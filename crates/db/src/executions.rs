use crate::models::ExecutionEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct ExecutionRepo {
    pool: PgPool,
}

impl ExecutionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> Result<Vec<ExecutionEntity>> {
        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            SELECT id, workflow_id, status, data, started_at, stopped_at
            FROM executions
            ORDER BY started_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<ExecutionEntity>> {
        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            SELECT id, workflow_id, status, data, started_at, stopped_at
            FROM executions
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_by_workflow_id(&self, workflow_id: Uuid) -> Result<Vec<ExecutionEntity>> {
        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            SELECT id, workflow_id, status, data, started_at, stopped_at
            FROM executions
            WHERE workflow_id = $1
            ORDER BY started_at DESC
            "#
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        workflow_id: Uuid,
        status: &str,
        data: serde_json::Value,
    ) -> Result<ExecutionEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            INSERT INTO executions (id, workflow_id, status, data, started_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, workflow_id, status, data, started_at, stopped_at
            "#
        )
        .bind(id)
        .bind(workflow_id)
        .bind(status)
        .bind(data)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_status_and_data(
        &self,
        id: Uuid,
        status: &str,
        data: serde_json::Value,
    ) -> Result<Option<ExecutionEntity>> {
        let now = Utc::now();
        let stopped_at = if status == "success" || status == "error" || status == "cancelled" || status == "crashed" {
            Some(now)
        } else {
            None
        };

        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            UPDATE executions
            SET status = $1, data = $2, stopped_at = COALESCE(stopped_at, $3)
            WHERE id = $4
            RETURNING id, workflow_id, status, data, started_at, stopped_at
            "#
        )
        .bind(status)
        .bind(data)
        .bind(stopped_at)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}
