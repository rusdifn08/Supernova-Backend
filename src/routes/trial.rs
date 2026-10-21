use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::trial::{CreateTrialRequest, TrialInquiry},
};

// 1. Submit Trial / Demo Inquiry
pub async fn submit_trial_inquiry(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateTrialRequest>,
) -> Result<Json<Value>, AppError> {
    if payload.business_name.trim().is_empty() {
        return Err(AppError::ValidationError("Nama usaha/bisnis wajib diisi.".to_string()));
    }
    if payload.phone_wa.trim().is_empty() {
        return Err(AppError::ValidationError("Nomor WhatsApp kontak wajib diisi.".to_string()));
    }

    let id = Uuid::new_v4();
    let outlets = payload.estimated_outlets.unwrap_or(1);

    sqlx::query(
        r#"
        INSERT INTO trial_inquiries (
            id, business_name, business_type, contact_name, phone_wa, email,
            interested_service, estimated_outlets, current_pain_points, status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'new')
        "#
    )
    .bind(id)
    .bind(&payload.business_name)
    .bind(&payload.business_type)
    .bind(&payload.contact_name)
    .bind(&payload.phone_wa)
    .bind(&payload.email)
    .bind(&payload.interested_service)
    .bind(outlets)
    .bind(&payload.current_pain_points)
    .execute(&pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "message": "Permintaan uji coba layanan & demo berhasil diterima! Lead Architect kami akan menghubungi dalam waktu < 15 menit.",
        "inquiry_id": id,
        "business_name": payload.business_name,
        "service": payload.interested_service
    })))
}

// 2. List Trial Inquiries
pub async fn list_trial_inquiries(State(pool): State<PgPool>) -> Result<Json<Vec<TrialInquiry>>, AppError> {
    let list = sqlx::query_as::<_, TrialInquiry>(
        r#"
        SELECT id, business_name, business_type, contact_name, phone_wa, email,
               interested_service, estimated_outlets, current_pain_points, status, created_at
        FROM trial_inquiries
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(list))
}
