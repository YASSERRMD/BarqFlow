use crate::auth::{require_authenticated_user, require_workspace_role, AuthenticatedUser};
use crate::contracts::{ExecutionLogResponse, ExecutionResponse};
use crate::execution_dispatch::{enqueue_workflow_execution, QueuedExecutionPayload};
use crate::execution_events::{
    extract_execution_events, merge_execution_events, with_execution_event_history,
    ExecutionEventHub,
};
use crate::operations::OperationsRuntime;
use crate::repositories::{
    api_key::ApiKeyRepository, credential::CredentialRepository, execution::ExecutionRepository,
    execution_dispatch::{ExecutionDispatchRepository, ExecutionQueueKind},
    execution_log::ExecutionLogRepository, governance::GovernanceRepository,
    workflow::WorkflowRepository, workspace::WorkspaceRepository,
};
use crate::routes::{ActiveExecutionControl, ActiveExecutionManager};
use crate::subworkflow_executor::RepositorySubWorkflowExecutor;
use async_stream::stream;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use barqflow_core::contracts::{ExecutionEvent, ExecutionEventType, ExecutionStatus};
use barqflow_core::types::{IDataObject, NodeId, RunId, WorkflowId};
use barqflow_db::models::ExecutionEntity;
use barqflow_db::users::UserRepo;
use chrono::{Duration as ChronoDuration, Utc};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tokio_util::sync::CancellationToken;
use tracing::warn;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub execution_repo: Arc<ExecutionRepository>,
    pub execution_dispatch_repo: Arc<ExecutionDispatchRepository>,
    pub execution_log_repo: Arc<ExecutionLogRepository>,
    pub workflow_repo: Arc<WorkflowRepository>,
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_repo: Arc<CredentialRepository>,
    pub governance_repo: Arc<GovernanceRepository>,
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
    pub active_executions: ActiveExecutionManager,
    pub execution_events: ExecutionEventHub,
    pub operations_runtime: OperationsRuntime,
}

