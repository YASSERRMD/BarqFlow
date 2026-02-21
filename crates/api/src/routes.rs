use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use barqflow_db::{WorkflowRepo, CredentialRepo, ExecutionRepo};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub workflow_repo: Arc<WorkflowRepo>,
    pub credential_repo: Arc<CredentialRepo>,
    pub exec_repo: Arc<ExecutionRepo>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/workflows", get(list_workflows).post(create_workflow))
        .route("/workflows/:id", get(get_workflow).put(update_workflow).delete(delete_workflow))
        .route("/workflows/:id/activate", post(activate_workflow))
        .route("/workflows/:id/deactivate", post(deactivate_workflow))
        .route("/credentials", get(list_credentials).post(create_credential))
        .route("/credentials/:id", get(get_credential).put(update_credential).delete(delete_credential))
        .route("/executions", get(list_executions))
        .route("/executions/:id", get(get_execution))
        .route("/workflows/:id/executions", get(list_workflow_executions))
        .with_state(state)
}

// Workflows handlers

async fn list_workflows(State(state): State<AppState>) -> impl IntoResponse {
    match state.workflow_repo.get_all().await {
        Ok(workflows) => (StatusCode::OK, Json(workflows)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.workflow_repo.get_by_id(id).await {
        Ok(Some(workflow)) => (StatusCode::OK, Json(workflow)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Deserialize)]
struct CreateWorkflowPayload {
    name: String,
    nodes: serde_json::Value,
    connections: serde_json::Value,
    settings: serde_json::Value,
}

async fn create_workflow(
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkflowPayload>,
) -> impl IntoResponse {
    match state.workflow_repo.create(&payload.name, payload.nodes, payload.connections, payload.settings).await {
        Ok(workflow) => (StatusCode::CREATED, Json(workflow)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn update_workflow() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.workflow_repo.delete(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn activate_workflow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.workflow_repo.toggle_active(id, true).await {
        Ok(Some(workflow)) => (StatusCode::OK, Json(workflow)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn deactivate_workflow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.workflow_repo.toggle_active(id, false).await {
        Ok(Some(workflow)) => (StatusCode::OK, Json(workflow)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// Credentials handlers

async fn list_credentials(State(state): State<AppState>) -> impl IntoResponse {
    match state.credential_repo.get_all().await {
        Ok(credentials) => (StatusCode::OK, Json(credentials)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_credential(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.credential_repo.get_by_id(id).await {
        Ok(Some(credential)) => (StatusCode::OK, Json(credential)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Deserialize)]
struct CreateCredentialPayload {
    name: String,
    cred_type: String,
    data: serde_json::Value,
}

async fn create_credential(
    State(state): State<AppState>,
    Json(payload): Json<CreateCredentialPayload>,
) -> impl IntoResponse {
    match state.credential_repo.create(&payload.name, &payload.cred_type, payload.data).await {
        Ok(credential) => (StatusCode::CREATED, Json(credential)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Deserialize)]
struct UpdateCredentialPayload {
    name: String,
    data: serde_json::Value,
}

async fn update_credential(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCredentialPayload>,
) -> impl IntoResponse {
    match state.credential_repo.update(id, &payload.name, payload.data).await {
        Ok(Some(credential)) => (StatusCode::OK, Json(credential)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_credential(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.credential_repo.delete(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// Executions handlers

async fn list_executions(State(state): State<AppState>) -> impl IntoResponse {
    match state.exec_repo.get_all().await {
        Ok(executions) => (StatusCode::OK, Json(executions)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.exec_repo.get_by_id(id).await {
        Ok(Some(execution)) => (StatusCode::OK, Json(execution)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn list_workflow_executions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.exec_repo.get_by_workflow_id(id).await {
        Ok(executions) => (StatusCode::OK, Json(executions)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
