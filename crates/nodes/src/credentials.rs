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

pub struct BarqDbApiCredential;

#[async_trait]
impl ICredentialType for BarqDbApiCredential {
    fn get_description(&self) -> ICredentialProperties {
        ICredentialProperties {
            name: "barqDbApi".to_string(),
            display_name: "BarqDB API".to_string(),
            notice: Some("Used by BarqDB vector integration nodes".to_string()),
            properties: vec![
                INodeProperty {
                    display_name: "Base URL".to_string(),
                    name: "baseUrl".to_string(),
                    r#type: NodePropertyType::String,
                    default: Some(serde_json::json!("http://localhost:7000")),
                    description: Some("BarqDB HTTP endpoint".to_string()),
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "API Key".to_string(),
                    name: "apiKey".to_string(),
                    r#type: NodePropertyType::String,
                    default: None,
                    description: Some("Secret key for BarqDB API".to_string()),
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
            ],
            documentation_url: None,
            authenticate: Some(AuthenticateRequestProperties {
                r#in: "header".to_string(),
                name: "x-api-key".to_string(),
                value: "={{$credentials.apiKey}}".to_string(),
            }),
        }
    }

    async fn test_credential(
        &self,
        credential_data: &HashMap<String, GenericValue>,
    ) -> Result<bool, BarqError> {
        let base_url_ok = credential_data
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let key_ok = credential_data
            .get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);

        Ok(base_url_ok && key_ok)
    }
}

pub struct TokenCredential {
    pub name: &'static str,
    pub display_name: &'static str,
    pub notice: &'static str,
    pub token_field: &'static str,
    pub token_label: &'static str,
    pub documentation_url: Option<&'static str>,
    pub authenticate_header: Option<&'static str>,
}

impl TokenCredential {
    fn authenticate_value(&self) -> String {
        format!("Bearer ={{$credentials.{}}}", self.token_field)
    }
}

#[async_trait]
impl ICredentialType for TokenCredential {
    fn get_description(&self) -> ICredentialProperties {
        ICredentialProperties {
            name: self.name.to_string(),
            display_name: self.display_name.to_string(),
            notice: Some(self.notice.to_string()),
            properties: vec![INodeProperty {
                display_name: self.token_label.to_string(),
                name: self.token_field.to_string(),
                r#type: NodePropertyType::String,
                default: None,
                description: Some(format!("Secret token used by {}", self.display_name)),
                hint: None,
                required: true,
                options: None,
                display_options: None,
            }],
            documentation_url: self.documentation_url.map(str::to_string),
            authenticate: self
                .authenticate_header
                .map(|header| AuthenticateRequestProperties {
                    r#in: "header".to_string(),
                    name: header.to_string(),
                    value: self.authenticate_value(),
                }),
        }
    }

    async fn test_credential(
        &self,
        credential_data: &HashMap<String, GenericValue>,
    ) -> Result<bool, BarqError> {
        Ok(credential_data
            .get(self.token_field)
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false))
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

    let _ = registry.register_credential(CredentialInfo {
        name: "barqDbApi".to_string(),
        cred_impl: Arc::new(BarqDbApiCredential),
    });

    let token_credentials = vec![
        TokenCredential {
            name: "slackApi",
            display_name: "Slack API",
            notice: "Used by Slack integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://api.slack.com/authentication/token-types"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "githubApi",
            display_name: "GitHub API",
            notice: "Used by GitHub integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://docs.github.com/en/rest/authentication"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "discordApi",
            display_name: "Discord API",
            notice: "Used by Discord integration nodes",
            token_field: "botToken",
            token_label: "Bot Token",
            documentation_url: Some("https://discord.com/developers/docs/topics/oauth2"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "notionApi",
            display_name: "Notion API",
            notice: "Used by Notion integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developers.notion.com/docs/authorization"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "jiraApi",
            display_name: "Jira API",
            notice: "Used by Jira integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developer.atlassian.com/cloud/jira/platform/basic-auth-for-rest-apis/",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "stripeApi",
            display_name: "Stripe API",
            notice: "Used by Stripe integration nodes",
            token_field: "apiKey",
            token_label: "API Key",
            documentation_url: Some("https://docs.stripe.com/keys"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "sendGridApi",
            display_name: "SendGrid API",
            notice: "Used by SendGrid integration nodes",
            token_field: "apiKey",
            token_label: "API Key",
            documentation_url: Some("https://docs.sendgrid.com/ui/account-and-settings/api-keys"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "hubspotApi",
            display_name: "HubSpot API",
            notice: "Used by HubSpot integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developers.hubspot.com/docs/api/private-apps"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "asanaApi",
            display_name: "Asana API",
            notice: "Used by Asana integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developers.asana.com/docs/personal-access-token"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "telegramApi",
            display_name: "Telegram Bot API",
            notice: "Used by Telegram integration nodes",
            token_field: "botToken",
            token_label: "Bot Token",
            documentation_url: Some("https://core.telegram.org/bots#how-do-i-create-a-bot"),
            authenticate_header: None,
        },
    ];

    for credential in token_credentials {
        let _ = registry.register_credential(CredentialInfo {
            name: credential.name.to_string(),
            cred_impl: Arc::new(credential),
        });
    }
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
        assert!(registry.get_credential("barqDbApi").is_some());
        assert!(registry.get_credential("slackApi").is_some());
        assert!(registry.get_credential("githubApi").is_some());
        assert!(registry.get_credential("discordApi").is_some());
        assert!(registry.get_credential("notionApi").is_some());
        assert!(registry.get_credential("jiraApi").is_some());
        assert!(registry.get_credential("stripeApi").is_some());
        assert!(registry.get_credential("sendGridApi").is_some());
        assert!(registry.get_credential("hubspotApi").is_some());
        assert!(registry.get_credential("asanaApi").is_some());
        assert!(registry.get_credential("telegramApi").is_some());
    }
}