pub fn execution_routes(state: AppState) -> Router {
    Router::new()
        .route("/executions", get(list_executions))
        .route("/executions/{id}/events", get(get_execution_events))
        .route("/executions/{id}/logs", get(get_execution_logs))
        .route(
            "/executions/{id}/events/stream",
            get(stream_execution_events),
        )
        .route(
            "/executions/{id}",
            get(get_execution).delete(delete_execution),
        )
        .route("/executions/{id}/retry", post(retry_execution))
        .route("/executions/{id}/stop", post(stop_execution))
        .route(
            "/executions/{id}/resume/{resume_token}",
            post(resume_execution),
        )
        .route("/executions/workflow/{workflow_id}", post(execute_workflow))
        .route(
            "/executions/workflow/{workflow_id}/test-node/{node_id}",
            post(test_workflow_node),
        )
        .with_state(state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionListQuery {
    #[serde(alias = "workflow_id")]
    pub workflow_id: Option<uuid::Uuid>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExecutionRequest {
    pub manual: Option<bool>,
    pub stop_at_node_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecutionStreamQuery {
    pub token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecutionLogsQuery {
    pub limit: Option<usize>,
}

fn is_terminal_contract_status(status: ExecutionStatus) -> bool {
    matches!(
        status,
        ExecutionStatus::Waiting
            | ExecutionStatus::Success
            | ExecutionStatus::Failed
            | ExecutionStatus::Stopped
            | ExecutionStatus::Cancelled
    )
}

fn execution_log_level(event: &ExecutionEvent) -> &'static str {
    match event.status {
        ExecutionStatus::Failed => "error",
        ExecutionStatus::Stopped | ExecutionStatus::Cancelled => "warn",
        ExecutionStatus::Waiting => "info",
        ExecutionStatus::Queued | ExecutionStatus::Running | ExecutionStatus::Success => "info",
    }
}

fn execution_event_type_label(event_type: ExecutionEventType) -> &'static str {
    match event_type {
        ExecutionEventType::Queued => "queued",
        ExecutionEventType::Started => "started",
        ExecutionEventType::NodeStarted => "nodeStarted",
        ExecutionEventType::NodeFinished => "nodeFinished",
        ExecutionEventType::Waiting => "waiting",
        ExecutionEventType::Resumed => "resumed",
        ExecutionEventType::Failed => "failed",
        ExecutionEventType::Stopped => "stopped",
        ExecutionEventType::Completed => "completed",
    }
}

#[derive(Clone)]
struct StateBackedExecutionEventReporter {
    state: AppState,
}

impl StateBackedExecutionEventReporter {
    fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl barqflow_exec::runner::ExecutionEventReporter for StateBackedExecutionEventReporter {
    async fn report(&self, event: ExecutionEvent) {
        append_execution_event(&self.state, event).await;
    }
}

pub(crate) fn build_execution_event(
    execution_id: Uuid,
    workflow_id: Uuid,
    run_id: RunId,
    sequence: u64,
    event_type: ExecutionEventType,
    status: ExecutionStatus,
    message: impl Into<String>,
    node_id: Option<NodeId>,
    node_name: Option<String>,
    data: serde_json::Value,
) -> ExecutionEvent {
    ExecutionEvent {
        execution_id,
        workflow_id: WorkflowId(workflow_id),
        run_id,
        event_type,
        status,
        node_id,
        node_name,
        message: message.into(),
        timestamp: Utc::now(),
        sequence,
        data: IDataObject::from(data),
    }
}

async fn load_execution_event_history(
    state: &AppState,
    execution: &ExecutionEntity,
) -> Vec<ExecutionEvent> {
    merge_execution_events(
        extract_execution_events(&execution.data),
        state.execution_events.snapshot(execution.id).await,
    )
}

pub(crate) async fn append_execution_event(state: &AppState, event: ExecutionEvent) {
    state.execution_events.append(event.clone()).await;

    if let Err(error) = state
        .execution_log_repo
        .create(
            event.execution_id,
            event.workflow_id.0,
            execution_log_level(&event),
            Some(execution_event_type_label(event.event_type)),
            event.message.as_str(),
            event.node_id.as_ref().map(|value| value.0.as_str()),
            event.node_name.as_deref(),
            serde_json::to_value(&event.data).unwrap_or(serde_json::Value::Null),
        )
        .await
    {
        warn!(
            execution_id = %event.execution_id,
            error = %error,
            "Failed to persist execution log entry"
        );
    }
}

pub(crate) async fn persist_execution_with_events(
    state: &AppState,
    execution_id: Uuid,
    status: &str,
    payload: serde_json::Value,
) -> Result<ExecutionEntity, (StatusCode, String)> {
    let existing = state
        .execution_repo
        .find_by_id(execution_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    let history = load_execution_event_history(state, &existing).await;
    let decorated = with_execution_event_history(payload, &history);

    state
        .execution_repo
        .update_status_and_data(execution_id, status, decorated)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))
}

async fn list_executions(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<ExecutionListQuery>,
) -> Result<Json<Vec<ExecutionResponse>>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    let workflow_ids = workspace_workflow_ids(&state, auth.workspace_id).await?;

    let mut executions = if let Some(workflow_id) = query.workflow_id {
        if !workflow_ids.contains(&workflow_id) {
            return Err((StatusCode::NOT_FOUND, "Workflow not found".into()));
        }
        state
            .execution_repo
            .find_by_workflow_id(workflow_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        state
            .execution_repo
            .find_all()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    executions.retain(|execution| workflow_ids.contains(&execution.workflow_id));

    if let Some(status) = query.status {
        executions.retain(|e| e.status.eq_ignore_ascii_case(&status));
    }

    if let Some(limit) = query.limit {
        executions.truncate(limit);
    }

    Ok(Json(
        executions
            .into_iter()
            .map(ExecutionResponse::from)
            .collect(),
    ))
}

async fn get_execution(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<ExecutionResponse>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    let exec = load_execution_in_workspace(&state, auth.workspace_id, id).await?;

    Ok(Json(ExecutionResponse::from(exec)))
}

async fn get_execution_events(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ExecutionEvent>>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    let execution = load_execution_in_workspace(&state, auth.workspace_id, id).await?;

    Ok(Json(load_execution_event_history(&state, &execution).await))
}

async fn get_execution_logs(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<ExecutionLogsQuery>,
) -> Result<Json<Vec<ExecutionLogResponse>>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    let execution = load_execution_in_workspace(&state, auth.workspace_id, id).await?;
    let limit = query.limit.unwrap_or(500).clamp(1, 1000);

    let logs = state
        .execution_log_repo
        .list_for_execution(execution.id, limit)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(
        logs.into_iter().map(ExecutionLogResponse::from).collect(),
    ))
}

async fn stream_execution_events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Query(query): Query<ExecutionStreamQuery>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    if let Some(token) = query.token.as_deref() {
        let mut auth_headers = HeaderMap::new();
        let header_value = format!("Bearer {}", token)
            .parse()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;
        auth_headers.insert(axum::http::header::AUTHORIZATION, header_value);
        let auth = require_execution_auth(&auth_headers, &state).await?;
        let execution = load_execution_in_workspace(&state, auth.workspace_id, id).await?;
        return stream_known_execution(state, id, execution).await;
    }

    let auth = require_execution_auth(&headers, &state).await?;
    let execution = load_execution_in_workspace(&state, auth.workspace_id, id).await?;

    stream_known_execution(state, id, execution).await
}

async fn stream_known_execution(
    state: AppState,
    id: Uuid,
    execution: ExecutionEntity,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    let snapshot = load_execution_event_history(&state, &execution).await;
    let is_active = {
        let active = state.active_executions.read().await;
        active.contains_key(&id)
    };

    let mut receiver = if is_active {
        Some(state.execution_events.subscribe(id).await)
    } else {
        None
    };

    let event_stream = stream! {
        for execution_event in snapshot {
            let terminal = is_terminal_contract_status(execution_event.status);
            yield Ok::<Event, Infallible>(
                Event::default()
                    .event("execution")
                    .json_data(&execution_event)
                    .unwrap_or_else(|_| Event::default().data("{}"))
            );
            if terminal {
                return;
            }
        }

        if let Some(receiver) = receiver.as_mut() {
            loop {
                match receiver.recv().await {
                    Ok(execution_event) => {
                        let terminal = is_terminal_contract_status(execution_event.status);
                        yield Ok::<Event, Infallible>(
                            Event::default()
                                .event("execution")
                                .json_data(&execution_event)
                                .unwrap_or_else(|_| Event::default().data("{}"))
                        );
                        if terminal {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    };
    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

async fn delete_execution(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    load_execution_in_workspace(&state, auth.workspace_id, id).await?;
    let deleted = state
        .execution_repo
        .delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        state.execution_events.remove(id).await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Execution not found".into()))
    }
}

async fn execute_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(workflow_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateExecutionRequest>,
) -> Result<Json<ExecutionResponse>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let execution = run_workflow_execution(
        &state,
        auth.workspace_id,
        workflow_id,
        payload.manual.unwrap_or(true),
        payload.stop_at_node_id,
    )
    .await?;
    Ok(Json(ExecutionResponse::from(execution)))
}

async fn test_workflow_node(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((workflow_id, node_id)): Path<(uuid::Uuid, String)>,
    payload: Option<Json<CreateExecutionRequest>>,
) -> Result<Json<ExecutionResponse>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let manual = payload
        .as_ref()
        .and_then(|json| json.manual)
        .unwrap_or(true);
    let execution = run_workflow_execution(
        &state,
        auth.workspace_id,
        workflow_id,
        manual,
        Some(node_id),
    )
    .await?;
    Ok(Json(ExecutionResponse::from(execution)))
}

async fn retry_execution(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionResponse>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let execution = load_execution_in_workspace(&state, auth.workspace_id, id).await?;

    let retried =
        run_workflow_execution(&state, auth.workspace_id, execution.workflow_id, true, None)
            .await?;
    Ok(Json(ExecutionResponse::from(retried)))
}

async fn stop_execution(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionResponse>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let execution = load_execution_in_workspace(&state, auth.workspace_id, id).await?;

    if !execution.status.eq_ignore_ascii_case("running")
        && !execution.status.eq_ignore_ascii_case("stopping")
    {
        return Err((StatusCode::CONFLICT, "Execution is not running".into()));
    }

    let control = {
        let active = state.active_executions.read().await;
        active.get(&id).cloned()
    };

    let Some(control) = control else {
        let next_sequence = load_execution_event_history(&state, &execution)
            .await
            .last()
            .map(|event| event.sequence + 1)
            .unwrap_or(1);
        append_execution_event(
            &state,
            build_execution_event(
                execution.id,
                execution.workflow_id,
                RunId(execution.id),
                next_sequence,
                ExecutionEventType::Stopped,
                ExecutionStatus::Stopped,
                "Execution stopped because it was not active in the runtime registry",
                None,
                None,
                serde_json::json!({
                    "reason": "Execution was not active in runtime registry"
                }),
            ),
        )
        .await;
        let updated = persist_execution_with_events(
            &state,
            execution.id,
            "stopped",
            serde_json::json!({
                "stopped": true,
                "reason": "Execution was not active in runtime registry"
            }),
        )
        .await?;
        return Ok(Json(ExecutionResponse::from(updated)));
    };

    control.cancellation_token.cancel();

    let _ = state
        .execution_repo
        .update_status_and_data(
            execution.id,
            "stopping",
            serde_json::json!({
                "stopping": true,
                "reason": "Stop requested by user"
            }),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let still_running = {
            let active = state.active_executions.read().await;
            active.contains_key(&id)
        };
        if !still_running {
            break;
        }

        if Instant::now() >= deadline {
            control.abort_handle.abort();
            let mut active = state.active_executions.write().await;
            active.remove(&id);
            break;
        }

        sleep(Duration::from_millis(200)).await;
    }

    let latest = state
        .execution_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    if !latest.status.eq_ignore_ascii_case("running")
        && !latest.status.eq_ignore_ascii_case("stopping")
    {
        return Ok(Json(ExecutionResponse::from(latest)));
    }

    let stopped = state
        .execution_repo
        .find_by_id(execution.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    let next_sequence = load_execution_event_history(&state, &stopped)
        .await
        .last()
        .map(|event| event.sequence + 1)
        .unwrap_or(1);
    append_execution_event(
        &state,
        build_execution_event(
            execution.id,
            execution.workflow_id,
            RunId(execution.id),
            next_sequence,
            ExecutionEventType::Stopped,
            ExecutionStatus::Stopped,
            "Execution stopped by user",
            None,
            None,
            serde_json::json!({
                "reason": "Stopped by user"
            }),
        ),
    )
    .await;

    let stopped = persist_execution_with_events(
        &state,
        execution.id,
        "stopped",
        serde_json::json!({
            "stopped": true,
            "reason": "Stopped by user",
        }),
    )
    .await?;

    Ok(Json(ExecutionResponse::from(stopped)))
}

fn summarize_node_results(
    results: std::collections::HashMap<String, barqflow_exec::runner::NodeExecutionResult>,
) -> (String, serde_json::Value) {
    let mut summary = serde_json::Map::new();
    let mut all_success = true;
    for (node, result) in results {
        if !result.success {
            all_success = false;
        }
        summary.insert(
            node,
            serde_json::json!({
                "success": result.success,
                "error": result.error,
                "outputs": result.outputs
            }),
        );
    }

    let status = if all_success { "success" } else { "failed" };
    (status.to_string(), serde_json::Value::Object(summary))
}

async fn build_waiting_execution_data(
    state: &AppState,
    execution_id: Uuid,
    node_name: &str,
    wait_config: serde_json::Value,
) -> Result<serde_json::Value, (StatusCode, String)> {
    let parsed: barqflow_exec::checkpoint::WaitConfig = serde_json::from_value(wait_config)
        .unwrap_or(barqflow_exec::checkpoint::WaitConfig {
            wait_type: barqflow_exec::checkpoint::WaitType::Time,
            duration_ms: None,
            webhook_path: None,
            external_id: None,
        });

    let wait_type = match parsed.wait_type {
        barqflow_exec::checkpoint::WaitType::Time => "time",
        barqflow_exec::checkpoint::WaitType::Webhook => "webhook",
        barqflow_exec::checkpoint::WaitType::External => "external",
        barqflow_exec::checkpoint::WaitType::SubWorkflow => "subworkflow",
    };

    let mut payload = serde_json::json!({
        "waiting": true,
        "nodeName": node_name,
        "waitType": wait_type,
        "durationMs": parsed.duration_ms,
    });

    if parsed.wait_type == barqflow_exec::checkpoint::WaitType::Webhook {
        let resume_token = parsed
            .webhook_path
            .clone()
            .filter(|token| !token.trim().is_empty())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let expires_at = Utc::now() + ChronoDuration::hours(24);

        state
            .execution_repo
            .create_wait_resume(execution_id, node_name, &resume_token, expires_at)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        payload["resumeToken"] = serde_json::json!(resume_token.clone());
        payload["resumeUrl"] = serde_json::json!(format!(
            "/rest/executions/{}/resume/{}",
            execution_id, resume_token
        ));
        payload["expiresAt"] = serde_json::json!(expires_at.to_rfc3339());
    }

    Ok(payload)
}

async fn resume_execution(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((execution_id, resume_token)): Path<(Uuid, String)>,
    payload: Option<Json<serde_json::Value>>,
) -> Result<Json<ExecutionResponse>, (StatusCode, String)> {
    let auth = require_execution_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    use barqflow_core::schema::{INodeExecutionData, ITaskDataConnections};
    use barqflow_core::types::{IDataObject, RunId};
    use barqflow_exec::runner::{ExecutionConfig, WorkflowRunContext, WorkflowRunner};

    let wait_resume = state
        .execution_repo
        .find_wait_resume(execution_id, &resume_token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Resume token not found".into()))?;

    if wait_resume.resumed_at.is_some() {
        return Err((StatusCode::CONFLICT, "Resume token already consumed".into()));
    }

    if Utc::now() > wait_resume.expires_at {
        let _ = state
            .execution_repo
            .delete_wait_resume(wait_resume.id)
            .await;
        return Err((StatusCode::GONE, "Resume token expired".into()));
    }

    let execution = load_execution_in_workspace(&state, auth.workspace_id, execution_id).await?;

    let wf_entity = state
        .workflow_repo
        .find_by_id_in_workspace(auth.workspace_id, execution.workflow_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workflow not found".into()))?;

    let nodes: Vec<barqflow_core::schema::INode> = serde_json::from_value(wf_entity.nodes.clone())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse nodes: {}", e),
            )
        })?;
    let connections: std::collections::HashMap<String, barqflow_core::schema::INodeConnections> =
        serde_json::from_value(wf_entity.connections.clone()).unwrap_or_default();
    let settings: barqflow_core::schema::IWorkflowSettings =
        serde_json::from_value(wf_entity.settings.clone()).unwrap_or_default();

    let credential_provider = Arc::new(
        crate::credentials_provider::RepositoryCredentialProvider::new(
            Arc::clone(&state.credential_repo),
            Arc::clone(&state.governance_repo),
            &nodes,
        ),
    );
    let subworkflow_executor = Arc::new(
        RepositorySubWorkflowExecutor::new(
            Arc::clone(&state.workflow_repo),
            Arc::clone(&state.credential_repo),
            Arc::clone(&state.governance_repo),
            Arc::clone(&state.node_registry),
        )
        .with_execution_repo(Arc::clone(&state.execution_repo)),
    );

    let core_wf = barqflow_core::schema::WorkflowDef {
        id: barqflow_core::types::WorkflowId(wf_entity.id),
        name: wf_entity.name.clone(),
        nodes,
        connections: connections.into_iter().collect(),
        active: wf_entity.active,
        settings,
    };

    let mut checkpoint_manager = barqflow_exec::checkpoint::CheckpointManager::with_filesystem(
        std::env::temp_dir().join("barqflow_checkpoints"),
    );
    let run_id = RunId(execution_id);
    let mut checkpoint = checkpoint_manager
        .load_checkpoint(&run_id)
        .await
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                "Checkpoint not found for execution".into(),
            )
        })?;

    let resume_payload = payload
        .map(|Json(value)| value)
        .unwrap_or_else(|| serde_json::json!({}));
    let normalized_payload = match resume_payload {
        serde_json::Value::Object(_) => resume_payload,
        other => serde_json::json!({ "value": other }),
    };
    let mut resume_input = ITaskDataConnections::new();
    resume_input.push(
        0,
        vec![INodeExecutionData::new(IDataObject::from(
            normalized_payload,
        ))],
    );
    checkpoint.node_data = serde_json::to_value(&resume_input).unwrap_or(serde_json::Value::Null);
    let existing_events = load_execution_event_history(&state, &execution).await;
    let resumed_sequence = existing_events
        .last()
        .map(|event| event.sequence + 1)
        .unwrap_or(1);
    let resumed_at = Utc::now().to_rfc3339();
    append_execution_event(
        &state,
        build_execution_event(
            execution_id,
            execution.workflow_id,
            run_id,
            resumed_sequence,
            ExecutionEventType::Resumed,
            ExecutionStatus::Running,
            "Execution resumed",
            None,
            Some(wait_resume.node_name.clone()),
            serde_json::json!({
                "resumedAt": resumed_at,
                "resumeToken": resume_token,
            }),
        ),
    )
    .await;
    let _ = persist_execution_with_events(
        &state,
        execution_id,
        "running",
        serde_json::json!({
            "resumed": true,
            "resumedAt": resumed_at,
        }),
    )
    .await?;

    let runner = WorkflowRunner::new(state.node_registry.clone(), ExecutionConfig::default())
        .with_credential_provider(credential_provider)
        .with_subworkflow_executor(subworkflow_executor)
        .with_event_reporter(Arc::new(StateBackedExecutionEventReporter::new(
            state.clone(),
        )));

    let context = WorkflowRunContext {
        run_id,
        workflow: core_wf,
        static_data: None,
        manual: true,
        execution_id: Some(execution_id),
        parent_execution_id: None,
        cancellation_token: None,
        stop_after_node_id: None,
        event_sequence_start: resumed_sequence,
    };

    let result = runner.resume_workflow(context, checkpoint).await;

    let (status, data, event_type, event_status, message, node_name_for_event) = match result {
        Ok(node_results) => {
            let _ = checkpoint_manager.delete_checkpoint(&run_id).await;
            let _ = state
                .execution_repo
                .delete_wait_resume(wait_resume.id)
                .await;
            let (status, data) = summarize_node_results(node_results);
            (
                status,
                data,
                ExecutionEventType::Completed,
                ExecutionStatus::Success,
                "Execution completed successfully".to_string(),
                None,
            )
        }
        Err(barqflow_core::errors::BarqError::SuspendExecution {
            node_name,
            wait_config,
        }) => {
            let _ = state
                .execution_repo
                .delete_wait_resume(wait_resume.id)
                .await;
            let waiting_data =
                build_waiting_execution_data(&state, execution_id, node_name.as_str(), wait_config)
                    .await?;
            (
                "waiting".to_string(),
                waiting_data,
                ExecutionEventType::Waiting,
                ExecutionStatus::Waiting,
                format!("Execution waiting at '{}'", node_name),
                Some(node_name),
            )
        }
        Err(err) => {
            let _ = state
                .execution_repo
                .delete_wait_resume(wait_resume.id)
                .await;
            (
                "failed".to_string(),
                serde_json::json!({"error": err.to_string()}),
                ExecutionEventType::Failed,
                ExecutionStatus::Failed,
                format!("Execution failed: {}", err),
                None,
            )
        }
    };

    let final_sequence = state
        .execution_events
        .snapshot(execution_id)
        .await
        .last()
        .map(|event| event.sequence + 1)
        .unwrap_or(resumed_sequence + 1);
    append_execution_event(
        &state,
        build_execution_event(
            execution_id,
            execution.workflow_id,
            run_id,
            final_sequence,
            event_type,
            event_status,
            message,
            None,
            node_name_for_event,
            data.clone(),
        ),
    )
    .await;

    let updated =
        persist_execution_with_events(&state, execution_id, status.as_str(), data).await?;

    Ok(Json(ExecutionResponse::from(updated)))
}

async fn run_workflow_execution(
    state: &AppState,
    workspace_id: Uuid,
    workflow_id: Uuid,
    manual: bool,
    stop_after_node_id: Option<String>,
) -> Result<ExecutionEntity, (StatusCode, String)> {
    let wf_entity = state
        .workflow_repo
        .find_by_id_in_workspace(workspace_id, workflow_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workflow not found".into()))?;

    // 2. Parse nodes and connections into CoreWorkflowDef
    let nodes: Vec<barqflow_core::schema::INode> = serde_json::from_value(wf_entity.nodes.clone())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse nodes: {}", e),
            )
        })?;

    if let Some(target) = stop_after_node_id.as_ref() {
        let target_exists = nodes
            .iter()
            .any(|node| node.id.to_string() == *target || node.name == *target);
        if !target_exists {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Target node '{}' was not found in this workflow", target),
            ));
        }
    }

    enqueue_workflow_execution(
        state,
        workspace_id,
        &wf_entity,
        if manual {
            ExecutionQueueKind::Run
        } else {
            ExecutionQueueKind::Trigger
        },
        if manual { "manual" } else { "api" },
        manual,
        stop_after_node_id,
        None,
    )
    .await
}

