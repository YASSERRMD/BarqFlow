use barqflow_api::AppState as ApiState;
use barqflow_db::users::UserRepo;
use barqflow_api::repositories::{
    credential::CredentialRepository, execution::ExecutionRepository,
    static_data::StaticDataRepository, workflow::WorkflowRepository,
};
use sqlx::PgPool;
use std::sync::Arc;
use barqflow_registry::registry::NodeRegistry;
use barqflow_registry::registry::CredentialRegistry;
use barqflow_api::controllers::webhooks::{WebhookRegistry, new_webhook_registry};
use tokio_cron_scheduler::JobScheduler;
use barqflow_api::routes::ActiveExecutionManager;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub workflow_repo: Arc<WorkflowRepository>,
    pub execution_repo: Arc<ExecutionRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub static_data_repo: Arc<StaticDataRepository>,
    pub user_repo: Arc<UserRepo>,
    pub node_registry: Arc<NodeRegistry>,
    pub credential_registry: Arc<CredentialRegistry>,
    pub webhook_registry: WebhookRegistry,
    pub job_scheduler: JobScheduler,
    pub active_executions: ActiveExecutionManager,
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
        let active_executions = Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));

        Ok(Self {
            db_pool: pool.clone(),
            workflow_repo: Arc::new(WorkflowRepository::new(pool.clone())),
            execution_repo: Arc::new(ExecutionRepository::new(pool.clone())),
            credential_repo: Arc::new(CredentialRepository::new(pool.clone())),
            static_data_repo: Arc::new(StaticDataRepository::new(pool.clone())),
            user_repo: Arc::new(UserRepo::new(pool)),
            node_registry,
            credential_registry,
            webhook_registry: new_webhook_registry(),
            job_scheduler,
            active_executions,
        })
    }

    pub fn into_api_state(&self) -> ApiState {
        ApiState {
            workflow_repo: Arc::clone(&self.workflow_repo),
            credential_repo: Arc::clone(&self.credential_repo),
            exec_repo: Arc::clone(&self.execution_repo),
            user_repo: Arc::clone(&self.user_repo),
            node_registry: Arc::clone(&self.node_registry),
            credential_registry: Arc::clone(&self.credential_registry),
            webhook_registry: Arc::clone(&self.webhook_registry),
            job_scheduler: self.job_scheduler.clone(),
            active_executions: Arc::clone(&self.active_executions),
        }
    }
}
