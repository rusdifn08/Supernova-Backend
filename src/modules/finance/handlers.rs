use axum::{extract::State, Json};

use crate::{
    error::AppError,
    http::{ApiResponse, AuthContext},
    modules::finance::dto::FinanceSummaryDto,
    state::AppState,
};

pub async fn get_summary(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<FinanceSummaryDto>>, AppError> {
    let rev = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT SUM(total_amount) FROM pos_orders WHERE tenant_id = $1",
    )
    .bind(auth.tenant_id)
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0.0);

    let orders_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pos_orders WHERE tenant_id = $1",
    )
    .bind(auth.tenant_id)
    .fetch_one(&state.pool)
    .await?;

    let low_stock = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM erp_products WHERE tenant_id = $1 AND stock_quantity <= min_stock_alert",
    )
    .bind(auth.tenant_id)
    .fetch_one(&state.pool)
    .await?;

    let active_emp = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM erp_employees WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(auth.tenant_id)
    .fetch_one(&state.pool)
    .await?;

    let cogs = rev * 0.58; // Standard 58% COGS estimate
    let gross_profit = rev - cogs;
    let profit_margin_pct = if rev > 0.0 { (gross_profit / rev) * 100.0 } else { 0.0 };

    Ok(ApiResponse::json(FinanceSummaryDto {
        total_revenue: rev,
        total_cogs: cogs,
        gross_profit,
        profit_margin_pct,
        total_orders_count: orders_count,
        low_stock_items_count: low_stock,
        active_employees_count: active_emp,
    }))
}
