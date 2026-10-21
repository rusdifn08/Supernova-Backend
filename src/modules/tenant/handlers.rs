use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    error::AppError,
    http::{extract::AuthContext, response::ApiResponse},
    modules::{auth::dto::TenantDto, tenant::dto::UpdateTenantRequest},
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

pub async fn update_tenant(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTenantRequest>,
) -> Result<Json<ApiResponse<TenantDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;

    let row = sqlx::query_as::<_, TenantDbRow>(
        r#"
        UPDATE tenants
        SET name = COALESCE($1, name),
            city = COALESCE($2, city),
            phone = COALESCE($3, phone),
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, slug, code, name, business_type, city, phone, service_plan, trial_expires_at, is_active
        "#,
    )
    .bind(payload.name)
    .bind(payload.city)
    .bind(payload.phone)
    .bind(auth.tenant_id)
    .fetch_one(&state.pool)
    .await?;

    let now = chrono::Utc::now();
    let days_remaining = (row.trial_expires_at - now).num_days().max(0);

    Ok(ApiResponse::json(TenantDto {
        id: row.id,
        slug: row.slug,
        code: row.code,
        name: row.name,
        business_type: row.business_type,
        city: row.city,
        phone: row.phone,
        plan: row.service_plan,
        trial_expires_at: row.trial_expires_at,
        days_remaining,
        is_trial_active: row.is_active && row.trial_expires_at > now,
    }))
}
