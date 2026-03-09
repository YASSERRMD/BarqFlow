pub mod code;
pub mod credentials;
pub mod deduplication;
pub mod http;
pub mod integration;
pub mod logic;
pub mod manipulation;
pub mod sandbox;
pub mod scheduler;
pub mod subworkflow;
pub mod trigger;
pub mod wait;

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
            | "barqflow-nodes.slack"
            | "barqflow-nodes.github"
            | "barqflow-nodes.discord"
            | "barqflow-nodes.notion"
            | "barqflow-nodes.airtable"
            | "barqflow-nodes.jira"
            | "barqflow-nodes.stripe"
            | "barqflow-nodes.sendGrid"
            | "barqflow-nodes.hubspot"
            | "barqflow-nodes.asana"
            | "barqflow-nodes.telegram"
            | "barqflow-nodes.googleSheets"
            | "barqflow-nodes.googleDrive"
            | "barqflow-nodes.outlook"
            | "barqflow-nodes.mailchimp"
            | "barqflow-nodes.salesforce"
            | "barqflow-nodes.redis"
            | "barqflow-nodes.mysql"
            | "barqflow-nodes.awsS3"
            | "barqflow-nodes.gmail"
            | "barqflow-nodes.twilio"
            | "barqflow-nodes.trello"
            | "barqflow-nodes.gitlab"
            | "barqflow-nodes.bitbucket"
            | "barqflow-nodes.dropbox"
            | "barqflow-nodes.oneDrive"
            | "barqflow-nodes.linear"
            | "barqflow-nodes.clickUp"
            | "barqflow-nodes.monday"
    )
}

#[cfg(test)]
mod tests {
    use super::{is_node_ui_exposed, register_all_nodes};
    use barqflow_registry::registry::NodeRegistry;

    #[test]
    fn test_ui_exposure_includes_implemented_integrations() {
        assert!(is_node_ui_exposed("barqflow-nodes.openai"));
        assert!(is_node_ui_exposed("barqflow-nodes.postgres"));
        assert!(is_node_ui_exposed("barqflow-nodes.slack"));
        assert!(is_node_ui_exposed("barqflow-nodes.github"));
        assert!(is_node_ui_exposed("barqflow-nodes.telegram"));
        assert!(is_node_ui_exposed("barqflow-nodes.googleSheets"));
        assert!(is_node_ui_exposed("barqflow-nodes.gmail"));
        assert!(is_node_ui_exposed("barqflow-nodes.twilio"));
    }

    #[test]
    fn test_webhook_schema_exposes_response_configuration_properties() {
        let registry = NodeRegistry::new();
        register_all_nodes(&registry);

        let webhook = registry
            .get_latest_node("barqflow-nodes.webhook")
            .expect("webhook node should be registered");

        let names: Vec<String> = webhook
            .properties
            .properties
            .iter()
            .map(|p| p.name.clone())
            .collect();

        assert!(names.contains(&"path".to_string()));
        assert!(names.contains(&"httpMethod".to_string()));
        assert!(names.contains(&"responseMode".to_string()));
        assert!(names.contains(&"responseCode".to_string()));
        assert!(names.contains(&"responseData".to_string()));
    }
}

