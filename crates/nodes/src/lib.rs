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
            description: Some("The operation to perform".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![barqflow_core::properties::NodePropertyOption {
                name: "Execute Query".into(),
                value: serde_json::json!("executeQuery"),
                description: None,
            }]),
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
            options: Some(vec![barqflow_core::properties::NodePropertyOption {
                name: "Generate Text".into(),
                value: serde_json::json!("generate"),
                description: None,
            }]),
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
    notion_props.properties = vec![barqflow_core::properties::INodeProperty {
        name: "databaseId".into(),
        display_name: "Database ID".into(),
        r#type: barqflow_core::properties::NodePropertyType::String,
        default: None,
        description: Some("Notion Database ID".into()),
        hint: None,
        required: true,
        display_options: None,
        options: None,
    }];

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
    mysql_props.properties = vec![barqflow_core::properties::INodeProperty {
        name: "query".into(),
        display_name: "Query".into(),
        r#type: barqflow_core::properties::NodePropertyType::Text,
        default: None,
        description: Some("SQL query to execute".into()),
        hint: None,
        required: true,
        display_options: None,
        options: None,
    }];

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
                },
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
    google_drive_props.properties = vec![barqflow_core::properties::INodeProperty {
        name: "fileId".into(),
        display_name: "File ID".into(),
        r#type: barqflow_core::properties::NodePropertyType::String,
        default: None,
        description: Some("Google Drive File ID".into()),
        hint: None,
        required: true,
        display_options: None,
        options: None,
    }];

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
    jira_props.properties = vec![barqflow_core::properties::INodeProperty {
        name: "issueKey".into(),
        display_name: "Issue Key".into(),
        r#type: barqflow_core::properties::NodePropertyType::String,
        default: None,
        description: Some("Jira Issue Key (e.g. PROJ-123)".into()),
        hint: None,
        required: true,
        display_options: None,
        options: None,
    }];

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
    stripe_props.properties = vec![barqflow_core::properties::INodeProperty {
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
            },
        ]),
    }];

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
            ]),
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
    hubspot_props.properties = vec![barqflow_core::properties::INodeProperty {
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
            },
        ]),
    }];

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
}
