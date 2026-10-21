use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RestoTable {
    pub id: Uuid,
    pub table_number: String,
    pub qr_token: String,
    pub section_area: String,
    pub capacity: i32,
    pub status: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RestoMenuItem {
    pub id: Uuid,
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub image_url: Option<String>,
    pub available_variants: serde_json::Value,
    pub is_available: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TableOrderItemInput {
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub selected_variant: Option<String>,
    pub special_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaceTableOrderRequest {
    pub table_qr_token: String,
    pub customer_name: Option<String>,
    pub items: Vec<TableOrderItemInput>,
    pub payment_method: String,
    pub customer_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RestoTableOrder {
    pub id: Uuid,
    pub order_code: String,
    pub table_id: Uuid,
    pub customer_name: Option<String>,
    pub status: String,
    pub payment_method: String,
    pub payment_status: String,
    pub subtotal: f64,
    pub tax_service: f64,
    pub total_amount: f64,
    pub customer_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KitchenTicketItem {
    pub item_name: String,
    pub quantity: i32,
    pub selected_variant: Option<String>,
    pub special_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KitchenTicket {
    pub order_id: Uuid,
    pub order_code: String,
    pub table_number: String,
    pub section_area: String,
    pub customer_name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub items: Vec<KitchenTicketItem>,
    pub customer_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}
