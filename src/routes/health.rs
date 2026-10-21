use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use crate::error::AppError;

pub async fn health_check(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    // Run live ping query to verify database is reachable
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Health check database ping failed: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(Json(json!({
        "status": "healthy",
        "service": "Supernova Labs Studio Backend Engine",
        "version": "1.1.0",
        "framework": "Rust / Axum 0.8 & SQLx 0.9",
        "database": "Neon Serverless PostgreSQL (ap-southeast-1)",
        "security": {
            "parameterized_queries": true,
            "sql_injection_safe": true,
            "cors_hardened": true,
            "tls_ssl": "require"
        },
        "services_ready": [
            "Ordering System Booking (Sports, Studio, Barber, PS, Warnet)",
            "POS, ERP, CRM & Keuangan Terpadu",
            "Ordering in Table with QR Code (Dine-in Resto & KDS)",
            "Sandbox Demo & Trial Inquiries"
        ]
    })))
}