pub(crate) async fn execute_queued_dispatch_item(
    state: AppState,
    queue_item: barqflow_db::models::ExecutionDispatchItemEntity,
) -> Result<(), (StatusCode, String)> {
    let payload: QueuedExecutionPayload = serde_json::from_value(queue_item.payload.clone())
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse queued execution payload: {error}"),
            )
        })?;

    let nodes: Vec<barqflow_core::schema::INode> =
        serde_json::from_value(payload.nodes.clone()).map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse queued workflow nodes: {error}"),
            )
        })?;
    let connections: std::collections::HashMap<String, barqflow_core::schema::INodeConnections> =
        serde_json::from_value(payload.connections.clone()).unwrap_or_default();
    let settings: barqflow_core::schema::IWorkflowSettings =
        serde_json::from_value(payload.settings.clone()).unwrap_or_default();

    let workflow = barqflow_core::schema::WorkflowDef {
        id: barqflow_core::types::WorkflowId(queue_item.workflow_id),
        name: payload.workflow_name,
        nodes,
        connections: connections.into_iter().collect(),
        active: payload.active,
        settings,
    };

    run_claimed_execution(
        state,
        queue_item.execution_id,
        workflow,
        payload.manual,
        payload.stop_after_node_id,
        payload.static_data,
    )
    .await
}

