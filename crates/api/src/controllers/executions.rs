use crate::auth::Claims;
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
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub execution_repo: Arc<ExecutionRepository>,
    pub workflow_repo: Arc<WorkflowRepository>,
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_repo: Arc<CredentialRepository>,
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

    if !execution.status.eq_ignore_ascii_case("running") {
        return Err((StatusCode::CONFLICT, "Execution is not running".into()));
    }

    let updated = state
        .execution_repo
        .update_status_and_data(
            execution.id,
            "cancelled",
            serde_json::json!({
                "cancelled": true,
                "reason": "Stopped by user"
            }),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    Ok(Json(updated))
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
    let runner = WorkflowRunner::new(state.node_registry.clone(), ExecutionConfig::default())
        .with_credential_provider(credential_provider);
    let run_id = RunId::new();
    let ctx = WorkflowRunContext {
        run_id,
        workflow: core_wf,
        static_data: None,
        manual,
    };

    // Save initial state
    let new_exec = state
        .execution_repo
        .create(workflow_id, "running", serde_json::Value::Null)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Run Engine
    let results = runner.run_workflow(ctx).await;

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
