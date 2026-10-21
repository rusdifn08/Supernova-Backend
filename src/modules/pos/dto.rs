use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ProductDto {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub category_id: Option<String>,
    pub unit: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub stock_quantity: i32,
    pub min_stock_alert: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub name: String,
    pub category_id: Option<String>,
    pub unit: Option<String>,
    pub buy_price: f64,
    pub sell_price: f64,
    pub stock_quantity: i32,
    pub min_stock_alert: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct StockAdjustmentRequest {
    pub quantity_delta: i32,
    pub reason: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderItemInput {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PosOrderItemDto {
    pub product_id: Uuid,
    pub name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub line_total: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PosOrderDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub subtotal: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub amount_paid: f64,
    pub change_amount: f64,
    pub payment_method: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePosOrderRequest {
    pub customer_id: Option<Uuid>,
    pub items: Vec<OrderItemInput>,
    pub payment_method: String,
    pub amount_paid: Option<f64>,
    pub discount_amount: Option<f64>,
    pub notes: Option<String>,
}
