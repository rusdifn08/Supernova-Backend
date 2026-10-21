use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    error::AppError,
    http::{extract::AuthContext, response::ApiResponse},
    modules::crm::dto::{CreateCustomerRequest, CustomerDto},
    state::AppState,
};

pub async fn list_customers(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CustomerDto>>>, AppError> {
    let rows = sqlx::query_as::<_, CustomerDto>(
        r#"
        SELECT id, name, phone, email, tier, loyalty_points, total_spent::FLOAT8
        FROM crm_customers
        WHERE tenant_id = $1
        ORDER BY total_spent DESC
        "#,
    )
    .bind(auth.tenant_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(ApiResponse::json(rows))
}

pub async fn create_customer(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Json<ApiResponse<CustomerDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;

    if payload.name.trim().is_empty() {
        return Err(AppError::ValidationError("Nama pelanggan wajib diisi.".to_string()));
    }

    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, CustomerDto>(
        r#"
        INSERT INTO crm_customers (id, tenant_id, name, phone, email, tier, loyalty_points, total_spent)
        VALUES ($1, $2, $3, $4, $5, 'silver', 0, 0)
        RETURNING id, name, phone, email, tier, loyalty_points, total_spent::FLOAT8
        "#,
    )
    .bind(id)
    .bind(auth.tenant_id)
    .bind(payload.name.trim())
    .bind(payload.phone.trim())
    .bind(payload.email)
    .fetch_one(&state.pool)
    .await?;

    Ok(ApiResponse::json(row))
}
