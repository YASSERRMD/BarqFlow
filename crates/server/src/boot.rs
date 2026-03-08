use crate::state::AppState;
use barqflow_api::active_workflows::ActiveWorkflowManager;
use tracing::info;

pub async fn run_boot_sequence(
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting BarqFlow production boot sequence...");

    let manager = ActiveWorkflowManager::new(
        std::sync::Arc::clone(&state.workflow_repo),
        std::sync::Arc::clone(&state.credential_repo),
        std::sync::Arc::clone(&state.node_registry),
        std::sync::Arc::clone(&state.webhook_registry),
        state.job_scheduler.clone(),
        std::sync::Arc::clone(&state.active_cron_jobs),
    );

    manager
        .reconcile_on_boot()
        .await
        .map_err(|e| format!("Failed to reconcile active workflows on boot: {}", e))?;

    state
        .job_scheduler
        .start()
        .await
        .map_err(|e| format!("Failed to start job scheduler: {}", e))?;

    info!("Boot sequence completed successfully.");
    Ok(())
}
