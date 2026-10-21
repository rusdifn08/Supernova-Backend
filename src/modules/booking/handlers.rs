use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    http::{ApiResponse, AuthContext},
    modules::booking::{
        dto::{
            BookingStatsDto, CreateReservationRequest, CreateResourceRequest, ReservationDto,
            ResourceDto, UpdateReservationRequest,
        },
        service,
    },
    state::AppState,
};

pub async fn list_resources(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ResourceDto>>>, AppError> {
    let list = service::list_resources(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::json(list))
}

pub async fn create_resource(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CreateResourceRequest>,
) -> Result<Json<ApiResponse<ResourceDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let res = service::create_resource(&state.pool, auth.tenant_id, payload).await?;
    Ok(ApiResponse::json(res))
}

pub async fn list_reservations(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ReservationDto>>>, AppError> {
    let list = service::list_reservations(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::json(list))
}

pub async fn create_reservation(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CreateReservationRequest>,
) -> Result<Json<ApiResponse<ReservationDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let res = service::create_reservation(&state.pool, auth.tenant_id, payload).await?;
    Ok(ApiResponse::json(res))
}

pub async fn update_reservation(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateReservationRequest>,
) -> Result<Json<ApiResponse<ReservationDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;
    let res = service::update_reservation_status(&state.pool, auth.tenant_id, id, payload).await?;
    Ok(ApiResponse::json(res))
}

pub async fn get_stats(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<BookingStatsDto>>, AppError> {
    let stats = service::get_stats(&state.pool, auth.tenant_id).await?;
    Ok(ApiResponse::json(stats))
}
