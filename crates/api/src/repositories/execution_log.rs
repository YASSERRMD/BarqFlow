use barqflow_db::models::ExecutionLogEntity;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct ExecutionLogRepository {
    pool: PgPool,
}

impl ExecutionLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        execution_id: Uuid,
        workflow_id: Uuid,
        level: &str,
        event_type: Option<&str>,
        message: &str,
        node_id: Option<&str>,
        node_name: Option<&str>,
        payload: serde_json::Value,
    ) -> Result<ExecutionLogEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, ExecutionLogEntity>(
            r#"
            INSERT INTO execution_logs (
                id,
                execution_id,
                workflow_id,
                level,
                event_type,
                message,
                node_id,
                node_name,
                payload,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, execution_id, workflow_id, level, event_type, message, node_id, node_name, payload, created_at
            "#,
        )
        .bind(id)
        .bind(execution_id)
        .bind(workflow_id)
        .bind(level)
        .bind(event_type)
        .bind(message)
        .bind(node_id)
        .bind(node_name)
        .bind(payload)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_for_execution(
        &self,
        execution_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ExecutionLogEntity>> {
        sqlx::query_as::<_, ExecutionLogEntity>(
            r#"
            SELECT id, execution_id, workflow_id, level, event_type, message, node_id, node_name, payload, created_at
            FROM execution_logs
            WHERE execution_id = $1
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(execution_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn count_prunable_for_executions_before(&self, before: DateTime<Utc>) -> Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM execution_logs logs
            INNER JOIN executions executions ON executions.id = logs.execution_id
            WHERE COALESCE(executions.stopped_at, executions.started_at) < $1
              AND LOWER(executions.status) IN ('success', 'error', 'failed', 'cancelled', 'stopped', 'crashed')
            "#,
        )
        .bind(before)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.max(0) as u64)
    }
}
