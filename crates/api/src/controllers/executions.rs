use crate::auth::Claims;
use crate::routes::{ActiveExecutionControl, ActiveExecutionManager};
use axum::http::StatusCode;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json,
    Router,
};
use crate::repositories::execution::ExecutionRepository;
use crate::repositories::workflow::WorkflowRepository;
use crate::repositories::credential::CredentialRepository;
use barqflow_db::models::ExecutionEntity;
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub execution_repo: Arc<ExecutionRepository>,
    pub workflow_repo: Arc<WorkflowRepository>,
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_repo: Arc<CredentialRepository>,
    pub active_executions: ActiveExecutionManager,
}

pub fn execution_routes(state: AppState) -> Router {
    Router::new()
        .route("/executions", get(list_executions))
        .route("/executions/{id}", get(get_execution).delete(delete_execution))
        .route("/executions/{id}/retry", post(retry_execution))
        .route("/executions/{id}/stop", post(stop_execution))
        .route("/executions/workflow/{workflow_id}", post(execute_workflow))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct ExecutionListQuery {
    pub workflow_id: Option<uuid::Uuid>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateExecutionRequest {
    pub manual: Option<bool>,
}

async fn list_executions(
    _claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<ExecutionListQuery>,
) -> Result<Json<Vec<ExecutionEntity>>, (StatusCode, String)> {
    let mut executions = if let Some(workflow_id) = query.workflow_id {
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

    if let Some(status) = query.status {
        executions.retain(|e| e.status.eq_ignore_ascii_case(&status));
    }

    if let Some(limit) = query.limit {
        executions.truncate(limit);
    }

    Ok(Json(executions))
}

async fn get_execution(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<ExecutionEntity>, (StatusCode, String)> {
    let exec = state
        .execution_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    Ok(Json(exec))
}

async fn delete_execution(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let deleted = state
        .execution_repo
        .delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Execution not found".into()))
    }
}

async fn execute_workflow(
    _claims: Claims,
    State(state): State<AppState>,
    Path(workflow_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateExecutionRequest>,
) -> Result<Json<ExecutionEntity>, (StatusCode, String)> {
    let execution = run_workflow_execution(&state, workflow_id, payload.manual.unwrap_or(true)).await?;
    Ok(Json(execution))
}

async fn retry_execution(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionEntity>, (StatusCode, String)> {
    let execution = state
        .execution_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    let retried = run_workflow_execution(&state, execution.workflow_id, true).await?;
    Ok(Json(retried))
}

async fn stop_execution(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionEntity>, (StatusCode, String)> {
    let execution = state
        .execution_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

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
        let updated = state
            .execution_repo
            .update_status_and_data(
                execution.id,
                "stopped",
                serde_json::json!({
                    "stopped": true,
                    "reason": "Execution was not active in runtime registry"
                }),
            )
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;
        return Ok(Json(updated));
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
        return Ok(Json(latest));
    }

    let stopped = state
        .execution_repo
        .update_status_and_data(
            execution.id,
            "stopped",
            serde_json::json!({
                "stopped": true,
                "reason": "Stopped by user",
            }),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    Ok(Json(stopped))
}

async fn run_workflow_execution(
    state: &AppState,
    workflow_id: Uuid,
    manual: bool,
) -> Result<ExecutionEntity, (StatusCode, String)> {
    use barqflow_exec::runner::{WorkflowRunner, ExecutionConfig, WorkflowRunContext};
    use barqflow_core::types::RunId;

    // 1. Fetch workflow
    let wf_entity = state
        .workflow_repo
        .find_by_id(workflow_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workflow not found".into()))?;

    // 2. Parse nodes and connections into CoreWorkflowDef
    let nodes: Vec<barqflow_core::schema::INode> = serde_json::from_value(wf_entity.nodes.clone())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse nodes: {}", e)))?;
    
    let connections: std::collections::HashMap<String, barqflow_core::schema::INodeConnections> = serde_json::from_value(wf_entity.connections.clone())
        .unwrap_or_default();
    
    let settings: barqflow_core::schema::IWorkflowSettings = serde_json::from_value(wf_entity.settings.clone())
        .unwrap_or_default();

    let credential_provider = Arc::new(crate::credentials_provider::RepositoryCredentialProvider::new(
        Arc::clone(&state.credential_repo),
        &nodes,
    ));

    let core_wf = barqflow_core::schema::WorkflowDef {
        id: barqflow_core::types::WorkflowId(wf_entity.id),
        name: wf_entity.name.clone(),
        nodes,
        connections: connections.into_iter().collect(),
        active: wf_entity.active,
        settings,
    };
    // Save initial state
    let new_exec = state
        .execution_repo
        .create(workflow_id, "running", serde_json::Value::Null)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let runner = WorkflowRunner::new(state.node_registry.clone(), ExecutionConfig::default())
        .with_credential_provider(credential_provider);
    let run_id = RunId::new();
    let cancellation_token = CancellationToken::new();
    let ctx = WorkflowRunContext {
        run_id,
        workflow: core_wf,
        static_data: None,
        manual,
        execution_id: Some(new_exec.id),
        parent_execution_id: None,
        cancellation_token: Some(cancellation_token.clone()),
    };

    let run_task = tokio::spawn(async move { runner.run_workflow(ctx).await });
    {
        let mut active = state.active_executions.write().await;
        active.insert(
            new_exec.id,
            ActiveExecutionControl {
                cancellation_token,
                abort_handle: run_task.abort_handle(),
            },
        );
    }

    let results = match run_task.await {
        Ok(result) => result,
        Err(join_err) if join_err.is_cancelled() => Err(barqflow_core::errors::BarqError::ExecutionCancelledError {
            execution_id: new_exec.id.to_string(),
        }),
        Err(join_err) => Err(barqflow_core::errors::BarqError::InternalError(format!(
            "Execution task join error: {}",
            join_err
        ))),
    };
    {
        let mut active = state.active_executions.write().await;
        active.remove(&new_exec.id);
    }

    // Convert results to generic Value
    let (status, data) = match results {
        Ok(res) => {
            let mut summary = serde_json::Map::new();
            let mut all_success = true;
            for (node, res) in res {
                if !res.success {
                    all_success = false;
                }
                summary.insert(node, serde_json::json!({
                    "success": res.success,
                    "error": res.error,
                    "outputs": res.outputs
                }));
            }
            let status = if all_success { "success" } else { "failed" };
            (status, serde_json::Value::Object(summary))
        },
        Err(barqflow_core::errors::BarqError::ExecutionCancelledError { .. }) => (
            "stopped",
            serde_json::json!({
                "stopped": true,
                "reason": "Execution cancelled",
            }),
        ),
        Err(e) => {
            ("failed", serde_json::json!({"error": e.to_string()}))
        }
    };

    let updated_exec = state
        .execution_repo
        .update_status_and_data(new_exec.id, status, data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap();

    Ok(updated_exec)
}
