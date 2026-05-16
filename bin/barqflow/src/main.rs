use anyhow::Result;
use barqflow_server::config::AppConfig;
use dotenvy::dotenv;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (ignored when not found).
    let _ = dotenv();

    // Validate all required environment variables before doing anything else.
    // Panics with a clear numbered list if any variable is missing or invalid.
    let config = AppConfig::from_env();

    // Initialize tracing (logging).
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting BarqFlow on port {}...", config.port);

    barqflow_server::server::run_server(&config.database_url, config.port).await?;

    Ok(())
}
