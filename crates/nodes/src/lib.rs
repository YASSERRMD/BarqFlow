pub mod code;
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
}
