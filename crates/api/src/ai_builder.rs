use crate::contracts::{AiWorkflowDraftResponse, ExtensionBundleResponse};
use barqflow_registry::registry::NodeRegistry;
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub fn generate_workflow_draft(
    prompt: &str,
    node_registry: &NodeRegistry,
    extensions: &[ExtensionBundleResponse],
) -> Result<AiWorkflowDraftResponse, String> {
    let trimmed = prompt.trim();
    if trimmed.len() < 12 {
        return Err("Prompt must be at least 12 characters so the builder has enough intent to plan against.".to_string());
    }

    let normalized = trimmed.to_lowercase();
    let mut nodes = Vec::new();
    let mut rationale = Vec::new();
    let mut warnings = Vec::new();
    let mut suggested_tags = BTreeSet::new();
    let mut required_credentials = BTreeSet::new();

    let trigger = select_trigger(trimmed, &normalized, &mut rationale, &mut suggested_tags);
    nodes.push(trigger);

    if should_add_http_fetch(&normalized) {
        nodes.push(build_http_request_node());
        rationale.push("Included an HTTP Request node because the prompt implies polling or enriching from an external API.".to_string());
        suggested_tags.insert("integration".to_string());
    }

    if normalized.contains("github") {
        nodes.push(build_github_node(&normalized));
        rationale.push("Added GitHub as the source system because the prompt explicitly references repository or issue data.".to_string());
        suggested_tags.insert("engineering".to_string());
        suggested_tags.insert("github".to_string());
        required_credentials.insert("githubApi".to_string());
    }

    if should_add_ai_step(&normalized) {
        if prefers_ollama(&normalized) {
            nodes.push(build_ollama_node(trimmed));
            rationale.push("Selected Ollama because the prompt signals local, private, or offline model execution.".to_string());
            suggested_tags.insert("local-ai".to_string());
        } else {
            nodes.push(build_openai_node(trimmed));
            rationale.push("Inserted an OpenAI step because the prompt asks for summarization, classification, extraction, or drafting.".to_string());
            suggested_tags.insert("ai".to_string());
            required_credentials.insert("openAiApi".to_string());
        }
    }

    if should_add_gate(&normalized) {
        nodes.push(build_if_node(&normalized));
        rationale.push("Inserted an If gate because the prompt describes conditional routing such as only high priority, critical, or qualified items.".to_string());
        warnings.push("The generated If node is connected on the primary branch only. Add explicit false-branch handling before production rollout.".to_string());
        suggested_tags.insert("triage".to_string());
    }

    if let Some(delivery) = select_delivery(&normalized) {
        nodes.push(delivery.node);
        rationale.push(delivery.rationale.to_string());
        for tag in delivery.tags {
            suggested_tags.insert(tag.to_string());
        }
        for credential in delivery.required_credentials {
            required_credentials.insert(credential.to_string());
        }
    } else if contains_any(&normalized, &["notify", "alert", "send", "post", "message"]) {
        nodes.push(build_slack_node());
        rationale.push("Defaulted the delivery step to Slack because the prompt requires notification but does not name a concrete channel.".to_string());
        warnings.push("Slack was selected as the default notification channel. Swap it if the destination system should be different.".to_string());
        suggested_tags.insert("ops".to_string());
        required_credentials.insert("slackApi".to_string());
    }

    if nodes.len() == 1 {
        nodes.push(build_openai_node(trimmed));
        rationale.push("Added a single AI step so the draft is actionable even though the prompt did not name a concrete source or delivery system.".to_string());
        warnings.push("This prompt did not specify a source system or delivery target. Refine it if you need a fully wired operational workflow.".to_string());
        suggested_tags.insert("ai".to_string());
        required_credentials.insert("openAiApi".to_string());
    }

    let node_types = nodes
        .iter()
        .filter_map(|node| node.get("type").and_then(Value::as_str))
        .collect::<Vec<_>>();

    for node_type in &node_types {
        if node_registry.get_latest_node(node_type).is_none() {
            warnings.push(format!(
                "The generated node type '{}' is not registered in the current BarqFlow build.",
                node_type
            ));
        }
    }

    let name = workflow_name_from_prompt(trimmed, &normalized);
    let connections = build_linear_connections(&nodes);
    let settings = json!({
        "saveExecutionProgress": true,
        "saveManualExecutions": true,
        "timezone": "Asia/Dubai"
    });
    let recommended_extensions = recommended_extensions_for_nodes(&node_types, extensions);

    let summary = summarize_workflow(&normalized, &node_types);

    Ok(AiWorkflowDraftResponse {
        generator: "heuristic-v1".to_string(),
        name,
        summary,
        rationale,
        warnings,
        suggested_tags: suggested_tags.into_iter().collect(),
        required_credentials: required_credentials.into_iter().collect(),
        recommended_extensions,
        nodes: Value::Array(nodes),
        connections,
        settings,
    })
}

