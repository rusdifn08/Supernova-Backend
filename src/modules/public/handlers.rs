use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::{error::AppError, http::response::ApiResponse, state::AppState};
use super::{dto::*, service::PublicService};

#[derive(Debug, Deserialize)]
pub struct DateQuery {
    pub date: Option<NaiveDate>,
}

pub async fn get_tenant(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let (_, dto) = PublicService::get_tenant(&state.pool, &slug).await?;
    Ok(ApiResponse::ok(dto))
}

pub async fn get_booking_availability(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<DateQuery>,
) -> Result<impl IntoResponse, AppError> {
    let date = query.date.unwrap_or_else(|| chrono::Local::now().date_naive());
    let slots = PublicService::get_booking_availability(&state.pool, &slug, date).await?;
    Ok(ApiResponse::ok(slots))
}

pub async fn reserve_booking(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<PublicReserveRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = PublicService::reserve_booking(&state.pool, &slug, req).await?;
    Ok(ApiResponse::created(res))
}

pub async fn get_pos_catalog(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let catalog = PublicService::get_pos_catalog(&state.pool, &slug).await?;
    Ok(ApiResponse::ok(catalog))
}

pub async fn get_resto_table(
    State(state): State<AppState>,
    Path((slug, qr_token)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let data = PublicService::get_resto_table_and_menu(&state.pool, &slug, &qr_token).await?;
    Ok(ApiResponse::ok(data))
}

pub async fn create_resto_order(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<PublicRestoOrderRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = PublicService::create_resto_order(&state.pool, &slug, req).await?;
    Ok(ApiResponse::created(res))
}

pub async fn call_waiter(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<CallWaiterRequest>,
) -> Result<impl IntoResponse, AppError> {
    PublicService::call_waiter(&state.pool, &slug, req).await?;
    Ok(ApiResponse::ok(serde_json::json!({ "sent": true, "message": "Permintaan layanan telah dikirim ke staf" })))
}
