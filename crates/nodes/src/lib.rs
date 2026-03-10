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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeSupportTier {
    Supported,
    Beta,
    Hidden,
}

pub fn register_all_credentials(registry: &barqflow_registry::registry::CredentialRegistry) {
    credentials::register_all_credentials(registry);
}

pub fn node_ui_category(name: &str) -> Option<&'static str> {
    match name {
        "n8n-nodes-base.manualTrigger"
        | "barqflow-nodes.wait"
        | "barqflow-nodes.errorTrigger"
        | "barqflow-nodes.webhook"
        | "barqflow-nodes.cronTrigger" => Some("Triggers"),
        "n8n-nodes-base.if"
        | "n8n-nodes-base.switch"
        | "n8n-nodes-base.merge"
        | "n8n-nodes-base.set"
        | "n8n-nodes-base.filter"
        | "n8n-nodes-base.itemLists"
        | "n8n-nodes-base.code" => Some("Data & Logic"),
        "n8n-nodes-base.httpRequest" | "barqflow-nodes.executeWorkflow" => Some("Core"),
        _ if name.starts_with("barqflow-nodes.") => Some("Integrations"),
        _ => None,
    }
}

pub fn node_support_tier(name: &str) -> Option<NodeSupportTier> {
    match name {
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
        | "barqflow-nodes.telegram"
        | "barqflow-nodes.googleSheets"
        | "barqflow-nodes.gmail"
        | "barqflow-nodes.twilio"
        | "barqflow-nodes.shopify"
        | "barqflow-nodes.barqDbInsert"
        | "barqflow-nodes.barqDbSearch"
        | "barqflow-nodes.barqDbDelete" => Some(NodeSupportTier::Supported),
        "barqflow-nodes.discord"
        | "barqflow-nodes.notion"
        | "barqflow-nodes.airtable"
        | "barqflow-nodes.jira"
        | "barqflow-nodes.stripe"
        | "barqflow-nodes.sendGrid"
        | "barqflow-nodes.hubspot"
        | "barqflow-nodes.asana"
        | "barqflow-nodes.googleDrive"
        | "barqflow-nodes.outlook"
        | "barqflow-nodes.mailchimp"
        | "barqflow-nodes.salesforce"
        | "barqflow-nodes.redis"
        | "barqflow-nodes.mysql"
        | "barqflow-nodes.awsS3"
        | "barqflow-nodes.trello"
        | "barqflow-nodes.gitlab"
        | "barqflow-nodes.bitbucket"
        | "barqflow-nodes.dropbox"
        | "barqflow-nodes.linear"
        | "barqflow-nodes.clickUp"
        | "barqflow-nodes.monday"
        | "barqflow-nodes.pipedrive" => Some(NodeSupportTier::Beta),
        "barqflow-nodes.oneDrive"
        | "barqflow-nodes.paypal"
        | "barqflow-nodes.zoom"
        | "barqflow-nodes.calendly"
        | "barqflow-nodes.zendesk"
        | "barqflow-nodes.intercom"
        | "barqflow-nodes.freshdesk"
        | "barqflow-nodes.quickbooks"
        | "barqflow-nodes.xero" => Some(NodeSupportTier::Hidden),
        _ => None,
    }
}

pub fn node_support_note(name: &str) -> Option<&'static str> {
    match node_support_tier(name)? {
        NodeSupportTier::Supported => Some(
            "Supported node with production-facing parameter and credential coverage in BarqFlow.",
        ),
        NodeSupportTier::Beta => Some(
            "Beta node: common operations are implemented, but depth and edge-case coverage are still expanding.",
        ),
        NodeSupportTier::Hidden => Some(
            "Hidden catalog entry until credential coverage and runtime depth are promoted.",
        ),
    }
}

