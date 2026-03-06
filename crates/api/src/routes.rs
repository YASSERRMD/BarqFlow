use axum::Router;
use barqflow_db::users::UserRepo;
use crate::repositories::{
    credential::CredentialRepository, execution::ExecutionRepository, workflow::WorkflowRepository,
};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

use crate::controllers::{
    credentials::{credential_routes, AppState as CredState},
    executions::{execution_routes, AppState as ExecState},
    oauth2::{oauth2_routes, OAuth2State},
    users::{user_routes, AppState as UserState},
    webhooks::{new_webhook_registry, webhook_routes, WebhookRegistry, WebhookState},
    workflows::{workflow_routes, AppState as WfState},
};

#[derive(Clone)]
pub struct AppState {
    pub workflow_repo: Arc<WorkflowRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub exec_repo: Arc<ExecutionRepository>,
    pub user_repo: Arc<UserRepo>,
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_registry: Arc<barqflow_registry::registry::CredentialRegistry>,
    pub webhook_registry: WebhookRegistry,
}

pub fn create_router(state: AppState) -> Router {
    let rest_routes = Router::new()
        .merge(user_routes(UserState {
            user_repo: Arc::clone(&state.user_repo),
        }))
        .merge(workflow_routes(WfState {
            workflow_repo: Arc::clone(&state.workflow_repo),
        }))
        .merge(execution_routes(ExecState {
            execution_repo: Arc::clone(&state.exec_repo),
            workflow_repo: Arc::clone(&state.workflow_repo),
            node_registry: Arc::clone(&state.node_registry),
        }))
        .merge(credential_routes(CredState {
            credential_repo: Arc::clone(&state.credential_repo),
            credential_registry: Arc::clone(&state.credential_registry),
        }))
        .merge(oauth2_routes(OAuth2State {
            credential_repo: Arc::clone(&state.credential_repo),
        }));

    let webh_routes = webhook_routes(WebhookState {
        workflow_repo: Arc::clone(&state.workflow_repo),
        webhook_registry: Arc::clone(&state.webhook_registry),
        node_registry: Arc::clone(&state.node_registry),
    });

    Router::new()
        .nest("/rest", rest_routes)
        .nest("/webhook", webh_routes)
        .fallback_service(
            ServeDir::new("web/dist").not_found_service(ServeFile::new("web/dist/index.html")),
        )
}
