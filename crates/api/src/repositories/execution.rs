use barqflow_db::models::{ExecutionEntity, WaitResumeEntity};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct ExecutionRepository {
    pool: PgPool,
}

impl ExecutionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<ExecutionEntity>> {
        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            SELECT id, workflow_id, status, data, started_at, stopped_at
            FROM executions
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ExecutionEntity>> {
        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            SELECT id, workflow_id, status, data, started_at, stopped_at
            FROM executions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_workflow_id(&self, workflow_id: Uuid) -> Result<Vec<ExecutionEntity>> {
        sqlx::query_as::<_, ExecutionEntity>(
            r#"
            SELECT id, workflow_id, status, data, started_at, stopped_at
            FROM executions
            WHERE workflow_id = $1
            ORDER BY started_at DESC
            "#,
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
            "#,
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
        let stopped_at = if status == "success"
            || status == "error"
            || status == "failed"
            || status == "cancelled"
            || status == "stopped"
            || status == "crashed"
        {
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
            "#,
        )
        .bind(status)
        .bind(data)
        .bind(stopped_at)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_older_than(&self, before: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM executions
            WHERE started_at < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM executions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_wait_resume(
        &self,
        execution_id: Uuid,
        node_name: &str,
        resume_token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<WaitResumeEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, WaitResumeEntity>(
            r#"
            INSERT INTO wait_resumes (
                id, execution_id, node_name, resume_token, created_at, expires_at, resumed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NULL)
            RETURNING id, execution_id, node_name, resume_token, created_at, expires_at, resumed_at
            "#,
        )
        .bind(id)
        .bind(execution_id)
        .bind(node_name)
        .bind(resume_token)
        .bind(now)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_wait_resume(
        &self,
        execution_id: Uuid,
        resume_token: &str,
    ) -> Result<Option<WaitResumeEntity>> {
        sqlx::query_as::<_, WaitResumeEntity>(
            r#"
            SELECT id, execution_id, node_name, resume_token, created_at, expires_at, resumed_at
            FROM wait_resumes
            WHERE execution_id = $1 AND resume_token = $2
            "#,
        )
        .bind(execution_id)
        .bind(resume_token)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn mark_wait_resume_resumed(&self, id: Uuid) -> Result<Option<WaitResumeEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, WaitResumeEntity>(
            r#"
            UPDATE wait_resumes
            SET resumed_at = $1
            WHERE id = $2
            RETURNING id, execution_id, node_name, resume_token, created_at, expires_at, resumed_at
            "#,
        )
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_wait_resume(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM wait_resumes
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
    use crate::repositories::workflow::WorkflowRepository;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_execution_lifecycle(pool: PgPool) {
        let workflow_repo = WorkflowRepository::new(pool.clone());
        let exec_repo = ExecutionRepository::new(pool);

        // Need a workflow first to satisfy foreign key constraint
        let workflow = workflow_repo.create("Test Flow", json!([]), json!({}), json!({})).await.unwrap();

        let initial = exec_repo.find_all().await.unwrap();
        assert_eq!(initial.len(), 0);

        // Huge JSON payload to test JSONB storage
        let mut large_array = Vec::new();
        for i in 0..10 {
            large_array.push(json!({"index": i, "data": "A".repeat(100)}));
        }
        let heavy_payload = json!({ "items": large_array });

        let created = exec_repo.create(workflow.id, "running", heavy_payload.clone()).await.unwrap();
        assert_eq!(created.status, "running");
        assert_eq!(created.workflow_id, workflow.id);

        exec_repo.update_status_and_data(created.id, "success", heavy_payload).await.unwrap();
        
        let found = exec_repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(found.status, "success");
        assert!(found.stopped_at.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_wait_resume_lifecycle(pool: PgPool) {
        let workflow_repo = WorkflowRepository::new(pool.clone());
        let exec_repo = ExecutionRepository::new(pool);

        let workflow = workflow_repo
            .create("Wait Flow", json!([]), json!({}), json!({}))
            .await
            .unwrap();
        let execution = exec_repo
            .create(workflow.id, "running", json!({}))
            .await
            .unwrap();

        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        let created = exec_repo
            .create_wait_resume(execution.id, "Wait Node", &token, expires_at)
            .await
            .unwrap();
        assert_eq!(created.execution_id, execution.id);

        let found = exec_repo
            .find_wait_resume(execution.id, &token)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.id, created.id);
        assert!(found.resumed_at.is_none());

        let resumed = exec_repo
            .mark_wait_resume_resumed(created.id)
            .await
            .unwrap()
            .unwrap();
        assert!(resumed.resumed_at.is_some());

        let deleted = exec_repo.delete_wait_resume(created.id).await.unwrap();
        assert!(deleted);
    }
}
