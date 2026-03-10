use crate::execution_events::ExecutionEventHub;
use crate::operations::OperationsRuntime;
use crate::repositories::{
    api_key::ApiKeyRepository, credential::CredentialRepository, execution::ExecutionRepository,
    execution_dispatch::ExecutionDispatchRepository, execution_log::ExecutionLogRepository, governance::GovernanceRepository,
    workflow::WorkflowRepository, workspace::WorkspaceRepository,
};
use axum::Router;
use barqflow_db::users::UserRepo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone)]
pub struct ActiveExecutionControl {
    pub cancellation_token: CancellationToken,
    pub abort_handle: tokio::task::AbortHandle,
}

pub type ActiveExecutionManager = Arc<RwLock<HashMap<Uuid, ActiveExecutionControl>>>;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::active_workflows::ActiveCronJobs;
use crate::controllers::{
    credentials::{credential_routes, AppState as CredState},
    executions::{execution_routes, AppState as ExecState},
    governance::{governance_routes, AppState as GovernanceState},
    health::{health_routes, AppState as HealthState},
    identity::{identity_routes, AppState as IdentityState},
    nodes::{node_routes, AppState as NodeState},
    oauth2::{oauth2_routes, OAuth2State},
    observability::{observability_routes, AppState as ObservabilityState},
    settings::{settings_routes, AppState as SettingsState},
    studio::{studio_routes, AppState as StudioState},
    users::{user_routes, AppState as UserState},
    webhooks::{webhook_routes, WebhookRegistry, WebhookState},
    workflows::{workflow_routes, AppState as WfState},
};

#[derive(Clone)]
pub struct AppState {
    pub workflow_repo: Arc<WorkflowRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub exec_repo: Arc<ExecutionRepository>,
    pub execution_dispatch_repo: Arc<ExecutionDispatchRepository>,
    pub execution_log_repo: Arc<ExecutionLogRepository>,
    pub governance_repo: Arc<GovernanceRepository>,
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_registry: Arc<barqflow_registry::registry::CredentialRegistry>,
    pub webhook_registry: WebhookRegistry,
    pub job_scheduler: JobScheduler,
    pub active_executions: ActiveExecutionManager,
    pub active_cron_jobs: ActiveCronJobs,
    pub execution_events: ExecutionEventHub,
    pub operations_runtime: OperationsRuntime,
}

pub fn create_router(state: AppState) -> Router {
    let rest_routes = Router::new()
        .merge(user_routes(UserState {
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
        }))
        .merge(identity_routes(IdentityState {
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
        }))
        .merge(workflow_routes(WfState {
            workflow_repo: Arc::clone(&state.workflow_repo),
            credential_repo: Arc::clone(&state.credential_repo),
            governance_repo: Arc::clone(&state.governance_repo),
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
            node_registry: Arc::clone(&state.node_registry),
            webhook_registry: Arc::clone(&state.webhook_registry),
            job_scheduler: state.job_scheduler.clone(),
            active_cron_jobs: Arc::clone(&state.active_cron_jobs),
        }))
        .merge(execution_routes(ExecState {
            execution_repo: Arc::clone(&state.exec_repo),
            execution_dispatch_repo: Arc::clone(&state.execution_dispatch_repo),
            execution_log_repo: Arc::clone(&state.execution_log_repo),
            workflow_repo: Arc::clone(&state.workflow_repo),
            node_registry: Arc::clone(&state.node_registry),
            credential_repo: Arc::clone(&state.credential_repo),
            governance_repo: Arc::clone(&state.governance_repo),
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
            active_executions: Arc::clone(&state.active_executions),
            execution_events: state.execution_events.clone(),
            operations_runtime: state.operations_runtime.clone(),
        }))
        .merge(credential_routes(CredState {
            credential_repo: Arc::clone(&state.credential_repo),
            governance_repo: Arc::clone(&state.governance_repo),
            credential_registry: Arc::clone(&state.credential_registry),
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
        }))
        .merge(governance_routes(GovernanceState {
            governance_repo: Arc::clone(&state.governance_repo),
            workflow_repo: Arc::clone(&state.workflow_repo),
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
        }))
        .merge(oauth2_routes(OAuth2State {
            credential_repo: Arc::clone(&state.credential_repo),
        }))
        .merge(settings_routes(SettingsState {
            node_registry: Arc::clone(&state.node_registry),
            credential_registry: Arc::clone(&state.credential_registry),
            execution_repo: Arc::clone(&state.exec_repo),
            execution_log_repo: Arc::clone(&state.execution_log_repo),
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
            webhook_registry: Arc::clone(&state.webhook_registry),
            active_cron_jobs: Arc::clone(&state.active_cron_jobs),
            active_executions: Arc::clone(&state.active_executions),
            operations_runtime: state.operations_runtime.clone(),
        }))
        .merge(observability_routes(ObservabilityState {
            workflow_repo: Arc::clone(&state.workflow_repo),
            credential_repo: Arc::clone(&state.credential_repo),
            execution_repo: Arc::clone(&state.exec_repo),
            execution_log_repo: Arc::clone(&state.execution_log_repo),
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
        }))
        .merge(studio_routes(StudioState {
            user_repo: Arc::clone(&state.user_repo),
            workspace_repo: Arc::clone(&state.workspace_repo),
            api_key_repo: Arc::clone(&state.api_key_repo),
            node_registry: Arc::clone(&state.node_registry),
        }))
        .merge(health_routes(HealthState {
            webhook_registry: Arc::clone(&state.webhook_registry),
            active_cron_jobs: Arc::clone(&state.active_cron_jobs),
            active_executions: Arc::clone(&state.active_executions),
            node_registry: Arc::clone(&state.node_registry),
            credential_registry: Arc::clone(&state.credential_registry),
            operations_runtime: state.operations_runtime.clone(),
        }))
        .nest(
            "/nodes",
            node_routes(NodeState {
                node_registry: Arc::clone(&state.node_registry),
            }),
        );

    let webh_routes = webhook_routes(WebhookState {
        workflow_repo: Arc::clone(&state.workflow_repo),
        webhook_registry: Arc::clone(&state.webhook_registry),
        node_registry: Arc::clone(&state.node_registry),
        credential_repo: Arc::clone(&state.credential_repo),
        governance_repo: Arc::clone(&state.governance_repo),
    });

    Router::new()
        .nest("/rest", rest_routes)
        .nest("/webhook", webh_routes)
        .fallback_service(
            ServeDir::new("web/dist").not_found_service(ServeFile::new("web/dist/index.html")),
        )
        .layer(TraceLayer::new_for_http())
}
