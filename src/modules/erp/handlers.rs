use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    error::AppError,
    http::{extract::AuthContext, response::ApiResponse},
    modules::erp::dto::{CreateEmployeeRequest, EmployeeDto, InventoryLogDto},
    state::AppState,
};

#[derive(sqlx::FromRow)]
struct InventoryLogRow {
    id: Uuid,
    product_id: Uuid,
    product_name: String,
    quantity: i32,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_employees(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<EmployeeDto>>>, AppError> {
    let rows = sqlx::query_as::<_, EmployeeDto>(
        r#"
        SELECT id, name, role, phone, shift, status
        FROM erp_employees
        WHERE tenant_id = $1
        ORDER BY name ASC
        "#,
    )
    .bind(auth.tenant_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(ApiResponse::json(rows))
}

pub async fn create_employee(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<CreateEmployeeRequest>,
) -> Result<Json<ApiResponse<EmployeeDto>>, AppError> {
    auth.ensure_active_trial(&state.pool).await?;

    if payload.name.trim().is_empty() {
        return Err(AppError::ValidationError("Nama staf/karyawan wajib diisi.".to_string()));
    }

    let id = Uuid::new_v4();
    let shift = payload.shift.unwrap_or_else(|| "morning".to_string());
    let role = match payload.role.to_lowercase().as_str() {
        "manager" => "manager",
        "cashier" | "kasir" => "cashier",
        "kitchen" | "dapur" => "kitchen",
        "inventory" | "gudang" => "inventory",
        _ => "staff",
    };

    let row = sqlx::query_as::<_, EmployeeDto>(
        r#"
        INSERT INTO erp_employees (id, tenant_id, name, role, phone, shift, status, attendance_status)
        VALUES ($1, $2, $3, $4, $5, $6, 'active', 'present')
        RETURNING id, name, role, phone, shift, status
        "#,
    )
    .bind(id)
    .bind(auth.tenant_id)
    .bind(payload.name.trim())
    .bind(role)
    .bind(payload.phone)
    .bind(&shift)
    .fetch_one(&state.pool)
    .await?;

    Ok(ApiResponse::json(row))
}

pub async fn list_inventory_logs(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<InventoryLogDto>>>, AppError> {
    let rows = sqlx::query_as::<_, InventoryLogRow>(
        r#"
        SELECT l.id, l.product_id, COALESCE(p.name, 'Produk') as product_name, l.quantity, l.notes, l.created_at
        FROM erp_inventory_logs l
        LEFT JOIN erp_products p ON l.product_id = p.id
        WHERE l.tenant_id = $1
        ORDER BY l.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(auth.tenant_id)
    .fetch_all(&state.pool)
    .await?;

    let list = rows
        .into_iter()
        .map(|r| InventoryLogDto {
            id: r.id,
            product_id: r.product_id,
            product_name: r.product_name,
            quantity_delta: r.quantity,
            reason: r.notes.unwrap_or_else(|| "Penyesuaian stok".to_string()),
            created_at: r.created_at,
        })
        .collect();

    Ok(ApiResponse::json(list))
}
