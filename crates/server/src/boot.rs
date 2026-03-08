use crate::state::AppState;
use tracing::info;
use barqflow_api::controllers::webhooks::WebhookEndpoint;
use barqflow_api::credentials_provider::RepositoryCredentialProvider;
use barqflow_core::{schema::{INode, WorkflowDef, INodeConnections, IWorkflowSettings}, types::{RunId, WorkflowId}};
use barqflow_exec::runner::{ExecutionConfig, WorkflowRunContext, WorkflowRunner};
use std::collections::HashMap;
use tokio_cron_scheduler::Job;

fn is_webhook_node_type(node_type: &str) -> bool {
    matches!(node_type, "webhook" | "barqflow-nodes.webhook")
}

pub async fn run_boot_sequence(
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting BarqFlow production boot sequence...");

    // 1. Fetch all active workflows
    let active_workflows = state.workflow_repo.find_all().await?;
    let active_count = active_workflows.iter().filter(|w| w.active).count();

    info!(
        "Found {} active workflows out of {} total.",
        active_count,
        active_workflows.len()
    );

    // 2. Mock initializing execution memory
    let mut registry_write = state.webhook_registry.write().map_err(|_| "Failed to lock WebhookRegistry")?;

    for wf in active_workflows.iter().filter(|w| w.active) {
        info!("Registering triggers for Active Workflow: {}", wf.id);

        // Parse nodes to find webhooks
        let nodes: Vec<INode> = match serde_json::from_value(wf.nodes.clone()) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!("Failed to parse nodes for workflow {}: {}", wf.id, e);
                continue;
            }
        };

        for node in &nodes {
            if is_webhook_node_type(&node.r#type) {
                // Extract path parameter, fallback to node ID if not present
                let path = node.parameters.0.get("path")
                    .and_then(|p| p.as_str())
                    .unwrap_or_else(|| node.id.0.as_str())
                    .to_string();

                let http_method = node.parameters.0.get("httpMethod")
                    .and_then(|m| m.as_str())
                    .unwrap_or("ANY")
                    .to_string();

                // Webhooks in n8n can define custom paths like "my-custom-webhook"
                registry_write.insert(path.clone(), WebhookEndpoint {
                    workflow_id: wf.id,
                    node_id: node.id.to_string(),
                    http_method,
                });

                info!("Registered webhook route: /webhook/{} -> Workflow: {}", path, wf.id);
            } else if node.r#type == "barqflow-nodes.cronTrigger" {
                let cron_expr = node.parameters.0.get("cron")
                    .and_then(|p| p.as_str())
                    .unwrap_or("0 * * * * *")
                    .to_string();

                let state_clone = state.clone();
                let wf_clone = wf.clone();
                let nodes_clone = nodes.clone();

                let job = Job::new_async(cron_expr.as_str(), move |_uuid, mut _l| {
                    let state = state_clone.clone();
                    let wf = wf_clone.clone();
                    let nodes = nodes_clone.clone();

                    Box::pin(async move {
                        let connections: HashMap<String, INodeConnections> = serde_json::from_value(wf.connections.clone()).unwrap_or_default();
                        let settings: IWorkflowSettings = serde_json::from_value(wf.settings.clone()).unwrap_or_default();

                        let workflow_def = WorkflowDef {
                            id: WorkflowId(wf.id),
                            name: wf.name.clone(),
                            nodes,
                            connections: connections.into_iter().collect(),
                            active: wf.active,
                            settings,
                        };

                        let credential_provider = std::sync::Arc::new(
                            RepositoryCredentialProvider::new(std::sync::Arc::clone(&state.credential_repo))
                        );
                        let runner = WorkflowRunner::new(state.node_registry.clone(), ExecutionConfig::default())
                            .with_credential_provider(credential_provider);
                        let ctx = WorkflowRunContext {
                            run_id: RunId::new(),
                            workflow: workflow_def,
                            static_data: None,
                            manual: false,
                        };

                        if let Err(e) = runner.run_workflow(ctx).await {
                            tracing::error!("Scheduled workflow {} execution failed: {:?}", wf.id, e);
                        }
                    })
                }).map_err(|e| format!("Failed to create scheduled job: {}", e))?;

                state.job_scheduler.add(job).await.map_err(|e| format!("Failed to add job: {}", e))?;
                info!("Registered cron trigger ({}) -> Workflow: {}", cron_expr, wf.id);
            }
        }
    }

    state.job_scheduler.start().await.map_err(|e| format!("Failed to start job scheduler: {}", e))?;
    info!("Background Job Scheduler started successfully.");

    info!("Boot sequence completed successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_webhook_node_type;

    #[test]
    fn test_webhook_type_detection_accepts_canonical_and_legacy() {
        assert!(is_webhook_node_type("barqflow-nodes.webhook"));
        assert!(is_webhook_node_type("webhook"));
        assert!(!is_webhook_node_type("barqflow-nodes.cronTrigger"));
        assert!(!is_webhook_node_type("barqflow-nodes.manualTrigger"));
    }
}
