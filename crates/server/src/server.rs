use anyhow::Result;
use barqflow_api::{create_router, AppState};
use barqflow_db::pool::init_db_pool;
use barqflow_db::users::UserRepo;
use barqflow_db::{CredentialRepo, ExecutionRepo, WorkflowRepo};
use std::net::SocketAddr;
use std::sync::Arc;

/// Start the BarqFlow server.
///
/// # Arguments
/// * `db_url` - The database connection URL
/// * `port` - The port to listen on
///
/// # Returns
/// Result indicating success or error
pub async fn run_server(db_url: &str, port: u16) -> Result<()> {
    // Create database connection pool
    let pool = init_db_pool(db_url, 10).await?;

    // Create application state
    let state = AppState {
        workflow_repo: Arc::new(WorkflowRepo::new(pool.clone())),
        credential_repo: Arc::new(CredentialRepo::new(pool.clone())),
        exec_repo: Arc::new(ExecutionRepo::new(pool.clone())),
        user_repo: Arc::new(UserRepo::new(pool)),
    };

    // Create router
    let app = create_router(state);

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("BarqFlow server listening on http://{}", addr);

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
