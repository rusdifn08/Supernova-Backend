use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use crate::error::AppError;

pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    tracing::info!("Initializing PostgreSQL connection pool to Neon database...");

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(300))
        .connect(database_url)
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to Neon PostgreSQL: {}", e);
            AppError::DatabaseError(e)
        })?;

    tracing::info!("Successfully connected to Neon PostgreSQL pool.");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    tracing::info!("Running database schema initialization and seed checks...");

    let schema_sql = include_str!("schema.sql");
    sqlx::raw_sql(schema_sql)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute schema migrations: {}", e);
            AppError::DatabaseError(e)
        })?;

    tracing::info!("Database schema and seed data are ready.");
    Ok(())
}