struct DeliverySelection {
    node: Value,
    rationale: &'static str,
    tags: &'static [&'static str],
    required_credentials: &'static [&'static str],
}

fn select_trigger(
    prompt: &str,
    normalized: &str,
    rationale: &mut Vec<String>,
    tags: &mut BTreeSet<String>,
) -> Value {
    if contains_any(
        normalized,
        &[
            "webhook",
            "lead",
            "form submission",
            "incoming event",
            "callback",
        ],
    ) {
        rationale.push(
            "Chose a Webhook trigger because the prompt describes an inbound event source."
                .to_string(),
        );
        tags.insert("webhook".to_string());
        return json!({
            "id": "trigger-webhook-1",
            "name": "Inbound Event",
            "type": "barqflow-nodes.webhook",
            "typeVersion": 1,
            "position": [120, 240],
            "parameters": {
                "path": slugify(prompt),
                "httpMethod": "POST",
                "responseMode": "onReceived",
                "responseCode": 200,
                "responseData": "{\"accepted\":true}"
            },
            "credentials": [],
            "disabled": false
        });
    }

    if contains_any(
        normalized,
        &[
            "hourly", "daily", "weekly", "nightly", "every ", "cron", "schedule",
        ],
    ) {
        rationale.push(
            "Chose a Cron trigger because the prompt reads like a scheduled automation."
                .to_string(),
        );
        tags.insert("scheduled".to_string());
        return json!({
            "id": "trigger-cron-1",
            "name": "Schedule",
            "type": "barqflow-nodes.cronTrigger",
            "typeVersion": 1,
            "position": [120, 240],
            "parameters": {
                "cron": cron_expression_for_prompt(normalized)
            },
            "credentials": [],
            "disabled": false
        });
    }

    rationale.push(
        "Chose a Manual Trigger because the prompt does not clearly specify an event source yet."
            .to_string(),
    );
    tags.insert("manual".to_string());
    json!({
        "id": "trigger-manual-1",
        "name": "Manual Trigger",
        "type": "n8n-nodes-base.manualTrigger",
        "typeVersion": 1,
        "position": [120, 240],
        "parameters": {},
        "credentials": [],
        "disabled": false
    })
}

fn should_add_http_fetch(normalized: &str) -> bool {
    contains_any(
        normalized,
        &[
            "status page",
            "status endpoint",
            "fetch api",
            "call api",
            "poll",
            "http",
            "rest api",
            "endpoint",
        ],
    )
}

fn should_add_ai_step(normalized: &str) -> bool {
    contains_any(
        normalized,
        &[
            "ai",
            "llm",
            "summarize",
            "summary",
            "classify",
            "classification",
            "extract",
            "rewrite",
            "draft",
            "diagnose",
            "triage",
        ],
    )
}

fn prefers_ollama(normalized: &str) -> bool {
    contains_any(
        normalized,
        &[
            "ollama",
            "local model",
            "offline",
            "private model",
            "on-prem",
        ],
    )
}

fn should_add_gate(normalized: &str) -> bool {
    contains_any(
        normalized,
        &[
            "only if",
            "only when",
            "critical",
            "high priority",
            "high-intent",
            "qualified",
            "severity",
            "score",
            "threshold",
        ],
    )
}

