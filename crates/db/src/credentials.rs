use crate::crypto::CryptoService;
use crate::models::CredentialEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct CredentialRepo {
    pool: PgPool,
    crypto: CryptoService,
}

impl CredentialRepo {
    pub fn new(pool: PgPool) -> Self {
        let crypto = CryptoService::new().unwrap_or_else(|e| {
            // Panic if crypto fails to load in production, or handle properly.
            // For BarqFlow, panicking on invalid encryption key prevents corrupted writes.
            panic!("Failed to initialize CryptoService: {}", e);
        });
        Self { pool, crypto }
    }

    pub async fn get_all(&self) -> Result<Vec<CredentialEntity>> {
        let mut entities = sqlx::query_as::<_, CredentialEntity>(
            r#"
            SELECT id, name, cred_type, data, created_at, updated_at
            FROM credentials
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for e in &mut entities {
            if let Some(enc) = e.data.get("encrypted").and_then(|v| v.as_str()) {
                if let Ok(dec) = self.crypto.decrypt(enc) {
                    if let Ok(json_val) = serde_json::from_str(&dec) {
                        e.data = json_val;
                    }
                }
            }
        }

        Ok(entities)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(
            r#"
            SELECT id, name, cred_type, data, created_at, updated_at
            FROM credentials
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut e) = entity_opt {
            if let Some(enc) = e.data.get("encrypted").and_then(|v| v.as_str()) {
                if let Ok(dec) = self.crypto.decrypt(enc) {
                    if let Ok(json_val) = serde_json::from_str(&dec) {
                        e.data = json_val;
                    }
                }
            }
        }

        Ok(entity_opt)
    }

    pub async fn create(
        &self,
        name: &str,
        cred_type: &str,
        data: serde_json::Value,
    ) -> Result<CredentialEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Encrypt the sensitive JSON data into a base64 string
        let plain_json = data.to_string();
        let encrypted_base64 = self
            .crypto
            .encrypt(&plain_json)
            .map_err(|e| sqlx::Error::Protocol(format!("Encryption error: {}", e)))?;
        let encrypted_data = serde_json::json!({ "encrypted": encrypted_base64 });

        sqlx::query_as::<_, CredentialEntity>(
            r#"
            INSERT INTO credentials (id, name, cred_type, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, cred_type, data, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(cred_type)
        .bind(encrypted_data)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: &str,
        data: serde_json::Value,
    ) -> Result<Option<CredentialEntity>> {
        let now = Utc::now();

        let plain_json = data.to_string();
        let encrypted_base64 = self
            .crypto
            .encrypt(&plain_json)
            .map_err(|e| sqlx::Error::Protocol(format!("Encryption error: {}", e)))?;
        let encrypted_data = serde_json::json!({ "encrypted": encrypted_base64 });

        sqlx::query_as::<_, CredentialEntity>(
            r#"
            UPDATE credentials
            SET name = $1, data = $2, updated_at = $3
            WHERE id = $4
            RETURNING id, name, cred_type, data, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(encrypted_data)
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
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
