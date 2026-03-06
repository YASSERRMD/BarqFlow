use barqflow_db::models::WorkflowEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct WorkflowRepository {
    pool: PgPool,
}

impl WorkflowRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(
            r#"
            SELECT id, name, active, nodes, connections, settings, created_at, updated_at
            FROM workflows
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_all_by_active(&self, active: bool) -> Result<Vec<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(
            r#"
            SELECT id, name, active, nodes, connections, settings, created_at, updated_at
            FROM workflows
            WHERE active = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(active)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkflowEntity>> {
        sqlx::query_as::<_, WorkflowEntity>(
            r#"
            SELECT id, name, active, nodes, connections, settings, created_at, updated_at
            FROM workflows
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        name: &str,
        nodes: serde_json::Value,
        connections: serde_json::Value,
        settings: serde_json::Value,
    ) -> Result<WorkflowEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, WorkflowEntity>(
            r#"
            INSERT INTO workflows (id, name, active, nodes, connections, settings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, active, nodes, connections, settings, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(name)
        .bind(false)
        .bind(nodes)
        .bind(connections)
        .bind(settings)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, name: &str, nodes: serde_json::Value, connections: serde_json::Value, settings: serde_json::Value) -> Result<Option<WorkflowEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, WorkflowEntity>(
            r#"
            UPDATE workflows
            SET name = $1, nodes = $2, connections = $3, settings = $4, updated_at = $5
            WHERE id = $6
            RETURNING id, name, active, nodes, connections, settings, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(nodes)
        .bind(connections)
        .bind(settings)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // We use sqlx::test which automatically creates a fresh database per test and runs migrations
    #[sqlx::test(migrations = "./migrations")]
    async fn test_workflow_lifecycle(pool: PgPool) {
        let repo = WorkflowRepository::new(pool);
        
        let initial = repo.find_all().await.unwrap();
        assert_eq!(initial.len(), 0);

        let created = repo.create(
            "Test Flow",
            json!([]),
            json!({}),
            json!({})
        ).await.unwrap();

        assert_eq!(created.name, "Test Flow");
        assert_eq!(created.active, false);

        let found = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(found.id, created.id);

        let active_flows = repo.find_all_by_active(false).await.unwrap();
        assert_eq!(active_flows.len(), 1);

        repo.update(created.id, "Updated Flow", json!([]), json!({}), json!({})).await.unwrap();
        let updated = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated Flow");

        repo.delete(created.id).await.unwrap();
        let final_count = repo.find_all().await.unwrap().len();
        assert_eq!(final_count, 0);
    }
}
