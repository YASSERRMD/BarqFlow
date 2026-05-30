use crate::active_workflows::ActiveCronJobs;
use crate::contracts::{
    ExtensionActionInvocationResponse, ExtensionActionResponse, ExtensionBundleResponse,
};
use crate::controllers::webhooks::WebhookRegistry;
use crate::operations::OperationsRuntime;
use crate::repositories::execution_dispatch::{ExecutionDispatchRepository, ExecutionQueueKind};
use crate::routes::ActiveExecutionManager;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct ExtensionRuntimeContext {
    pub webhook_registry: WebhookRegistry,
    pub active_cron_jobs: ActiveCronJobs,
    pub active_executions: ActiveExecutionManager,
    pub execution_dispatch_repo: Arc<ExecutionDispatchRepository>,
    pub operations_runtime: OperationsRuntime,
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
}

pub async fn invoke_extension_action(
    bundle: &ExtensionBundleResponse,
    action_id: &str,
    context: Value,
    runtime: &ExtensionRuntimeContext,
) -> Result<ExtensionActionInvocationResponse, String> {
    if !bundle.signature_status.eq_ignore_ascii_case("verified") {
        return Err(format!(
            "Extension '{}' is not trusted for runtime invocation (signature status: {}).",
            bundle.name, bundle.signature_status
        ));
    }

    if !bundle.runtime.eq_ignore_ascii_case("builtin-pack") {
        return Err(format!(
            "Runtime '{}' is not yet invokable through the capability runtime.",
            bundle.runtime
        ));
    }

    let action = bundle
        .actions
        .iter()
        .find(|candidate| candidate.id == action_id)
        .ok_or_else(|| {
            format!(
                "Extension action '{}' was not found in bundle '{}'.",
                action_id, bundle.id
            )
        })?;

    for capability in &action.required_capabilities {
        if !bundle.capabilities.iter().any(|value| value == capability) {
            return Err(format!(
                "Extension action '{}' requires capability '{}' which is not declared by the bundle.",
                action.id, capability
            ));
        }
    }

    let (status, summary, output) = match (bundle.id.as_str(), action.id.as_str()) {
        ("barqflow.ops.observability-pack", "runtime-health") => {
            let webhook_count = {
                let registry = runtime
                    .webhook_registry
                    .read()
                    .map_err(|_| "Webhook registry lock poisoned".to_string())?;
                registry.len()
            };
            let cron_job_count = {
                let cron_jobs = runtime.active_cron_jobs.read().await;
                cron_jobs
                    .values()
                    .map(|job_ids| job_ids.len())
                    .sum::<usize>()
            };
            let active_execution_count = runtime.active_executions.read().await.len();
            let queued_run_count = runtime
                .execution_dispatch_repo
                .count_open_items_by_kind(ExecutionQueueKind::Run)
                .await
                .map_err(|error| error.to_string())?
                .max(0) as usize;
            let queued_trigger_count = runtime
                .execution_dispatch_repo
                .count_open_items_by_kind(ExecutionQueueKind::Trigger)
                .await
                .map_err(|error| error.to_string())?
                .max(0) as usize;
            let dispatch = runtime.operations_runtime.dispatch_metrics_snapshot().await;
            let advice = if queued_run_count + queued_trigger_count > dispatch.worker_concurrency {
                "Queue depth is above worker concurrency. Consider scaling workers or reviewing trigger volume."
            } else {
                "Queue depth is within the current worker envelope."
            };
            let summary = format!(
                "Runtime health summary generated from {} active executions, {} queued run jobs, and {} queued trigger jobs.",
                active_execution_count, queued_run_count, queued_trigger_count
            );
            (
                "ok".to_string(),
                summary,
                json!({
                    "activeExecutions": active_execution_count,
                    "queuedRunExecutions": queued_run_count,
                    "queuedTriggerExecutions": queued_trigger_count,
                    "webhookEndpointCount": webhook_count,
                    "cronJobCount": cron_job_count,
                    "workerConcurrency": {
                        "total": dispatch.worker_concurrency,
                        "run": runtime.operations_runtime.run_worker_concurrency(),
                        "trigger": runtime.operations_runtime.trigger_worker_concurrency()
                    },
                    "advice": advice
                }),
            )
        }
        ("barqflow.ops.observability-pack", "incident-brief") => {
            let title = context
                .get("incidentTitle")
                .and_then(|value| value.as_str())
                .unwrap_or("Operational incident");
            let severity = context
                .get("severity")
                .and_then(|value| value.as_str())
                .unwrap_or("medium");
            let current_error = context
                .get("currentError")
                .and_then(|value| value.as_str())
                .unwrap_or("No explicit runtime error supplied.");
            let active_execution_count = runtime.active_executions.read().await.len();
            let summary = format!(
                "{} ({}) brief generated with {} active executions in flight.",
                title, severity, active_execution_count
            );
            (
                "ok".to_string(),
                summary,
                json!({
                    "title": title,
                    "severity": severity,
                    "currentError": current_error,
                    "brief": [
                        format!("{} active executions are currently running.", active_execution_count),
                        "Review the execution monitor for the affected workflow timeline.",
                        "Validate the credential and webhook posture before resuming traffic."
                    ],
                    "nextChecks": [
                        "Confirm whether trigger volume spiked before the incident.",
                        "Inspect the failing node's last successful run for configuration drift.",
                        "Review governance audit entries for recent credential or promotion changes."
                    ]
                }),
            )
        }
        ("barqflow.ai.automation-pack", "prompt-planner") => {
            let prompt = context
                .get("prompt")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let lower = prompt.to_ascii_lowercase();
            let mut steps = vec!["Capture the trigger and inbound context.".to_string()];
            if lower.contains("slack") {
                steps.push("Add a Slack notification or approval step.".to_string());
            }
            if lower.contains("github") {
                steps.push("Pull repository or issue context from GitHub.".to_string());
            }
            if lower.contains("openai") || lower.contains("ai") {
                steps.push("Use an LLM node for summarization or classification.".to_string());
            }
            steps.push("Persist or route the final outcome to the target system.".to_string());

            let summary = format!(
                "Prompt planner produced {} implementation steps across {} registered node types.",
                steps.len(),
                runtime.node_registry.get_all_node_names().len()
            );
            (
                "ok".to_string(),
                summary,
                json!({
                    "prompt": prompt,
                    "steps": steps,
                    "recommendedCapabilities": action.required_capabilities,
                    "nodeCatalogSize": runtime.node_registry.get_all_node_names().len()
                }),
            )
        }
        ("barqflow.ai.automation-pack", "run-diagnosis") => {
            let failing_node = context
                .get("failingNode")
                .and_then(|value| value.as_str())
                .unwrap_or("Unknown node");
            let error_message = context
                .get("error")
                .and_then(|value| value.as_str())
                .unwrap_or("No error payload provided.");
            let status = context
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("failed");
            let summary = format!(
                "Run diagnosis prepared for node '{}' in '{}' status.",
                failing_node, status
            );
            (
                "ok".to_string(),
                summary,
                json!({
                    "failingNode": failing_node,
                    "status": status,
                    "error": error_message,
                    "hypotheses": [
                        "The credential binding may be stale or missing required scopes.",
                        "A node parameter may no longer match the upstream API contract.",
                        "The execution may have resumed with outdated workflow data after a deployment."
                    ],
                    "recommendedChecks": [
                        format!("Re-test the credential used by '{}'.", failing_node),
                        "Inspect the latest execution timeline for wait/resume boundaries.",
                        "Compare the current workflow revision against the last successful snapshot."
                    ]
                }),
            )
        }
        _ => {
            return Err(format!(
                "Extension action '{}' is not implemented by the builtin runtime.",
                action.id
            ))
        }
    };

    Ok(ExtensionActionInvocationResponse {
        bundle_id: bundle.id.clone(),
        action_id: action.id.clone(),
        status,
        summary,
        capability_trace: capability_trace(action),
        output,
        signature_status: bundle.signature_status.clone(),
    })
}