pub fn register_all_nodes(registry: &barqflow_registry::registry::NodeRegistry) {
    use barqflow_core::properties::INodeProperties;
    use barqflow_registry::registry::NodeInfo;
    use std::sync::Arc;

    let empty_props = INodeProperties {
        properties: vec![],
        display_name: None,
        required_values: None,
    };

    let mut http_props = empty_props.clone();
    http_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "url".into(),
            display_name: "URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://example.com")),
            description: Some("Target URL for the request.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Request headers as JSON array/object.".into()),
            hint: Some(r#"[{"name":"Authorization","value":"Bearer ..."}]"#.into()),
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Query parameters as JSON array/object.".into()),
            hint: Some(r#"[{"name":"q","value":"search"}]"#.into()),
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("")),
            description: Some("Request body (text or JSON string).".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "authentication".into(),
            display_name: "Authentication".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("none")),
            description: Some("Authentication mode.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "None".into(),
                    value: serde_json::json!("none"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Bearer Token".into(),
                    value: serde_json::json!("bearer"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Bearer Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("")),
            description: Some("Token used when Authentication is Bearer.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "authentication".into(),
                    values: vec![serde_json::json!("bearer")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "responseFormat".into(),
            display_name: "Response Format".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("autodetect")),
            description: Some("How response body should be parsed.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Auto Detect".into(),
                    value: serde_json::json!("autodetect"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "JSON".into(),
                    value: serde_json::json!("json"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Text".into(),
                    value: serde_json::json!("text"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "File".into(),
                    value: serde_json::json!("file"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(30000)),
            description: Some("Request timeout in milliseconds.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.httpRequest".into(),
        display_name: "HTTP Request".into(),
        version: 1.0,
        description: "Make an HTTP request".into(),
        properties: http_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::http::HttpRequestNode::new()),
    });

    let mut if_props = empty_props.clone();
    if_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "combineOperation".into(),
            display_name: "Combine".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("all")),
            description: Some("How multiple conditions are combined.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "All".into(),
                    value: serde_json::json!("all"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Any".into(),
                    value: serde_json::json!("any"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "conditions".into(),
            display_name: "Conditions".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("JSON array of conditions.".into()),
            hint: Some(
                r#"[{"value1":"={{$json.count}}","operation":"larger","value2":10}]"#.into(),
            ),
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("equals")),
            description: Some("Legacy single-condition operation.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Equals".into(),
                    value: serde_json::json!("equals"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Not Equals".into(),
                    value: serde_json::json!("notEquals"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Contains".into(),
                    value: serde_json::json!("contains"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Larger".into(),
                    value: serde_json::json!("larger"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Smaller".into(),
                    value: serde_json::json!("smaller"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Exists".into(),
                    value: serde_json::json!("exists"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Not Exists".into(),
                    value: serde_json::json!("notExists"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "value1".into(),
            display_name: "Value 1".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("")),
            description: Some("Legacy single-condition left value.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "value2".into(),
            display_name: "Value 2".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("")),
            description: Some("Legacy single-condition right value.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.if".into(),
        display_name: "If".into(),
        version: 1.0,
        description: "Split flow conditionally".into(),
        properties: if_props,
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
        properties: INodeProperties {
            properties: vec![
                barqflow_core::properties::INodeProperty {
                    name: "mode".into(),
                    display_name: "Mode".into(),
                    r#type: barqflow_core::properties::NodePropertyType::Options,
                    default: Some(serde_json::json!("append")),
                    description: Some("Merge strategy.".into()),
                    hint: None,
                    required: true,
                    display_options: None,
                    options: Some(vec![
                        barqflow_core::properties::NodePropertyOption {
                            name: "Append".into(),
                            value: serde_json::json!("append"),
                            description: None,
                        },
                        barqflow_core::properties::NodePropertyOption {
                            name: "Merge By Fields".into(),
                            value: serde_json::json!("merge"),
                            description: None,
                        },
                        barqflow_core::properties::NodePropertyOption {
                            name: "Multiplex".into(),
                            value: serde_json::json!("multiplex"),
                            description: None,
                        },
                    ]),
                },
                barqflow_core::properties::INodeProperty {
                    name: "property1".into(),
                    display_name: "Input 1 Field".into(),
                    r#type: barqflow_core::properties::NodePropertyType::String,
                    default: Some(serde_json::json!("id")),
                    description: Some("Join field from input 1 for merge mode.".into()),
                    hint: None,
                    required: false,
                    display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                        r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                            property: "mode".into(),
                            values: vec![serde_json::json!("merge")],
                        }),
                    }),
                    options: None,
                },
                barqflow_core::properties::INodeProperty {
                    name: "property2".into(),
                    display_name: "Input 2 Field".into(),
                    r#type: barqflow_core::properties::NodePropertyType::String,
                    default: Some(serde_json::json!("id")),
                    description: Some("Join field from input 2 for merge mode.".into()),
                    hint: None,
                    required: false,
                    display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                        r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                            property: "mode".into(),
                            values: vec![serde_json::json!("merge")],
                        }),
                    }),
                    options: None,
                },
            ],
            display_name: None,
            required_values: None,
        },
        is_trigger: false,
        max_inputs: 2,
        node_impl: Arc::new(crate::logic::MergeNode),
    });

    let mut set_props = empty_props.clone();
    set_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "keepOnlySet".into(),
            display_name: "Keep Only Set".into(),
            r#type: barqflow_core::properties::NodePropertyType::Boolean,
            default: Some(serde_json::json!(false)),
            description: Some("Discard incoming fields and keep only configured values.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "values".into(),
            display_name: "Values".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("JSON array of assignments: {name,value,type}.".into()),
            hint: Some(r#"[{"name":"result","value":"ok","type":"string"}]"#.into()),
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.set".into(),
        display_name: "Set".into(),
        version: 1.0,
        description: "Set values".into(),
        properties: set_props,
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
        properties: INodeProperties {
            properties: vec![
                barqflow_core::properties::INodeProperty {
                    name: "mode".into(),
                    display_name: "Mode".into(),
                    r#type: barqflow_core::properties::NodePropertyType::Options,
                    default: Some(serde_json::json!("splitInBatches")),
                    description: Some("Item list operation mode.".into()),
                    hint: None,
                    required: true,
                    display_options: None,
                    options: Some(vec![
                        barqflow_core::properties::NodePropertyOption {
                            name: "Split In Batches".into(),
                            value: serde_json::json!("splitInBatches"),
                            description: None,
                        },
                        barqflow_core::properties::NodePropertyOption {
                            name: "No Op".into(),
                            value: serde_json::json!("passthrough"),
                            description: None,
                        },
                    ]),
                },
                barqflow_core::properties::INodeProperty {
                    name: "batchSize".into(),
                    display_name: "Batch Size".into(),
                    r#type: barqflow_core::properties::NodePropertyType::Number,
                    default: Some(serde_json::json!(1)),
                    description: Some("Items per batch when splitting.".into()),
                    hint: None,
                    required: false,
                    display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                        r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                            property: "mode".into(),
                            values: vec![serde_json::json!("splitInBatches")],
                        }),
                    }),
                    options: None,
                },
            ],
            display_name: None,
            required_values: None,
        },
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::manipulation::ItemListsNode),
    });

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.splitInBatches".into(),
        display_name: "Split In Batches".into(),
        version: 1.0,
        description: "Split incoming items into configured batches".into(),
        properties: INodeProperties {
            properties: vec![barqflow_core::properties::INodeProperty {
                name: "batchSize".into(),
                display_name: "Batch Size".into(),
                r#type: barqflow_core::properties::NodePropertyType::Number,
                default: Some(serde_json::json!(1)),
                description: Some("Items per batch.".into()),
                hint: None,
                required: true,
                display_options: None,
                options: None,
            }],
            display_name: None,
            required_values: None,
        },
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(crate::manipulation::ItemListsNode),
    });

    let mut code_props = empty_props.clone();
    code_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "language".into(),
            display_name: "Language".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("javascript")),
            description: Some("Script language.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "JavaScript".into(),
                    value: serde_json::json!("javascript"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Python".into(),
                    value: serde_json::json!("python"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "jsCode".into(),
            display_name: "JavaScript Code".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("items[0].json.result = \"ok\";\nitems")),
            description: Some("Script body for JavaScript mode.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "language".into(),
                    values: vec![serde_json::json!("javascript")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "pythonCode".into(),
            display_name: "Python Code".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("items")),
            description: Some("Script body for Python mode.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "language".into(),
                    values: vec![serde_json::json!("python")],
                }),
            }),
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.code".into(),
        display_name: "Code".into(),
        version: 1.0,
        description: "Run custom script".into(),
        properties: code_props,
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

    let mut wait_props = empty_props.clone();
    wait_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "resume".into(),
            display_name: "Resume".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("time")),
            description: Some("How this execution should be resumed.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "After Time Interval".into(),
                    value: serde_json::json!("time"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "By Webhook".into(),
                    value: serde_json::json!("webhook"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "amount".into(),
            display_name: "Amount".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(1)),
            description: Some("Wait duration amount when using time-based resume.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "resume".into(),
                    values: vec![serde_json::json!("time")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "unit".into(),
            display_name: "Unit".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("seconds")),
            description: Some("Time unit used by Amount.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "resume".into(),
                    values: vec![serde_json::json!("time")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Milliseconds".into(),
                    value: serde_json::json!("milliseconds"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Seconds".into(),
                    value: serde_json::json!("seconds"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Minutes".into(),
                    value: serde_json::json!("minutes"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Hours".into(),
                    value: serde_json::json!("hours"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Days".into(),
                    value: serde_json::json!("days"),
                    description: None,
                },
            ]),
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.wait".into(),
        display_name: "Wait".into(),
        version: 1.0,
        description: "Suspend execution for a time or webhook".into(),
        properties: wait_props,
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

    let mut webhook_props = empty_props.clone();
    webhook_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "path".into(),
            display_name: "Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("webhook")),
            description: Some("Webhook path segment used under /webhook/{path}.".into()),
            hint: Some("Example: lead-capture".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "httpMethod".into(),
            display_name: "HTTP Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("ANY")),
            description: Some("Method this webhook endpoint accepts.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "ANY".into(),
                    value: serde_json::json!("ANY"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "responseMode".into(),
            display_name: "Response Mode".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("onReceived")),
            description: Some("Choose when to respond to the webhook request.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "On Received".into(),
                    value: serde_json::json!("onReceived"),
                    description: Some("Respond immediately after receiving the request.".into()),
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "After Workflow Completes".into(),
                    value: serde_json::json!("lastNode"),
                    description: Some("Respond only after execution finishes.".into()),
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "responseCode".into(),
            display_name: "Response Code".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(200)),
            description: Some("HTTP status code returned to the caller.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "responseData".into(),
            display_name: "Response Data".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{\"success\":true}")),
            description: Some("Optional JSON response payload for On Received mode.".into()),
            hint: Some("Leave empty to use default payload.".into()),
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "responseMode".into(),
                    values: vec![serde_json::json!("onReceived")],
                }),
            }),
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.webhook".into(),
        display_name: "Webhook".into(),
        version: 1.0,
        description: "Triggered via webhook".into(),
        properties: webhook_props,
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

    let mut execute_workflow_props = empty_props.clone();
    execute_workflow_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "workflowId".into(),
            display_name: "Workflow ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("UUID of the child workflow to execute.".into()),
            hint: Some("Use a workflow UUID from the workflows list.".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "mode".into(),
            display_name: "Mode".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("wait")),
            description: Some("Execution mode for sub-workflow invocation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Wait for Completion".into(),
                    value: serde_json::json!("wait"),
                    description: Some("Run child workflow and return its outputs.".into()),
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Run".into(),
                    value: serde_json::json!("run"),
                    description: Some("Trigger child execution inline.".into()),
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "inputData".into(),
            display_name: "Input Data".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some(
                "Optional JSON payload for the child workflow. If empty, incoming items are forwarded."
                    .into(),
            ),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.executeWorkflow".into(),
        display_name: "Execute Workflow".into(),
        version: 1.0,
        description: "Execute another workflow".into(),
        properties: execute_workflow_props,
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
            description: Some("Postgres operation to perform.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Execute Query".into(),
                    value: serde_json::json!("executeQuery"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Select Rows".into(),
                    value: serde_json::json!("selectRows"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Insert Row".into(),
                    value: serde_json::json!("insertRow"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "query".into(),
            display_name: "Query".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("SELECT * FROM users;")),
            description: Some("SQL query for Execute Query operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("executeQuery")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "table".into(),
            display_name: "Table".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Table name for Select/Insert operations.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("selectRows"),
                        serde_json::json!("insertRow"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "columns".into(),
            display_name: "Columns".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("*")),
            description: Some("Comma-separated columns for Select Rows.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("selectRows")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "whereClause".into(),
            display_name: "Where Clause".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Optional SQL where clause for Select Rows.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("selectRows")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "limit".into(),
            display_name: "Limit".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(100)),
            description: Some("Optional result limit for Select Rows.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("selectRows")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "data".into(),
            display_name: "Data (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("JSON object for Insert Row operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("insertRow")],
                }),
            }),
            options: None,
        },
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
            options: Some(vec![barqflow_core::properties::NodePropertyOption {
                name: "Chat Completion".into(),
                value: serde_json::json!("chatCompletion"),
                description: None,
            }]),
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.openai.com/v1")),
            description: Some("OpenAI API base URL.".into()),
            hint: Some("Override only if you use a custom compatible endpoint.".into()),
            required: false,
            display_options: None,
            options: None,
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
            name: "systemPrompt".into(),
            display_name: "System Prompt".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("")),
            description: Some("Optional system instructions for model behavior.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "prompt".into(),
            display_name: "Prompt".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("")),
            description: Some("The user prompt to send".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "temperature".into(),
            display_name: "Temperature".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(0.7)),
            description: Some("Sampling temperature.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "maxTokens".into(),
            display_name: "Max Tokens".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(512)),
            description: Some("Maximum tokens in model response.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout in milliseconds.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            default: Some(serde_json::json!("http://localhost:11434")),
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
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "List Models".into(),
                    value: serde_json::json!("listModels"),
                    description: None,
                },
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
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("generate")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "systemPrompt".into(),
            display_name: "System Prompt".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("")),
            description: Some("Optional system instructions.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("generate")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "prompt".into(),
            display_name: "Prompt".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("")),
            description: Some("The user prompt to send".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("generate")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "temperature".into(),
            display_name: "Temperature".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(0.7)),
            description: Some("Optional sampling temperature.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("generate")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout in milliseconds.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("sendMessage")),
            description: Some("Telegram action to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Send Message".into(),
                    value: serde_json::json!("sendMessage"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "botToken".into(),
            display_name: "Bot Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Telegram bot token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.telegram.org")),
            description: Some("Telegram API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "chatId".into(),
            display_name: "Chat ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Target Chat ID".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendMessage")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "text".into(),
            display_name: "Text".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Message to send".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendMessage")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("getMe")),
            description: Some("Telegram method name for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("postMessage")),
            description: Some("Slack action to run.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Post Message".into(),
                    value: serde_json::json!("postMessage"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Slack Bot token.".into()),
            hint: Some("Starts with xoxb-...".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://slack.com")),
            description: Some("Slack API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "channel".into(),
            display_name: "Channel".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Slack Channel Name or ID".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("postMessage")],
                }),
            }),
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
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("postMessage")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/api/users.list")),
            description: Some("API path for custom Slack API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional extra headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout in milliseconds.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("getRepo")),
            description: Some("GitHub action to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Get Repository".into(),
                    value: serde_json::json!("getRepo"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "List Issues".into(),
                    value: serde_json::json!("listIssues"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Issue".into(),
                    value: serde_json::json!("createIssue"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("GitHub personal access token.".into()),
            hint: Some("Starts with ghp_...".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.github.com")),
            description: Some("GitHub API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "owner".into(),
            display_name: "Repository Owner".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Owner of the repository".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("getRepo"),
                        serde_json::json!("listIssues"),
                        serde_json::json!("createIssue"),
                    ],
                }),
            }),
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
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("getRepo"),
                        serde_json::json!("listIssues"),
                        serde_json::json!("createIssue"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "issueTitle".into(),
            display_name: "Issue Title".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Issue title when creating an issue.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "issueBody".into(),
            display_name: "Issue Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Issue body when creating an issue.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/user/repos")),
            description: Some("Resource path for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("readRange")),
            description: Some("Google Sheets operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Read Range".into(),
                    value: serde_json::json!("readRange"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Append Values".into(),
                    value: serde_json::json!("appendValues"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("OAuth access token for Google APIs.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://sheets.googleapis.com")),
            description: Some("Google Sheets API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "spreadsheetId".into(),
            display_name: "Spreadsheet ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("The ID of the spreadsheet".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("readRange"),
                        serde_json::json!("appendValues"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "range".into(),
            display_name: "Range".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("A1 notation range (e.g. Sheet1!A1:B2)".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("readRange"),
                        serde_json::json!("appendValues"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "values".into(),
            display_name: "Values (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("[[\"value\"]]")),
            description: Some("2D array values for append operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("appendValues")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/v4/spreadsheets")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("sendWebhook")),
            description: Some("Discord action to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Send Webhook".into(),
                    value: serde_json::json!("sendWebhook"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "webhookUrl".into(),
            display_name: "Webhook URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Discord Webhook URL".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendWebhook")],
                }),
            }),
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
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendWebhook")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Bot token for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://discord.com/api/v10")),
            description: Some("Discord API base URL.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/users/@me")),
            description: Some("Resource path for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional extra headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("queryDatabase")),
            description: Some("Notion action to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Query Database".into(),
                    value: serde_json::json!("queryDatabase"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Page".into(),
                    value: serde_json::json!("createPage"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Notion integration token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.notion.com")),
            description: Some("Notion API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "databaseId".into(),
            display_name: "Database ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Notion Database ID".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("queryDatabase"),
                        serde_json::json!("createPage"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "filter".into(),
            display_name: "Filter (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Optional query filter body for database query.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("queryDatabase")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "properties".into(),
            display_name: "Properties (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Page properties for create page operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createPage")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/v1/users")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listRecords")),
            description: Some("Airtable action to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Records".into(),
                    value: serde_json::json!("listRecords"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Record".into(),
                    value: serde_json::json!("createRecord"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Update Record".into(),
                    value: serde_json::json!("updateRecord"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Airtable API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.airtable.com")),
            description: Some("Airtable API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseId".into(),
            display_name: "Base ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Airtable Base ID".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("listRecords"),
                        serde_json::json!("createRecord"),
                        serde_json::json!("updateRecord"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "table".into(),
            display_name: "Table".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Airtable Table Name".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("listRecords"),
                        serde_json::json!("createRecord"),
                        serde_json::json!("updateRecord"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "recordId".into(),
            display_name: "Record ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Record ID for update operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("updateRecord")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "fields".into(),
            display_name: "Fields (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Record fields for create/update operations.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("createRecord"),
                        serde_json::json!("updateRecord"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/v0/base/table")),
            description: Some("Resource path for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("executeQuery")),
            description: Some("MySQL operation to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Execute Query".into(),
                    value: serde_json::json!("executeQuery"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Gateway API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://sql-gateway.example.com")),
            description: Some("MySQL HTTP gateway base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "query".into(),
            display_name: "Query".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("SELECT 1")),
            description: Some("SQL query to execute.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("executeQuery")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/query")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            default: Some(serde_json::json!("get")),
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
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Delete".into(),
                    value: serde_json::json!("delete"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Redis REST API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!(
                "https://your-upstash-endpoint.upstash.io"
            )),
            description: Some("Redis REST endpoint base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "key".into(),
            display_name: "Key".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Redis Key".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("get"),
                        serde_json::json!("set"),
                        serde_json::json!("delete"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "value".into(),
            display_name: "Value".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Redis value for set operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("set")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/ping")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("getObject")),
            description: Some("AWS S3 operation to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Get Object".into(),
                    value: serde_json::json!("getObject"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Put Object".into(),
                    value: serde_json::json!("putObject"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Gateway auth token (not needed with pre-signed URL).".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://s3.amazonaws.com")),
            description: Some("S3 or S3-compatible endpoint base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "preSignedUrl".into(),
            display_name: "Pre-signed URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Optional pre-signed URL for get/put operations.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("getObject"),
                        serde_json::json!("putObject"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "bucketName".into(),
            display_name: "Bucket Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("AWS S3 Bucket Name".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("getObject"),
                        serde_json::json!("putObject"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "objectKey".into(),
            display_name: "Object Key".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Key of the object to access".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("getObject"),
                        serde_json::json!("putObject"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/bucket/key")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for putObject or API Call operations.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("getFile")),
            description: Some("Google Drive operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Get File".into(),
                    value: serde_json::json!("getFile"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "List Files".into(),
                    value: serde_json::json!("listFiles"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("OAuth access token for Google APIs.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://www.googleapis.com/drive/v3")),
            description: Some("Google Drive API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "fileId".into(),
            display_name: "File ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Google Drive file ID.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("getFile")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/files")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("getIssue")),
            description: Some("Jira action to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Get Issue".into(),
                    value: serde_json::json!("getIssue"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Search Issues".into(),
                    value: serde_json::json!("searchIssues"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Issue".into(),
                    value: serde_json::json!("createIssue"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Jira API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://your-domain.atlassian.net")),
            description: Some("Your Jira instance URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "issueKey".into(),
            display_name: "Issue Key".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Jira issue key, e.g. PROJ-123.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("getIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "jql".into(),
            display_name: "JQL".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("order by created DESC")),
            description: Some("Query for issue search.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("searchIssues")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "issueFields".into(),
            display_name: "Issue Fields (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Jira fields object for create operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/rest/api/3/project")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            description: Some("Stripe resource.".into()),
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
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("list")),
            description: Some("Stripe operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List".into(),
                    value: serde_json::json!("list"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Retrieve".into(),
                    value: serde_json::json!("retrieve"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Stripe secret key.".into()),
            hint: Some("Starts with sk_...".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.stripe.com")),
            description: Some("Stripe API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "resourceId".into(),
            display_name: "Resource ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Resource ID for retrieve operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("retrieve")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/v1/customers")),
            description: Some("Resource path for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("sendEmail")),
            description: Some("SendGrid action to execute.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Send Email".into(),
                    value: serde_json::json!("sendEmail"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("SendGrid API key.".into()),
            hint: Some("Starts with SG....".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.sendgrid.com")),
            description: Some("SendGrid API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "fromEmail".into(),
            display_name: "From Email".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Sender email address.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendEmail")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "toEmail".into(),
            display_name: "Recipient Email".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email address to send to".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendEmail")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "subject".into(),
            display_name: "Subject".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email Subject".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendEmail")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "content".into(),
            display_name: "Content".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Email Body".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendEmail")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/v3/mail/send")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Salesforce access token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://your-instance.salesforce.com")),
            description: Some("Salesforce instance URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "apiVersion".into(),
            display_name: "API Version".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("v59.0")),
            description: Some("Salesforce API version.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
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
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "recordId".into(),
            display_name: "Record ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Record ID for get operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("get")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "fields".into(),
            display_name: "Fields (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Record fields for create operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("create")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/services/data/v59.0/sobjects/Contact")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            description: Some("HubSpot resource type.".into()),
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
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("list")),
            description: Some("HubSpot operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List".into(),
                    value: serde_json::json!("list"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create".into(),
                    value: serde_json::json!("create"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("HubSpot private app token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.hubapi.com")),
            description: Some("HubSpot API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "properties".into(),
            display_name: "Properties (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Properties object for create operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("create")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/crm/v3/objects/contacts")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("sendMail")),
            description: Some("Outlook operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Send Mail".into(),
                    value: serde_json::json!("sendMail"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "List Messages".into(),
                    value: serde_json::json!("listMessages"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Microsoft Graph access token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://graph.microsoft.com/v1.0")),
            description: Some("Microsoft Graph API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "toEmail".into(),
            display_name: "Recipient Email".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email address to send to".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendMail")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "subject".into(),
            display_name: "Subject".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email Subject".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendMail")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "content".into(),
            display_name: "Content".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Email Body".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendMail")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/me/messages")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("addMember")),
            description: Some("Mailchimp operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Add Member".into(),
                    value: serde_json::json!("addMember"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "List Members".into(),
                    value: serde_json::json!("listMembers"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "apiKey".into(),
            display_name: "API Key".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Mailchimp API key.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://us1.api.mailchimp.com/3.0")),
            description: Some("Mailchimp API base URL (datacenter-specific).".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "listId".into(),
            display_name: "List".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Mailchimp List (Audience) ID".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("addMember"),
                        serde_json::json!("listMembers"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "emailAddress".into(),
            display_name: "Email Address".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Email address to add".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("addMember")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "memberStatus".into(),
            display_name: "Member Status".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("subscribed")),
            description: Some("Member status for add member operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("addMember")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Subscribed".into(),
                    value: serde_json::json!("subscribed"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Pending".into(),
                    value: serde_json::json!("pending"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "mergeFields".into(),
            display_name: "Merge Fields (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Optional merge fields object for add member operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("addMember")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/lists")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("createTask")),
            description: Some("Asana operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Task".into(),
                    value: serde_json::json!("createTask"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "List Project Tasks".into(),
                    value: serde_json::json!("listProjectTasks"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Asana personal access token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://app.asana.com/api/1.0")),
            description: Some("Asana API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "workspace".into(),
            display_name: "Workspace ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Asana Workspace ID".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTask")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "project".into(),
            display_name: "Project ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Asana Project ID".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("createTask"),
                        serde_json::json!("listProjectTasks"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "name".into(),
            display_name: "Task Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Name of task to create".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTask")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "notes".into(),
            display_name: "Notes".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Optional task notes.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTask")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/tasks")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
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

    let mut gmail_props = empty_props.clone();
    gmail_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listMessages")),
            description: Some("Gmail operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Messages".into(),
                    value: serde_json::json!("listMessages"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Send Message".into(),
                    value: serde_json::json!("sendMessage"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Google OAuth access token.".into()),
            hint: Some("Connect credentials and paste token if required.".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://gmail.googleapis.com")),
            description: Some("Gmail API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "rawMessage".into(),
            display_name: "Raw Message".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Base64URL encoded MIME message.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendMessage")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/gmail/v1/users/me/messages")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.gmail".into(),
        display_name: "Gmail".into(),
        version: 1.0,
        description: "Read and send Gmail messages".into(),
        properties: gmail_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::gmail::GmailNode::new()),
    });

    let mut twilio_props = empty_props.clone();
    twilio_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("sendSms")),
            description: Some("Twilio operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Send SMS".into(),
                    value: serde_json::json!("sendSms"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "accountSid".into(),
            display_name: "Account SID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Twilio account SID.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Twilio auth token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.twilio.com")),
            description: Some("Twilio API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "to".into(),
            display_name: "To".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Destination phone number.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendSms")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "from".into(),
            display_name: "From".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Twilio sender number.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendSms")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "message".into(),
            display_name: "Message".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("SMS message text.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("sendSms")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!(
                "/2010-04-01/Accounts/{AccountSid}/Messages.json"
            )),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.twilio".into(),
        display_name: "Twilio".into(),
        version: 1.0,
        description: "Send SMS and call Twilio API".into(),
        properties: twilio_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::twilio::TwilioNode::new()),
    });

    let mut trello_props = empty_props.clone();
    trello_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listBoards")),
            description: Some("Trello operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Boards".into(),
                    value: serde_json::json!("listBoards"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Card".into(),
                    value: serde_json::json!("createCard"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Trello API token.".into()),
            hint: Some("Create token in Trello developer settings.".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.trello.com")),
            description: Some("Trello API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "listId".into(),
            display_name: "List ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Target list ID for card creation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createCard")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "name".into(),
            display_name: "Card Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Card title.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createCard")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "description".into(),
            display_name: "Description".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Optional card description.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createCard")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/1/members/me/boards")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.trello".into(),
        display_name: "Trello".into(),
        version: 1.0,
        description: "Manage Trello boards and cards".into(),
        properties: trello_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::trello::TrelloNode::new()),
    });

    let mut gitlab_props = empty_props.clone();
    gitlab_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("getProject")),
            description: Some("GitLab operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Get Project".into(),
                    value: serde_json::json!("getProject"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Issue".into(),
                    value: serde_json::json!("createIssue"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("GitLab personal access token.".into()),
            hint: Some("Generate PAT with API scope.".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://gitlab.com")),
            description: Some("GitLab base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "projectId".into(),
            display_name: "Project ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("GitLab project ID.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("getProject"),
                        serde_json::json!("createIssue"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "title".into(),
            display_name: "Title".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Issue title.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "description".into(),
            display_name: "Description".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Issue description.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/api/v4/projects")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.gitlab".into(),
        display_name: "GitLab".into(),
        version: 1.0,
        description: "Fetch projects and create GitLab issues".into(),
        properties: gitlab_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::gitlab::GitlabNode::new()),
    });

    let mut bitbucket_props = empty_props.clone();
    bitbucket_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listRepositories")),
            description: Some("Bitbucket operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Repositories".into(),
                    value: serde_json::json!("listRepositories"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Issue".into(),
                    value: serde_json::json!("createIssue"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Bitbucket app password or token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.bitbucket.org")),
            description: Some("Bitbucket API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "workspace".into(),
            display_name: "Workspace".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Workspace slug.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("listRepositories"),
                        serde_json::json!("createIssue"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "repoSlug".into(),
            display_name: "Repository".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Repository slug.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "title".into(),
            display_name: "Title".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Issue title.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/2.0/repositories")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.bitbucket".into(),
        display_name: "Bitbucket".into(),
        version: 1.0,
        description: "Use Bitbucket repositories and issues".into(),
        properties: bitbucket_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::bitbucket::BitbucketNode::new()),
    });

    let mut dropbox_props = empty_props.clone();
    dropbox_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listFolder")),
            description: Some("Dropbox operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Folder".into(),
                    value: serde_json::json!("listFolder"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Folder".into(),
                    value: serde_json::json!("createFolder"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Dropbox OAuth token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.dropboxapi.com")),
            description: Some("Dropbox API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "path".into(),
            display_name: "Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("")),
            description: Some("Folder path.".into()),
            hint: Some("Example: /New Folder".into()),
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("listFolder"),
                        serde_json::json!("createFolder"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/2/files/list_folder")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.dropbox".into(),
        display_name: "Dropbox".into(),
        version: 1.0,
        description: "List and create folders in Dropbox".into(),
        properties: dropbox_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::dropbox::DropboxNode::new()),
    });

    let mut onedrive_props = empty_props.clone();
    onedrive_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listRoot")),
            description: Some("OneDrive operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Root".into(),
                    value: serde_json::json!("listRoot"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Folder".into(),
                    value: serde_json::json!("createFolder"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Microsoft Graph access token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://graph.microsoft.com")),
            description: Some("Microsoft Graph API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "name".into(),
            display_name: "Folder Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Folder name for create operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createFolder")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/v1.0/me/drive/root/children")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.oneDrive".into(),
        display_name: "OneDrive".into(),
        version: 1.0,
        description: "Read and create folders in OneDrive".into(),
        properties: onedrive_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::onedrive::OnedriveNode::new()),
    });

    let mut linear_props = empty_props.clone();
    linear_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listTeams")),
            description: Some("Linear operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Teams".into(),
                    value: serde_json::json!("listTeams"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Issue".into(),
                    value: serde_json::json!("createIssue"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Linear API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.linear.app")),
            description: Some("Linear API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "teamId".into(),
            display_name: "Team ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Team ID for issue creation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "title".into(),
            display_name: "Title".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Issue title.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createIssue")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/graphql")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.linear".into(),
        display_name: "Linear".into(),
        version: 1.0,
        description: "List teams and create issues in Linear".into(),
        properties: linear_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::linear::LinearNode::new()),
    });

    let mut clickup_props = empty_props.clone();
    clickup_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listSpaces")),
            description: Some("ClickUp operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Spaces".into(),
                    value: serde_json::json!("listSpaces"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Task".into(),
                    value: serde_json::json!("createTask"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("ClickUp API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.clickup.com")),
            description: Some("ClickUp API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "teamId".into(),
            display_name: "Team ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Team ID for list spaces operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("listSpaces")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "listId".into(),
            display_name: "List ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("List ID for create task operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTask")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "name".into(),
            display_name: "Task Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Task name.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTask")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "description".into(),
            display_name: "Description".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Task description.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTask")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/api/v2/team/{teamId}/space")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.clickUp".into(),
        display_name: "ClickUp".into(),
        version: 1.0,
        description: "List spaces and create ClickUp tasks".into(),
        properties: clickup_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::clickup::ClickupNode::new()),
    });

    let mut monday_props = empty_props.clone();
    monday_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listBoards")),
            description: Some("Monday.com operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Boards".into(),
                    value: serde_json::json!("listBoards"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Item".into(),
                    value: serde_json::json!("createItem"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "API Call".into(),
                    value: serde_json::json!("apiCall"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "authToken".into(),
            display_name: "Auth Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Monday.com API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.monday.com")),
            description: Some("Monday.com API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "boardId".into(),
            display_name: "Board ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Board ID for create item operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createItem")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "itemName".into(),
            display_name: "Item Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("New item name.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createItem")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("GET")),
            description: Some("HTTP method for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "GET".into(),
                    value: serde_json::json!("GET"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "POST".into(),
                    value: serde_json::json!("POST"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PUT".into(),
                    value: serde_json::json!("PUT"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "PATCH".into(),
                    value: serde_json::json!("PATCH"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "DELETE".into(),
                    value: serde_json::json!("DELETE"),
                    description: None,
                },
            ]),
        },
        barqflow_core::properties::INodeProperty {
            name: "resourcePath".into(),
            display_name: "Resource Path".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("/v2")),
            description: Some("Resource path for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "queryParameters".into(),
            display_name: "Query Parameters".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional query parameters.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "headers".into(),
            display_name: "Headers".into(),
            r#type: barqflow_core::properties::NodePropertyType::Collection,
            default: Some(serde_json::json!([])),
            description: Some("Optional custom headers.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "body".into(),
            display_name: "Body".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Body for API Call operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("apiCall")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "timeout".into(),
            display_name: "Timeout (ms)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(60000)),
            description: Some("Request timeout.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.monday".into(),
        display_name: "Monday.com".into(),
        version: 1.0,
        description: "List boards and create items in Monday.com".into(),
        properties: monday_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::monday::MondayNode::new()),
    });
}
