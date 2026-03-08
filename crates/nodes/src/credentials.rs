use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::properties::{
    AuthenticateRequestProperties, ICredentialProperties, INodeProperty, NodePropertyType,
};
use barqflow_core::traits::ICredentialType;
use barqflow_core::types::GenericValue;
use barqflow_registry::registry::{CredentialInfo, CredentialRegistry};
use std::collections::HashMap;
use std::sync::Arc;

pub struct OpenAiApiCredential;

#[async_trait]
impl ICredentialType for OpenAiApiCredential {
    fn get_description(&self) -> ICredentialProperties {
        ICredentialProperties {
            name: "openAiApi".to_string(),
            display_name: "OpenAI API".to_string(),
            notice: Some("Used by OpenAI integration nodes".to_string()),
            properties: vec![
                INodeProperty {
                    display_name: "API Key".to_string(),
                    name: "apiKey".to_string(),
                    r#type: NodePropertyType::String,
                    default: None,
                    description: Some("Secret key from your OpenAI account".to_string()),
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "Base URL".to_string(),
                    name: "baseUrl".to_string(),
                    r#type: NodePropertyType::String,
                    default: Some(serde_json::json!("https://api.openai.com/v1")),
                    description: Some("Optional custom API base URL".to_string()),
                    hint: None,
                    required: false,
                    options: None,
                    display_options: None,
                },
            ],
            documentation_url: Some("https://platform.openai.com/docs/quickstart".to_string()),
            authenticate: Some(AuthenticateRequestProperties {
                r#in: "header".to_string(),
                name: "Authorization".to_string(),
                value: "Bearer ={{$credentials.apiKey}}".to_string(),
            }),
        }
    }

    async fn test_credential(
        &self,
        credential_data: &HashMap<String, GenericValue>,
    ) -> Result<bool, BarqError> {
        Ok(credential_data
            .get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false))
    }
}

pub struct PostgresApiCredential;

#[async_trait]
impl ICredentialType for PostgresApiCredential {
    fn get_description(&self) -> ICredentialProperties {
        ICredentialProperties {
            name: "postgresApi".to_string(),
            display_name: "PostgreSQL".to_string(),
            notice: Some("Used by PostgreSQL integration nodes".to_string()),
            properties: vec![
                INodeProperty {
                    display_name: "Host".to_string(),
                    name: "host".to_string(),
                    r#type: NodePropertyType::String,
                    default: Some(serde_json::json!("localhost")),
                    description: None,
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "Port".to_string(),
                    name: "port".to_string(),
                    r#type: NodePropertyType::Number,
                    default: Some(serde_json::json!(5432)),
                    description: None,
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "Database".to_string(),
                    name: "database".to_string(),
                    r#type: NodePropertyType::String,
                    default: Some(serde_json::json!("postgres")),
                    description: None,
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "User".to_string(),
                    name: "user".to_string(),
                    r#type: NodePropertyType::String,
                    default: Some(serde_json::json!("postgres")),
                    description: None,
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "Password".to_string(),
                    name: "password".to_string(),
                    r#type: NodePropertyType::String,
                    default: None,
                    description: None,
                    hint: None,
                    required: false,
                    options: None,
                    display_options: None,
                },
            ],
            documentation_url: None,
            authenticate: None,
        }
    }

    async fn test_credential(
        &self,
        credential_data: &HashMap<String, GenericValue>,
    ) -> Result<bool, BarqError> {
        let host_ok = credential_data
            .get("host")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let db_ok = credential_data
            .get("database")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let user_ok = credential_data
            .get("user")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);

        Ok(host_ok && db_ok && user_ok)
    }
}

pub fn register_all_credentials(registry: &CredentialRegistry) {
    let _ = registry.register_credential(CredentialInfo {
        name: "openAiApi".to_string(),
        cred_impl: Arc::new(OpenAiApiCredential),
    });

    let _ = registry.register_credential(CredentialInfo {
        name: "postgresApi".to_string(),
        cred_impl: Arc::new(PostgresApiCredential),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all_credentials_registers_core_types() {
        let registry = CredentialRegistry::new();
        register_all_credentials(&registry);

        assert!(registry.get_credential("openAiApi").is_some());
        assert!(registry.get_credential("postgresApi").is_some());
    }
}
