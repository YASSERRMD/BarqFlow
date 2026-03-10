use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use barqflow_core::properties::INodeProperty;
use barqflow_core::schema::CredentialReference;
use barqflow_nodes::is_node_ui_exposed;
use barqflow_registry::registry::NodeRegistry;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub node_registry: Arc<NodeRegistry>,
}

#[derive(Serialize)]
pub struct NodeSchema {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub is_trigger: bool,
    pub properties: Vec<INodeProperty>,
    pub credentials: Vec<CredentialReference>,
    pub defaults: Option<Value>,
}

pub fn node_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_node_schemas))
        .with_state(state)
}

async fn list_node_schemas(State(state): State<AppState>) -> impl IntoResponse {
    let mut schemas = Vec::new();

    let names = state.node_registry.get_all_node_names();
    for name in names {
        if !is_node_ui_exposed(&name) {
            continue;
        }

        if let Some(info) = state.node_registry.get_latest_node(&name) {
            let node_name = info.name.clone();
            let properties = info.properties.properties.clone();
            schemas.push(NodeSchema {
                name: node_name.clone(),
                display_name: info.display_name,
                description: info.description,
                is_trigger: info.is_trigger,
                defaults: build_defaults(&properties),
                properties,
                credentials: node_credential_references(&node_name),
            });
        }
    }

    Json(schemas)
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

fn build_defaults(properties: &[INodeProperty]) -> Option<Value> {
    let mut defaults = serde_json::Map::new();

    for property in properties {
        if let Some(default) = property.default.clone() {
            defaults.insert(property.name.clone(), default);
        }
    }

    if defaults.is_empty() {
        None
    } else {
        Some(Value::Object(defaults))
    }
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

        let defaults = build_defaults(&props).expect("defaults should be present");
        assert_eq!(defaults["method"], "GET");
        assert_eq!(defaults["retry"], 3);
        assert!(defaults.get("url").is_none());
    }

    #[test]
    fn build_defaults_returns_none_when_no_defaults_exist() {
        let props = vec![build_prop("url", None), build_prop("body", None)];
        assert!(build_defaults(&props).is_none());
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
        ];

        for (node_name, credential_type) in cases {
            let refs = node_credential_references(node_name);
            assert_eq!(refs.len(), 1);
            assert_eq!(refs[0].credential_type, credential_type);
            assert!(refs[0].required);
        }
    }
}
