pub mod migrations;

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::info;

use crate::error::AppError;

pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    info!("Initializing connection pool to Neon PostgreSQL...");

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(300))
        .connect(database_url)
        .await
        .map_err(|e| {
            tracing::error!("Neon PostgreSQL connection error: {}", e);
            AppError::DatabaseError(e)
        })?;

    info!("Connected to Neon PostgreSQL successfully.");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    migrations::run_migrations(pool).await
}
