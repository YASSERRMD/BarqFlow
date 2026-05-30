use crate::active_workflows::ActiveCronJobs;
use crate::auth::{require_authenticated_user, require_workspace_role, AuthenticatedUser};
use crate::contracts::{
    ExecutionDispatchMetricsResponse, ExecutionPruningStatusResponse, OperationsOverviewResponse,
    PruneExecutionsResponse, RuntimeSettingsResponse, TelemetrySettingsResponse,
};
use crate::controllers::webhooks::{WebhookEndpoint, WebhookRegistry};
use crate::operations::{
    run_execution_pruning, ExecutionDispatchMetricsSnapshot, ExecutionDispatchMode,
    ExecutionPruningSnapshot, OperationsRuntime, TelemetrySnapshot,
};
use crate::repositories::{
    api_key::ApiKeyRepository,
    execution::ExecutionRepository,
    execution_dispatch::{ExecutionDispatchRepository, ExecutionQueueKind},
    execution_log::ExecutionLogRepository,
    workspace::WorkspaceRepository,
};
use crate::routes::ActiveExecutionManager;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use barqflow_db::users::UserRepo;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_registry: Arc<barqflow_registry::registry::CredentialRegistry>,
    pub execution_repo: Arc<ExecutionRepository>,
    pub execution_dispatch_repo: Arc<ExecutionDispatchRepository>,
    pub execution_log_repo: Arc<ExecutionLogRepository>,
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
    pub webhook_registry: WebhookRegistry,
    pub active_cron_jobs: ActiveCronJobs,
    pub active_executions: ActiveExecutionManager,
    pub operations_runtime: OperationsRuntime,
}

pub fn settings_routes(state: AppState) -> Router {
    Router::new()
        .route("/settings/runtime", get(get_runtime_settings))
        .route("/settings/operations", get(get_operations_overview))
        .route("/settings/operations/prune", post(prune_operations_data))
        .with_state(state)
}

async fn get_runtime_settings(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<RuntimeSettingsResponse>, (StatusCode, String)> {
    let _auth = require_settings_auth(&headers, &state).await?;

    let encryption_key_configured = std::env::var("BARQFLOW_ENCRYPTION_KEY")
        .map(|key| key.len() >= 32)
        .unwrap_or(false);
    let dispatch = state.operations_runtime.dispatch_metrics_snapshot().await;
    let pruning = state.operations_runtime.pruning_snapshot().await;
    let telemetry = state.operations_runtime.telemetry_snapshot();

    Ok(Json(RuntimeSettingsResponse {
        server_time: Utc::now(),
        environment: std::env::var("BARQFLOW_ENV").unwrap_or_else(|_| "development".to_string()),
        node_types_count: state.node_registry.get_all_node_names().len(),
        credential_types_count: state.credential_registry.get_all_credentials().len(),
        encryption_key_configured,
        execution_mode: dispatch_mode_label(dispatch.mode),
        worker_concurrency: dispatch.worker_concurrency,
        run_worker_concurrency: state.operations_runtime.run_worker_concurrency(),
        trigger_worker_concurrency: state.operations_runtime.trigger_worker_concurrency(),
        queue_capacity: dispatch.queue_capacity,
        pruning_enabled: pruning.enabled,
        execution_retention_days: pruning.retention_days,
        tracing_enabled: telemetry.enabled,
        trace_format: telemetry.format,
    }))
}

async fn get_operations_overview(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<OperationsOverviewResponse>, (StatusCode, String)> {
    let _auth = require_settings_auth(&headers, &state).await?;

    let (webhook_endpoint_count, webhook_workflow_count) = {
        let registry = state.webhook_registry.read().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Registry lock poisoned".to_string(),
            )
        })?;
        summarize_webhooks(&registry)
    };

    let (cron_workflow_count, cron_job_count) = {
        let cron_jobs = state.active_cron_jobs.read().await;
        summarize_cron_jobs(&cron_jobs)
    };

    let active_executions = state.active_executions.read().await.len();
    let dispatch = state.operations_runtime.dispatch_metrics_snapshot().await;
    let run_queued_count = state
        .execution_dispatch_repo
        .count_open_items_by_kind(ExecutionQueueKind::Run)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .max(0) as usize;
    let trigger_queued_count = state
        .execution_dispatch_repo
        .count_open_items_by_kind(ExecutionQueueKind::Trigger)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .max(0) as usize;
    let pruning = state.operations_runtime.pruning_snapshot().await;
    let telemetry = state.operations_runtime.telemetry_snapshot();

    Ok(Json(OperationsOverviewResponse {
        dispatch: map_dispatch_metrics(
            dispatch,
            state.operations_runtime.run_worker_concurrency(),
            state.operations_runtime.trigger_worker_concurrency(),
            run_queued_count,
            trigger_queued_count,
        ),
        pruning: map_pruning_snapshot(pruning),
        telemetry: map_telemetry_snapshot(telemetry),
        active_executions,
        webhook_endpoint_count,
        webhook_workflow_count,
        cron_workflow_count,
        cron_job_count,
        node_types_count: state.node_registry.get_all_node_names().len(),
        credential_types_count: state.credential_registry.get_all_credentials().len(),
        generated_at: Utc::now(),
    }))
}

