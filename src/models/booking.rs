use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BusinessType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BookingResource {
    pub id: Uuid,
    pub business_type_id: String,
    pub name: String,
    pub category: String,
    pub hourly_rate: f64,
    pub capacity: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Booking {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub booking_code: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_hours: f64,
    pub total_amount: f64,
    pub down_payment: f64,
    pub booking_status: String,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
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
pub struct UpdateBookingStatusRequest {
    pub booking_status: Option<String>,
    pub payment_status: Option<String>,
}
