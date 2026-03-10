use crate::ai_builder::generate_workflow_draft;
use crate::auth::{require_authenticated_user, AuthenticatedUser};
use crate::contracts::{AiWorkflowDraftResponse, ExtensionBundleResponse};
use crate::extensions::discover_extensions;
use crate::repositories::{api_key::ApiKeyRepository, workspace::WorkspaceRepository};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use barqflow_db::users::UserRepo;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
}

pub fn studio_routes(state: AppState) -> Router {
    Router::new()
        .route("/studio/extensions", get(list_extensions))
        .route("/studio/workflow-drafts", post(create_workflow_draft))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDraftRequest {
    pub prompt: String,
}

async fn list_extensions(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ExtensionBundleResponse>>, (StatusCode, String)> {
    let _auth = require_studio_auth(&headers, &state).await?;
    let bundles = discover_extensions(&state.node_registry)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    Ok(Json(bundles))
}

async fn create_workflow_draft(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<WorkflowDraftRequest>,
) -> Result<Json<AiWorkflowDraftResponse>, (StatusCode, String)> {
    let _auth = require_studio_auth(&headers, &state).await?;
    let bundles = discover_extensions(&state.node_registry)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    let draft = generate_workflow_draft(&payload.prompt, &state.node_registry, &bundles)
        .map_err(|error| (StatusCode::BAD_REQUEST, error))?;
    Ok(Json(draft))
}

async fn require_studio_auth(
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
