use barqflow_db::models::ExecutionDispatchItemEntity;
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Result};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionQueueKind {
    Run,
    Trigger,
}

impl ExecutionQueueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Trigger => "trigger",
        }
    }
}

pub struct ExecutionDispatchRepository {
    pool: PgPool,
}

impl ExecutionDispatchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        execution_id: Uuid,
        workspace_id: Uuid,
        workflow_id: Uuid,
        queue_kind: ExecutionQueueKind,
        source: &str,
        priority: i32,
        payload: serde_json::Value,
        available_at: DateTime<Utc>,
    ) -> Result<ExecutionDispatchItemEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, ExecutionDispatchItemEntity>(
            r#"
            INSERT INTO execution_dispatch_queue (
                id,
                execution_id,
                workspace_id,
                workflow_id,
                queue_kind,
                source,
                status,
                priority,
                payload,
                available_at,
                claimed_at,
                lease_expires_at,
                worker_id,
                attempt_count,
                last_error,
                created_at,
                updated_at,
                finished_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, 'queued', $7, $8, $9,
                NULL, NULL, NULL, 0, NULL, $10, $10, NULL
            )
            RETURNING
                id, execution_id, workspace_id, workflow_id, queue_kind, source, status,
                priority, payload, available_at, claimed_at, lease_expires_at, worker_id,
                attempt_count, last_error, created_at, updated_at, finished_at
            "#,
        )
        .bind(id)
        .bind(execution_id)
        .bind(workspace_id)
        .bind(workflow_id)
        .bind(queue_kind.as_str())
        .bind(source)
        .bind(priority)
        .bind(payload)
        .bind(available_at)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn claim_next(
        &self,
        queue_kind: ExecutionQueueKind,
        worker_id: &str,
        lease_seconds: i64,
    ) -> Result<Option<ExecutionDispatchItemEntity>> {
        let now = Utc::now();
        let lease_expires_at = now + Duration::seconds(lease_seconds.max(30));

        sqlx::query_as::<_, ExecutionDispatchItemEntity>(
            r#"
            WITH candidate AS (
                SELECT id
                FROM execution_dispatch_queue
                WHERE queue_kind = $1
                  AND (
                    (status = 'queued' AND available_at <= $2)
                    OR (status = 'leased' AND lease_expires_at < $2)
                  )
                ORDER BY priority ASC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE execution_dispatch_queue AS queue
            SET status = 'leased',
                claimed_at = $2,
                lease_expires_at = $3,
                worker_id = $4,
                attempt_count = attempt_count + 1,
                updated_at = $2,
                last_error = NULL
            FROM candidate
            WHERE queue.id = candidate.id
            RETURNING
                queue.id,
                queue.execution_id,
                queue.workspace_id,
                queue.workflow_id,
                queue.queue_kind,
                queue.source,
                queue.status,
                queue.priority,
                queue.payload,
                queue.available_at,
                queue.claimed_at,
                queue.lease_expires_at,
                queue.worker_id,
                queue.attempt_count,
                queue.last_error,
                queue.created_at,
                queue.updated_at,
                queue.finished_at
            "#,
        )
        .bind(queue_kind.as_str())
        .bind(now)
        .bind(lease_expires_at)
        .bind(worker_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn renew_lease(
        &self,
        id: Uuid,
        lease_seconds: i64,
    ) -> Result<Option<ExecutionDispatchItemEntity>> {
        let now = Utc::now();
        let lease_expires_at = now + Duration::seconds(lease_seconds.max(30));

        sqlx::query_as::<_, ExecutionDispatchItemEntity>(
            r#"
            UPDATE execution_dispatch_queue
            SET lease_expires_at = $2,
                updated_at = $1
            WHERE id = $3
              AND status = 'leased'
            RETURNING
                id, execution_id, workspace_id, workflow_id, queue_kind, source, status,
                priority, payload, available_at, claimed_at, lease_expires_at, worker_id,
                attempt_count, last_error, created_at, updated_at, finished_at
            "#,
        )
        .bind(now)
        .bind(lease_expires_at)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn mark_completed(&self, id: Uuid) -> Result<Option<ExecutionDispatchItemEntity>> {
        let now = Utc::now();

        sqlx::query_as::<_, ExecutionDispatchItemEntity>(
            r#"
            UPDATE execution_dispatch_queue
            SET status = 'completed',
                updated_at = $1,
                finished_at = $1,
                lease_expires_at = NULL,
                last_error = NULL
            WHERE id = $2
            RETURNING
                id, execution_id, workspace_id, workflow_id, queue_kind, source, status,
                priority, payload, available_at, claimed_at, lease_expires_at, worker_id,
                attempt_count, last_error, created_at, updated_at, finished_at
            "#,
        )
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn mark_failed(
        &self,
        id: Uuid,
        error: &str,
    ) -> Result<Option<ExecutionDispatchItemEntity>> {
        let now = Utc::now();

        sqlx::query_as::<_, ExecutionDispatchItemEntity>(
            r#"
            UPDATE execution_dispatch_queue
            SET status = 'failed',
                updated_at = $1,
                finished_at = $1,
                lease_expires_at = NULL,
                last_error = $2
            WHERE id = $3
            RETURNING
                id, execution_id, workspace_id, workflow_id, queue_kind, source, status,
                priority, payload, available_at, claimed_at, lease_expires_at, worker_id,
                attempt_count, last_error, created_at, updated_at, finished_at
            "#,
        )
        .bind(now)
        .bind(error)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn count_open_items(&self) -> Result<i64> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM execution_dispatch_queue
            WHERE status IN ('queued', 'leased')
            "#,
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn count_open_items_by_kind(
        &self,
        queue_kind: ExecutionQueueKind,
    ) -> Result<i64> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM execution_dispatch_queue
            WHERE queue_kind = $1
              AND status IN ('queued', 'leased')
            "#,
        )
        .bind(queue_kind.as_str())
        .fetch_one(&self.pool)
        .await
    }
}
