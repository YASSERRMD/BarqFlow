use barqflow_db::models::ApiKeyEntity;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub const API_KEY_TOKEN_PREFIX: &str = "bf_api";

pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_workspace(&self, workspace_id: Uuid) -> Result<Vec<ApiKeyEntity>> {
        sqlx::query_as::<_, ApiKeyEntity>(
            r#"
            SELECT
                id,
                workspace_id,
                user_id,
                name,
                key_prefix,
                key_hash,
                last_used_at,
                expires_at,
                revoked_at,
                created_at,
                updated_at
            FROM api_keys
            WHERE workspace_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        name: &str,
        key_prefix: &str,
        key_hash: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ApiKeyEntity> {
        let now = Utc::now();
        sqlx::query_as::<_, ApiKeyEntity>(
            r#"
            INSERT INTO api_keys (
                id,
                workspace_id,
                user_id,
                name,
                key_prefix,
                key_hash,
                last_used_at,
                expires_at,
                revoked_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, NULL, $8, $9)
            RETURNING
                id,
                workspace_id,
                user_id,
                name,
                key_prefix,
                key_hash,
                last_used_at,
                expires_at,
                revoked_at,
                created_at,
                updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(user_id)
        .bind(name)
        .bind(key_prefix)
        .bind(key_hash)
        .bind(expires_at)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKeyEntity>> {
        sqlx::query_as::<_, ApiKeyEntity>(
            r#"
            SELECT
                id,
                workspace_id,
                user_id,
                name,
                key_prefix,
                key_hash,
                last_used_at,
                expires_at,
                revoked_at,
                created_at,
                updated_at
            FROM api_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn record_usage(&self, id: Uuid) -> Result<bool> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = $1, updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn revoke(&self, workspace_id: Uuid, id: Uuid) -> Result<bool> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET revoked_at = $1, updated_at = $1
            WHERE id = $2 AND workspace_id = $3 AND revoked_at IS NULL
            "#,
        )
        .bind(now)
        .bind(id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

pub fn build_api_key_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

pub fn build_api_key_token(id: Uuid, secret: &str) -> String {
    format!("{}_{}_{}", API_KEY_TOKEN_PREFIX, id, secret)
}

pub fn parse_api_key_token(token: &str) -> Option<(Uuid, &str)> {
    let suffix = token.strip_prefix(&format!("{}_", API_KEY_TOKEN_PREFIX))?;
    let (id, secret) = suffix.split_once('_')?;
    let key_id = Uuid::parse_str(id).ok()?;
    if secret.trim().is_empty() {
        return None;
    }
    Some((key_id, secret))
}
