use crate::active_workflows::ActiveCronJobs;
use crate::ai_builder::generate_workflow_draft;
use crate::auth::{require_authenticated_user, AuthenticatedUser};
use crate::contracts::{
    AiWorkflowDraftResponse, ExtensionActionInvocationResponse, ExtensionBundleResponse,
    InvokeExtensionActionRequest,
};
use crate::controllers::webhooks::WebhookRegistry;
use crate::extension_runtime::{invoke_extension_action, ExtensionRuntimeContext};
use crate::extensions::discover_extensions;
use crate::operations::OperationsRuntime;
use crate::repositories::execution_dispatch::ExecutionDispatchRepository;
use crate::repositories::{api_key::ApiKeyRepository, workspace::WorkspaceRepository};
use crate::routes::ActiveExecutionManager;
use axum::http::{HeaderMap, StatusCode};
use axum::{
    extract::{Json, Path, State},
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
    pub webhook_registry: WebhookRegistry,
    pub active_cron_jobs: ActiveCronJobs,
    pub active_executions: ActiveExecutionManager,
    pub execution_dispatch_repo: Arc<ExecutionDispatchRepository>,
    pub operations_runtime: OperationsRuntime,
}

pub fn studio_routes(state: AppState) -> Router {
    Router::new()
        .route("/studio/extensions", get(list_extensions))
        .route(
            "/studio/extensions/{bundle_id}/invoke",
            post(invoke_extension_bundle_action),
        )
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

async fn invoke_extension_bundle_action(
    headers: HeaderMap,
    Path(bundle_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<InvokeExtensionActionRequest>,
) -> Result<Json<ExtensionActionInvocationResponse>, (StatusCode, String)> {
    let _auth = require_studio_auth(&headers, &state).await?;
    let bundles = discover_extensions(&state.node_registry)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    let Some(bundle) = bundles.iter().find(|bundle| bundle.id == bundle_id) else {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Extension bundle '{}' was not found.", bundle_id),
        ));
    };
    if !bundle
        .actions
        .iter()
        .any(|action| action.id == payload.action_id)
    {
        return Err((
            StatusCode::NOT_FOUND,
            format!(
                "Extension action '{}' was not found in bundle '{}'.",
                payload.action_id, bundle_id
            ),
        ));
    }

    let runtime = ExtensionRuntimeContext {
        webhook_registry: Arc::clone(&state.webhook_registry),
        active_cron_jobs: Arc::clone(&state.active_cron_jobs),
        active_executions: Arc::clone(&state.active_executions),
        execution_dispatch_repo: Arc::clone(&state.execution_dispatch_repo),
        operations_runtime: state.operations_runtime.clone(),
        node_registry: Arc::clone(&state.node_registry),
    };
    let invocation = invoke_extension_action(
        bundle,
        &payload.action_id,
        payload.context.unwrap_or_else(empty_context),
        &runtime,
    )
    .await
    .map_err(|error| (StatusCode::FORBIDDEN, error))?;

    Ok(Json(invocation))
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

fn empty_context() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_context_builder_returns_object() {
        assert!(empty_context().is_object());
    }
}
