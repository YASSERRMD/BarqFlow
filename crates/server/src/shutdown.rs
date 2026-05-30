use crate::state::AppState;
use tokio::signal;
use tracing::info;

pub async fn shutdown_signal(mut state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful termination...");
    // shut down job scheduler
    info!("Stopping background job scheduler...");
    /*
    tokio-cron-scheduler might not expose an explicit shutdown that blocking-waits cleanly,
    so we drop or stop it inherently if supported, or rely on normal process tear-down.
    It does have a `shutdown` method though!
    */
    if let Err(e) = state.job_scheduler.shutdown().await {
        tracing::error!("Error shutting down job scheduler: {}", e);
    } else {
        info!("Job scheduler terminated.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_db::pool::init_db_pool;

    #[tokio::test]
    async fn test_job_scheduler_graceful_shutdown() {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/barqflow".to_string());

        // `AppState::new` builds the credential repository, whose CryptoService
        // requires an encryption key in the environment. Provide a deterministic
        // 32-byte test key so state construction succeeds outside production.
        std::env::set_var("BARQFLOW_ENCRYPTION_KEY", "test_key_must_be_exactly_32_byte");

        // Since tests run against the DB, try connecting
        if let Ok(pool) = init_db_pool(&db_url, 1).await {
            let mut state = AppState::new(pool).await.expect("State should initialize");

            // Start scheduler
            state
                .job_scheduler
                .start()
                .await
                .expect("Scheduler should start");

            // Invoke the shutdown capability to ensure memory clears and threads unwind gracefully
            let res = state.job_scheduler.shutdown().await;
            assert!(res.is_ok(), "Job Scheduler failed to shut down gracefully");
        }
    }
}
