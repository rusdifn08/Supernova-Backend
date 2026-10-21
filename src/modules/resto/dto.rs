use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TableDto {
    pub id: Uuid,
    pub table_number: String,
    pub qr_token: String,
    pub section_area: String,
    pub capacity: i32,
    pub status: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTableRequest {
    pub table_number: String,
    pub section_area: Option<String>,
    pub capacity: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MenuItemDto {
    pub id: Uuid,
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub is_available: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateMenuItemRequest {
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KitchenTicketItemDto {
    pub name: String,
    pub quantity: i32,
    pub variant: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KitchenTicketDto {
    pub id: Uuid,
    pub order_code: String,
    pub table_number: String,
    pub section_area: String,
    pub status: String,
    pub time_elapsed: String,
    pub items: Vec<KitchenTicketItemDto>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ServiceRequestDto {
    pub id: Uuid,
    pub table_number: String,
    pub section_area: String,
    pub request_type: String,
    pub note: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoStatsDto {
    pub revenue_today: f64,
    pub active_tables: i64,
    pub total_tables: i64,
    pub avg_cook_minutes: f64,
    pub open_requests: i64,
}
