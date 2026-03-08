use axum::Router;
use barqflow_db::users::UserRepo;
use crate::repositories::{
    credential::CredentialRepository, execution::ExecutionRepository, workflow::WorkflowRepository,
};
use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActiveExecutionControl {
    pub cancellation_token: CancellationToken,
    pub abort_handle: tokio::task::AbortHandle,
}

pub type ActiveExecutionManager = Arc<RwLock<HashMap<Uuid, ActiveExecutionControl>>>;
use tower_http::services::{ServeDir, ServeFile};

use crate::controllers::{
    credentials::{credential_routes, AppState as CredState},
    executions::{execution_routes, AppState as ExecState},
    settings::{settings_routes, AppState as SettingsState},
    oauth2::{oauth2_routes, OAuth2State},
    users::{user_routes, AppState as UserState},
    webhooks::{webhook_routes, WebhookRegistry, WebhookState},
    workflows::{workflow_routes, AppState as WfState},
    nodes::{node_routes, AppState as NodeState},
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
    pub job_scheduler: JobScheduler,
    pub active_executions: ActiveExecutionManager,
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
            credential_repo: Arc::clone(&state.credential_repo),
            active_executions: Arc::clone(&state.active_executions),
        }))
        .merge(credential_routes(CredState {
            credential_repo: Arc::clone(&state.credential_repo),
            credential_registry: Arc::clone(&state.credential_registry),
        }))
        .merge(oauth2_routes(OAuth2State {
            credential_repo: Arc::clone(&state.credential_repo),
        }))
        .merge(settings_routes(SettingsState {
            node_registry: Arc::clone(&state.node_registry),
            credential_registry: Arc::clone(&state.credential_registry),
        }))
        .nest("/nodes", node_routes(NodeState {
            node_registry: Arc::clone(&state.node_registry),
        }));

    let webh_routes = webhook_routes(WebhookState {
        workflow_repo: Arc::clone(&state.workflow_repo),
        webhook_registry: Arc::clone(&state.webhook_registry),
        node_registry: Arc::clone(&state.node_registry),
        credential_repo: Arc::clone(&state.credential_repo),
    });

    Router::new()
        .nest("/rest", rest_routes)
        .nest("/webhook", webh_routes)
        .fallback_service(
            ServeDir::new("web/dist").not_found_service(ServeFile::new("web/dist/index.html")),
        )
}
