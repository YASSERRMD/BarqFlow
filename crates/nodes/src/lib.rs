pub mod code;
pub mod credentials;
pub mod deduplication;
pub mod http;
pub mod logic;
pub mod manipulation;
pub mod sandbox;
pub mod scheduler;
pub mod subworkflow;
pub mod trigger;
pub mod wait;
pub mod integration;

pub fn register_all_credentials(registry: &barqflow_registry::registry::CredentialRegistry) {
    credentials::register_all_credentials(registry);
}

pub fn is_node_ui_exposed(name: &str) -> bool {
    matches!(
        name,
        "n8n-nodes-base.httpRequest"
            | "n8n-nodes-base.if"
            | "n8n-nodes-base.switch"
            | "n8n-nodes-base.merge"
            | "n8n-nodes-base.set"
            | "n8n-nodes-base.filter"
            | "n8n-nodes-base.itemLists"
            | "n8n-nodes-base.code"
            | "n8n-nodes-base.manualTrigger"
            | "barqflow-nodes.wait"
            | "barqflow-nodes.errorTrigger"
            | "barqflow-nodes.webhook"
            | "barqflow-nodes.cronTrigger"
            | "barqflow-nodes.executeWorkflow"
            | "barqflow-nodes.postgres"
            | "barqflow-nodes.openai"
            | "barqflow-nodes.ollama"
    )
}

#[cfg(test)]
mod tests {
    use super::is_node_ui_exposed;

    #[test]
    fn test_ui_exposure_hides_scaffold_integrations() {
        assert!(is_node_ui_exposed("barqflow-nodes.openai"));
        assert!(is_node_ui_exposed("barqflow-nodes.postgres"));
        assert!(!is_node_ui_exposed("barqflow-nodes.slack"));
        assert!(!is_node_ui_exposed("barqflow-nodes.github"));
    }
}

