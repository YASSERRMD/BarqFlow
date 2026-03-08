use crate::repositories::credential::CredentialRepository;
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::types::GenericValue;
use barqflow_exec::context::CredentialProvider;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct RepositoryCredentialProvider {
    repo: Arc<CredentialRepository>,
}

impl RepositoryCredentialProvider {
    pub fn new(repo: Arc<CredentialRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl CredentialProvider for RepositoryCredentialProvider {
    async fn get_credentials(
        &self,
        name: &str,
    ) -> Result<HashMap<String, GenericValue>, BarqError> {
        let by_type = self
            .repo
            .find_latest_by_type(name)
            .await
            .map_err(|e| BarqError::NodeOperationError {
                node_name: "CredentialResolver".to_string(),
                message: format!("Credential lookup failed for type '{}': {}", name, e),
            })?;

        let by_name = if by_type.is_none() {
            self.repo
                .find_by_name(name)
                .await
                .map_err(|e| BarqError::NodeOperationError {
                    node_name: "CredentialResolver".to_string(),
                    message: format!("Credential lookup failed for name '{}': {}", name, e),
                })?
        } else {
            None
        };

        let credential = by_type.or(by_name).ok_or_else(|| BarqError::NodeOperationError {
                node_name: "CredentialResolver".to_string(),
                message: format!("No credential found for reference '{}'", name),
            })?;

        let object = credential.data.as_object().ok_or_else(|| BarqError::NodeOperationError {
            node_name: "CredentialResolver".to_string(),
            message: format!(
                "Credential '{}' resolved to non-object payload and cannot be used",
                credential.name
            ),
        })?;

        Ok(object
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_repository_provider_resolves_credentials_by_type(pool: PgPool) {
        std::env::set_var("BARQFLOW_ENCRYPTION_KEY", "12345678901234567890123456789012");

        let repo = Arc::new(CredentialRepository::new(pool));
        repo.create(
            "OpenAI Prod",
            "openAiApi",
            serde_json::json!({
                "apiKey": "sk-test-123",
                "baseUrl": "https://api.openai.com/v1"
            }),
        )
        .await
        .unwrap();

        let provider = RepositoryCredentialProvider::new(Arc::clone(&repo));
        let creds = provider.get_credentials("openAiApi").await.unwrap();

        assert_eq!(creds.get("apiKey").and_then(|v| v.as_str()), Some("sk-test-123"));
    }
}
