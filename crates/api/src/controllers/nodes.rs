use crate::contracts::NodeSchemaResponse;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use barqflow_core::properties::{INodeProperty, NodePropertyOption};
use barqflow_core::schema::CredentialReference;
use barqflow_nodes::{
    is_node_ui_exposed, node_documentation_url, node_support_note, node_support_tier,
    node_ui_category, NodeSupportTier,
};
use barqflow_registry::registry::NodeRegistry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub node_registry: Arc<NodeRegistry>,
}

pub fn node_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_node_schemas))
        .route("/dynamic-options", post(resolve_dynamic_options))
        .with_state(state)
}

async fn list_node_schemas(State(state): State<AppState>) -> impl IntoResponse {
    let mut schemas: Vec<NodeSchemaResponse> = Vec::new();

    let names = state.node_registry.get_all_node_names();
    for name in names {
        if !is_node_ui_exposed(&name) {
            continue;
        }

        if let Some(info) = state.node_registry.get_latest_node(&name) {
            let node_name = info.name.clone();
            let properties = info.properties.properties.clone();
            let category = node_ui_category(&node_name).unwrap_or("Core").to_string();
            let support_tier =
                tier_label(node_support_tier(&node_name).unwrap_or(NodeSupportTier::Beta));
            schemas.push(NodeSchemaResponse::from_node_info(
                info,
                category,
                support_tier.to_string(),
                node_support_note(&node_name).map(str::to_string),
                node_documentation_url(&node_name).map(str::to_string),
                node_credential_references(&node_name),
                build_defaults(&properties),
            ));
        }
    }

    schemas.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.display_name.cmp(&right.display_name))
    });

    Json(schemas)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DynamicNodeOptionsRequest {
    node_type: String,
    property_name: String,
    #[serde(default)]
    current_parameters: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DynamicNodeOptionsResponse {
    options: Vec<NodePropertyOption>,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

async fn resolve_dynamic_options(
    Json(request): Json<DynamicNodeOptionsRequest>,
) -> Result<Json<DynamicNodeOptionsResponse>, (StatusCode, String)> {
    match (request.node_type.as_str(), request.property_name.as_str()) {
        ("barqflow-nodes.openai", "model") => Ok(Json(DynamicNodeOptionsResponse {
            options: default_openai_model_options(),
            source: "catalog".to_string(),
            note: Some(
                "Curated OpenAI-compatible models. Bind a credential and override manually if your endpoint exposes a custom model set."
                    .to_string(),
            ),
        })),
        ("barqflow-nodes.ollama", "model") => {
            let base_url = request
                .current_parameters
                .get("baseUrl")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("http://localhost:11434");

            match fetch_ollama_model_options(base_url).await {
                Ok(options) if !options.is_empty() => Ok(Json(DynamicNodeOptionsResponse {
                    options,
                    source: "remote".to_string(),
                    note: Some(format!("Loaded from {}", normalized_base_url(base_url))),
                })),
                Ok(_) | Err(_) => Ok(Json(DynamicNodeOptionsResponse {
                    options: default_ollama_model_options(),
                    source: "fallback".to_string(),
                    note: Some(
                        "Could not reach the Ollama instance. Showing a local fallback catalog instead."
                            .to_string(),
                    ),
                })),
            }
        }
        _ => Err((
            StatusCode::NOT_FOUND,
            format!(
                "Dynamic options are not implemented for '{}.{}'",
                request.node_type, request.property_name
            ),
        )),
    }
}

async fn fetch_ollama_model_options(
    base_url: &str,
) -> Result<Vec<NodePropertyOption>, reqwest::Error> {
    let endpoint = format!("{}/api/tags", normalized_base_url(base_url));
    let response = reqwest::Client::new()
        .get(endpoint)
        .timeout(Duration::from_secs(5))
        .send()
        .await?
        .error_for_status()?;

    let payload: Value = response.json().await?;
    let mut options = payload
        .get("models")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let name = entry
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| entry.get("model").and_then(Value::as_str))?;
            let clean_name = name.trim();
            if clean_name.is_empty() {
                return None;
            }

            Some(NodePropertyOption {
                name: clean_name.to_string(),
                value: Value::String(clean_name.to_string()),
                description: entry
                    .get("size")
                    .and_then(Value::as_u64)
                    .map(|size| format!("{} bytes", size)),
            })
        })
        .collect::<Vec<_>>();

    options.sort_by(|left, right| left.name.cmp(&right.name));
    options.dedup_by(|left, right| left.name == right.name);
    Ok(options)
}

fn normalized_base_url(base_url: &str) -> String {
    base_url.trim().trim_end_matches('/').to_string()
}