pub fn register_all_nodes(registry: &barqflow_registry::registry::NodeRegistry) {
    use barqflow_registry::registry::NodeInfo;
    use barqflow_core::properties::INodeProperties;
    use std::sync::Arc;

    let empty_props = INodeProperties {
        properties: vec![],
        display_name: None,
        required_values: None,
    };

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.httpRequest".into(),
        display_name: "HTTP Request".into(),
        version: 1.0,
        description: "Make an HTTP request".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::http::HttpRequestNode::new()),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.if".into(),
        display_name: "If".into(),
        version: 1.0,
        description: "Split flow conditionally".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::logic::IfNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.switch".into(),
        display_name: "Switch".into(),
        version: 1.0,
        description: "Route items based on matching values".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::logic::SwitchNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.merge".into(),
        display_name: "Merge".into(),
        version: 1.0,
        description: "Merge two branches".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 2,
        node_impl: Arc::new(crate::logic::MergeNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.set".into(),
        display_name: "Set".into(),
        version: 1.0,
        description: "Set values".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::manipulation::SetNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.filter".into(),
        display_name: "Filter".into(),
        version: 1.0,
        description: "Filters items based on conditions".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::manipulation::FilterNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.itemLists".into(),
        display_name: "Item Lists".into(),
        version: 1.0,
        description: "Split items into batches or combine items".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::manipulation::ItemListsNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.code".into(),
        display_name: "Code".into(),
        version: 1.0,
        description: "Run custom script".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::code::CodeNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.manualTrigger".into(),
        display_name: "Manual Trigger".into(),
        version: 1.0,
        description: "Start workflow manually".into(),
        properties: empty_props.clone(),
        is_trigger: true,
        max_inputs: 0,
        node_impl: Arc::new(crate::trigger::ManualTriggerNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.wait".into(),
        display_name: "Wait".into(),
        version: 1.0,
        description: "Suspend execution for a time or webhook".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(wait::WaitNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.errorTrigger".into(),
        display_name: "Error Trigger".into(),
        version: 1.0,
        description: "Triggers error workflow on failure".into(),
        properties: empty_props.clone(),
        is_trigger: true,
        max_inputs: 0,
        node_impl: Arc::new(trigger::ErrorTriggerNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.webhook".into(),
        display_name: "Webhook".into(),
        version: 1.0,
        description: "Triggered via webhook".into(),
        properties: empty_props.clone(),
        is_trigger: true,
        max_inputs: 0,
        node_impl: Arc::new(trigger::WebhookNode::new()),
    });

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.cronTrigger".into(),
        display_name: "Cron Trigger".into(),
        version: 1.0,
        description: "Triggers on schedule".into(),
        properties: empty_props.clone(),
        is_trigger: true,
        max_inputs: 0,
        node_impl: Arc::new(trigger::CronTriggerNode::new("0 * * * *")),
    });

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.executeWorkflow".into(),
        display_name: "Execute Workflow".into(),
        version: 1.0,
        description: "Execute another workflow".into(),
        properties: empty_props.clone(),
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(subworkflow::ExecuteWorkflowNode),
    });

    let mut postgres_props = empty_props.clone();
    postgres_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("executeQuery")),
            description: Some("The operation to perform".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Execute Query".into(),
                    value: serde_json::json!("executeQuery"),
                    description: None,
                }
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "query".into(),
            display_name: "Query".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("SELECT * FROM users;")),
            description: Some("The SQL query to execute".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.postgres".into(),
        display_name: "PostgreSQL".into(),
        version: 1.0,
        description: "Execute SQL queries on PostgreSQL".into(),
        properties: postgres_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::postgres::PostgresNode::new()),
    });

    let mut openai_props = empty_props.clone();
    openai_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("chatCompletion")),
            description: Some("The operation to perform".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Chat Completion".into(),
                    value: serde_json::json!("chatCompletion"),
                    description: None,
                }
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "model".into(),
            display_name: "Model".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("gpt-4o-mini")),
            description: Some("The OpenAI model to use".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "prompt".into(),
            display_name: "Prompt".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("The user prompt to send".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.openai".into(),
        display_name: "OpenAI".into(),
        version: 1.0,
        description: "Interact with OpenAI APIs".into(),
        properties: openai_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::openai::OpenAINode::new()),
    });

    let mut ollama_props = empty_props.clone();
    ollama_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("http://host.docker.internal:11434")),
            description: Some("The Ollama instance base URL".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("generate")),
            description: Some("The operation to perform".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Generate Text".into(),
                    value: serde_json::json!("generate"),
                    description: None,
                }
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "model".into(),
            display_name: "Model".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("llama3")),
            description: Some("The Ollama model to use".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "prompt".into(),
            display_name: "Prompt".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("The user prompt to send".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.ollama".into(),
        display_name: "Ollama".into(),
        version: 1.0,
        description: "Interact with a local Ollama instance".into(),
        properties: ollama_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::ollama::OllamaNode::new()),
    });

    // --- Phase 59 Expanded Integrations ---

    let mut telegram_props = empty_props.clone();
    telegram_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "chatId".into(),
            display_name: "Chat ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Target Chat ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "text".into(),
            display_name: "Text".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Message to send".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.telegram".into(),
        display_name: "Telegram".into(),
        version: 1.0,
        description: "Send messages via Telegram Bot API".into(),
        properties: telegram_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::telegram::TelegramNode::new()),
    });

    let mut slack_props = empty_props.clone();
    slack_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "channel".into(),
            display_name: "Channel".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Slack Channel Name or ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "text".into(),
            display_name: "Text".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Message to send".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.slack".into(),
        display_name: "Slack".into(),
        version: 1.0,
        description: "Send messages to Slack channels".into(),
        properties: slack_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::slack::SlackNode::new()),
    });

    let mut github_props = empty_props.clone();
    github_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "owner".into(),
            display_name: "Repository Owner".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Owner of the repository".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "repo".into(),
            display_name: "Repository Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Name of the repository".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.github".into(),
        display_name: "GitHub".into(),
        version: 1.0,
        description: "Interact with GitHub API".into(),
        properties: github_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::github::GithubNode::new()),
    });

    let mut sheets_props = empty_props.clone();
    sheets_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "spreadsheetId".into(),
            display_name: "Spreadsheet ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("The ID of the spreadsheet".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "range".into(),
            display_name: "Range".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("A1 notation range (e.g. Sheet1!A1:B2)".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.googleSheets".into(),
        display_name: "Google Sheets".into(),
        version: 1.0,
        description: "Read, write to Google Sheets".into(),
        properties: sheets_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::google_sheets::SheetsNode::new()),
    });

    // --- Phase 60 Integration Nodes (Batch 2) ---

    let mut discord_props = empty_props.clone();
    discord_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "webhookUrl".into(),
            display_name: "Webhook URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Discord Webhook URL".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "content".into(),
            display_name: "Content".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Message to send".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.discord".into(),
        display_name: "Discord".into(),
        version: 1.0,
        description: "Send messages to Discord channels".into(),
        properties: discord_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::discord::DiscordNode::new()),
    });

    let mut notion_props = empty_props.clone();
    notion_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "databaseId".into(),
            display_name: "Database ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Notion Database ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.notion".into(),
        display_name: "Notion".into(),
        version: 1.0,
        description: "Interact with Notion API".into(),
        properties: notion_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::notion::NotionNode::new()),
    });

    let mut airtable_props = empty_props.clone();
    airtable_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "baseId".into(),
            display_name: "Base ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Airtable Base ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "table".into(),
            display_name: "Table".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Airtable Table Name".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.airtable".into(),
        display_name: "Airtable".into(),
        version: 1.0,
        description: "Interact with Airtable".into(),
        properties: airtable_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::airtable::AirtableNode::new()),
    });

    let mut mysql_props = empty_props.clone();
    mysql_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "query".into(),
            display_name: "Query".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("SQL query to execute".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.mysql".into(),
        display_name: "MySQL".into(),
        version: 1.0,
        description: "Execute SQL queries on MySQL".into(),
        properties: mysql_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::mysql::MysqlNode::new()),
    });

    let mut redis_props = empty_props.clone();
    redis_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("set")),
            description: Some("Redis Operation".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Get".into(),
                    value: serde_json::json!("get"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Set".into(),
                    value: serde_json::json!("set"),
                    description: None,
                }
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "key".into(),
            display_name: "Key".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Redis Key".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.redis".into(),
        display_name: "Redis".into(),
        version: 1.0,
        description: "Interact with Redis key/value store".into(),
        properties: redis_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::redis::RedisNode::new()),
    });

    // --- Phase 61 Integration Nodes (Batch 3) ---

    let mut aws_s3_props = empty_props.clone();
    aws_s3_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "bucketName".into(),
            display_name: "Bucket Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("AWS S3 Bucket Name".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "objectKey".into(),
            display_name: "Object Key".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Key of the object to access".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.awsS3".into(),
        display_name: "AWS S3".into(),
        version: 1.0,
        description: "Interact with AWS Simple Storage Service".into(),
        properties: aws_s3_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::aws_s3::AwsS3Node::new()),
    });

    let mut google_drive_props = empty_props.clone();
    google_drive_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "fileId".into(),
            display_name: "File ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Google Drive File ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.googleDrive".into(),
        display_name: "Google Drive".into(),
        version: 1.0,
        description: "Access and modify Google Drive files".into(),
        properties: google_drive_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::google_drive::GoogleDriveNode::new()),
    });

    let mut jira_props = empty_props.clone();
    jira_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "issueKey".into(),
            display_name: "Issue Key".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Jira Issue Key (e.g. PROJ-123)".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.jira".into(),
        display_name: "Jira".into(),
        version: 1.0,
        description: "Manage issues and projects in Jira".into(),
        properties: jira_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::jira::JiraNode::new()),
    });

    let mut stripe_props = empty_props.clone();
    stripe_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "resource".into(),
            display_name: "Resource".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("customer")),
            description: Some("Stripe Resource".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Customer".into(),
                    value: serde_json::json!("customer"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Charge".into(),
                    value: serde_json::json!("charge"),
                    description: None,
                }
            ]),
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.stripe".into(),
        display_name: "Stripe".into(),
        version: 1.0,
        description: "Process payments and manage customers in Stripe".into(),
        properties: stripe_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::stripe::StripeNode::new()),
    });

    let mut sendgrid_props = empty_props.clone();
    sendgrid_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "toEmail".into(),
            display_name: "Recipient Email".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email address to send to".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "subject".into(),
            display_name: "Subject".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email Subject".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "content".into(),
            display_name: "Content".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Email Body".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.sendGrid".into(),
        display_name: "SendGrid".into(),
        version: 1.0,
        description: "Send emails via SendGrid".into(),
        properties: sendgrid_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::sendgrid::SendGridNode::new()),
    });

    // --- Phase 62 Integration Nodes (Batch 4) ---

    let mut salesforce_props = empty_props.clone();
    salesforce_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "resource".into(),
            display_name: "Resource".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("contact")),
            description: Some("Salesforce Resource".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Contact".into(),
                    value: serde_json::json!("contact"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Account".into(),
                    value: serde_json::json!("account"),
                    description: None,
                }
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("get")),
            description: Some("Salesforce Operation".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Get".into(),
                    value: serde_json::json!("get"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create".into(),
                    value: serde_json::json!("create"),
                    description: None,
                }
            ]),
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.salesforce".into(),
        display_name: "Salesforce".into(),
        version: 1.0,
        description: "Consume Salesforce API".into(),
        properties: salesforce_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::salesforce::SalesforceNode::new()),
    });

    let mut hubspot_props = empty_props.clone();
    hubspot_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "resource".into(),
            display_name: "Resource".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("contact")),
            description: Some("HubSpot Resource".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Contact".into(),
                    value: serde_json::json!("contact"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Company".into(),
                    value: serde_json::json!("company"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Deal".into(),
                    value: serde_json::json!("deal"),
                    description: None,
                }
            ]),
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.hubspot".into(),
        display_name: "HubSpot".into(),
        version: 1.0,
        description: "Consume HubSpot API".into(),
        properties: hubspot_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::hubspot::HubspotNode::new()),
    });

    let mut outlook_props = empty_props.clone();
    outlook_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "toEmail".into(),
            display_name: "Recipient Email".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email address to send to".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "subject".into(),
            display_name: "Subject".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email Subject".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "content".into(),
            display_name: "Content".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Email Body".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.outlook".into(),
        display_name: "Microsoft Outlook".into(),
        version: 1.0,
        description: "Send emails and manage events in Outlook".into(),
        properties: outlook_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::outlook::OutlookNode::new()),
    });

    let mut mailchimp_props = empty_props.clone();
    mailchimp_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "listId".into(),
            display_name: "List".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Mailchimp List (Audience) ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "emailAddress".into(),
            display_name: "Email Address".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email address to add".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.mailchimp".into(),
        display_name: "Mailchimp".into(),
        version: 1.0,
        description: "Manage lists and campaigns in Mailchimp".into(),
        properties: mailchimp_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::mailchimp::MailchimpNode::new()),
    });

    let mut asana_props = empty_props.clone();
    asana_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "workspace".into(),
            display_name: "Workspace ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Asana Workspace ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "project".into(),
            display_name: "Project ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Asana Project ID".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "name".into(),
            display_name: "Task Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Name of task to create".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        }
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.asana".into(),
        display_name: "Asana".into(),
        version: 1.0,
        description: "Manage projects and tasks in Asana".into(),
        properties: asana_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::asana::AsanaNode::new()),
    });
}