async fn run_claimed_execution(
    state: AppState,
    execution_id: Uuid,
    workflow: barqflow_core::schema::WorkflowDef,
    manual: bool,
    stop_after_node_id: Option<String>,
    static_data: Option<serde_json::Value>,
) -> Result<(), (StatusCode, String)> {
    use barqflow_core::types::RunId;
    use barqflow_exec::runner::{ExecutionConfig, WorkflowRunContext, WorkflowRunner};

    let credential_provider = Arc::new(
        crate::credentials_provider::RepositoryCredentialProvider::new(
            Arc::clone(&state.credential_repo),
            Arc::clone(&state.governance_repo),
            &workflow.nodes,
        ),
    );
    let subworkflow_executor = Arc::new(
        RepositorySubWorkflowExecutor::new(
            Arc::clone(&state.workflow_repo),
            Arc::clone(&state.credential_repo),
            Arc::clone(&state.governance_repo),
            Arc::clone(&state.node_registry),
        )
        .with_execution_repo(Arc::clone(&state.execution_repo)),
    );
    let run_id = RunId(execution_id);

    let _ = persist_execution_with_events(
        &state,
        execution_id,
        "running",
        serde_json::json!({
            "queued": false,
            "manual": manual,
            "workflowName": workflow.name.clone(),
            "stopAfterNodeId": stop_after_node_id.clone(),
        }),
    )
    .await?;

    let runner = WorkflowRunner::new(state.node_registry.clone(), ExecutionConfig::default())
        .with_credential_provider(credential_provider)
        .with_subworkflow_executor(subworkflow_executor)
        .with_event_reporter(Arc::new(StateBackedExecutionEventReporter::new(
            state.clone(),
        )));
    let cancellation_token = CancellationToken::new();
    let ctx = WorkflowRunContext {
        run_id,
        workflow: workflow.clone(),
        static_data: static_data.map(IDataObject::from),
        manual,
        execution_id: Some(execution_id),
        parent_execution_id: None,
        cancellation_token: Some(cancellation_token.clone()),
        stop_after_node_id,
        event_sequence_start: 1,
    };

    let run_task = tokio::spawn(async move { runner.run_workflow(ctx).await });
    {
        let mut active = state.active_executions.write().await;
        active.insert(
            execution_id,
            ActiveExecutionControl {
                cancellation_token,
                abort_handle: run_task.abort_handle(),
            },
        );
    }

    let results = match run_task.await {
        Ok(result) => result,
        Err(join_err) if join_err.is_cancelled() => {
            Err(barqflow_core::errors::BarqError::ExecutionCancelledError {
                execution_id: execution_id.to_string(),
            })
        }
        Err(join_err) => Err(barqflow_core::errors::BarqError::InternalError(format!(
            "Execution task join error: {}",
            join_err
        ))),
    };

    let (status, data, event_type, event_status, message, node_name_for_event) = match results {
        Ok(res) => {
            let (status, data) = summarize_node_results(res);
            if status.eq_ignore_ascii_case("success") {
                (
                    status,
                    data,
                    ExecutionEventType::Completed,
                    ExecutionStatus::Success,
                    "Execution completed successfully".to_string(),
                    None,
                )
            } else {
                (
                    status,
                    data,
                    ExecutionEventType::Failed,
                    ExecutionStatus::Failed,
                    "Execution completed with node failures".to_string(),
                    None,
                )
            }
        }
        Err(barqflow_core::errors::BarqError::SuspendExecution {
            node_name,
            wait_config,
        }) => match build_waiting_execution_data(&state, execution_id, node_name.as_str(), wait_config)
            .await
        {
            Ok(waiting_data) => (
                "waiting".to_string(),
                waiting_data,
                ExecutionEventType::Waiting,
                ExecutionStatus::Waiting,
                format!("Execution waiting at '{}'", node_name),
                Some(node_name),
            ),
            Err((code, message)) => (
                "failed".to_string(),
                serde_json::json!({
                    "error": format!(
                        "Failed to persist wait state ({}): {}",
                        code.as_u16(),
                        message
                    )
                }),
                ExecutionEventType::Failed,
                ExecutionStatus::Failed,
                format!("Execution failed while persisting wait state: {}", message),
                None,
            ),
        },
        Err(barqflow_core::errors::BarqError::ExecutionCancelledError { .. }) => (
            "stopped".to_string(),
            serde_json::json!({
                "stopped": true,
                "reason": "Execution cancelled",
            }),
            ExecutionEventType::Stopped,
            ExecutionStatus::Stopped,
            "Execution cancelled".to_string(),
            None,
        ),
        Err(e) => (
            "failed".to_string(),
            serde_json::json!({"error": e.to_string()}),
            ExecutionEventType::Failed,
            ExecutionStatus::Failed,
            format!("Execution failed: {}", e),
            None,
        ),
    };
    let next_sequence = state
        .execution_events
        .snapshot(execution_id)
        .await
        .last()
        .map(|event| event.sequence + 1)
        .unwrap_or(2);
    append_execution_event(
        &state,
        build_execution_event(
            execution_id,
            workflow.id.0,
            run_id,
            next_sequence,
            event_type,
            event_status,
            message,
            None,
            node_name_for_event,
            data.clone(),
        ),
    )
    .await;

    let _ = persist_execution_with_events(&state, execution_id, status.as_str(), data).await;

    let mut active = state.active_executions.write().await;
    active.remove(&execution_id);

    Ok(())
}

