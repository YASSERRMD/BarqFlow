use crate::state::AppState;
use barqflow_api::active_workflows::ActiveWorkflowManager;
use barqflow_api::controllers::executions::AppState as ExecutionControllerState;
use barqflow_api::execution_workers::start_dispatch_workers;
use tracing::info;

pub async fn run_boot_sequence(
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting BarqFlow production boot sequence...");

    let manager = ActiveWorkflowManager::new(
        std::sync::Arc::clone(&state.workflow_repo),
        ExecutionControllerState {
            execution_repo: std::sync::Arc::clone(&state.execution_repo),
            execution_dispatch_repo: std::sync::Arc::clone(&state.execution_dispatch_repo),
            execution_log_repo: std::sync::Arc::clone(&state.execution_log_repo),
            workflow_repo: std::sync::Arc::clone(&state.workflow_repo),
            node_registry: std::sync::Arc::clone(&state.node_registry),
            credential_repo: std::sync::Arc::clone(&state.credential_repo),
            governance_repo: std::sync::Arc::clone(&state.governance_repo),
            user_repo: std::sync::Arc::clone(&state.user_repo),
            workspace_repo: std::sync::Arc::clone(&state.workspace_repo),
            api_key_repo: std::sync::Arc::clone(&state.api_key_repo),
            active_executions: std::sync::Arc::clone(&state.active_executions),
            execution_events: state.execution_events.clone(),
            operations_runtime: state.operations_runtime.clone(),
        },
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

    start_dispatch_workers(ExecutionControllerState {
        execution_repo: std::sync::Arc::clone(&state.execution_repo),
        execution_dispatch_repo: std::sync::Arc::clone(&state.execution_dispatch_repo),
        execution_log_repo: std::sync::Arc::clone(&state.execution_log_repo),
        workflow_repo: std::sync::Arc::clone(&state.workflow_repo),
        node_registry: std::sync::Arc::clone(&state.node_registry),
        credential_repo: std::sync::Arc::clone(&state.credential_repo),
        governance_repo: std::sync::Arc::clone(&state.governance_repo),
        user_repo: std::sync::Arc::clone(&state.user_repo),
        workspace_repo: std::sync::Arc::clone(&state.workspace_repo),
        api_key_repo: std::sync::Arc::clone(&state.api_key_repo),
        active_executions: std::sync::Arc::clone(&state.active_executions),
        execution_events: state.execution_events.clone(),
        operations_runtime: state.operations_runtime.clone(),
    });

    info!("Boot sequence completed successfully.");
    Ok(())
}
