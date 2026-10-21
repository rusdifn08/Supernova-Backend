use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    http::{ApiResponse, AuthContext},
    modules::pos::{
        dto::{
            CategoryDto, CreatePosOrderRequest, CreateProductRequest, PosOrderDto, ProductDto,
            StockAdjustmentRequest,
        },
        service,
    },
    state::AppState,
};

pub async fn list_categories(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CategoryDto>>>, AppError> {
    let list = service::list_categories(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::json(list))
}

pub async fn list_products(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ProductDto>>>, AppError> {
    let list = service::list_products(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::json(list))
}

pub async fn create_product(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<ApiResponse<ProductDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let res = service::create_product(&state.pool, auth.tenant_id, payload).await?;
    Ok(ApiResponse::json(res))
}

pub async fn adjust_stock(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<StockAdjustmentRequest>,
) -> Result<Json<ApiResponse<ProductDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let res = service::adjust_stock(&state.pool, auth.tenant_id, id, payload).await?;
    Ok(ApiResponse::json(res))
}

pub async fn create_order(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CreatePosOrderRequest>,
) -> Result<Json<ApiResponse<PosOrderDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let res = service::create_pos_order(&state.pool, auth.tenant_id, payload).await?;
    Ok(ApiResponse::json(res))
}

pub async fn list_orders(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PosOrderDto>>>, AppError> {
    let list = service::list_pos_orders(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::json(list))
}
