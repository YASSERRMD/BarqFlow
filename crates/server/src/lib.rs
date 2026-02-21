use std::sync::Arc;
use sqlx::PgPool;
use tokio::net::TcpListener;
use barqflow_api::{create_router, AppState};
use barqflow_db::{WorkflowRepo, CredentialRepo, ExecutionRepo};
use tracing::{info, error};

pub async fn run_server(db_url: &str, port: u16) -> anyhow::Result<()> {
    info!("Connecting to the database: {}", db_url);
    
    let pool = PgPool::connect(db_url).await?;
    info!("Database connection established");

    let state = AppState {
        workflow_repo: Arc::new(WorkflowRepo::new(pool.clone())),
        credential_repo: Arc::new(CredentialRepo::new(pool.clone())),
        exec_repo: Arc::new(ExecutionRepo::new(pool.clone())),
    };

    let app = create_router(state);
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("Starting BarqFlow server on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
