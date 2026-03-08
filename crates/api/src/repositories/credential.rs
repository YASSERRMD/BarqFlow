use crate::crypto::CryptoService;
use barqflow_db::models::CredentialEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct CredentialRepository {
    pool: PgPool,
    crypto: CryptoService,
}

impl CredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        let crypto = CryptoService::new().unwrap_or_else(|e| {
            // Panic if crypto fails to load in production, or handle properly.
            // For BarqFlow, panicking on invalid encryption key prevents corrupted writes.
            panic!("Failed to initialize CryptoService: {}", e);
        });
        Self { pool, crypto }
    }

    pub async fn find_all(&self) -> Result<Vec<CredentialEntity>> {
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
            self.decrypt_entity_data(e);
        }

        Ok(entities)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<CredentialEntity>> {
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
            self.decrypt_entity_data(e);
        }

        Ok(entity_opt)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(
            r#"
            SELECT id, name, cred_type, data, created_at, updated_at
            FROM credentials
            WHERE name = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut e) = entity_opt {
            self.decrypt_entity_data(e);
        }

        Ok(entity_opt)
    }

    pub async fn find_latest_by_type(&self, cred_type: &str) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(
            r#"
            SELECT id, name, cred_type, data, created_at, updated_at
            FROM credentials
            WHERE cred_type = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(cred_type)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut e) = entity_opt {
            self.decrypt_entity_data(e);
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
        let encrypted_base64 = self
            .crypto
            .encrypt_value(&data)
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

        let encrypted_base64 = self
            .crypto
            .encrypt_value(&data)
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

    fn decrypt_entity_data(&self, entity: &mut CredentialEntity) {
        if let Some(enc) = entity.data.get("encrypted").and_then(|v| v.as_str()) {
            if let Ok(dec) = self.crypto.decrypt_value(enc) {
                entity.data = dec;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::env;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_credential_lifecycle(pool: PgPool) {
        env::set_var("BARQFLOW_ENCRYPTION_KEY", "test_key_must_be_exactly_32_byte");
        let repo = CredentialRepository::new(pool.clone());

        let secret_payload = json!({ "api_token" : "super-secret-123", "domain" : "api.acme.com" });

        // CREATE
        let created = repo.create("My ACME Creds", "acmeApi", secret_payload.clone()).await.unwrap();
        assert_eq!(created.name, "My ACME Creds");
        assert_eq!(created.cred_type, "acmeApi");

        // The returned entity from create/update might still be encrypted because we didn't explicitly decrypt returning clauses,
        // Wait, the `create` method returns the RAW inserted entity! So created.data has `{"encrypted":"..."}`.
        let is_encrypted = created.data.get("encrypted").is_some();
        assert!(is_encrypted);
        assert!(created.data.get("api_token").is_none());

        // READ (which decrypts automatically)
        let found = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(found.data, secret_payload);

        // UPDATE
        let updated_payload = json!({ "api_token" : "new-token-456" });
        repo.update(created.id, "Renamed ACME Creds", updated_payload.clone()).await.unwrap();

        let refound = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(refound.name, "Renamed ACME Creds");
        assert_eq!(refound.data, updated_payload);

        // DELETE
        repo.delete(created.id).await.unwrap();
        let deleted = repo.find_by_id(created.id).await.unwrap();
        assert!(deleted.is_none());
    }
}
