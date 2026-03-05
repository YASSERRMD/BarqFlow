use crate::auth::Claims;
use axum::{
    extract::{Path, State, Json},
    routing::{get, post},
    Router,
};
use barqflow_db::executions::ExecutionRepo;
use barqflow_db::models::ExecutionEntity;
use axum::http::StatusCode;
use serde::Deserialize;

#[derive(Clone)]
pub struct AppState {
    pub execution_repo: std::sync::Arc<ExecutionRepo>,
}

pub fn execution_routes(state: AppState) -> Router {
    Router::new()
        .route("/executions/{id}", get(get_execution))
        .route("/executions/workflow/{workflow_id}", post(create_execution))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct CreateExecutionRequest {
    pub status: String,
    pub data: serde_json::Value,
}

async fn get_execution(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<ExecutionEntity>, (StatusCode, String)> {
    let exec = state
        .execution_repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found".into()))?;

    Ok(Json(exec))
}

async fn create_execution(
    _claims: Claims,
    State(state): State<AppState>,
    Path(workflow_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateExecutionRequest>,
) -> Result<Json<ExecutionEntity>, (StatusCode, String)> {
    let new_exec = state
        .execution_repo
        .create(
            workflow_id,
            &payload.status,
            payload.data,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(new_exec))
}
