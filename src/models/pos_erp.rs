use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub outlet_id: Option<Uuid>,
    pub sku: String,
    pub name: String,
    pub category_id: Option<String>,
    pub unit: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub stock_quantity: i32,
    pub min_stock_alert: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub tier: String,
    pub loyalty_points: i32,
    pub total_spent: f64,
    pub last_visit: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Employee {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub phone: Option<String>,
    pub status: String,
    pub shift: String,
    pub attendance_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct OrderItemInput {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreatePosOrderRequest {
    pub customer_id: Option<Uuid>,
    pub cashier_id: Option<Uuid>,
    pub items: Vec<OrderItemInput>,
    pub payment_method: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PosOrder {
    pub id: Uuid,
    pub invoice_number: String,
    pub subtotal: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub payment_method: String,
    pub payment_status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinanceSummary {
    pub total_revenue: f64,
    pub total_cogs: f64,
    pub gross_profit: f64,
    pub profit_margin_pct: f64,
    pub total_orders_count: i64,
    pub low_stock_items_count: i64,
    pub active_employees_count: i64,
}
