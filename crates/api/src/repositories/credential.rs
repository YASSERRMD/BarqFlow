use crate::crypto::CryptoService;
use barqflow_db::models::CredentialEntity;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Result, Transaction};
use uuid::Uuid;

pub struct CredentialRepository {
    pool: PgPool,
    crypto: CryptoService,
}

const CREDENTIAL_COLUMNS: &str = r#"
    id,
    workspace_id,
    owner_user_id,
    name,
    cred_type,
    data,
    created_at,
    updated_at,
    last_tested_at,
    last_test_status,
    last_test_message,
    last_used_at,
    usage_count,
    rotated_at
"#;

impl CredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        let crypto = CryptoService::new().unwrap_or_else(|e| {
            panic!("Failed to initialize CryptoService: {}", e);
        });
        Self { pool, crypto }
    }

    pub async fn find_all(&self) -> Result<Vec<CredentialEntity>> {
        let mut entities = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            ORDER BY name ASC
            "#
        ))
        .fetch_all(&self.pool)
        .await?;

        for entity in &mut entities {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entities)
    }

    pub async fn find_all_in_workspace(&self, workspace_id: Uuid) -> Result<Vec<CredentialEntity>> {
        let mut entities = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE workspace_id = $1
            ORDER BY name ASC
            "#
        ))
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;

        for entity in &mut entities {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entities)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE id = $1
            "#
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut entity) = entity_opt {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entity_opt)
    }

    pub async fn find_by_id_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE id = $1 AND workspace_id = $2
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut entity) = entity_opt {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entity_opt)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE name = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#
        ))
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut entity) = entity_opt {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entity_opt)
    }

    pub async fn find_by_name_in_workspace(
        &self,
        workspace_id: Uuid,
        name: &str,
    ) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE workspace_id = $1 AND name = $2
            ORDER BY updated_at DESC
            LIMIT 1
            "#
        ))
        .bind(workspace_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut entity) = entity_opt {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entity_opt)
    }

    pub async fn find_by_type(&self, cred_type: &str) -> Result<Vec<CredentialEntity>> {
        let mut entities = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE cred_type = $1
            ORDER BY name ASC
            "#
        ))
        .bind(cred_type)
        .fetch_all(&self.pool)
        .await?;

        for entity in &mut entities {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entities)
    }

    pub async fn find_by_type_in_workspace(
        &self,
        workspace_id: Uuid,
        cred_type: &str,
    ) -> Result<Vec<CredentialEntity>> {
        let mut entities = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE workspace_id = $1 AND cred_type = $2
            ORDER BY name ASC
            "#
        ))
        .bind(workspace_id)
        .bind(cred_type)
        .fetch_all(&self.pool)
        .await?;

        for entity in &mut entities {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entities)
    }

    pub async fn find_latest_by_type(&self, cred_type: &str) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE cred_type = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#
        ))
        .bind(cred_type)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut entity) = entity_opt {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entity_opt)
    }

    pub async fn find_latest_by_type_in_workspace(
        &self,
        workspace_id: Uuid,
        cred_type: &str,
    ) -> Result<Option<CredentialEntity>> {
        let mut entity_opt = sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            SELECT {CREDENTIAL_COLUMNS}
            FROM credentials
            WHERE workspace_id = $1 AND cred_type = $2
            ORDER BY updated_at DESC
            LIMIT 1
            "#
        ))
        .bind(workspace_id)
        .bind(cred_type)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut entity) = entity_opt {
            self.decrypt_entity_data(entity)?;
        }

        Ok(entity_opt)
    }

    pub async fn create(
        &self,
        name: &str,
        cred_type: &str,
        data: serde_json::Value,
    ) -> Result<CredentialEntity> {
        let mut tx = self.pool.begin().await?;
        let (workspace_id, owner_user_id) = self.resolve_workspace_context_tx(&mut tx).await?;
        let entity = self
            .create_in_tx(&mut tx, workspace_id, owner_user_id, name, cred_type, data)
            .await?;
        tx.commit().await?;
        Ok(entity)
    }

    pub async fn create_in_workspace(
        &self,
        workspace_id: Uuid,
        owner_user_id: Option<Uuid>,
        name: &str,
        cred_type: &str,
        data: serde_json::Value,
    ) -> Result<CredentialEntity> {
        let mut tx = self.pool.begin().await?;
        let entity = self
            .create_in_tx(&mut tx, workspace_id, owner_user_id, name, cred_type, data)
            .await?;
        tx.commit().await?;
        Ok(entity)
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: &str,
        data: serde_json::Value,
    ) -> Result<Option<CredentialEntity>> {
        self.update_scoped(None, id, name, data).await
    }

    pub async fn update_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        name: &str,
        data: serde_json::Value,
    ) -> Result<Option<CredentialEntity>> {
        self.update_scoped(Some(workspace_id), id, name, data).await
    }

    pub async fn rotate(
        &self,
        id: Uuid,
        name: &str,
        data: serde_json::Value,
    ) -> Result<Option<CredentialEntity>> {
        self.rotate_scoped(None, id, name, data).await
    }

    pub async fn rotate_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        name: &str,
        data: serde_json::Value,
    ) -> Result<Option<CredentialEntity>> {
        self.rotate_scoped(Some(workspace_id), id, name, data).await
    }

    pub async fn record_test_result(
        &self,
        id: Uuid,
        status: &str,
        message: Option<&str>,
    ) -> Result<Option<CredentialEntity>> {
        self.record_test_result_scoped(None, id, status, message)
            .await
    }

    pub async fn record_test_result_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: &str,
        message: Option<&str>,
    ) -> Result<Option<CredentialEntity>> {
        self.record_test_result_scoped(Some(workspace_id), id, status, message)
            .await
    }

    pub async fn clear_test_result(&self, id: Uuid) -> Result<Option<CredentialEntity>> {
        self.clear_test_result_scoped(None, id).await
    }

    pub async fn clear_test_result_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<CredentialEntity>> {
        self.clear_test_result_scoped(Some(workspace_id), id).await
    }

    pub async fn record_usage(&self, id: Uuid) -> Result<bool> {
        self.record_usage_scoped(None, id).await
    }

    pub async fn record_usage_in_workspace(&self, workspace_id: Uuid, id: Uuid) -> Result<bool> {
        self.record_usage_scoped(Some(workspace_id), id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        self.delete_scoped(None, id).await
    }

    pub async fn delete_in_workspace(&self, workspace_id: Uuid, id: Uuid) -> Result<bool> {
        self.delete_scoped(Some(workspace_id), id).await
    }

    async fn create_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: Uuid,
        owner_user_id: Option<Uuid>,
        name: &str,
        cred_type: &str,
        data: serde_json::Value,
    ) -> Result<CredentialEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let encrypted_data = self.encrypt_data(data)?;

        sqlx::query_as::<_, CredentialEntity>(&format!(
            r#"
            INSERT INTO credentials (
                id,
                workspace_id,
                owner_user_id,
                name,
                cred_type,
                data,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING {CREDENTIAL_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .bind(owner_user_id)
        .bind(name)
        .bind(cred_type)
        .bind(encrypted_data)
        .bind(now)
        .bind(now)
        .fetch_one(&mut **tx)
        .await
    }

    async fn update_scoped(
        &self,
        workspace_id: Option<Uuid>,
        id: Uuid,
        name: &str,
        data: serde_json::Value,
    ) -> Result<Option<CredentialEntity>> {
        let now = Utc::now();
        let encrypted_data = self.encrypt_data(data)?;
        let sql = if workspace_id.is_some() {
            format!(
                r#"
                UPDATE credentials
                SET name = $1, data = $2, updated_at = $3
                WHERE id = $4 AND workspace_id = $5
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        } else {
            format!(
                r#"
                UPDATE credentials
                SET name = $1, data = $2, updated_at = $3
                WHERE id = $4
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        };

        let mut query = sqlx::query_as::<_, CredentialEntity>(&sql)
            .bind(name)
            .bind(encrypted_data)
            .bind(now)
            .bind(id);

        if let Some(workspace_id) = workspace_id {
            query = query.bind(workspace_id);
        }

        query.fetch_optional(&self.pool).await
    }

    async fn rotate_scoped(
        &self,
        workspace_id: Option<Uuid>,
        id: Uuid,
        name: &str,
        data: serde_json::Value,
    ) -> Result<Option<CredentialEntity>> {
        let now = Utc::now();
        let encrypted_data = self.encrypt_data(data)?;
        let sql = if workspace_id.is_some() {
            format!(
                r#"
                UPDATE credentials
                SET
                    name = $1,
                    data = $2,
                    updated_at = $3,
                    rotated_at = $3,
                    last_tested_at = NULL,
                    last_test_status = NULL,
                    last_test_message = NULL
                WHERE id = $4 AND workspace_id = $5
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        } else {
            format!(
                r#"
                UPDATE credentials
                SET
                    name = $1,
                    data = $2,
                    updated_at = $3,
                    rotated_at = $3,
                    last_tested_at = NULL,
                    last_test_status = NULL,
                    last_test_message = NULL
                WHERE id = $4
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        };

        let mut query = sqlx::query_as::<_, CredentialEntity>(&sql)
            .bind(name)
            .bind(encrypted_data)
            .bind(now)
            .bind(id);

        if let Some(workspace_id) = workspace_id {
            query = query.bind(workspace_id);
        }

        query.fetch_optional(&self.pool).await
    }

    async fn record_test_result_scoped(
        &self,
        workspace_id: Option<Uuid>,
        id: Uuid,
        status: &str,
        message: Option<&str>,
    ) -> Result<Option<CredentialEntity>> {
        let now = Utc::now();
        let sql = if workspace_id.is_some() {
            format!(
                r#"
                UPDATE credentials
                SET
                    last_tested_at = $1,
                    last_test_status = $2,
                    last_test_message = $3
                WHERE id = $4 AND workspace_id = $5
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        } else {
            format!(
                r#"
                UPDATE credentials
                SET
                    last_tested_at = $1,
                    last_test_status = $2,
                    last_test_message = $3
                WHERE id = $4
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        };

        let mut query = sqlx::query_as::<_, CredentialEntity>(&sql)
            .bind(now)
            .bind(status)
            .bind(message)
            .bind(id);

        if let Some(workspace_id) = workspace_id {
            query = query.bind(workspace_id);
        }

        query.fetch_optional(&self.pool).await
    }

    async fn clear_test_result_scoped(
        &self,
        workspace_id: Option<Uuid>,
        id: Uuid,
    ) -> Result<Option<CredentialEntity>> {
        let sql = if workspace_id.is_some() {
            format!(
                r#"
                UPDATE credentials
                SET
                    last_tested_at = NULL,
                    last_test_status = NULL,
                    last_test_message = NULL
                WHERE id = $1 AND workspace_id = $2
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        } else {
            format!(
                r#"
                UPDATE credentials
                SET
                    last_tested_at = NULL,
                    last_test_status = NULL,
                    last_test_message = NULL
                WHERE id = $1
                RETURNING {CREDENTIAL_COLUMNS}
                "#
            )
        };

        let mut query = sqlx::query_as::<_, CredentialEntity>(&sql).bind(id);
        if let Some(workspace_id) = workspace_id {
            query = query.bind(workspace_id);
        }

        query.fetch_optional(&self.pool).await
    }

    async fn record_usage_scoped(&self, workspace_id: Option<Uuid>, id: Uuid) -> Result<bool> {
        let now = Utc::now();
        let sql = if workspace_id.is_some() {
            r#"
            UPDATE credentials
            SET
                last_used_at = $1,
                usage_count = usage_count + 1
            WHERE id = $2 AND workspace_id = $3
            "#
        } else {
            r#"
            UPDATE credentials
            SET
                last_used_at = $1,
                usage_count = usage_count + 1
            WHERE id = $2
            "#
        };

        let mut query = sqlx::query(sql).bind(now).bind(id);
        if let Some(workspace_id) = workspace_id {
            query = query.bind(workspace_id);
        }

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_scoped(&self, workspace_id: Option<Uuid>, id: Uuid) -> Result<bool> {
        let sql = if workspace_id.is_some() {
            "DELETE FROM credentials WHERE id = $1 AND workspace_id = $2"
        } else {
            "DELETE FROM credentials WHERE id = $1"
        };

        let mut query = sqlx::query(sql).bind(id);
        if let Some(workspace_id) = workspace_id {
            query = query.bind(workspace_id);
        }

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    fn encrypt_data(&self, data: serde_json::Value) -> Result<serde_json::Value> {
        let encrypted_base64 = self
            .crypto
            .encrypt_value(&data)
            .map_err(|e| sqlx::Error::Protocol(format!("Encryption error: {}", e)))?;
        Ok(serde_json::json!({ "encrypted": encrypted_base64 }))
    }

    fn decrypt_entity_data(&self, entity: &mut CredentialEntity) -> sqlx::Result<()> {
        if let Some(enc) = entity.data.get("encrypted").and_then(|v| v.as_str()) {
            let dec = self.crypto.decrypt_value(enc).map_err(|e| {
                sqlx::Error::Protocol(format!(
                    "Decryption failed for credential {}: {}",
                    entity.id, e
                ))
            })?;
            entity.data = dec;
        }
        Ok(())
    }

    async fn resolve_workspace_context_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(Uuid, Option<Uuid>)> {
        let existing = sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
            r#"
            SELECT id, created_by_user_id
            FROM workspaces
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(context) = existing {
            return Ok(context);
        }

        let workspace_id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO workspaces (id, name, slug, created_by_user_id, created_at, updated_at)
            VALUES ($1, 'Imported Workspace', 'imported-workspace', NULL, $2, $3)
            "#,
        )
        .bind(workspace_id)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        Ok((workspace_id, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::env;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_credential_lifecycle(pool: PgPool) {
        env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "test_key_must_be_exactly_32_byte",
        );
        let repo = CredentialRepository::new(pool.clone());

        let secret_payload = json!({ "api_token" : "super-secret-123", "domain" : "api.acme.com" });

        let created = repo
            .create("My ACME Creds", "acmeApi", secret_payload.clone())
            .await
            .unwrap();
        assert_eq!(created.name, "My ACME Creds");
        assert_eq!(created.cred_type, "acmeApi");
        assert_ne!(created.workspace_id, Uuid::nil());

        let is_encrypted = created.data.get("encrypted").is_some();
        assert!(is_encrypted);
        assert!(created.data.get("api_token").is_none());

        let found = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(found.data, secret_payload);

        repo.update(
            created.id,
            "Renamed ACME Creds",
            json!({ "api_token" : "new-token-456" }),
        )
        .await
        .unwrap();

        let refound = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(refound.name, "Renamed ACME Creds");
        assert_eq!(refound.data, json!({ "api_token" : "new-token-456" }));
        assert_eq!(refound.usage_count, 0);
        assert!(refound.last_tested_at.is_none());

        repo.delete(created.id).await.unwrap();
        let deleted = repo.find_by_id(created.id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_credential_usage_and_test_metadata(pool: PgPool) {
        env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "test_key_must_be_exactly_32_byte",
        );
        let repo = CredentialRepository::new(pool.clone());

        let created = repo
            .create(
                "Usage Creds",
                "openAiApi",
                json!({ "apiKey": "sk-test", "baseUrl": "https://api.openai.com/v1" }),
            )
            .await
            .unwrap();

        let used = repo.record_usage(created.id).await.unwrap();
        assert!(used);

        let tested = repo
            .record_test_result(created.id, "valid", Some("Credential validated"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(tested.last_test_status.as_deref(), Some("valid"));

        let reloaded = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(reloaded.usage_count, 1);
        assert!(reloaded.last_used_at.is_some());
        assert!(reloaded.last_tested_at.is_some());
        assert_eq!(reloaded.last_test_status.as_deref(), Some("valid"));
        assert_eq!(
            reloaded.last_test_message.as_deref(),
            Some("Credential validated")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_decryption_failure_propagates_error(pool: PgPool) {
        env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "test_key_must_be_exactly_32_byte",
        );
        let repo = CredentialRepository::new(pool.clone());

        let created = repo
            .create("Tamper Test", "testApi", json!({ "secret": "value" }))
            .await
            .unwrap();

        // Overwrite the encrypted payload with garbage so decryption must fail.
        sqlx::query("UPDATE credentials SET data = $1 WHERE id = $2")
            .bind(json!({ "encrypted": "not_valid_base64:not_valid_cipher" }))
            .bind(created.id)
            .execute(&pool)
            .await
            .unwrap();

        let result = repo.find_by_id(created.id).await;
        assert!(
            result.is_err(),
            "Expected decryption failure to propagate as Err"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Decryption failed"),
            "Error should mention decryption: {}",
            err_msg
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_rotate_clears_test_metadata_and_sets_rotated_at(pool: PgPool) {
        env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "test_key_must_be_exactly_32_byte",
        );
        let repo = CredentialRepository::new(pool.clone());

        let created = repo
            .create("Rotate Me", "openAiApi", json!({ "apiKey": "sk-old" }))
            .await
            .unwrap();

        repo.record_test_result(created.id, "valid", Some("before rotation"))
            .await
            .unwrap();

        repo.rotate(
            created.id,
            "Rotate Me",
            json!({ "apiKey": "sk-new", "baseUrl": "https://api.openai.com/v1" }),
        )
        .await
        .unwrap();

        let reloaded = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(reloaded.data["apiKey"], "sk-new");
        assert!(reloaded.rotated_at.is_some());
        assert!(reloaded.last_tested_at.is_none());
        assert!(reloaded.last_test_status.is_none());
        assert!(reloaded.last_test_message.is_none());
    }
}
