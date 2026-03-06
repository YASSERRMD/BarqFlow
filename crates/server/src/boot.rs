use crate::state::AppState;
use tracing::info;
use barqflow_api::controllers::webhooks::WebhookEndpoint;
use barqflow_core::schema::INode;

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

        for node in nodes {
            if node.r#type == "webhook" {
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
            }
        }
    }

    info!("Boot sequence completed successfully.");
    Ok(())
}
