use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FinanceSummaryDto {
    pub total_revenue: f64,
    pub total_cogs: f64,
    pub gross_profit: f64,
    pub profit_margin_pct: f64,
    pub total_orders_count: i64,
    pub low_stock_items_count: i64,
    pub active_employees_count: i64,
}