async fn prune_operations_data(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<PruneExecutionsResponse>, (StatusCode, String)> {
    let auth = require_settings_auth(&headers, &state).await?;
    require_workspace_role(&auth, "admin")?;

    if state.operations_runtime.pruning_cutoff().is_none() {
        return Err((
            StatusCode::CONFLICT,
            "Execution pruning is disabled for this deployment".to_string(),
        ));
    }

    let summary = run_execution_pruning(
        &state.operations_runtime,
        &state.execution_repo,
        &state.execution_log_repo,
    )
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(PruneExecutionsResponse {
        cutoff: summary.cutoff,
        ran_at: summary.ran_at,
        executions_deleted: summary.executions_deleted,
        wait_resumes_deleted: summary.wait_resumes_deleted,
        logs_deleted: summary.logs_deleted,
    }))
}

async fn require_settings_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, (StatusCode, String)> {
    require_authenticated_user(
        headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await
}

fn summarize_webhooks(registry: &HashMap<String, WebhookEndpoint>) -> (usize, usize) {
    let mut workflow_ids = HashSet::new();
    for endpoint in registry.values() {
        workflow_ids.insert(endpoint.workflow_id);
    }
    (registry.len(), workflow_ids.len())
}

fn summarize_cron_jobs(active_cron_jobs: &HashMap<uuid::Uuid, Vec<uuid::Uuid>>) -> (usize, usize) {
    let workflow_count = active_cron_jobs.len();
    let job_count = active_cron_jobs
        .values()
        .map(|job_ids| job_ids.len())
        .sum::<usize>();
    (workflow_count, job_count)
}

fn dispatch_mode_label(mode: ExecutionDispatchMode) -> String {
    match mode {
        ExecutionDispatchMode::Inline => "inline".to_string(),
        ExecutionDispatchMode::Queue => "queue".to_string(),
    }
}

fn map_dispatch_metrics(
    snapshot: ExecutionDispatchMetricsSnapshot,
    run_worker_concurrency: usize,
    trigger_worker_concurrency: usize,
    run_queued_count: usize,
    trigger_queued_count: usize,
) -> ExecutionDispatchMetricsResponse {
    ExecutionDispatchMetricsResponse {
        mode: dispatch_mode_label(snapshot.mode),
        worker_concurrency: snapshot.worker_concurrency,
        run_worker_concurrency,
        trigger_worker_concurrency,
        queue_capacity: snapshot.queue_capacity,
        queued_count: run_queued_count + trigger_queued_count,
        run_queued_count,
        trigger_queued_count,
        running_count: snapshot.running_count,
        total_enqueued: snapshot.total_enqueued,
        total_started: snapshot.total_started,
        total_finished: snapshot.total_finished,
        total_failed_to_dispatch: snapshot.total_failed_to_dispatch,
        last_enqueued_at: snapshot.last_enqueued_at,
        last_started_at: snapshot.last_started_at,
        last_finished_at: snapshot.last_finished_at,
    }
}

fn map_pruning_snapshot(snapshot: ExecutionPruningSnapshot) -> ExecutionPruningStatusResponse {
    ExecutionPruningStatusResponse {
        enabled: snapshot.enabled,
        retention_days: snapshot.retention_days,
        interval_minutes: snapshot.interval_minutes,
        last_run_at: snapshot.last_run_at,
        last_cutoff_at: snapshot.last_cutoff_at,
        last_executions_deleted: snapshot.last_executions_deleted,
        last_wait_resumes_deleted: snapshot.last_wait_resumes_deleted,
        last_logs_deleted: snapshot.last_logs_deleted,
    }
}

fn map_telemetry_snapshot(snapshot: TelemetrySnapshot) -> TelemetrySettingsResponse {
    TelemetrySettingsResponse {
        enabled: snapshot.enabled,
        format: snapshot.format,
        service_name: snapshot.service_name,
        environment: snapshot.environment,
        request_id_header: snapshot.request_id_header,
    }
}
