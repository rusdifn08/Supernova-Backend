use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrialInquiry {
    pub id: Uuid,
    pub business_name: String,
    pub business_type: String,
    pub contact_name: String,
    pub phone_wa: String,
    pub email: Option<String>,
    pub interested_service: String,
    pub estimated_outlets: i32,
    pub current_pain_points: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrialRequest {
    pub business_name: String,
    pub business_type: String,
    pub contact_name: String,
    pub phone_wa: String,
    pub email: Option<String>,
    pub interested_service: String,
    pub estimated_outlets: Option<i32>,
    pub current_pain_points: Option<String>,
}
