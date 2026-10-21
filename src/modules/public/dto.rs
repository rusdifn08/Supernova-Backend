use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicTenantDto {
    pub slug: String,
    pub code: String,
    pub name: String,
    pub business_type: String,
    pub city: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicResourceDto {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub hourly_price: f64,
    pub capacity: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicResourceAvailabilityDto {
    pub resource: PublicResourceDto,
    pub booked_slots: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublicReserveRequest {
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub resource_id: Uuid,
    pub booking_date: NaiveDate,
    pub start_time: NaiveTime,
    pub duration_hours: i32,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicReserveResponse {
    pub booking_code: String,
    pub resource_name: String,
    pub booking_date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub total_amount: f64,
    pub down_payment: f64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicProductDto {
    pub id: Uuid,
    pub name: String,
    pub sku: String,
    pub category_name: String,
    pub price: f64,
    pub stock: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PublicPosOrderItemRequest {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PublicPosOrderRequest {
    pub customer_name: String,
    pub customer_phone: String,
    pub payment_method: Option<String>,
    pub items: Vec<PublicPosOrderItemRequest>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicPosOrderResponse {
    pub invoice_number: String,
    pub total_amount: f64,
    pub payment_status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicMenuItemDto {
    pub id: Uuid,
    pub category_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub is_available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicRestoTableDto {
    pub table_id: Uuid,
    pub table_number: String,
    pub section_area: String,
    pub menu: Vec<PublicMenuItemDto>,
}

#[derive(Debug, Deserialize)]
pub struct PublicRestoOrderItemRequest {
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub variant: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublicRestoOrderRequest {
    pub qr_token: String,
    pub customer_name: Option<String>,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<PublicRestoOrderItemRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicRestoOrderResponse {
    pub order_code: String,
    pub table_number: String,
    pub subtotal: f64,
    pub tax_service: f64,
    pub total_amount: f64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CallWaiterRequest {
    pub qr_token: String,
    pub request_type: String,
    pub note: Option<String>,
}
