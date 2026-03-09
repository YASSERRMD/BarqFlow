use crate::repositories::credential::CredentialRepository;
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INode;
use barqflow_core::types::GenericValue;
use barqflow_exec::context::CredentialProvider;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct RepositoryCredentialProvider {
    repo: Arc<CredentialRepository>,
    node_bindings: HashMap<String, HashMap<String, Uuid>>,
}

impl RepositoryCredentialProvider {
    pub fn new(repo: Arc<CredentialRepository>, nodes: &[INode]) -> Self {
        let mut node_bindings: HashMap<String, HashMap<String, Uuid>> = HashMap::new();
        for node in nodes {
            let mut bindings = HashMap::new();
            for binding in &node.credentials {
                bindings.insert(binding.credential_type.clone(), binding.credential_id);
            }
            node_bindings.insert(node.id.0.clone(), bindings);
        }

        Self {
            repo,
            node_bindings,
        }
    }

    pub async fn resolve_for_node(
        &self,
        node_id: &str,
        credential_type: &str,
    ) -> Result<HashMap<String, GenericValue>, BarqError> {
        let credential_id = self
            .node_bindings
            .get(node_id)
            .and_then(|bindings| bindings.get(credential_type))
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: node_id.to_string(),
                message: format!(
                    "Credential binding missing for type '{}'. Open the node and select a credential.",
                    credential_type
                ),
            })?;

        let credential = self
            .repo
            .find_by_id(*credential_id)
            .await
            .map_err(|e| BarqError::NodeOperationError {
                node_name: node_id.to_string(),
                message: format!("Credential lookup failed for id '{}': {}", credential_id, e),
            })?
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: node_id.to_string(),
                message: format!(
                    "Bound credential '{}' was not found. Open /credentials and reselect it in the node.",
                    credential_id
                ),
            })?;

        if credential.cred_type != credential_type {
            return Err(BarqError::NodeOperationError {
                node_name: node_id.to_string(),
                message: format!(
                    "Credential type mismatch: node expects '{}', but selected credential is '{}'.",
                    credential_type, credential.cred_type
                ),
            });
        }

        let object = credential
            .data
            .as_object()
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: node_id.to_string(),
                message: format!(
                    "Credential '{}' resolved to non-object payload and cannot be used",
                    credential.name
                ),
            })?;

        Ok(object.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }
}

#[async_trait]
impl CredentialProvider for RepositoryCredentialProvider {
    async fn get_credentials(
        &self,
        node_id: &str,
        credential_type: &str,
    ) -> Result<HashMap<String, GenericValue>, BarqError> {
        self.resolve_for_node(node_id, credential_type).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::schema::{INode, INodeParameters, NodeCredentialBinding};
    use barqflow_core::types::NodeId;
    use sqlx::PgPool;

    fn node_with_binding(node_id: &str, credential_type: &str, credential_id: Uuid) -> INode {
        INode {
            id: NodeId::new(node_id),
            name: "Test Node".to_string(),
            r#type: "barqflow-nodes.openai".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![NodeCredentialBinding {
                node_id: node_id.to_string(),
                credential_type: credential_type.to_string(),
                credential_id,
            }],
            disabled: false,
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_repository_provider_resolves_credentials_for_bound_node(pool: PgPool) {
        std::env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "12345678901234567890123456789012",
        );

        let repo = Arc::new(CredentialRepository::new(pool));
        let credential = repo
            .create(
                "OpenAI Prod",
                "openAiApi",
                serde_json::json!({
                    "apiKey": "sk-test-123",
                    "baseUrl": "https://api.openai.com/v1"
                }),
            )
            .await
            .unwrap();

        let nodes = vec![node_with_binding("node-1", "openAiApi", credential.id)];
        let provider = RepositoryCredentialProvider::new(Arc::clone(&repo), &nodes);
        let creds = provider
            .get_credentials("node-1", "openAiApi")
            .await
            .unwrap();

        assert_eq!(
            creds.get("apiKey").and_then(|v| v.as_str()),
            Some("sk-test-123")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_repository_provider_returns_not_bound_when_missing(pool: PgPool) {
        std::env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "12345678901234567890123456789012",
        );

        let repo = Arc::new(CredentialRepository::new(pool));
        let nodes = vec![INode {
            id: NodeId::new("node-2"),
            name: "No Binding".to_string(),
            r#type: "barqflow-nodes.openai".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![],
            disabled: false,
        }];

        let provider = RepositoryCredentialProvider::new(Arc::clone(&repo), &nodes);
        let err = provider
            .get_credentials("node-2", "openAiApi")
            .await
            .unwrap_err();

        assert!(err
            .to_string()
            .contains("Credential binding missing for type 'openAiApi'"));
    }
}
