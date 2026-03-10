use barqflow_db::models::{WorkspaceEntity, WorkspaceMembershipEntity};
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct WorkspaceRepository {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct WorkspaceMembershipDocument {
    pub workspace: WorkspaceEntity,
    pub membership: WorkspaceMembershipEntity,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WorkspaceMemberRecord {
    pub membership_id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkspaceMembershipRow {
    workspace_id: Uuid,
    workspace_name: String,
    workspace_slug: String,
    workspace_created_by_user_id: Option<Uuid>,
    workspace_created_at: chrono::DateTime<Utc>,
    workspace_updated_at: chrono::DateTime<Utc>,
    membership_id: Uuid,
    membership_user_id: Uuid,
    membership_role: String,
    membership_created_at: chrono::DateTime<Utc>,
    membership_updated_at: chrono::DateTime<Utc>,
}

impl WorkspaceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_workspace(
        &self,
        name: &str,
        created_by_user_id: Uuid,
    ) -> Result<WorkspaceMembershipDocument> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();
        let workspace = sqlx::query_as::<_, WorkspaceEntity>(
            r#"
            INSERT INTO workspaces (id, name, slug, created_by_user_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, slug, created_by_user_id, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(build_workspace_slug(name))
        .bind(created_by_user_id)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        let membership = sqlx::query_as::<_, WorkspaceMembershipEntity>(
            r#"
            INSERT INTO workspace_memberships (id, workspace_id, user_id, role, created_at, updated_at)
            VALUES ($1, $2, $3, 'owner', $4, $5)
            RETURNING id, workspace_id, user_id, role, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(workspace.id)
        .bind(created_by_user_id)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE users
            SET active_workspace_id = COALESCE(active_workspace_id, $1), updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(workspace.id)
        .bind(now)
        .bind(created_by_user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(WorkspaceMembershipDocument {
            workspace,
            membership,
        })
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<WorkspaceMembershipDocument>> {
        let rows = sqlx::query_as::<_, WorkspaceMembershipRow>(
            r#"
            SELECT
                w.id AS workspace_id,
                w.name AS workspace_name,
                w.slug AS workspace_slug,
                w.created_by_user_id AS workspace_created_by_user_id,
                w.created_at AS workspace_created_at,
                w.updated_at AS workspace_updated_at,
                wm.id AS membership_id,
                wm.user_id AS membership_user_id,
                wm.role AS membership_role,
                wm.created_at AS membership_created_at,
                wm.updated_at AS membership_updated_at
            FROM workspace_memberships wm
            INNER JOIN workspaces w ON w.id = wm.workspace_id
            WHERE wm.user_id = $1
            ORDER BY LOWER(w.name) ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(map_membership_row).collect())
    }

    pub async fn find_membership(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<WorkspaceMembershipDocument>> {
        let row = sqlx::query_as::<_, WorkspaceMembershipRow>(
            r#"
            SELECT
                w.id AS workspace_id,
                w.name AS workspace_name,
                w.slug AS workspace_slug,
                w.created_by_user_id AS workspace_created_by_user_id,
                w.created_at AS workspace_created_at,
                w.updated_at AS workspace_updated_at,
                wm.id AS membership_id,
                wm.user_id AS membership_user_id,
                wm.role AS membership_role,
                wm.created_at AS membership_created_at,
                wm.updated_at AS membership_updated_at
            FROM workspace_memberships wm
            INNER JOIN workspaces w ON w.id = wm.workspace_id
            WHERE wm.user_id = $1 AND wm.workspace_id = $2
            "#,
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(map_membership_row))
    }

    pub async fn get_current_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Option<WorkspaceMembershipDocument>> {
        let row = sqlx::query_as::<_, WorkspaceMembershipRow>(
            r#"
            SELECT
                w.id AS workspace_id,
                w.name AS workspace_name,
                w.slug AS workspace_slug,
                w.created_by_user_id AS workspace_created_by_user_id,
                w.created_at AS workspace_created_at,
                w.updated_at AS workspace_updated_at,
                wm.id AS membership_id,
                wm.user_id AS membership_user_id,
                wm.role AS membership_role,
                wm.created_at AS membership_created_at,
                wm.updated_at AS membership_updated_at
            FROM users u
            INNER JOIN workspace_memberships wm
                ON wm.user_id = u.id AND wm.workspace_id = u.active_workspace_id
            INNER JOIN workspaces w ON w.id = wm.workspace_id
            WHERE u.id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(map_membership_row))
    }

    pub async fn add_or_update_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<WorkspaceMembershipEntity> {
        let now = Utc::now();
        sqlx::query_as::<_, WorkspaceMembershipEntity>(
            r#"
            INSERT INTO workspace_memberships (id, workspace_id, user_id, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (workspace_id, user_id)
            DO UPDATE SET role = EXCLUDED.role, updated_at = EXCLUDED.updated_at
            RETURNING id, workspace_id, user_id, role, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(user_id)
        .bind(role)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_members(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceMemberRecord>> {
        sqlx::query_as::<_, WorkspaceMemberRecord>(
            r#"
            SELECT
                wm.id AS membership_id,
                wm.workspace_id,
                wm.user_id,
                wm.role,
                u.email,
                u.first_name,
                u.last_name,
                wm.created_at,
                wm.updated_at
            FROM workspace_memberships wm
            INNER JOIN users u ON u.id = wm.user_id
            WHERE wm.workspace_id = $1
            ORDER BY LOWER(u.email) ASC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
    }
}

fn build_workspace_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in name.chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            slug.push(normalized);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-');
    let base = if trimmed.is_empty() {
        "workspace"
    } else {
        trimmed
    };
    format!("{}-{}", base, &Uuid::new_v4().simple().to_string()[..8])
}

fn map_membership_row(row: WorkspaceMembershipRow) -> WorkspaceMembershipDocument {
    WorkspaceMembershipDocument {
        workspace: WorkspaceEntity {
            id: row.workspace_id,
            name: row.workspace_name,
            slug: row.workspace_slug,
            created_by_user_id: row.workspace_created_by_user_id,
            created_at: row.workspace_created_at,
            updated_at: row.workspace_updated_at,
        },
        membership: WorkspaceMembershipEntity {
            id: row.membership_id,
            workspace_id: row.workspace_id,
            user_id: row.membership_user_id,
            role: row.membership_role,
            created_at: row.membership_created_at,
            updated_at: row.membership_updated_at,
        },
    }
}
