use anyhow::Result;
use barqflow_api::{create_router, AppState};
use barqflow_db::pool::init_db_pool;
use barqflow_db::users::UserRepo;
use barqflow_api::repositories::{
    credential::CredentialRepository, execution::ExecutionRepository, workflow::WorkflowRepository,
};
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

    // Initialize the Node Registry and populate standard core nodes
    let node_registry = Arc::new(barqflow_registry::registry::NodeRegistry::new());
    barqflow_nodes::register_all_nodes(&node_registry);

    // Initialize Credential Registry
    let credential_registry = Arc::new(barqflow_registry::registry::CredentialRegistry::new());

    // Create application state
    let state = AppState {
        workflow_repo: Arc::new(WorkflowRepository::new(pool.clone())),
        credential_repo: Arc::new(CredentialRepository::new(pool.clone())),
        exec_repo: Arc::new(ExecutionRepository::new(pool.clone())),
        user_repo: Arc::new(UserRepo::new(pool)),
        node_registry,
        credential_registry,
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
