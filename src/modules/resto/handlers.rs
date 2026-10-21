use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    http::{extract::AuthContext, response::ApiResponse},
    state::AppState,
};
use super::{dto::*, service::RestoService};

#[derive(Debug, Deserialize)]
pub struct MenuQuery {
    pub category: Option<String>,
}

pub async fn list_tables(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    let tables = RestoService::list_tables(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::ok(tables))
}

pub async fn create_table(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<CreateTableRequest>,
) -> Result<impl IntoResponse, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let table = RestoService::create_table(&state.pool, auth.tenant_id, req).await?;
    Ok(ApiResponse::created(table))
}

pub async fn list_menu(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<MenuQuery>,
) -> Result<impl IntoResponse, AppError> {
    let items = RestoService::list_menu(&state.pool, auth.tenant_id, query.category).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn create_menu_item(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<CreateMenuItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let item = RestoService::create_menu_item(&state.pool, auth.tenant_id, req).await?;
    Ok(ApiResponse::created(item))
}

pub async fn list_kitchen_tickets(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    let tickets = RestoService::list_kitchen_tickets(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::ok(tickets))
}

pub async fn update_order_status(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateOrderStatusRequest>,
) -> Result<impl IntoResponse, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    RestoService::update_order_status(&state.pool, auth.tenant_id, id, &req.status).await?;
    Ok(ApiResponse::ok(serde_json::json!({ "updated": true, "status": req.status })))
}

pub async fn list_service_requests(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    let requests = RestoService::list_service_requests(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::ok(requests))
}

pub async fn resolve_service_request(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    RestoService::resolve_service_request(&state.pool, auth.tenant_id, id).await?;
    Ok(ApiResponse::ok(serde_json::json!({ "resolved": true })))
}

pub async fn get_stats(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    let stats = RestoService::get_stats(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::ok(stats))
}
