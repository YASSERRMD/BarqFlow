mod boot;
mod shutdown;
mod state;

use boot::run_boot_sequence;
use state::AppState;

use dotenvy::dotenv;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env

    // 1. Initialize Telemetry
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,barqflow=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting BarqFlow Engine Server...");

    // 2. Initialize Database Pool
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = barqflow_api::db::init_pool(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    info!("Connected to PostgreSQL Database.");

    // Run Migrations
    barqflow_api::db::run_migrations(&pool)
        .await
        .expect("Database Migrations failed");

    info!("Database Migrations successfully completed.");

    // 3. Construct Global State
    let app_state = AppState::new(pool)
        .await
        .expect("Failed to create AppState");

    // 4. Run Active Boot Sequence
    if let Err(e) = run_boot_sequence(&app_state).await {
        error!("Boot sequence failed: {:?}", e);
        std::process::exit(1);
    }

    // 5. Mount API Routes
    let api_router = barqflow_api::create_router(app_state.to_api_state());

    // 6. Bind to Port
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!(
        "Axum Server listening on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, api_router)
        .with_graceful_shutdown(shutdown::shutdown_signal(app_state))
        .await
        .unwrap();
}
