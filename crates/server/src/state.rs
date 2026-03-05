use barqflow_api::AppState as ApiState;
use barqflow_db::users::UserRepo;
use barqflow_db::{CredentialRepo, ExecutionRepo, StaticDataRepo, WorkflowRepo};
use sqlx::PgPool;
use std::sync::Arc;
use barqflow_registry::registry::NodeRegistry;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub workflow_repo: Arc<WorkflowRepo>,
    pub execution_repo: Arc<ExecutionRepo>,
    pub credential_repo: Arc<CredentialRepo>,
    pub static_data_repo: Arc<StaticDataRepo>,
    pub user_repo: Arc<UserRepo>,
    pub node_registry: Arc<NodeRegistry>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let node_registry = Arc::new(NodeRegistry::new());
        barqflow_nodes::register_all_nodes(&node_registry);

        Self {
            db_pool: pool.clone(),
            workflow_repo: Arc::new(WorkflowRepo::new(pool.clone())),
            execution_repo: Arc::new(ExecutionRepo::new(pool.clone())),
            credential_repo: Arc::new(CredentialRepo::new(pool.clone())),
            static_data_repo: Arc::new(StaticDataRepo::new(pool.clone())),
            user_repo: Arc::new(UserRepo::new(pool)),
            node_registry,
        }
    }

    pub fn into_api_state(&self) -> ApiState {
        ApiState {
            workflow_repo: Arc::clone(&self.workflow_repo),
            credential_repo: Arc::clone(&self.credential_repo),
            exec_repo: Arc::clone(&self.execution_repo),
            user_repo: Arc::clone(&self.user_repo),
            node_registry: Arc::clone(&self.node_registry),
        }
    }
}
