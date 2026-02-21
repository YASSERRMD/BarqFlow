use axum::Router;
use barqflow_db::{CredentialRepo, ExecutionRepo, WorkflowRepo};
use barqflow_db::users::UserRepo;
use std::sync::Arc;

use crate::controllers::{
    credentials::{credential_routes, AppState as CredState},
    executions::{execution_routes, AppState as ExecState},
    users::{user_routes, AppState as UserState},
    webhooks::{webhook_routes, WebhookState},
    workflows::{workflow_routes, AppState as WfState},
};

#[derive(Clone)]
pub struct AppState {
    pub workflow_repo: Arc<WorkflowRepo>,
    pub credential_repo: Arc<CredentialRepo>,
    pub exec_repo: Arc<ExecutionRepo>,
    pub user_repo: Arc<UserRepo>,
}

pub fn create_router(state: AppState) -> Router {
    let rest_routes = Router::new()
        .merge(user_routes(UserState { user_repo: Arc::clone(&state.user_repo) }))
        .merge(workflow_routes(WfState { workflow_repo: Arc::clone(&state.workflow_repo) }))
        .merge(execution_routes(ExecState { execution_repo: Arc::clone(&state.exec_repo) }))
        .merge(credential_routes(CredState { credential_repo: Arc::clone(&state.credential_repo) }));

    let webh_routes = webhook_routes(WebhookState { 
        workflow_repo: Arc::clone(&state.workflow_repo) 
    });

    Router::new()
        .nest("/rest", rest_routes)
        .nest("/webhook", webh_routes)
}
