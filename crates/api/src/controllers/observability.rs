use crate::auth::{require_authenticated_user, AuthenticatedUser};
use crate::contracts::ObservabilityOverviewResponse;
use crate::observability::{build_observability_overview, clamp_observability_window};
use crate::repositories::{
    api_key::ApiKeyRepository, credential::CredentialRepository, execution::ExecutionRepository,
    execution_log::ExecutionLogRepository, workflow::WorkflowRepository,
    workspace::WorkspaceRepository,
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use barqflow_db::users::UserRepo;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub workflow_repo: Arc<WorkflowRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub execution_repo: Arc<ExecutionRepository>,
    pub execution_log_repo: Arc<ExecutionLogRepository>,
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ObservabilityQuery {
    hours: Option<u32>,
}

pub fn observability_routes(state: AppState) -> Router {
    Router::new()
        .route("/observability/overview", get(get_observability_overview))
        .with_state(state)
}

async fn get_observability_overview(
    headers: HeaderMap,
    Query(query): Query<ObservabilityQuery>,
    State(state): State<AppState>,
) -> Result<Json<ObservabilityOverviewResponse>, (StatusCode, String)> {
    let auth = require_observability_auth(&headers, &state).await?;
    let window_hours = clamp_observability_window(query.hours);

    let overview = build_observability_overview(
        &state.workflow_repo,
        &state.execution_repo,
        &state.execution_log_repo,
        &state.credential_repo,
        auth.workspace_id,
        window_hours,
    )
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(overview))
}

async fn require_observability_auth(
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
