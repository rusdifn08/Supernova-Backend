use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BusinessUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub business_name: String,
    pub business_type: String,
    pub phone_number: String,
    pub city: Option<String>,
    pub service_plan: String,
    pub trial_starts_at: DateTime<Utc>,
    pub trial_expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub business_name: String,
    pub business_type: String,
    pub phone_number: String,
    pub city: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: String,
    pub user: BusinessUser,
    pub days_remaining: i64,
    pub is_trial_active: bool,
}
