use barqflow_db::models::{TagEntity, WorkflowEntity, WorkflowHistorySnapshotEntity};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Result, Transaction};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use uuid::Uuid;

const WORKFLOW_COLUMNS: &str = r#"
    id,
    workspace_id,
    owner_user_id,
    name,
    active,
    nodes,
    connections,
    settings,
    created_at,
    updated_at
"#;

const TAG_COLUMNS: &str = r#"
    id,
    workspace_id,
    name,
    created_at,
    updated_at
"#;

pub struct WorkflowRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkflowSortBy {
    #[default]
    UpdatedAt,
    CreatedAt,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Default)]
pub struct WorkflowListFilters {
    pub active: Option<bool>,
    pub search: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
    pub sort_by: WorkflowSortBy,
    pub sort_direction: SortDirection,
}

#[derive(Debug, Clone)]
pub struct WorkflowUpsert {
    pub name: String,
    pub nodes: Value,
    pub connections: Value,
    pub settings: Value,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TagRecordEntity {
    pub tag: TagEntity,
    pub workflow_count: i64,
}

#[derive(Debug, Clone)]
pub struct WorkflowSummaryEntity {
    pub node_count: usize,
    pub trigger_count: usize,
    pub credential_binding_count: usize,
    pub tag_count: usize,
    pub latest_version: i32,
}

#[derive(Debug, Clone)]
pub struct WorkflowDocumentEntity {
    pub workflow: WorkflowEntity,
    pub tags: Vec<TagEntity>,
    pub latest_version: i32,
}

impl WorkflowDocumentEntity {
    pub fn summary(&self) -> WorkflowSummaryEntity {
        summarize_workflow(&self.workflow.nodes, self.tags.len(), self.latest_version)
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowHistoryEntryEntity {
    pub snapshot: WorkflowHistorySnapshotEntity,
    pub summary: WorkflowSummaryEntity,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTagJoinRow {
    workflow_id: Uuid,
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowVersionRow {
    workflow_id: Uuid,
    latest_version: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct TagCountRow {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    workflow_count: i64,
}

impl WorkflowRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            SELECT {WORKFLOW_COLUMNS}
            FROM workflows
            ORDER BY created_at DESC
            "#
        ))
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_all_by_active(&self, active: bool) -> Result<Vec<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            SELECT {WORKFLOW_COLUMNS}
            FROM workflows
            WHERE active = $1
            ORDER BY created_at ASC
            "#
        ))
        .bind(active)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_all_for_workspace(&self, workspace_id: Uuid) -> Result<Vec<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            SELECT {WORKFLOW_COLUMNS}
            FROM workflows
            WHERE workspace_id = $1
            ORDER BY created_at DESC
            "#
        ))
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_all_by_active_in_workspace(
        &self,
        workspace_id: Uuid,
        active: bool,
    ) -> Result<Vec<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            SELECT {WORKFLOW_COLUMNS}
            FROM workflows
            WHERE workspace_id = $1 AND active = $2
            ORDER BY created_at ASC
            "#
        ))
        .bind(workspace_id)
        .bind(active)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            SELECT {WORKFLOW_COLUMNS}
            FROM workflows
            WHERE id = $1
            "#
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_id_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            SELECT {WORKFLOW_COLUMNS}
            FROM workflows
            WHERE id = $1 AND workspace_id = $2
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_documents(
        &self,
        filters: WorkflowListFilters,
    ) -> Result<Vec<WorkflowDocumentEntity>> {
        let workflows = match filters.active {
            Some(active) => self.find_all_by_active(active).await?,
            None => self.find_all().await?,
        };

        self.filter_documents(workflows, filters).await
    }

    pub async fn find_documents_for_workspace(
        &self,
        workspace_id: Uuid,
        filters: WorkflowListFilters,
    ) -> Result<Vec<WorkflowDocumentEntity>> {
        let workflows = match filters.active {
            Some(active) => {
                self.find_all_by_active_in_workspace(workspace_id, active)
                    .await?
            }
            None => self.find_all_for_workspace(workspace_id).await?,
        };

        self.filter_documents(workflows, filters).await
    }

    pub async fn find_document_by_id(&self, id: Uuid) -> Result<Option<WorkflowDocumentEntity>> {
        let Some(workflow) = self.find_by_id(id).await? else {
            return Ok(None);
        };

        let mut documents = self.collect_documents(vec![workflow]).await?;
        Ok(documents.pop())
    }

    pub async fn find_document_by_id_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<WorkflowDocumentEntity>> {
        let Some(workflow) = self.find_by_id_in_workspace(workspace_id, id).await? else {
            return Ok(None);
        };

        let mut documents = self.collect_documents(vec![workflow]).await?;
        Ok(documents.pop())
    }

    pub async fn create(
        &self,
        name: &str,
        nodes: serde_json::Value,
        connections: serde_json::Value,
        settings: serde_json::Value,
    ) -> Result<WorkflowEntity> {
        let mut tx = self.pool.begin().await?;
        let (workspace_id, owner_user_id) = self.resolve_workspace_context_tx(&mut tx).await?;
        let workflow = self
            .insert_workflow_tx_with_scope(
                &mut tx,
                workspace_id,
                owner_user_id,
                name,
                nodes,
                connections,
                settings,
            )
            .await?;
        tx.commit().await?;
        Ok(workflow)
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: &str,
        nodes: serde_json::Value,
        connections: serde_json::Value,
        settings: serde_json::Value,
    ) -> Result<Option<WorkflowEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            UPDATE workflows
            SET name = $1, nodes = $2, connections = $3, settings = $4, updated_at = $5
            WHERE id = $6
            RETURNING {WORKFLOW_COLUMNS}
            "#
        ))
        .bind(name)
        .bind(nodes)
        .bind(connections)
        .bind(settings)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_document(
        &self,
        input: WorkflowUpsert,
        source: &str,
    ) -> Result<WorkflowDocumentEntity> {
        let mut tx = self.pool.begin().await?;
        let (workspace_id, owner_user_id) = self.resolve_workspace_context_tx(&mut tx).await?;
        let workflow = self
            .insert_workflow_tx(&mut tx, workspace_id, owner_user_id, &input)
            .await?;
        self.sync_workflow_tags_tx(&mut tx, &workflow, &input.tags)
            .await?;
        self.insert_history_snapshot_tx(&mut tx, &workflow, source)
            .await?;
        tx.commit().await?;

        Ok(self
            .find_document_by_id(workflow.id)
            .await?
            .expect("created workflow should exist after commit"))
    }

    pub async fn create_document_in_workspace(
        &self,
        workspace_id: Uuid,
        owner_user_id: Option<Uuid>,
        input: WorkflowUpsert,
        source: &str,
    ) -> Result<WorkflowDocumentEntity> {
        let mut tx = self.pool.begin().await?;
        let workflow = self
            .insert_workflow_tx(&mut tx, workspace_id, owner_user_id, &input)
            .await?;
        self.sync_workflow_tags_tx(&mut tx, &workflow, &input.tags)
            .await?;
        self.insert_history_snapshot_tx(&mut tx, &workflow, source)
            .await?;
        tx.commit().await?;

        Ok(self
            .find_document_by_id_in_workspace(workspace_id, workflow.id)
            .await?
            .expect("created workflow should exist after commit"))
    }

    pub async fn update_document(
        &self,
        id: Uuid,
        input: WorkflowUpsert,
        source: &str,
    ) -> Result<Option<WorkflowDocumentEntity>> {
        let mut tx = self.pool.begin().await?;
        let Some(workflow) = self.update_workflow_tx(&mut tx, id, None, &input).await? else {
            return Ok(None);
        };

        self.sync_workflow_tags_tx(&mut tx, &workflow, &input.tags)
            .await?;
        self.insert_history_snapshot_tx(&mut tx, &workflow, source)
            .await?;
        tx.commit().await?;

        self.find_document_by_id(id).await
    }

    pub async fn update_document_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        input: WorkflowUpsert,
        source: &str,
    ) -> Result<Option<WorkflowDocumentEntity>> {
        let mut tx = self.pool.begin().await?;
        let Some(workflow) = self
            .update_workflow_tx(&mut tx, id, Some(workspace_id), &input)
            .await?
        else {
            return Ok(None);
        };

        self.sync_workflow_tags_tx(&mut tx, &workflow, &input.tags)
            .await?;
        self.insert_history_snapshot_tx(&mut tx, &workflow, source)
            .await?;
        tx.commit().await?;

        self.find_document_by_id_in_workspace(workspace_id, id)
            .await
    }

    pub async fn toggle_active(&self, id: Uuid, active: bool) -> Result<Option<WorkflowEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            UPDATE workflows
            SET active = $1, updated_at = $2
            WHERE id = $3
            RETURNING {WORKFLOW_COLUMNS}
            "#
        ))
        .bind(active)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn toggle_active_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        active: bool,
    ) -> Result<Option<WorkflowEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            UPDATE workflows
            SET active = $1, updated_at = $2
            WHERE id = $3 AND workspace_id = $4
            RETURNING {WORKFLOW_COLUMNS}
            "#
        ))
        .bind(active)
        .bind(now)
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn record_snapshot_by_workflow_id(
        &self,
        workflow_id: Uuid,
        source: &str,
    ) -> Result<Option<WorkflowHistoryEntryEntity>> {
        let mut tx = self.pool.begin().await?;
        let Some(workflow) = sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            SELECT {WORKFLOW_COLUMNS}
            FROM workflows
            WHERE id = $1
            "#
        ))
        .bind(workflow_id)
        .fetch_optional(&mut *tx)
        .await?
        else {
            return Ok(None);
        };

        let snapshot = self
            .insert_history_snapshot_tx(&mut tx, &workflow, source)
            .await?;
        tx.commit().await?;

        Ok(Some(to_history_entry(snapshot)))
    }

    pub async fn list_history(&self, workflow_id: Uuid) -> Result<Vec<WorkflowHistoryEntryEntity>> {
        let snapshots = sqlx::query_as::<_, WorkflowHistorySnapshotEntity>(
            r#"
            SELECT id, workflow_id, version, source, name, active, tags, nodes, connections, settings, created_at
            FROM workflow_history_snapshots
            WHERE workflow_id = $1
            ORDER BY version DESC, created_at DESC
            "#,
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(snapshots.into_iter().map(to_history_entry).collect())
    }

    pub async fn find_history_entry(
        &self,
        workflow_id: Uuid,
        version: i32,
    ) -> Result<Option<WorkflowHistoryEntryEntity>> {
        let snapshot = sqlx::query_as::<_, WorkflowHistorySnapshotEntity>(
            r#"
            SELECT id, workflow_id, version, source, name, active, tags, nodes, connections, settings, created_at
            FROM workflow_history_snapshots
            WHERE workflow_id = $1 AND version = $2
            "#,
        )
        .bind(workflow_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(snapshot.map(to_history_entry))
    }

    pub async fn list_tags(&self) -> Result<Vec<TagRecordEntity>> {
        let rows = sqlx::query_as::<_, TagCountRow>(
            r#"
            SELECT t.id, t.workspace_id, t.name, t.created_at, t.updated_at, COUNT(wt.workflow_id) AS workflow_count
            FROM tags t
            LEFT JOIN workflow_tags wt ON wt.tag_id = t.id
            GROUP BY t.id, t.workspace_id, t.name, t.created_at, t.updated_at
            ORDER BY LOWER(t.name) ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(map_tag_record_row).collect())
    }

    pub async fn list_tags_in_workspace(&self, workspace_id: Uuid) -> Result<Vec<TagRecordEntity>> {
        let rows = sqlx::query_as::<_, TagCountRow>(
            r#"
            SELECT t.id, t.workspace_id, t.name, t.created_at, t.updated_at, COUNT(wt.workflow_id) AS workflow_count
            FROM tags t
            LEFT JOIN workflow_tags wt ON wt.tag_id = t.id
            WHERE t.workspace_id = $1
            GROUP BY t.id, t.workspace_id, t.name, t.created_at, t.updated_at
            ORDER BY LOWER(t.name) ASC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(map_tag_record_row).collect())
    }

    pub async fn create_tag(&self, name: &str) -> Result<TagRecordEntity> {
        let mut tx = self.pool.begin().await?;
        let (workspace_id, _) = self.resolve_workspace_context_tx(&mut tx).await?;
        let tag = self.ensure_tag_tx(&mut tx, workspace_id, name).await?;
        tx.commit().await?;

        Ok(self
            .find_tag_record_by_id(tag.id)
            .await?
            .unwrap_or(TagRecordEntity {
                tag,
                workflow_count: 0,
            }))
    }

    pub async fn create_tag_in_workspace(
        &self,
        workspace_id: Uuid,
        name: &str,
    ) -> Result<TagRecordEntity> {
        let mut tx = self.pool.begin().await?;
        let tag = self.ensure_tag_tx(&mut tx, workspace_id, name).await?;
        tx.commit().await?;

        Ok(self
            .find_tag_record_by_id(tag.id)
            .await?
            .unwrap_or(TagRecordEntity {
                tag,
                workflow_count: 0,
            }))
    }

    pub async fn delete_tag(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM tags
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_tag_in_workspace(&self, workspace_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM tags
            WHERE id = $1 AND workspace_id = $2
            "#,
        )
        .bind(id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM workflows
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_in_workspace(&self, workspace_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM workflows
            WHERE id = $1 AND workspace_id = $2
            "#,
        )
        .bind(id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn filter_documents(
        &self,
        workflows: Vec<WorkflowEntity>,
        filters: WorkflowListFilters,
    ) -> Result<Vec<WorkflowDocumentEntity>> {
        let mut documents = self.collect_documents(workflows).await?;

        if let Some(needle) = filters
            .search
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_lowercase())
        {
            documents.retain(|document| {
                document.workflow.name.to_lowercase().contains(&needle)
                    || document
                        .tags
                        .iter()
                        .any(|tag| tag.name.to_lowercase().contains(&needle))
            });
        }

        let required_tags = normalize_tag_names(&filters.tags);
        if !required_tags.is_empty() {
            documents.retain(|document| {
                let document_tags: HashSet<String> = document
                    .tags
                    .iter()
                    .map(|tag| tag.name.to_lowercase())
                    .collect();

                required_tags
                    .iter()
                    .all(|tag| document_tags.contains(&tag.to_lowercase()))
            });
        }

        documents.sort_by(|left, right| {
            compare_workflow_documents(left, right, filters.sort_by, filters.sort_direction)
        });

        if let Some(limit) = filters.limit {
            documents.truncate(limit);
        }

        Ok(documents)
    }

    async fn collect_documents(
        &self,
        workflows: Vec<WorkflowEntity>,
    ) -> Result<Vec<WorkflowDocumentEntity>> {
        if workflows.is_empty() {
            return Ok(Vec::new());
        }

        let workflow_ids: Vec<Uuid> = workflows.iter().map(|workflow| workflow.id).collect();
        let mut tags_by_workflow = self.load_tags_for_workflow_ids(&workflow_ids).await?;
        let versions_by_workflow = self
            .load_latest_versions_for_workflow_ids(&workflow_ids)
            .await?;

        Ok(workflows
            .into_iter()
            .map(|workflow| WorkflowDocumentEntity {
                latest_version: versions_by_workflow
                    .get(&workflow.id)
                    .copied()
                    .unwrap_or_default(),
                tags: tags_by_workflow.remove(&workflow.id).unwrap_or_default(),
                workflow,
            })
            .collect())
    }

    async fn load_tags_for_workflow_ids(
        &self,
        workflow_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<TagEntity>>> {
        if workflow_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query_as::<_, WorkflowTagJoinRow>(
            r#"
            SELECT wt.workflow_id, t.id, t.workspace_id, t.name, t.created_at, t.updated_at
            FROM workflow_tags wt
            INNER JOIN tags t ON t.id = wt.tag_id
            WHERE wt.workflow_id = ANY($1)
            ORDER BY LOWER(t.name) ASC
            "#,
        )
        .bind(workflow_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut tags_by_workflow: HashMap<Uuid, Vec<TagEntity>> = HashMap::new();
        for row in rows {
            tags_by_workflow
                .entry(row.workflow_id)
                .or_default()
                .push(TagEntity {
                    id: row.id,
                    workspace_id: row.workspace_id,
                    name: row.name,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                });
        }

        Ok(tags_by_workflow)
    }

    async fn load_latest_versions_for_workflow_ids(
        &self,
        workflow_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, i32>> {
        if workflow_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query_as::<_, WorkflowVersionRow>(
            r#"
            SELECT workflow_id, MAX(version) AS latest_version
            FROM workflow_history_snapshots
            WHERE workflow_id = ANY($1)
            GROUP BY workflow_id
            "#,
        )
        .bind(workflow_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.workflow_id, row.latest_version.unwrap_or_default()))
            .collect())
    }

    async fn find_tag_record_by_id(&self, id: Uuid) -> Result<Option<TagRecordEntity>> {
        let row = sqlx::query_as::<_, TagCountRow>(
            r#"
            SELECT t.id, t.workspace_id, t.name, t.created_at, t.updated_at, COUNT(wt.workflow_id) AS workflow_count
            FROM tags t
            LEFT JOIN workflow_tags wt ON wt.tag_id = t.id
            WHERE t.id = $1
            GROUP BY t.id, t.workspace_id, t.name, t.created_at, t.updated_at
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(map_tag_record_row))
    }

    async fn insert_workflow_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: Uuid,
        owner_user_id: Option<Uuid>,
        input: &WorkflowUpsert,
    ) -> Result<WorkflowEntity> {
        self.insert_workflow_tx_with_scope(
            tx,
            workspace_id,
            owner_user_id,
            &input.name,
            input.nodes.clone(),
            input.connections.clone(),
            input.settings.clone(),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_workflow_tx_with_scope(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: Uuid,
        owner_user_id: Option<Uuid>,
        name: &str,
        nodes: serde_json::Value,
        connections: serde_json::Value,
        settings: serde_json::Value,
    ) -> Result<WorkflowEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, WorkflowEntity>(&format!(
            r#"
            INSERT INTO workflows (
                id, workspace_id, owner_user_id, name, active, nodes, connections, settings, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING {WORKFLOW_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .bind(owner_user_id)
        .bind(name)
        .bind(false)
        .bind(nodes)
        .bind(connections)
        .bind(settings)
        .bind(now)
        .bind(now)
        .fetch_one(&mut **tx)
        .await
    }

    async fn update_workflow_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        workspace_id: Option<Uuid>,
        input: &WorkflowUpsert,
    ) -> Result<Option<WorkflowEntity>> {
        let now = Utc::now();

        let mut sql = r#"
            UPDATE workflows
            SET name = $1, nodes = $2, connections = $3, settings = $4, updated_at = $5
            WHERE id = $6
            "#
        .to_string();
        if workspace_id.is_some() {
            sql.push_str(" AND workspace_id = $7");
        }
        sql.push_str(&format!(" RETURNING {WORKFLOW_COLUMNS}"));

        let mut query = sqlx::query_as::<_, WorkflowEntity>(&sql)
            .bind(&input.name)
            .bind(input.nodes.clone())
            .bind(input.connections.clone())
            .bind(input.settings.clone())
            .bind(now)
            .bind(id);

        if let Some(workspace_id) = workspace_id {
            query = query.bind(workspace_id);
        }

        query.fetch_optional(&mut **tx).await
    }

    async fn sync_workflow_tags_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        workflow: &WorkflowEntity,
        tags: &[String],
    ) -> Result<()> {
        let normalized_tags = normalize_tag_names(tags);

        sqlx::query(
            r#"
            DELETE FROM workflow_tags
            WHERE workflow_id = $1
            "#,
        )
        .bind(workflow.id)
        .execute(&mut **tx)
        .await?;

        for tag_name in normalized_tags {
            let tag = self
                .ensure_tag_tx(tx, workflow.workspace_id, &tag_name)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO workflow_tags (workflow_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT (workflow_id, tag_id) DO NOTHING
                "#,
            )
            .bind(workflow.id)
            .bind(tag.id)
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    async fn ensure_tag_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: Uuid,
        name: &str,
    ) -> Result<TagEntity> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return sqlx::query_as::<_, TagEntity>(&format!(
                r#"
                SELECT {TAG_COLUMNS}
                FROM tags
                WHERE workspace_id = $1
                ORDER BY created_at ASC
                LIMIT 0
                "#
            ))
            .bind(workspace_id)
            .fetch_one(&mut **tx)
            .await;
        }

        if let Some(existing) = sqlx::query_as::<_, TagEntity>(&format!(
            r#"
            SELECT {TAG_COLUMNS}
            FROM tags
            WHERE workspace_id = $1 AND LOWER(name) = LOWER($2)
            LIMIT 1
            "#
        ))
        .bind(workspace_id)
        .bind(trimmed)
        .fetch_optional(&mut **tx)
        .await?
        {
            return Ok(existing);
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query_as::<_, TagEntity>(&format!(
            r#"
            INSERT INTO tags (id, workspace_id, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING {TAG_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .bind(trimmed)
        .bind(now)
        .bind(now)
        .fetch_one(&mut **tx)
        .await
    }

    async fn insert_history_snapshot_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        workflow: &WorkflowEntity,
        source: &str,
    ) -> Result<WorkflowHistorySnapshotEntity> {
        let version: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(MAX(version), 0) + 1
            FROM workflow_history_snapshots
            WHERE workflow_id = $1
            "#,
        )
        .bind(workflow.id)
        .fetch_one(&mut **tx)
        .await?;

        let tag_names: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT t.name
            FROM workflow_tags wt
            INNER JOIN tags t ON t.id = wt.tag_id
            WHERE wt.workflow_id = $1
            ORDER BY LOWER(t.name) ASC
            "#,
        )
        .bind(workflow.id)
        .fetch_all(&mut **tx)
        .await?;

        sqlx::query_as::<_, WorkflowHistorySnapshotEntity>(
            r#"
            INSERT INTO workflow_history_snapshots (
                id, workflow_id, version, source, name, active, tags, nodes, connections, settings, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, workflow_id, version, source, name, active, tags, nodes, connections, settings, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(workflow.id)
        .bind(version)
        .bind(source)
        .bind(&workflow.name)
        .bind(workflow.active)
        .bind(serde_json::to_value(tag_names).unwrap_or_else(|_| Value::Array(Vec::new())))
        .bind(workflow.nodes.clone())
        .bind(workflow.connections.clone())
        .bind(workflow.settings.clone())
        .bind(Utc::now())
        .fetch_one(&mut **tx)
        .await
    }

    async fn resolve_workspace_context_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(Uuid, Option<Uuid>)> {
        let existing = sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
            r#"
            SELECT id, created_by_user_id
            FROM workspaces
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(context) = existing {
            return Ok(context);
        }

        let workspace_id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO workspaces (id, name, slug, created_by_user_id, created_at, updated_at)
            VALUES ($1, 'Imported Workspace', 'imported-workspace', NULL, $2, $3)
            "#,
        )
        .bind(workspace_id)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        Ok((workspace_id, None))
    }
}

fn normalize_tag_names(tags: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::new();

    for raw_tag in tags {
        let trimmed = raw_tag.trim();
        if trimmed.is_empty() {
            continue;
        }

        let key = trimmed.to_lowercase();
        if seen.insert(key) {
            normalized.push(trimmed.to_string());
        }
    }

    normalized
}

fn compare_workflow_documents(
    left: &WorkflowDocumentEntity,
    right: &WorkflowDocumentEntity,
    sort_by: WorkflowSortBy,
    sort_direction: SortDirection,
) -> Ordering {
    let ordering = match sort_by {
        WorkflowSortBy::Name => left
            .workflow
            .name
            .to_lowercase()
            .cmp(&right.workflow.name.to_lowercase()),
        WorkflowSortBy::CreatedAt => left.workflow.created_at.cmp(&right.workflow.created_at),
        WorkflowSortBy::UpdatedAt => left.workflow.updated_at.cmp(&right.workflow.updated_at),
    };

    match sort_direction {
        SortDirection::Asc => ordering,
        SortDirection::Desc => ordering.reverse(),
    }
}

fn summarize_workflow(
    nodes: &Value,
    tag_count: usize,
    latest_version: i32,
) -> WorkflowSummaryEntity {
    let mut node_count = 0;
    let mut trigger_count = 0;
    let mut credential_binding_count = 0;

    for node in nodes.as_array().into_iter().flatten() {
        node_count += 1;

        if is_trigger_node_type(node.get("type").and_then(Value::as_str).unwrap_or_default()) {
            trigger_count += 1;
        }

        credential_binding_count += node
            .get("credentials")
            .and_then(Value::as_array)
            .map(|credentials| credentials.len())
            .unwrap_or_default();
    }

    WorkflowSummaryEntity {
        node_count,
        trigger_count,
        credential_binding_count,
        tag_count,
        latest_version,
    }
}

fn is_trigger_node_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "manualTrigger"
            | "n8n-nodes-base.manualTrigger"
            | "webhook"
            | "barqflow-nodes.webhook"
            | "cron"
            | "barqflow-nodes.cronTrigger"
    )
}

fn to_history_entry(snapshot: WorkflowHistorySnapshotEntity) -> WorkflowHistoryEntryEntity {
    let tag_count = snapshot
        .tags
        .as_array()
        .map(|tags| tags.len())
        .unwrap_or_default();
    let summary = summarize_workflow(&snapshot.nodes, tag_count, snapshot.version);

    WorkflowHistoryEntryEntity { snapshot, summary }
}

fn map_tag_record_row(row: TagCountRow) -> TagRecordEntity {
    TagRecordEntity {
        tag: TagEntity {
            id: row.id,
            workspace_id: row.workspace_id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        },
        workflow_count: row.workflow_count,
    }
}
