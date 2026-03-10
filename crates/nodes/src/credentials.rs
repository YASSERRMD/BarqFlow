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

pub struct TwilioApiCredential;

#[async_trait]
impl ICredentialType for TwilioApiCredential {
    fn get_description(&self) -> ICredentialProperties {
        ICredentialProperties {
            name: "twilioApi".to_string(),
            display_name: "Twilio API".to_string(),
            notice: Some("Used by Twilio integration nodes".to_string()),
            properties: vec![
                INodeProperty {
                    display_name: "Account SID".to_string(),
                    name: "accountSid".to_string(),
                    r#type: NodePropertyType::String,
                    default: None,
                    description: Some("Twilio account SID".to_string()),
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "Auth Token".to_string(),
                    name: "authToken".to_string(),
                    r#type: NodePropertyType::String,
                    default: None,
                    description: Some("Twilio auth token".to_string()),
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
            ],
            documentation_url: Some(
                "https://www.twilio.com/docs/usage/requests-to-twilio".to_string(),
            ),
            authenticate: None,
        }
    }

    async fn test_credential(
        &self,
        credential_data: &HashMap<String, GenericValue>,
    ) -> Result<bool, BarqError> {
        let sid_ok = credential_data
            .get("accountSid")
            .and_then(|value| value.as_str())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let token_ok = credential_data
            .get("authToken")
            .and_then(|value| value.as_str())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);

        Ok(sid_ok && token_ok)
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

    let _ = registry.register_credential(CredentialInfo {
        name: "twilioApi".to_string(),
        cred_impl: Arc::new(TwilioApiCredential),
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
        TokenCredential {
            name: "airtableApi",
            display_name: "Airtable API",
            notice: "Used by Airtable integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://airtable.com/developers/web/api/authentication#personal-access-tokens",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "awsS3Api",
            display_name: "AWS S3 API",
            notice: "Used by AWS S3 integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://docs.aws.amazon.com/AmazonS3/latest/API/Welcome.html"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "bitbucketApi",
            display_name: "Bitbucket API",
            notice: "Used by Bitbucket integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developer.atlassian.com/cloud/bitbucket/rest/intro/#authentication",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "calendlyApi",
            display_name: "Calendly API",
            notice: "Used by Calendly integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developer.calendly.com/how-to-authenticate-with-personal-access-tokens",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "dropboxApi",
            display_name: "Dropbox API",
            notice: "Used by Dropbox integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developers.dropbox.com/oauth-guide#creating-an-access-token",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "gitlabApi",
            display_name: "GitLab API",
            notice: "Used by GitLab integration nodes",
            token_field: "privateToken",
            token_label: "Private Token",
            documentation_url: Some(
                "https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html",
            ),
            authenticate_header: None,
        },
        TokenCredential {
            name: "gmailApi",
            display_name: "Gmail API",
            notice: "Used by Gmail integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developers.google.com/workspace/gmail/api/auth/web-server",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "googleDriveApi",
            display_name: "Google Drive API",
            notice: "Used by Google Drive integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developers.google.com/drive/api/guides/api-specific-auth",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "googleSheetsApi",
            display_name: "Google Sheets API",
            notice: "Used by Google Sheets integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developers.google.com/sheets/api/guides/authorizing"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "oneDriveApi",
            display_name: "OneDrive API",
            notice: "Used by OneDrive integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://learn.microsoft.com/en-us/graph/auth/"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "linearApi",
            display_name: "Linear API",
            notice: "Used by Linear integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developers.linear.app/docs/graphql/working-with-the-graphql-api",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "clickUpApi",
            display_name: "ClickUp API",
            notice: "Used by ClickUp integration nodes",
            token_field: "authToken",
            token_label: "Auth Token",
            documentation_url: Some("https://developer.clickup.com/docs/authentication"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "mondayApi",
            display_name: "Monday.com API",
            notice: "Used by Monday.com integration nodes",
            token_field: "authToken",
            token_label: "Auth Token",
            documentation_url: Some(
                "https://developer.monday.com/api-reference/docs/authentication",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "mysqlApi",
            display_name: "MySQL API",
            notice: "Used by MySQL integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://dev.mysql.com/doc/"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "redisApi",
            display_name: "Redis API",
            notice: "Used by Redis integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://redis.io/docs/latest/"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "zendeskApi",
            display_name: "Zendesk API",
            notice: "Used by Zendesk integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developer.zendesk.com/api-reference/"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "salesforceApi",
            display_name: "Salesforce API",
            notice: "Used by Salesforce integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developer.salesforce.com/docs/apis"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "quickbooksApi",
            display_name: "QuickBooks API",
            notice: "Used by QuickBooks integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developer.intuit.com/app/developer/qbo/docs/develop"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "zoomApi",
            display_name: "Zoom API",
            notice: "Used by Zoom integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developers.zoom.us/docs/api/"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "trelloApi",
            display_name: "Trello API",
            notice: "Used by Trello integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developer.atlassian.com/cloud/trello/guides/rest-api/authorization/",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "outlookApi",
            display_name: "Outlook API",
            notice: "Used by Outlook integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://learn.microsoft.com/en-us/graph/outlook-concept-overview",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "shopifyApi",
            display_name: "Shopify Admin API",
            notice: "Used by Shopify integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://shopify.dev/docs/api/admin-rest"),
            authenticate_header: None,
        },
        TokenCredential {
            name: "paypalApi",
            display_name: "PayPal API",
            notice: "Used by PayPal integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some("https://developer.paypal.com/api/rest/"),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "intercomApi",
            display_name: "Intercom API",
            notice: "Used by Intercom integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developers.intercom.com/docs/references/rest-api/api.intercom.io",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "xeroApi",
            display_name: "Xero API",
            notice: "Used by Xero integration nodes",
            token_field: "accessToken",
            token_label: "Access Token",
            documentation_url: Some(
                "https://developer.xero.com/documentation/api/accounting/overview",
            ),
            authenticate_header: Some("Authorization"),
        },
        TokenCredential {
            name: "mailchimpApi",
            display_name: "Mailchimp API",
            notice: "Used by Mailchimp integration nodes",
            token_field: "apiKey",
            token_label: "API Key",
            documentation_url: Some("https://mailchimp.com/developer/marketing/docs/fundamentals/"),
            authenticate_header: None,
        },
        TokenCredential {
            name: "freshdeskApi",
            display_name: "Freshdesk API",
            notice: "Used by Freshdesk integration nodes",
            token_field: "apiKey",
            token_label: "API Key",
            documentation_url: Some("https://developers.freshdesk.com/api/"),
            authenticate_header: None,
        },
        TokenCredential {
            name: "pipedriveApi",
            display_name: "Pipedrive API",
            notice: "Used by Pipedrive integration nodes",
            token_field: "apiToken",
            token_label: "API Token",
            documentation_url: Some("https://developers.pipedrive.com/docs/api/v1"),
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
        assert!(registry.get_credential("airtableApi").is_some());
        assert!(registry.get_credential("awsS3Api").is_some());
        assert!(registry.get_credential("bitbucketApi").is_some());
        assert!(registry.get_credential("calendlyApi").is_some());
        assert!(registry.get_credential("dropboxApi").is_some());
        assert!(registry.get_credential("gitlabApi").is_some());
        assert!(registry.get_credential("gmailApi").is_some());
        assert!(registry.get_credential("googleDriveApi").is_some());
        assert!(registry.get_credential("googleSheetsApi").is_some());
        assert!(registry.get_credential("oneDriveApi").is_some());
        assert!(registry.get_credential("linearApi").is_some());
        assert!(registry.get_credential("clickUpApi").is_some());
        assert!(registry.get_credential("mondayApi").is_some());
        assert!(registry.get_credential("mysqlApi").is_some());
        assert!(registry.get_credential("redisApi").is_some());
        assert!(registry.get_credential("zendeskApi").is_some());
        assert!(registry.get_credential("salesforceApi").is_some());
        assert!(registry.get_credential("quickbooksApi").is_some());
        assert!(registry.get_credential("zoomApi").is_some());
        assert!(registry.get_credential("trelloApi").is_some());
        assert!(registry.get_credential("outlookApi").is_some());
        assert!(registry.get_credential("shopifyApi").is_some());
        assert!(registry.get_credential("paypalApi").is_some());
        assert!(registry.get_credential("intercomApi").is_some());
        assert!(registry.get_credential("xeroApi").is_some());
        assert!(registry.get_credential("mailchimpApi").is_some());
        assert!(registry.get_credential("freshdeskApi").is_some());
        assert!(registry.get_credential("pipedriveApi").is_some());
        assert!(registry.get_credential("twilioApi").is_some());
    }
}
