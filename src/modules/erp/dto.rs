use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct EmployeeDto {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub phone: Option<String>,
    pub shift: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub role: String,
    pub phone: Option<String>,
    pub shift: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryLogDto {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity_delta: i32,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}
