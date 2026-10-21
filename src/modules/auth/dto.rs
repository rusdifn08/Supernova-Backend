use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub business_name: String,
    pub business_type: String,
    pub phone: String,
    pub city: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TenantDto {
    pub id: Uuid,
    pub slug: String,
    pub code: String,
    pub name: String,
    pub business_type: String,
    pub city: Option<String>,
    pub phone: Option<String>,
    pub plan: String,
    pub trial_expires_at: DateTime<Utc>,
    pub days_remaining: i64,
    pub is_trial_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionDto {
    pub token: String,
    pub user: UserDto,
    pub tenant: TenantDto,
}
