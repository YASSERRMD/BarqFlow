use crate::auth::Claims;
use axum::http::StatusCode;
use axum::{
    extract::{Json, Path, State},
    routing::{get, put},
    Router,
    response::IntoResponse,
};
use barqflow_db::models::WorkflowEntity;
use crate::repositories::workflow::WorkflowRepository;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub workflow_repo: Arc<WorkflowRepository>,
}

pub fn workflow_routes(state: AppState) -> Router {
    Router::new()
        .route("/workflows", get(get_workflows).post(create_workflow))
        .route("/workflows/{id}/activate", put(toggle_workflow_active))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub nodes: serde_json::Value,
    pub connections: serde_json::Value,
    pub settings: serde_json::Value,
}

#[derive(Deserialize)]
pub struct ToggleActiveRequest {
    pub active: bool,
}

async fn get_workflows(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkflowEntity>>, (StatusCode, String)> {
    let workflows = state
        .workflow_repo
        .find_all()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(workflows))
}

async fn create_workflow(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkflowRequest>,
) -> Result<Json<WorkflowEntity>, (StatusCode, String)> {
    let new_wf = state
        .workflow_repo
        .create(
            &payload.name,
            payload.nodes,
            payload.connections,
            payload.settings,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(new_wf))
}

async fn toggle_workflow_active(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<ToggleActiveRequest>,
) -> Result<Json<WorkflowEntity>, (StatusCode, String)> {
    let updated_wf = state
        .workflow_repo
        .toggle_active(id, payload.active)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workflow not found".into()))?;

    Ok(Json(updated_wf))
}