pub fn node_documentation_url(name: &str) -> Option<&'static str> {
    match name {
        "n8n-nodes-base.httpRequest" => {
            Some("https://docs.n8n.io/integrations/builtin/core-nodes/n8n-nodes-base.httprequest/")
        }
        "barqflow-nodes.postgres" => Some("https://www.postgresql.org/docs/current/index.html"),
        "barqflow-nodes.openai" => Some("https://platform.openai.com/docs/overview"),
        "barqflow-nodes.ollama" => Some("https://ollama.com/library"),
        "barqflow-nodes.slack" => Some("https://api.slack.com/web"),
        "barqflow-nodes.github" => Some("https://docs.github.com/en/rest"),
        "barqflow-nodes.telegram" => Some("https://core.telegram.org/bots/api"),
        "barqflow-nodes.googleSheets" => Some("https://developers.google.com/sheets/api"),
        "barqflow-nodes.gmail" => Some("https://developers.google.com/gmail/api"),
        "barqflow-nodes.twilio" => Some("https://www.twilio.com/docs/usage/api"),
        "barqflow-nodes.shopify" => Some("https://shopify.dev/docs/api/admin-rest"),
        _ => None,
    }
}

pub fn is_node_ui_exposed(name: &str) -> bool {
    matches!(
        node_support_tier(name),
        Some(NodeSupportTier::Supported | NodeSupportTier::Beta)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        is_node_ui_exposed, node_documentation_url, node_support_tier, node_ui_category,
        register_all_nodes, NodeSupportTier,
    };
    use barqflow_registry::registry::NodeRegistry;

    fn property_names_for(registry: &NodeRegistry, node_name: &str) -> Vec<String> {
        registry
            .get_latest_node(node_name)
            .unwrap_or_else(|| panic!("node '{node_name}' should be registered"))
            .properties
            .properties
            .iter()
            .map(|property| property.name.clone())
            .collect()
    }

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
        assert!(is_node_ui_exposed("barqflow-nodes.shopify"));
        assert!(is_node_ui_exposed("barqflow-nodes.barqDbInsert"));
    }

    #[test]
    fn test_node_catalog_metadata_classifies_tiers_and_hidden_entries() {
        assert_eq!(
            node_support_tier("barqflow-nodes.openai"),
            Some(NodeSupportTier::Supported)
        );
        assert_eq!(
            node_support_tier("barqflow-nodes.monday"),
            Some(NodeSupportTier::Beta)
        );
        assert_eq!(
            node_support_tier("barqflow-nodes.paypal"),
            Some(NodeSupportTier::Hidden)
        );
        assert!(!is_node_ui_exposed("barqflow-nodes.paypal"));
        assert_eq!(node_ui_category("barqflow-nodes.webhook"), Some("Triggers"));
        assert_eq!(
            node_documentation_url("barqflow-nodes.openai"),
            Some("https://platform.openai.com/docs/overview")
        );
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

    #[test]
    fn test_tier1_node_schemas_expose_runtime_parameters() {
        let registry = NodeRegistry::new();
        register_all_nodes(&registry);

        let cases = vec![
            (
                "n8n-nodes-base.httpRequest",
                vec!["url", "method", "authentication", "responseFormat"],
            ),
            (
                "n8n-nodes-base.if",
                vec![
                    "combineOperation",
                    "conditions",
                    "operation",
                    "value1",
                    "value2",
                ],
            ),
            (
                "n8n-nodes-base.switch",
                vec![
                    "dataProperty",
                    "fallbackOutput",
                    "case0",
                    "case1",
                    "case2",
                    "case3",
                ],
            ),
            (
                "n8n-nodes-base.filter",
                vec![
                    "combineOperation",
                    "conditions",
                    "operation",
                    "value1",
                    "value2",
                ],
            ),
            (
                "n8n-nodes-base.code",
                vec!["mode", "language", "jsCode", "pythonCode"],
            ),
            ("n8n-nodes-base.manualTrigger", vec![]),
            ("barqflow-nodes.wait", vec!["resume", "amount", "unit"]),
            (
                "barqflow-nodes.webhook",
                vec!["path", "httpMethod", "responseMode"],
            ),
            ("barqflow-nodes.cronTrigger", vec!["cron"]),
            (
                "barqflow-nodes.executeWorkflow",
                vec!["workflowId", "mode", "inputData"],
            ),
        ];

        for (node_name, expected_properties) in cases {
            let property_names = property_names_for(&registry, node_name);
            for expected_property in expected_properties {
                assert!(
                    property_names.contains(&expected_property.to_string()),
                    "expected '{}' schema to expose '{}', got {:?}",
                    node_name,
                    expected_property,
                    property_names
                );
            }
        }
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

    let mut switch_props = empty_props.clone();
    switch_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "dataProperty".into(),
            display_name: "Data Property".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("route")),
            description: Some("Field name used to decide the output branch.".into()),
            hint: Some("Use a top-level JSON property from the incoming item.".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "fallbackOutput".into(),
            display_name: "Fallback Output".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(9)),
            description: Some("Output index used when no configured case matches.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "case0".into(),
            display_name: "Case 1".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("A")),
            description: Some("Value routed to output 0.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "case1".into(),
            display_name: "Case 2".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("B")),
            description: Some("Value routed to output 1.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "case2".into(),
            display_name: "Case 3".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("")),
            description: Some("Value routed to output 2.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "case3".into(),
            display_name: "Case 4".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("")),
            description: Some("Value routed to output 3.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
    ];

    let _ = registry.register_node(NodeInfo {
        name: "n8n-nodes-base.switch".into(),
        display_name: "Switch".into(),
        version: 1.0,
        description: "Route items based on matching values".into(),
        properties: switch_props,
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

    let mut filter_props = empty_props.clone();
    filter_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "combineOperation".into(),
            display_name: "Combine".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("all")),
            description: Some("How multiple filter conditions are combined.".into()),
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
            description: Some("JSON array of filter conditions.".into()),
            hint: Some(
                r#"[{"value1":"={{$json.status}}","operation":"equals","value2":"ready"}]"#.into(),
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
            description: Some("Legacy single-condition operator.".into()),
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
                    name: "Exists".into(),
                    value: serde_json::json!("exists"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Not Exists".into(),
                    value: serde_json::json!("notExists"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Larger".into(),
                    value: serde_json::json!("larger"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Larger or Equal".into(),
                    value: serde_json::json!("largerEqual"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Smaller".into(),
                    value: serde_json::json!("smaller"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Smaller or Equal".into(),
                    value: serde_json::json!("smallerEqual"),
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
        name: "n8n-nodes-base.filter".into(),
        display_name: "Filter".into(),
        version: 1.0,
        description: "Filters items based on conditions".into(),
        properties: filter_props,
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
            name: "mode".into(),
            display_name: "Mode".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("runOnceForAllItems")),
            description: Some(
                "Choose whether the script runs once for all items or once per item.".into(),
            ),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Run Once For All Items".into(),
                    value: serde_json::json!("runOnceForAllItems"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Run Once For Each Item".into(),
                    value: serde_json::json!("runOnceForEachItem"),
                    description: None,
                },
            ]),
        },
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

    let mut cron_props = empty_props.clone();
    cron_props.properties = vec![barqflow_core::properties::INodeProperty {
        name: "cron".into(),
        display_name: "Cron Expression".into(),
        r#type: barqflow_core::properties::NodePropertyType::String,
        default: Some(serde_json::json!("0 * * * * *")),
        description: Some("Six-field cron expression used by the workflow scheduler.".into()),
        hint: Some("Example: 0 */5 * * * * for every five minutes.".into()),
        required: true,
        display_options: None,
        options: None,
    }];

    let _ = registry.register_node(NodeInfo {
        name: "barqflow-nodes.cronTrigger".into(),
        display_name: "Cron Trigger".into(),
        version: 1.0,
        description: "Triggers on schedule".into(),
        properties: cron_props,
        is_trigger: true,
        max_inputs: 0,
        node_impl: Arc::new(trigger::CronTriggerNode::new("0 * * * * *")),
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
            r#type: barqflow_core::properties::NodePropertyType::LoadOptions,
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
            r#type: barqflow_core::properties::NodePropertyType::LoadOptions,
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
            name: "perPage".into(),
            display_name: "Per Page".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(50)),
            description: Some("Page size for list issues.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("listIssues")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "autoPaginate".into(),
            display_name: "Auto Paginate".into(),
            r#type: barqflow_core::properties::NodePropertyType::Boolean,
            default: Some(serde_json::json!(false)),
            description: Some("Follow response next links automatically.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("listIssues")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "maxPages".into(),
            display_name: "Max Pages".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(3)),
            description: Some("Maximum pages to fetch when auto paginate is enabled.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("listIssues")],
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

    let mut shopify_props = empty_props.clone();
    shopify_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listProducts")),
            description: Some("Shopify operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Products".into(),
                    value: serde_json::json!("listProducts"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Product".into(),
                    value: serde_json::json!("createProduct"),
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
            name: "accessToken".into(),
            display_name: "Access Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Shopify Admin API access token.".into()),
            hint: Some("Create token from Shopify app settings.".into()),
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://your-store.myshopify.com")),
            description: Some("Shopify store URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "title".into(),
            display_name: "Title".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Product title for create operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createProduct")],
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
            default: Some(serde_json::json!("/admin/api/2024-01/products.json")),
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
        name: "barqflow-nodes.shopify".into(),
        display_name: "Shopify".into(),
        version: 1.0,
        description: "List and create Shopify products".into(),
        properties: shopify_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::shopify::ShopifyNode::new()),
    });

    let mut paypal_props = empty_props.clone();
    paypal_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("createOrder")),
            description: Some("PayPal operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Order".into(),
                    value: serde_json::json!("createOrder"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Capture Order".into(),
                    value: serde_json::json!("captureOrder"),
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
            description: Some("PayPal OAuth token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api-m.sandbox.paypal.com")),
            description: Some("PayPal API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "orderId".into(),
            display_name: "Order ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Order ID for capture operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("captureOrder")],
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
            default: Some(serde_json::json!("/v2/checkout/orders")),
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
        name: "barqflow-nodes.paypal".into(),
        display_name: "PayPal".into(),
        version: 1.0,
        description: "Create and capture PayPal orders".into(),
        properties: paypal_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::paypal::PaypalNode::new()),
    });

    let mut zoom_props = empty_props.clone();
    zoom_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listMeetings")),
            description: Some("Zoom operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Meetings".into(),
                    value: serde_json::json!("listMeetings"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Meeting".into(),
                    value: serde_json::json!("createMeeting"),
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
            description: Some("Zoom OAuth/JWT token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.zoom.us")),
            description: Some("Zoom API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "topic".into(),
            display_name: "Topic".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Meeting topic for create operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createMeeting")],
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
            default: Some(serde_json::json!("/v2/users/me/meetings")),
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
        name: "barqflow-nodes.zoom".into(),
        display_name: "Zoom".into(),
        version: 1.0,
        description: "List and create Zoom meetings".into(),
        properties: zoom_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::zoom::ZoomNode::new()),
    });

    let mut calendly_props = empty_props.clone();
    calendly_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listEventTypes")),
            description: Some("Calendly operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Event Types".into(),
                    value: serde_json::json!("listEventTypes"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "List Scheduled Events".into(),
                    value: serde_json::json!("listScheduledEvents"),
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
            description: Some("Calendly personal access token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.calendly.com")),
            description: Some("Calendly API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
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
            default: Some(serde_json::json!("/event_types")),
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
        name: "barqflow-nodes.calendly".into(),
        display_name: "Calendly".into(),
        version: 1.0,
        description: "Read Calendly event types and scheduled events".into(),
        properties: calendly_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::calendly::CalendlyNode::new()),
    });

    let mut zendesk_props = empty_props.clone();
    zendesk_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listTickets")),
            description: Some("Zendesk operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Tickets".into(),
                    value: serde_json::json!("listTickets"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Ticket".into(),
                    value: serde_json::json!("createTicket"),
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
            description: Some("Zendesk API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://example.zendesk.com")),
            description: Some("Zendesk subdomain URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "subject".into(),
            display_name: "Subject".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Ticket subject.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTicket")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "comment".into(),
            display_name: "Comment".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Ticket comment/body.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTicket")],
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
            default: Some(serde_json::json!("/api/v2/tickets.json")),
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
        name: "barqflow-nodes.zendesk".into(),
        display_name: "Zendesk".into(),
        version: 1.0,
        description: "List and create Zendesk tickets".into(),
        properties: zendesk_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::zendesk::ZendeskNode::new()),
    });

    let mut intercom_props = empty_props.clone();
    intercom_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listContacts")),
            description: Some("Intercom operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Contacts".into(),
                    value: serde_json::json!("listContacts"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Contact".into(),
                    value: serde_json::json!("createContact"),
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
            description: Some("Intercom API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.intercom.io")),
            description: Some("Intercom API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "email".into(),
            display_name: "Email".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Contact email for create operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createContact")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "name".into(),
            display_name: "Name".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Contact name for create operation.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createContact")],
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
            default: Some(serde_json::json!("/contacts")),
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
        name: "barqflow-nodes.intercom".into(),
        display_name: "Intercom".into(),
        version: 1.0,
        description: "List and create Intercom contacts".into(),
        properties: intercom_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::intercom::IntercomNode::new()),
    });

    let mut freshdesk_props = empty_props.clone();
    freshdesk_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listTickets")),
            description: Some("Freshdesk operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Tickets".into(),
                    value: serde_json::json!("listTickets"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Ticket".into(),
                    value: serde_json::json!("createTicket"),
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
            description: Some("Freshdesk API key.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://example.freshdesk.com")),
            description: Some("Freshdesk domain URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "subject".into(),
            display_name: "Subject".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Ticket subject.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTicket")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "email".into(),
            display_name: "Email".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Requester email.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTicket")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "description".into(),
            display_name: "Description".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: None,
            description: Some("Ticket description.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createTicket")],
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
            default: Some(serde_json::json!("/api/v2/tickets")),
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
        name: "barqflow-nodes.freshdesk".into(),
        display_name: "Freshdesk".into(),
        version: 1.0,
        description: "List and create Freshdesk tickets".into(),
        properties: freshdesk_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::freshdesk::FreshdeskNode::new()),
    });

    let mut pipedrive_props = empty_props.clone();
    pipedrive_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listDeals")),
            description: Some("Pipedrive operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Deals".into(),
                    value: serde_json::json!("listDeals"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Deal".into(),
                    value: serde_json::json!("createDeal"),
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
            name: "apiToken".into(),
            display_name: "API Token".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Pipedrive API token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.pipedrive.com")),
            description: Some("Pipedrive API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "title".into(),
            display_name: "Title".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Deal title for create operation.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("createDeal")],
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
            default: Some(serde_json::json!("/api/v1/deals")),
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
        name: "barqflow-nodes.pipedrive".into(),
        display_name: "Pipedrive".into(),
        version: 1.0,
        description: "List and create Pipedrive deals".into(),
        properties: pipedrive_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::pipedrive::PipedriveNode::new()),
    });

    let mut quickbooks_props = empty_props.clone();
    quickbooks_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listCustomers")),
            description: Some("QuickBooks operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Customers".into(),
                    value: serde_json::json!("listCustomers"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Invoice".into(),
                    value: serde_json::json!("createInvoice"),
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
            description: Some("QuickBooks OAuth access token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "companyId".into(),
            display_name: "Company ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("QuickBooks realm/company ID.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://quickbooks.api.intuit.com")),
            description: Some("QuickBooks API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
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
            default: Some(serde_json::json!("/v3/company/{companyId}/query")),
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
        name: "barqflow-nodes.quickbooks".into(),
        display_name: "QuickBooks".into(),
        version: 1.0,
        description: "Query customers and create invoices in QuickBooks".into(),
        properties: quickbooks_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::quickbooks::QuickbooksNode::new()),
    });

    let mut xero_props = empty_props.clone();
    xero_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("listContacts")),
            description: Some("Xero operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "List Contacts".into(),
                    value: serde_json::json!("listContacts"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Create Invoice".into(),
                    value: serde_json::json!("createInvoice"),
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
            description: Some("Xero OAuth access token.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "tenantId".into(),
            display_name: "Tenant ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Xero tenant/organization ID.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "baseUrl".into(),
            display_name: "Base URL".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("https://api.xero.com")),
            description: Some("Xero API base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
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
            default: Some(serde_json::json!("/api.xro/2.0/Contacts")),
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
        name: "barqflow-nodes.xero".into(),
        display_name: "Xero".into(),
        version: 1.0,
        description: "Read contacts and create invoices in Xero".into(),
        properties: xero_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::xero::XeroNode::new()),
    });

    let mut barqdb_insert_props = empty_props.clone();
    barqdb_insert_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("insert")),
            description: Some("BarqDB insert operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Insert".into(),
                    value: serde_json::json!("insert"),
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
            name: "baseUrl".into(),
            display_name: "Base URL Override".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Optional override for credential base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "collection".into(),
            display_name: "Collection".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Target BarqDB collection.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("insert")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "embedField".into(),
            display_name: "Embed Field".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("text")),
            description: Some("JSON field to embed.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("insert")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "metadataFields".into(),
            display_name: "Metadata Fields".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("")),
            description: Some("Comma-separated metadata field names.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("insert")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "embeddingModel".into(),
            display_name: "Embedding Model".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: Some(serde_json::json!("text-embedding-3-small")),
            description: Some("Embedding model identifier.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("insert")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "item".into(),
            display_name: "Item (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Optional item payload when no input item is present.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("insert")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("POST")),
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
            default: Some(serde_json::json!("/v1/collections/default/items")),
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
        name: "barqflow-nodes.barqDbInsert".into(),
        display_name: "BarqDB Insert".into(),
        version: 1.0,
        description: "Insert items into BarqDB collections with embeddings".into(),
        properties: barqdb_insert_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::barqdb_insert::BarqDbInsertNode::new()),
    });

    let mut barqdb_search_props = empty_props.clone();
    barqdb_search_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("search")),
            description: Some("BarqDB search operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Search".into(),
                    value: serde_json::json!("search"),
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
            name: "baseUrl".into(),
            display_name: "Base URL Override".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Optional override for credential base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "collection".into(),
            display_name: "Collection".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Target BarqDB collection.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("search")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "query".into(),
            display_name: "Query".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Semantic search query text.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("search")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "topK".into(),
            display_name: "Top K".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(5)),
            description: Some("Number of results to return.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("search")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "scoreThreshold".into(),
            display_name: "Score Threshold".into(),
            r#type: barqflow_core::properties::NodePropertyType::Number,
            default: Some(serde_json::json!(0.0)),
            description: Some("Minimum similarity score.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("search")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "filters".into(),
            display_name: "Filters (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Optional metadata filter object.".into()),
            hint: None,
            required: false,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("search")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("POST")),
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
            default: Some(serde_json::json!("/v1/collections/default/search")),
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
        name: "barqflow-nodes.barqDbSearch".into(),
        display_name: "BarqDB Search".into(),
        version: 1.0,
        description: "Semantic search in BarqDB vector collections".into(),
        properties: barqdb_search_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::barqdb_search::BarqDbSearchNode::new()),
    });

    let mut barqdb_delete_props = empty_props.clone();
    barqdb_delete_props.properties = vec![
        barqflow_core::properties::INodeProperty {
            name: "operation".into(),
            display_name: "Operation".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("deleteById")),
            description: Some("BarqDB delete operation.".into()),
            hint: None,
            required: true,
            display_options: None,
            options: Some(vec![
                barqflow_core::properties::NodePropertyOption {
                    name: "Delete By ID".into(),
                    value: serde_json::json!("deleteById"),
                    description: None,
                },
                barqflow_core::properties::NodePropertyOption {
                    name: "Delete By Filter".into(),
                    value: serde_json::json!("deleteByFilter"),
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
            name: "baseUrl".into(),
            display_name: "Base URL Override".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Optional override for credential base URL.".into()),
            hint: None,
            required: false,
            display_options: None,
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "collection".into(),
            display_name: "Collection".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Target BarqDB collection.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![
                        serde_json::json!("deleteById"),
                        serde_json::json!("deleteByFilter"),
                    ],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "id".into(),
            display_name: "ID".into(),
            r#type: barqflow_core::properties::NodePropertyType::String,
            default: None,
            description: Some("Item ID for delete by ID.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("deleteById")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "filter".into(),
            display_name: "Filter (JSON)".into(),
            r#type: barqflow_core::properties::NodePropertyType::Text,
            default: Some(serde_json::json!("{}")),
            description: Some("Filter object for delete by filter.".into()),
            hint: None,
            required: true,
            display_options: Some(barqflow_core::properties::NodeDisplayOptions {
                r#show: Some(barqflow_core::properties::NodeDisplayCondition {
                    property: "operation".into(),
                    values: vec![serde_json::json!("deleteByFilter")],
                }),
            }),
            options: None,
        },
        barqflow_core::properties::INodeProperty {
            name: "method".into(),
            display_name: "Method".into(),
            r#type: barqflow_core::properties::NodePropertyType::Options,
            default: Some(serde_json::json!("POST")),
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
            default: Some(serde_json::json!("/v1/collections/default/items/delete")),
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
        name: "barqflow-nodes.barqDbDelete".into(),
        display_name: "BarqDB Delete".into(),
        version: 1.0,
        description: "Delete vectors/documents from BarqDB".into(),
        properties: barqdb_delete_props,
        is_trigger: false,
        max_inputs: 1,
        node_impl: Arc::new(integration::barqdb_delete::BarqDbDeleteNode::new()),
    });
}