async fn require_execution_auth(
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

async fn workspace_workflow_ids(
    state: &AppState,
    workspace_id: Uuid,
) -> Result<std::collections::HashSet<Uuid>, (StatusCode, String)> {
    let workflows = state
        .workflow_repo
        .find_all_for_workspace(workspace_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(workflows.into_iter().map(|workflow| workflow.id).collect())
}

async fn load_execution_in_workspace(
    state: &AppState,
    workspace_id: Uuid,
    execution_id: Uuid,
) -> Result<ExecutionEntity, (StatusCode, String)> {
    let execution = state
        .execution_repo
        .find_by_id(execution_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    state
        .workflow_repo
        .find_by_id_in_workspace(workspace_id, execution.workflow_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    Ok(execution)
}

#[cfg(test)]
mod tests {
    use super::CreateExecutionRequest;

    #[test]
    fn create_execution_request_deserializes_stop_at_node_id() {
        let payload = serde_json::json!({
            "manual": true,
            "stopAtNodeId": "node-123",
        });

        let request: CreateExecutionRequest = serde_json::from_value(payload).unwrap();
        assert_eq!(request.manual, Some(true));
        assert_eq!(request.stop_at_node_id.as_deref(), Some("node-123"));
    }

    #[test]
    fn create_execution_request_allows_missing_optional_fields() {
        let payload = serde_json::json!({});
        let request: CreateExecutionRequest = serde_json::from_value(payload).unwrap();
        assert!(request.manual.is_none());
        assert!(request.stop_at_node_id.is_none());
    }
}