fn capability_trace(action: &ExtensionActionResponse) -> Vec<String> {
    action.required_capabilities.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::execution_dispatch::ExecutionDispatchRepository;
    use sqlx::postgres::PgPoolOptions;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    fn test_bundle(signature_status: &str, action_id: &str) -> ExtensionBundleResponse {
        ExtensionBundleResponse {
            id: "barqflow.ai.automation-pack".to_string(),
            name: "AI Automation Pack".to_string(),
            vendor: "BarqFlow Labs".to_string(),
            version: "0.1.0".to_string(),
            runtime: "builtin-pack".to_string(),
            description: "Test bundle".to_string(),
            homepage: None,
            entrypoint: None,
            capabilities: vec![
                "ai:draft".to_string(),
                "nodes:llm".to_string(),
                "execution:annotate".to_string(),
            ],
            actions: vec![ExtensionActionResponse {
                id: action_id.to_string(),
                name: "Planner".to_string(),
                description: "Generate a plan".to_string(),
                required_capabilities: vec!["ai:draft".to_string(), "nodes:llm".to_string()],
            }],
            permissions: crate::contracts::ExtensionPermissionScopeResponse::default(),
            provided_assets: crate::contracts::ExtensionProvidedAssetsResponse::default(),
            source_path: "extensions/ai".to_string(),
            digest: "digest".to_string(),
            signature_status: signature_status.to_string(),
            signature_key_id: Some("barqflow-dev-rsa-2026".to_string()),
            status: "validated".to_string(),
            warnings: Vec::new(),
        }
    }

    fn test_runtime() -> ExtensionRuntimeContext {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://barqflow:barqflow@localhost/barqflow")
            .unwrap();
        let registry = Arc::new(barqflow_registry::registry::NodeRegistry::new());
        barqflow_nodes::register_all_nodes(&registry);

        ExtensionRuntimeContext {
            webhook_registry: Arc::new(std::sync::RwLock::new(HashMap::new())),
            active_cron_jobs: Arc::new(RwLock::new(HashMap::new())),
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_dispatch_repo: Arc::new(ExecutionDispatchRepository::new(pool)),
            operations_runtime: OperationsRuntime::from_env(),
            node_registry: registry,
        }
    }

    #[tokio::test]
    async fn rejects_unsigned_runtime_invocation() {
        let bundle = test_bundle("unsigned", "prompt-planner");
        let error = invoke_extension_action(&bundle, "prompt-planner", json!({}), &test_runtime())
            .await
            .unwrap_err();
        assert!(error.contains("not trusted"));
    }

    #[tokio::test]
    async fn invokes_builtin_prompt_planner_action() {
        let bundle = test_bundle("verified", "prompt-planner");
        let response = invoke_extension_action(
            &bundle,
            "prompt-planner",
            json!({ "prompt": "Summarize GitHub issues with AI and notify Slack" }),
            &test_runtime(),
        )
        .await
        .unwrap();

        assert_eq!(response.bundle_id, bundle.id);
        assert_eq!(response.action_id, "prompt-planner");
        assert_eq!(response.signature_status, "verified");
        assert!(response.summary.contains("implementation steps"));
        assert!(response.output["steps"].is_array());
    }
}
