use anyhow::Result;
use barqflow_api::create_router;
use barqflow_db::pool::init_db_pool;
use crate::state::AppState;
use crate::boot::run_boot_sequence;
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
    let state = AppState::new(pool).await?;

    // Run active boot sequence
    if let Err(e) = run_boot_sequence(&state).await {
        anyhow::bail!("Boot sequence failed: {:?}", e);
    }

    // Create router using state converted for the API layer
    let app = create_router(state.into_api_state());

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("BarqFlow server listening on http://{}", addr);

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
