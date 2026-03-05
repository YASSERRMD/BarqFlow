use crate::models::StaticDataEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct StaticDataRepo {
    pool: PgPool,
}

impl StaticDataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_node_and_workflow(
        &self,
        node_id: &str,
        workflow_id: Uuid,
    ) -> Result<Option<StaticDataEntity>> {
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
    ) -> Result<StaticDataEntity> {
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
