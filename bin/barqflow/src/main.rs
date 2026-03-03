use anyhow::Result;
use dotenvy::dotenv;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    let _ = dotenv();

    // Initialize tracing (logging)
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting BarqFlow...");

    // Extract configuration
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    // Start server
    barqflow_server::run_server(&db_url, port).await?;

    Ok(())
}
