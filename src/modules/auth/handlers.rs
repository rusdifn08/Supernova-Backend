use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    error::AppError,
    http::{extract::AuthContext, response::ApiResponse},
    modules::auth::{
        dto::{LoginRequest, RegisterRequest, SessionDto, TenantDto, UserDto},
        service,
    },
    state::AppState,
};

#[derive(sqlx::FromRow)]
struct TenantDbRow {
    id: Uuid,
    slug: String,
    code: String,
    name: String,
    business_type: String,
    city: Option<String>,
    phone: Option<String>,
    service_plan: String,
    trial_expires_at: chrono::DateTime<chrono::Utc>,
    is_active: bool,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<SessionDto>>, AppError> {
    let session = service::register(&state.pool, &state.jwt_keys, payload).await?;
    Ok(ApiResponse::json(session))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<SessionDto>>, AppError> {
    let session = service::login(&state.pool, &state.jwt_keys, payload).await?;
    Ok(ApiResponse::json(session))
}

pub async fn demo(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SessionDto>>, AppError> {
    if !state.config.demo_mode {
        return Err(AppError::Forbidden("Akun demo dinonaktifkan di mode produksi.".to_string()));
    }
    let session = service::demo_login(&state.pool, &state.jwt_keys).await?;
    Ok(ApiResponse::json(session))
}

pub async fn me(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user = sqlx::query_as::<_, UserDto>(
        "SELECT id, email, full_name, role FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    let tenant_row = sqlx::query_as::<_, TenantDbRow>(
        "SELECT id, slug, code, name, business_type, city, phone, service_plan, trial_expires_at, is_active FROM tenants WHERE id = $1",
    )
    .bind(auth.tenant_id)
    .fetch_one(&state.pool)
    .await?;

    let now = chrono::Utc::now();
    let days_remaining = (tenant_row.trial_expires_at - now).num_days().max(0);
    let is_trial_active = tenant_row.is_active && tenant_row.trial_expires_at > now;

    let tenant = TenantDto {
        id: tenant_row.id,
        slug: tenant_row.slug,
        code: tenant_row.code,
        name: tenant_row.name,
        business_type: tenant_row.business_type,
        city: tenant_row.city,
        phone: tenant_row.phone,
        plan: tenant_row.service_plan,
        trial_expires_at: tenant_row.trial_expires_at,
        days_remaining,
        is_trial_active,
    };

    Ok(ApiResponse::json(serde_json::json!({
        "user": user,
        "tenant": tenant,
    })))
}
