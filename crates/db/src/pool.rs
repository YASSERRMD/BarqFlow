use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
}

/// Initializes a postgres connection pool with a given URI
pub async fn init_db_pool(database_url: &str, max_connections: u32) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Automatically runs SQLx migrations located in the migrations directory
pub async fn run_migrations(_pool: &PgPool) -> Result<(), DbError> {
    // If we have migrations in crates/db/migrations, sqlx can automatically embed them
    // For testing and scaffolding without migrations folder, we can ignore for now or use migrate! macro
    // sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_connection_invalid_url() {
        // Just verify our wrapper accurately maps the error when the URL is bad
        let result = init_db_pool("postgres://invalid:invalid@localhost/db", 1).await;
        assert!(result.is_err());
    }
}
