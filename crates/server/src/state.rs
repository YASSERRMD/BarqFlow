use barqflow_api::AppState as ApiState;
use barqflow_db::{CredentialRepo, ExecutionRepo, StaticDataRepo, WorkflowRepo};
use barqflow_db::users::UserRepo;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub workflow_repo: Arc<WorkflowRepo>,
    pub execution_repo: Arc<ExecutionRepo>,
    pub credential_repo: Arc<CredentialRepo>,
    pub static_data_repo: Arc<StaticDataRepo>,
    pub user_repo: Arc<UserRepo>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            db_pool: pool.clone(),
            workflow_repo: Arc::new(WorkflowRepo::new(pool.clone())),
            execution_repo: Arc::new(ExecutionRepo::new(pool.clone())),
            credential_repo: Arc::new(CredentialRepo::new(pool.clone())),
            static_data_repo: Arc::new(StaticDataRepo::new(pool.clone())),
            user_repo: Arc::new(UserRepo::new(pool)),
        }
    }

    pub fn into_api_state(&self) -> ApiState {
        ApiState {
            workflow_repo: Arc::clone(&self.workflow_repo),
            credential_repo: Arc::clone(&self.credential_repo),
            exec_repo: Arc::clone(&self.execution_repo),
            user_repo: Arc::clone(&self.user_repo),
        }
    }
}
