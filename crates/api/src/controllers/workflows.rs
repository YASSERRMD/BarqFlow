use crate::auth::Claims;
use axum::http::StatusCode;
use axum::{
    extract::{Json, Path, Query, State},
    routing::{get, put},
    Router,
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
        .route(
            "/workflows/{id}",
            get(get_workflow).put(update_workflow).delete(delete_workflow),
        )
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

#[derive(Deserialize)]
pub struct WorkflowListQuery {
    pub active: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

async fn get_workflows(
    _claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<WorkflowListQuery>,
) -> Result<Json<Vec<WorkflowEntity>>, (StatusCode, String)> {
    let mut workflows = if let Some(active) = query.active {
        state
            .workflow_repo
            .find_all_by_active(active)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        state
            .workflow_repo
            .find_all()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    if let Some(search) = query.search {
        let needle = search.trim().to_lowercase();
        if !needle.is_empty() {
            workflows.retain(|wf| wf.name.to_lowercase().contains(&needle));
        }
    }

    if let Some(limit) = query.limit {
        workflows.truncate(limit);
    }

    Ok(Json(workflows))
}

async fn get_workflow(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<WorkflowEntity>, (StatusCode, String)> {
    let workflow = state
        .workflow_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workflow not found".into()))?;

    Ok(Json(workflow))
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

async fn update_workflow(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<CreateWorkflowRequest>,
) -> Result<Json<WorkflowEntity>, (StatusCode, String)> {
    let updated_wf = state
        .workflow_repo
        .update(
            id,
            &payload.name,
            payload.nodes,
            payload.connections,
            payload.settings,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workflow not found".into()))?;

    Ok(Json(updated_wf))
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

async fn delete_workflow(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let deleted = state
        .workflow_repo
        .delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Workflow not found".into()))
    }
}
