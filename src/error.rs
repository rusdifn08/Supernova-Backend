use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Conflict detected: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[allow(dead_code)]
    #[error("Internal server error")]
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, client_message, error_code) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), "NOT_FOUND"),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.clone(), "VALIDATION_FAILED"),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone(), "SLOT_CONFLICT"),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone(), "BAD_REQUEST"),
            AppError::DatabaseError(err) => {
                tracing::error!("Internal SQLx Database Error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected database error occurred. Please contact support.".to_string(),
                    "DATABASE_ERROR",
                )
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal Server Error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error occurred.".to_string(),
                    "INTERNAL_SERVER_ERROR",
                )
            }
        };

        let body = Json(json!({
            "success": false,
            "error_code": error_code,
            "message": client_message
        }));

        (status, body).into_response()
    }
}
