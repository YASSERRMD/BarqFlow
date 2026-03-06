use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use uuid::Uuid;

/// Initialize the Database Pool with max connections
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}

/// Run pending migrations automatically on boot
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
}

/// Generate a new UUID v4
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}
