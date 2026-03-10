use barqflow_api::active_workflows::ActiveCronJobs;
use barqflow_api::controllers::webhooks::{new_webhook_registry, WebhookRegistry};
use barqflow_api::execution_events::ExecutionEventHub;
use barqflow_api::operations::OperationsRuntime;
use barqflow_api::repositories::{
    api_key::ApiKeyRepository, credential::CredentialRepository, execution::ExecutionRepository,
    execution_log::ExecutionLogRepository, governance::GovernanceRepository,
    static_data::StaticDataRepository, workflow::WorkflowRepository,
    workspace::WorkspaceRepository,
};
use barqflow_api::routes::ActiveExecutionManager;
use barqflow_api::AppState as ApiState;
use barqflow_db::users::UserRepo;
use barqflow_registry::registry::CredentialRegistry;
use barqflow_registry::registry::NodeRegistry;
use sqlx::PgPool;
use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub workflow_repo: Arc<WorkflowRepository>,
    pub execution_repo: Arc<ExecutionRepository>,
    pub execution_log_repo: Arc<ExecutionLogRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub governance_repo: Arc<GovernanceRepository>,
    pub static_data_repo: Arc<StaticDataRepository>,
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
    pub node_registry: Arc<NodeRegistry>,
    pub credential_registry: Arc<CredentialRegistry>,
    pub webhook_registry: WebhookRegistry,
    pub job_scheduler: JobScheduler,
    pub active_executions: ActiveExecutionManager,
    pub active_cron_jobs: ActiveCronJobs,
    pub execution_events: ExecutionEventHub,
    pub operations_runtime: OperationsRuntime,
}

impl AppState {
    pub async fn new(pool: PgPool) -> anyhow::Result<Self> {
        let node_registry = Arc::new(NodeRegistry::new());
        barqflow_nodes::register_all_nodes(&node_registry);

        // Setup Credential Registry
        let credential_registry = Arc::new(CredentialRegistry::new());
        barqflow_nodes::register_all_credentials(&credential_registry);

        // Setup Scheduler and active execution map
        let job_scheduler = JobScheduler::new().await?;
        let active_executions =
            Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
        let active_cron_jobs = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
        let execution_events = ExecutionEventHub::new();
        let operations_runtime = OperationsRuntime::from_env();

        Ok(Self {
            db_pool: pool.clone(),
            workflow_repo: Arc::new(WorkflowRepository::new(pool.clone())),
            execution_repo: Arc::new(ExecutionRepository::new(pool.clone())),
            execution_log_repo: Arc::new(ExecutionLogRepository::new(pool.clone())),
            credential_repo: Arc::new(CredentialRepository::new(pool.clone())),
            governance_repo: Arc::new(GovernanceRepository::new(pool.clone())),
            static_data_repo: Arc::new(StaticDataRepository::new(pool.clone())),
            user_repo: Arc::new(UserRepo::new(pool.clone())),
            workspace_repo: Arc::new(WorkspaceRepository::new(pool.clone())),
            api_key_repo: Arc::new(ApiKeyRepository::new(pool)),
            node_registry,
            credential_registry,
            webhook_registry: new_webhook_registry(),
            job_scheduler,
            active_executions,
            active_cron_jobs,
            execution_events,
            operations_runtime,
        })
    }

    pub fn into_api_state(&self) -> ApiState {
        ApiState {
            workflow_repo: Arc::clone(&self.workflow_repo),
            credential_repo: Arc::clone(&self.credential_repo),
            governance_repo: Arc::clone(&self.governance_repo),
            exec_repo: Arc::clone(&self.execution_repo),
            execution_log_repo: Arc::clone(&self.execution_log_repo),
            user_repo: Arc::clone(&self.user_repo),
            workspace_repo: Arc::clone(&self.workspace_repo),
            api_key_repo: Arc::clone(&self.api_key_repo),
            node_registry: Arc::clone(&self.node_registry),
            credential_registry: Arc::clone(&self.credential_registry),
            webhook_registry: Arc::clone(&self.webhook_registry),
            job_scheduler: self.job_scheduler.clone(),
            active_executions: Arc::clone(&self.active_executions),
            active_cron_jobs: Arc::clone(&self.active_cron_jobs),
            execution_events: self.execution_events.clone(),
            operations_runtime: self.operations_runtime.clone(),
        }
    }
}