fn default_openai_model_options() -> Vec<NodePropertyOption> {
    [
        ("gpt-4.1", "Frontier reasoning model"),
        ("gpt-4.1-mini", "Balanced latency and quality"),
        ("gpt-4o", "Omni model for multimodal workflows"),
        ("gpt-4o-mini", "Fast default for agent steps"),
        ("o3-mini", "Reasoning-focused compact model"),
        (
            "text-embedding-3-large",
            "Embedding model for retrieval workflows",
        ),
    ]
    .into_iter()
    .map(|(name, description)| NodePropertyOption {
        name: name.to_string(),
        value: Value::String(name.to_string()),
        description: Some(description.to_string()),
    })
    .collect()
}

fn default_ollama_model_options() -> Vec<NodePropertyOption> {
    [
        ("llama3.2", "General-purpose local assistant"),
        ("codellama", "Coding-focused local model"),
        ("mistral", "Compact local instruction model"),
        ("phi4", "Small high-quality local model"),
        ("qwen2.5", "Broad multilingual local model"),
    ]
    .into_iter()
    .map(|(name, description)| NodePropertyOption {
        name: name.to_string(),
        value: Value::String(name.to_string()),
        description: Some(description.to_string()),
    })
    .collect()
}

fn tier_label(tier: NodeSupportTier) -> &'static str {
    match tier {
        NodeSupportTier::Supported => "supported",
        NodeSupportTier::Beta => "beta",
        NodeSupportTier::Hidden => "hidden",
    }
}

