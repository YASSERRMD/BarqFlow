use barqflow_db::models::StaticDataEntity;
use chrono::Utc;
use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;
use barqflow_core::traits::IStaticDataStorage;
use barqflow_core::types::IDataObject;
use barqflow_core::errors::BarqError;
use async_trait::async_trait;

pub struct StaticDataRepository {
    pool: PgPool,
}

impl StaticDataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_node_and_workflow(
        &self,
        node_id: &str,
        workflow_id: Uuid,
    ) -> SqlxResult<Option<StaticDataEntity>> {
        sqlx::query_as::<_, StaticDataEntity>(
            r#"
            SELECT id, node_id, workflow_id, data, created_at, updated_at
            FROM static_data
            WHERE node_id = $1 AND workflow_id = $2
            "#,
        )
        .bind(node_id)
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn upsert(
        &self,
        node_id: &str,
        workflow_id: Uuid,
        data: serde_json::Value,
    ) -> SqlxResult<StaticDataEntity> {
        let now = Utc::now();

        sqlx::query_as::<_, StaticDataEntity>(
            r#"
            INSERT INTO static_data (id, node_id, workflow_id, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (node_id, workflow_id) 
            DO UPDATE SET data = EXCLUDED.data, updated_at = EXCLUDED.updated_at
            RETURNING id, node_id, workflow_id, data, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(node_id)
        .bind(workflow_id)
        .bind(data)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }
}

#[async_trait]
impl IStaticDataStorage for StaticDataRepository {
    async fn get(
        &self,
        node_id: String,
        workflow_id: Uuid,
    ) -> std::result::Result<Option<IDataObject>, BarqError> {
        match self.find_by_node_and_workflow(&node_id, workflow_id).await {
            Ok(Some(entity)) => {
                if let Some(obj) = entity.data.as_object() {
                    Ok(Some(IDataObject(obj.clone())))
                } else {
                    Ok(Some(IDataObject::default()))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BarqError::NodeOperationError {
                node_name: node_id.clone(),
                message: format!("DB Error getting static data: {}", e),
            }),
        }
    }

    async fn upsert(
        &self,
        node_id: String,
        workflow_id: Uuid,
        data: IDataObject,
    ) -> std::result::Result<(), BarqError> {
        self.upsert(&node_id, workflow_id, serde_json::Value::Object(data.0))
            .await
            .map(|_| ())
            .map_err(|e| BarqError::NodeOperationError {
                node_name: node_id.clone(),
                message: format!("DB Error upserting static data: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::repositories::workflow::WorkflowRepository;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_static_data_lifecycle(pool: PgPool) {
        let workflow_repo = WorkflowRepository::new(pool.clone());
        let static_repo = StaticDataRepository::new(pool);

        let workflow = workflow_repo.create("Test Flow", json!([]), json!({}), json!({})).await.unwrap();
        let node_id = "PollingNode_123";

        // Initial fetch should be None
        let initial = static_repo.find_by_node_and_workflow(node_id, workflow.id).await.unwrap();
        assert!(initial.is_none());

        // Upsert 1
        let payload_1 = json!({ "last_poll_timestamp": "2024-01-01T00:00:00Z" });
        let inserted = static_repo.upsert(node_id, workflow.id, payload_1.clone()).await.unwrap();
        assert_eq!(inserted.node_id, node_id);
        assert_eq!(inserted.workflow_id, workflow.id);
        assert_eq!(inserted.data, payload_1);

        // Fetch
        let found = static_repo.find_by_node_and_workflow(node_id, workflow.id).await.unwrap().unwrap();
        assert_eq!(found.data, payload_1);

        // Upsert 2 (Update)
        let payload_2 = json!({ "last_poll_timestamp": "2024-01-02T00:00:00Z" });
        let updated = static_repo.upsert(node_id, workflow.id, payload_2.clone()).await.unwrap();
        
        // Assert it updated the existing row (IDs should match)
        assert_eq!(updated.id, inserted.id);
        assert_eq!(updated.data, payload_2);

        // Try a different node on the same workflow
        let payload_3 = json!({ "last_poll_timestamp": "2024-01-03T00:00:00Z" });
        let new_node = static_repo.upsert("DifferentNode_456", workflow.id, payload_3).await.unwrap();
        assert_ne!(new_node.id, inserted.id);
    }
}
