use crate::models::WorkflowEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct WorkflowRepo {
    pool: PgPool,
}

impl WorkflowRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> Result<Vec<WorkflowEntity>> {
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

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<WorkflowEntity>> {
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

    pub async fn toggle_active(&self, id: Uuid, active: bool) -> Result<Option<WorkflowEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, WorkflowEntity>(
            r#"
            UPDATE workflows
            SET active = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, name, active, nodes, connections, settings, created_at, updated_at
            "#,
        )
        .bind(active)
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