fn node_credential_references(node_name: &str) -> Vec<CredentialReference> {
    match node_name {
        "barqflow-nodes.openai" => vec![CredentialReference {
            credential_type: "openAiApi".to_string(),
            required: true,
            display_name: "OpenAI API".to_string(),
        }],
        "barqflow-nodes.postgres" => vec![CredentialReference {
            credential_type: "postgresApi".to_string(),
            required: true,
            display_name: "Postgres".to_string(),
        }],
        "barqflow-nodes.slack" => vec![CredentialReference {
            credential_type: "slackApi".to_string(),
            required: true,
            display_name: "Slack API".to_string(),
        }],
        "barqflow-nodes.github" => vec![CredentialReference {
            credential_type: "githubApi".to_string(),
            required: true,
            display_name: "GitHub API".to_string(),
        }],
        "barqflow-nodes.discord" => vec![CredentialReference {
            credential_type: "discordApi".to_string(),
            required: true,
            display_name: "Discord API".to_string(),
        }],
        "barqflow-nodes.notion" => vec![CredentialReference {
            credential_type: "notionApi".to_string(),
            required: true,
            display_name: "Notion API".to_string(),
        }],
        "barqflow-nodes.jira" => vec![CredentialReference {
            credential_type: "jiraApi".to_string(),
            required: true,
            display_name: "Jira API".to_string(),
        }],
        "barqflow-nodes.stripe" => vec![CredentialReference {
            credential_type: "stripeApi".to_string(),
            required: true,
            display_name: "Stripe API".to_string(),
        }],
        "barqflow-nodes.sendGrid" => vec![CredentialReference {
            credential_type: "sendGridApi".to_string(),
            required: true,
            display_name: "SendGrid API".to_string(),
        }],
        "barqflow-nodes.hubspot" => vec![CredentialReference {
            credential_type: "hubspotApi".to_string(),
            required: true,
            display_name: "HubSpot API".to_string(),
        }],
        "barqflow-nodes.asana" => vec![CredentialReference {
            credential_type: "asanaApi".to_string(),
            required: true,
            display_name: "Asana API".to_string(),
        }],
        "barqflow-nodes.telegram" => vec![CredentialReference {
            credential_type: "telegramApi".to_string(),
            required: true,
            display_name: "Telegram Bot API".to_string(),
        }],
        "barqflow-nodes.airtable" => vec![CredentialReference {
            credential_type: "airtableApi".to_string(),
            required: true,
            display_name: "Airtable API".to_string(),
        }],
        "barqflow-nodes.awsS3" => vec![CredentialReference {
            credential_type: "awsS3Api".to_string(),
            required: true,
            display_name: "AWS S3 API".to_string(),
        }],
        "barqflow-nodes.bitbucket" => vec![CredentialReference {
            credential_type: "bitbucketApi".to_string(),
            required: true,
            display_name: "Bitbucket API".to_string(),
        }],
        "barqflow-nodes.calendly" => vec![CredentialReference {
            credential_type: "calendlyApi".to_string(),
            required: true,
            display_name: "Calendly API".to_string(),
        }],
        "barqflow-nodes.dropbox" => vec![CredentialReference {
            credential_type: "dropboxApi".to_string(),
            required: true,
            display_name: "Dropbox API".to_string(),
        }],
        "barqflow-nodes.gitlab" => vec![CredentialReference {
            credential_type: "gitlabApi".to_string(),
            required: true,
            display_name: "GitLab API".to_string(),
        }],
        "barqflow-nodes.gmail" => vec![CredentialReference {
            credential_type: "gmailApi".to_string(),
            required: true,
            display_name: "Gmail API".to_string(),
        }],
        "barqflow-nodes.googleDrive" => vec![CredentialReference {
            credential_type: "googleDriveApi".to_string(),
            required: true,
            display_name: "Google Drive API".to_string(),
        }],
        "barqflow-nodes.googleSheets" => vec![CredentialReference {
            credential_type: "googleSheetsApi".to_string(),
            required: true,
            display_name: "Google Sheets API".to_string(),
        }],
        "barqflow-nodes.oneDrive" => vec![CredentialReference {
            credential_type: "oneDriveApi".to_string(),
            required: true,
            display_name: "OneDrive API".to_string(),
        }],
        "barqflow-nodes.linear" => vec![CredentialReference {
            credential_type: "linearApi".to_string(),
            required: true,
            display_name: "Linear API".to_string(),
        }],
        "barqflow-nodes.mysql" => vec![CredentialReference {
            credential_type: "mysqlApi".to_string(),
            required: true,
            display_name: "MySQL API".to_string(),
        }],
        "barqflow-nodes.redis" => vec![CredentialReference {
            credential_type: "redisApi".to_string(),
            required: true,
            display_name: "Redis API".to_string(),
        }],
        "barqflow-nodes.zendesk" => vec![CredentialReference {
            credential_type: "zendeskApi".to_string(),
            required: true,
            display_name: "Zendesk API".to_string(),
        }],
        "barqflow-nodes.salesforce" => vec![CredentialReference {
            credential_type: "salesforceApi".to_string(),
            required: true,
            display_name: "Salesforce API".to_string(),
        }],
        "barqflow-nodes.quickbooks" => vec![CredentialReference {
            credential_type: "quickbooksApi".to_string(),
            required: true,
            display_name: "QuickBooks API".to_string(),
        }],
        "barqflow-nodes.zoom" => vec![CredentialReference {
            credential_type: "zoomApi".to_string(),
            required: true,
            display_name: "Zoom API".to_string(),
        }],
        "barqflow-nodes.trello" => vec![CredentialReference {
            credential_type: "trelloApi".to_string(),
            required: true,
            display_name: "Trello API".to_string(),
        }],
        "barqflow-nodes.outlook" => vec![CredentialReference {
            credential_type: "outlookApi".to_string(),
            required: true,
            display_name: "Outlook API".to_string(),
        }],
        "barqflow-nodes.paypal" => vec![CredentialReference {
            credential_type: "paypalApi".to_string(),
            required: true,
            display_name: "PayPal API".to_string(),
        }],
        "barqflow-nodes.intercom" => vec![CredentialReference {
            credential_type: "intercomApi".to_string(),
            required: true,
            display_name: "Intercom API".to_string(),
        }],
        "barqflow-nodes.xero" => vec![CredentialReference {
            credential_type: "xeroApi".to_string(),
            required: true,
            display_name: "Xero API".to_string(),
        }],
        "barqflow-nodes.mailchimp" => vec![CredentialReference {
            credential_type: "mailchimpApi".to_string(),
            required: true,
            display_name: "Mailchimp API".to_string(),
        }],
        "barqflow-nodes.freshdesk" => vec![CredentialReference {
            credential_type: "freshdeskApi".to_string(),
            required: true,
            display_name: "Freshdesk API".to_string(),
        }],
        "barqflow-nodes.twilio" => vec![CredentialReference {
            credential_type: "twilioApi".to_string(),
            required: true,
            display_name: "Twilio API".to_string(),
        }],
        "barqflow-nodes.shopify" => vec![CredentialReference {
            credential_type: "shopifyApi".to_string(),
            required: true,
            display_name: "Shopify Admin API".to_string(),
        }],
        "barqflow-nodes.clickUp" => vec![CredentialReference {
            credential_type: "clickUpApi".to_string(),
            required: true,
            display_name: "ClickUp API".to_string(),
        }],
        "barqflow-nodes.monday" => vec![CredentialReference {
            credential_type: "mondayApi".to_string(),
            required: true,
            display_name: "Monday.com API".to_string(),
        }],
        "barqflow-nodes.pipedrive" => vec![CredentialReference {
            credential_type: "pipedriveApi".to_string(),
            required: true,
            display_name: "Pipedrive API".to_string(),
        }],
        "barqflow-nodes.barqDbInsert"
        | "barqflow-nodes.barqDbSearch"
        | "barqflow-nodes.barqDbDelete" => vec![CredentialReference {
            credential_type: "barqDbApi".to_string(),
            required: true,
            display_name: "BarqDB API".to_string(),
        }],
        _ => vec![],
    }
}