fn select_delivery(normalized: &str) -> Option<DeliverySelection> {
    if contains_any(normalized, &["slack", "channel", "incident room"]) {
        return Some(DeliverySelection {
            node: build_slack_node(),
            rationale: "Added Slack as the delivery layer because the prompt explicitly names a chat-room style notification handoff.",
            tags: &["ops", "slack"],
            required_credentials: &["slackApi"],
        });
    }

    if contains_any(normalized, &["telegram", "bot message"]) {
        return Some(DeliverySelection {
            node: build_telegram_node(),
            rationale:
                "Added Telegram because the prompt asks for a bot-driven messaging destination.",
            tags: &["telegram", "messaging"],
            required_credentials: &["telegramApi"],
        });
    }

    if contains_any(normalized, &["sms", "text message", "twilio"]) {
        return Some(DeliverySelection {
            node: build_twilio_node(),
            rationale: "Added Twilio because the prompt requires SMS delivery.",
            tags: &["sms", "twilio"],
            required_credentials: &["twilioApi"],
        });
    }

    None
}

fn build_http_request_node() -> Value {
    json!({
        "id": "http-request-1",
        "name": "Fetch Source",
        "type": "n8n-nodes-base.httpRequest",
        "typeVersion": 1,
        "position": [380, 240],
        "parameters": {
            "url": "https://api.example.com/resource",
            "method": "GET",
            "headers": [],
            "queryParameters": [],
            "body": "",
            "authentication": "none",
            "responseFormat": "json",
            "timeout": 30000
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_github_node(normalized: &str) -> Value {
    let operation = if contains_any(normalized, &["release", "issue", "bug", "backlog"]) {
        "listIssues"
    } else {
        "getRepo"
    };

    json!({
        "id": "github-1",
        "name": "GitHub Source",
        "type": "barqflow-nodes.github",
        "typeVersion": 1,
        "position": [380, 240],
        "parameters": {
            "operation": operation,
            "baseUrl": "https://api.github.com",
            "owner": "your-org",
            "repo": "your-repo",
            "perPage": 20,
            "autoPaginate": true,
            "maxPages": 2,
            "timeout": 60000
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_openai_node(prompt: &str) -> Value {
    json!({
        "id": "openai-1",
        "name": "AI Analysis",
        "type": "barqflow-nodes.openai",
        "typeVersion": 1,
        "position": [640, 240],
        "parameters": {
            "operation": "chatCompletion",
            "baseUrl": "https://api.openai.com/v1",
            "model": "gpt-4o-mini",
            "systemPrompt": "Structure the incoming payload into a concise operational summary with next actions.",
            "prompt": format!("Use the incoming workflow data to fulfil this request: {}", prompt),
            "temperature": 0.2,
            "maxTokens": 600,
            "timeout": 60000
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_ollama_node(prompt: &str) -> Value {
    json!({
        "id": "ollama-1",
        "name": "Local AI Analysis",
        "type": "barqflow-nodes.ollama",
        "typeVersion": 1,
        "position": [640, 240],
        "parameters": {
            "baseUrl": "http://localhost:11434",
            "operation": "generate",
            "model": "llama3.2",
            "systemPrompt": "Summarize the incoming payload into actionable operational steps.",
            "prompt": format!("Use the incoming workflow data to fulfil this request: {}", prompt),
            "temperature": 0.2,
            "timeout": 60000
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_if_node(normalized: &str) -> Value {
    let (value1, operation, value2) = if contains_any(normalized, &["critical", "severity"]) {
        (
            "={{$json.severity || $json.status}}",
            "equals",
            json!("critical"),
        )
    } else {
        (
            "={{$json.score || $json.priority || 0}}",
            "larger",
            json!(70),
        )
    };

    json!({
        "id": "if-1",
        "name": "Route High Signal",
        "type": "n8n-nodes-base.if",
        "typeVersion": 1,
        "position": [900, 240],
        "parameters": {
            "combineOperation": "all",
            "conditions": [{
                "value1": value1,
                "operation": operation,
                "value2": value2
            }],
            "operation": operation,
            "value1": value1,
            "value2": if operation == "larger" { json!("70") } else { value2.clone() }
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_slack_node() -> Value {
    json!({
        "id": "slack-1",
        "name": "Notify Slack",
        "type": "barqflow-nodes.slack",
        "typeVersion": 1,
        "position": [1160, 240],
        "parameters": {
            "operation": "postMessage",
            "baseUrl": "https://slack.com",
            "channel": "#ops-automation",
            "text": "BarqFlow AI Studio generated a draft notification. Replace this with production-ready message formatting.",
            "timeout": 60000
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_telegram_node() -> Value {
    json!({
        "id": "telegram-1",
        "name": "Notify Telegram",
        "type": "barqflow-nodes.telegram",
        "typeVersion": 1,
        "position": [1160, 240],
        "parameters": {
            "operation": "sendMessage",
            "baseUrl": "https://api.telegram.org",
            "chatId": "replace-with-chat-id",
            "text": "BarqFlow AI Studio generated a draft Telegram notification.",
            "timeout": 60000
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_twilio_node() -> Value {
    json!({
        "id": "twilio-1",
        "name": "Send SMS",
        "type": "barqflow-nodes.twilio",
        "typeVersion": 1,
        "position": [1160, 240],
        "parameters": {
            "operation": "sendSms",
            "baseUrl": "https://api.twilio.com",
            "to": "+10000000000",
            "from": "+10000000000",
            "message": "BarqFlow AI Studio generated a draft SMS notification.",
            "timeout": 60000
        },
        "credentials": [],
        "disabled": false
    })
}

fn build_linear_connections(nodes: &[Value]) -> Value {
    let mut map = serde_json::Map::new();

    for window in nodes.windows(2) {
        let Some(source_name) = window[0].get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(target_name) = window[1].get("name").and_then(Value::as_str) else {
            continue;
        };
        map.insert(
            source_name.to_string(),
            json!({
                "main": [[{
                    "node": target_name,
                    "type": "main",
                    "index": 0
                }]]
            }),
        );
    }

    Value::Object(map)
}

fn recommended_extensions_for_nodes(
    node_types: &[&str],
    extensions: &[ExtensionBundleResponse],
) -> Vec<String> {
    let requested = node_types.iter().copied().collect::<BTreeSet<_>>();
    extensions
        .iter()
        .filter(|bundle| {
            bundle
                .provided_assets
                .nodes
                .iter()
                .any(|node_type| requested.contains(node_type.as_str()))
        })
        .map(|bundle| bundle.id.clone())
        .collect()
}

fn summarize_workflow(normalized: &str, node_types: &[&str]) -> String {
    let trigger = if node_types.contains(&"barqflow-nodes.webhook") {
        "Webhook-triggered"
    } else if node_types.contains(&"barqflow-nodes.cronTrigger") {
        "Scheduled"
    } else {
        "Manual"
    };

    let ai = if node_types.contains(&"barqflow-nodes.openai") {
        " with OpenAI analysis"
    } else if node_types.contains(&"barqflow-nodes.ollama") {
        " with local model analysis"
    } else {
        ""
    };

    let destination = if node_types.contains(&"barqflow-nodes.slack") {
        " delivered to Slack"
    } else if node_types.contains(&"barqflow-nodes.telegram") {
        " delivered to Telegram"
    } else if node_types.contains(&"barqflow-nodes.twilio") {
        " delivered by SMS"
    } else {
        ""
    };

    if contains_any(normalized, &["github"]) {
        format!("{trigger} GitHub workflow{ai}{destination}.")
    } else if contains_any(normalized, &["status", "api", "endpoint"]) {
        format!("{trigger} API monitoring workflow{ai}{destination}.")
    } else {
        format!("{trigger} automation draft{ai}{destination}.")
    }
}

fn workflow_name_from_prompt(prompt: &str, normalized: &str) -> String {
    if contains_any(normalized, &["github", "release"]) {
        return "GitHub Release Operations".to_string();
    }
    if contains_any(normalized, &["lead", "webhook"]) {
        return "Webhook Intake Triage".to_string();
    }
    if contains_any(normalized, &["incident", "status", "alert"]) {
        return "Incident Response Monitor".to_string();
    }

    let significant = prompt
        .split_whitespace()
        .take(5)
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    if significant.is_empty() {
        "AI Workflow Draft".to_string()
    } else {
        format!("{} Workflow", significant)
    }
}

fn cron_expression_for_prompt(normalized: &str) -> &'static str {
    if contains_any(normalized, &["every 5", "five minutes"]) {
        "0 */5 * * * *"
    } else if contains_any(normalized, &["hourly", "every hour"]) {
        "0 0 * * * *"
    } else if contains_any(normalized, &["weekly", "every monday"]) {
        "0 0 9 * * 1"
    } else if contains_any(normalized, &["nightly", "every night"]) {
        "0 0 1 * * *"
    } else {
        "0 0 9 * * *"
    }
}

fn slugify(input: &str) -> String {
    let slug = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "workflow-intake".to_string()
    } else {
        slug.split('-')
            .filter(|part| !part.is_empty())
            .take(4)
            .collect::<Vec<_>>()
            .join("-")
    }
}

fn contains_any(input: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| input.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{ExtensionPermissionScopeResponse, ExtensionProvidedAssetsResponse};
    use barqflow_registry::registry::NodeRegistry;

    fn test_extension(id: &str, nodes: &[&str]) -> ExtensionBundleResponse {
        ExtensionBundleResponse {
            id: id.to_string(),
            name: id.to_string(),
            vendor: "BarqFlow".to_string(),
            version: "0.1.0".to_string(),
            runtime: "builtin-pack".to_string(),
            description: "Test bundle".to_string(),
            homepage: None,
            entrypoint: None,
            capabilities: vec![],
            permissions: ExtensionPermissionScopeResponse {
                network: vec![],
                credentials: vec![],
                workflow: vec![],
                filesystem: vec![],
            },
            provided_assets: ExtensionProvidedAssetsResponse {
                nodes: nodes.iter().map(|node| (*node).to_string()).collect(),
                templates: vec![],
                panels: vec![],
            },
            source_path: "extensions/test/barqflow-plugin.json".to_string(),
            digest: "digest".to_string(),
            signature_status: "verified".to_string(),
            signature_key_id: Some("test-key".to_string()),
            status: "validated".to_string(),
            warnings: vec![],
        }
    }

    #[test]
    fn generate_workflow_draft_builds_github_ai_flow() {
        let registry = NodeRegistry::new();
        barqflow_nodes::register_all_nodes(&registry);
        let draft = generate_workflow_draft(
            "Every morning fetch GitHub issues, summarize blockers with AI, and notify Slack.",
            &registry,
            &[test_extension(
                "barqflow.ai.pack",
                &[
                    "barqflow-nodes.github",
                    "barqflow-nodes.openai",
                    "barqflow-nodes.slack",
                ],
            )],
        )
        .unwrap();

        let nodes = draft.nodes.as_array().unwrap();
        assert!(nodes
            .iter()
            .any(|node| node.get("type") == Some(&json!("barqflow-nodes.github"))));
        assert!(nodes
            .iter()
            .any(|node| node.get("type") == Some(&json!("barqflow-nodes.openai"))));
        assert!(nodes
            .iter()
            .any(|node| node.get("type") == Some(&json!("barqflow-nodes.slack"))));
        assert!(draft
            .recommended_extensions
            .contains(&"barqflow.ai.pack".to_string()));
    }

    #[test]
    fn generate_workflow_draft_prefers_webhook_for_inbound_prompts() {
        let registry = NodeRegistry::new();
        barqflow_nodes::register_all_nodes(&registry);
        let draft = generate_workflow_draft(
            "Receive webhook leads, score high-intent submissions with AI, and send SMS for urgent ones.",
            &registry,
            &[],
        )
        .unwrap();

        let nodes = draft.nodes.as_array().unwrap();
        assert_eq!(nodes[0].get("type"), Some(&json!("barqflow-nodes.webhook")));
        assert!(nodes
            .iter()
            .any(|node| node.get("type") == Some(&json!("n8n-nodes-base.if"))));
        assert!(nodes
            .iter()
            .any(|node| node.get("type") == Some(&json!("barqflow-nodes.twilio"))));
    }
}
