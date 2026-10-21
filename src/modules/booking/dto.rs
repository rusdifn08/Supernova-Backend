use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ResourceDto {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub hourly_rate: f64,
    pub capacity: i32,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateResourceRequest {
    pub name: String,
    pub category: String,
    pub hourly_rate: f64,
    pub capacity: i32,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReservationDto {
    pub id: Uuid,
    pub code: String,
    pub resource_id: Uuid,
    pub resource_name: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_hours: f64,
    pub total_amount: f64,
    pub down_payment: f64,
    pub status: String,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReservationRequest {
    pub resource_id: Uuid,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub down_payment: Option<f64>,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReservationRequest {
    pub status: Option<String>,
    #[allow(dead_code)]
    pub payment_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookingStatsDto {
    pub revenue_today: f64,
    pub reservations_today: i64,
    pub occupancy_pct: f64,
    pub upcoming_count: i64,
    pub pending_payment_count: i64,
}