fn build_defaults(properties: &[INodeProperty]) -> Value {
    let mut defaults = serde_json::Map::new();

    for property in properties {
        if let Some(default) = property.default.clone() {
            defaults.insert(property.name.clone(), default);
        }
    }

    Value::Object(defaults)
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::properties::NodePropertyType;

    fn build_prop(name: &str, default: Option<Value>) -> INodeProperty {
        INodeProperty {
            display_name: name.to_string(),
            name: name.to_string(),
            r#type: NodePropertyType::String,
            default,
            description: None,
            hint: None,
            required: false,
            options: None,
            display_options: None,
        }
    }

    #[test]
    fn build_defaults_returns_object_for_properties_with_defaults() {
        let props = vec![
            build_prop("method", Some(serde_json::json!("GET"))),
            build_prop("url", None),
            build_prop("retry", Some(serde_json::json!(3))),
        ];

        let defaults = build_defaults(&props);
        assert_eq!(defaults["method"], "GET");
        assert_eq!(defaults["retry"], 3);
        assert!(defaults.get("url").is_none());
    }

    #[test]
    fn build_defaults_returns_empty_object_when_no_defaults_exist() {
        let props = vec![build_prop("url", None), build_prop("body", None)];
        assert_eq!(defaults_object(), build_defaults(&props));
    }

    #[test]
    fn node_credential_references_includes_barqdb_nodes() {
        let refs = node_credential_references("barqflow-nodes.barqDbSearch");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].credential_type, "barqDbApi");
        assert!(refs[0].required);
    }

    #[test]
    fn node_credential_references_include_core_integration_bindings() {
        let cases = vec![
            ("barqflow-nodes.slack", "slackApi"),
            ("barqflow-nodes.github", "githubApi"),
            ("barqflow-nodes.discord", "discordApi"),
            ("barqflow-nodes.notion", "notionApi"),
            ("barqflow-nodes.jira", "jiraApi"),
            ("barqflow-nodes.stripe", "stripeApi"),
            ("barqflow-nodes.sendGrid", "sendGridApi"),
            ("barqflow-nodes.hubspot", "hubspotApi"),
            ("barqflow-nodes.asana", "asanaApi"),
            ("barqflow-nodes.telegram", "telegramApi"),
            ("barqflow-nodes.airtable", "airtableApi"),
            ("barqflow-nodes.awsS3", "awsS3Api"),
            ("barqflow-nodes.bitbucket", "bitbucketApi"),
            ("barqflow-nodes.calendly", "calendlyApi"),
            ("barqflow-nodes.dropbox", "dropboxApi"),
            ("barqflow-nodes.gitlab", "gitlabApi"),
            ("barqflow-nodes.gmail", "gmailApi"),
            ("barqflow-nodes.googleDrive", "googleDriveApi"),
            ("barqflow-nodes.googleSheets", "googleSheetsApi"),
            ("barqflow-nodes.oneDrive", "oneDriveApi"),
            ("barqflow-nodes.linear", "linearApi"),
            ("barqflow-nodes.mysql", "mysqlApi"),
            ("barqflow-nodes.redis", "redisApi"),
            ("barqflow-nodes.zendesk", "zendeskApi"),
            ("barqflow-nodes.salesforce", "salesforceApi"),
            ("barqflow-nodes.quickbooks", "quickbooksApi"),
            ("barqflow-nodes.zoom", "zoomApi"),
            ("barqflow-nodes.trello", "trelloApi"),
            ("barqflow-nodes.outlook", "outlookApi"),
            ("barqflow-nodes.paypal", "paypalApi"),
            ("barqflow-nodes.intercom", "intercomApi"),
            ("barqflow-nodes.xero", "xeroApi"),
            ("barqflow-nodes.mailchimp", "mailchimpApi"),
            ("barqflow-nodes.freshdesk", "freshdeskApi"),
            ("barqflow-nodes.twilio", "twilioApi"),
            ("barqflow-nodes.shopify", "shopifyApi"),
            ("barqflow-nodes.clickUp", "clickUpApi"),
            ("barqflow-nodes.monday", "mondayApi"),
            ("barqflow-nodes.pipedrive", "pipedriveApi"),
        ];

        for (node_name, credential_type) in cases {
            let refs = node_credential_references(node_name);
            assert_eq!(refs.len(), 1);
            assert_eq!(refs[0].credential_type, credential_type);
        }
    }

    #[test]
    fn default_dynamic_options_include_expected_models() {
        let openai = default_openai_model_options();
        let ollama = default_ollama_model_options();

        assert!(openai.iter().any(|option| option.name == "gpt-4o-mini"));
        assert!(ollama.iter().any(|option| option.name == "llama3.2"));
    }

    fn defaults_object() -> Value {
        Value::Object(serde_json::Map::new())
    }
}
