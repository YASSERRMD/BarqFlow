use crate::models::CredentialEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct CredentialRepo {
    pool: PgPool,
}

impl CredentialRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> Result<Vec<CredentialEntity>> {
        sqlx::query_as::<_, CredentialEntity>(
            r#"
            SELECT id, name, cred_type, data, created_at, updated_at
            FROM credentials
            ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<CredentialEntity>> {
        sqlx::query_as::<_, CredentialEntity>(
            r#"
            SELECT id, name, cred_type, data, created_at, updated_at
            FROM credentials
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        name: &str,
        cred_type: &str,
        data: serde_json::Value,
    ) -> Result<CredentialEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query_as::<_, CredentialEntity>(
            r#"
            INSERT INTO credentials (id, name, cred_type, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, cred_type, data, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(name)
        .bind(cred_type)
        .bind(data)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, name: &str, data: serde_json::Value) -> Result<Option<CredentialEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, CredentialEntity>(
            r#"
            UPDATE credentials
            SET name = $1, data = $2, updated_at = $3
            WHERE id = $4
            RETURNING id, name, cred_type, data, created_at, updated_at
            "#
        )
        .bind(name)
        .bind(data)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM credentials
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
