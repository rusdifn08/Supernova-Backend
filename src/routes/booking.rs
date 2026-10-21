use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::booking::{Booking, BookingResource, BusinessType, CreateBookingRequest, UpdateBookingStatusRequest},
};

// 1. Get All Supported Business Types
pub async fn get_business_types(State(pool): State<PgPool>) -> Result<Json<Vec<BusinessType>>, AppError> {
    let types = sqlx::query_as::<_, BusinessType>(
        "SELECT id, name, description, icon, created_at FROM booking_business_types ORDER BY name ASC"
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(types))
}

// 2. Get All Booking Resources
pub async fn get_resources(State(pool): State<PgPool>) -> Result<Json<Vec<BookingResource>>, AppError> {
    let resources = sqlx::query_as::<_, BookingResource>(
        r#"
        SELECT id, business_type_id, name, category, hourly_rate::FLOAT8, capacity, description, is_active, created_at, updated_at
        FROM booking_resources
        WHERE is_active = TRUE
        ORDER BY business_type_id ASC, name ASC
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(resources))
}

// 3. Create a New Booking with Strict Conflict Detection
pub async fn create_booking(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBookingRequest>,
) -> Result<Json<Value>, AppError> {
    // Input Validation
    if payload.customer_name.trim().is_empty() {
        return Err(AppError::ValidationError("Nama pemesan tidak boleh kosong.".to_string()));
    }
    if payload.customer_phone.trim().is_empty() {
        return Err(AppError::ValidationError("Nomor telepon/WhatsApp wajib diisi.".to_string()));
    }
    if payload.end_time <= payload.start_time {
        return Err(AppError::ValidationError("Waktu selesai booking harus lebih besar dari waktu mulai.".to_string()));
    }

    // Fetch resource details & verify it exists
    let resource = sqlx::query_as::<_, BookingResource>(
        r#"
        SELECT id, business_type_id, name, category, hourly_rate::FLOAT8, capacity, description, is_active, created_at, updated_at
        FROM booking_resources
        WHERE id = $1 AND is_active = TRUE
        "#
    )
    .bind(payload.resource_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Sarana atau resource booking tidak ditemukan.".to_string()))?;

    // Conflict Check: Ensure no existing active booking overlaps this period
    let conflict = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM bookings
        WHERE resource_id = $1
          AND booking_status NOT IN ('cancelled')
          AND (start_time < $3 AND end_time > $2)
        "#
    )
    .bind(payload.resource_id)
    .bind(payload.start_time)
    .bind(payload.end_time)
    .fetch_one(&pool)
    .await?;

    if conflict > 0 {
        return Err(AppError::Conflict(
            "Slot waktu yang Anda pilih sudah terisi oleh pemesan lain. Silakan pilih jam lain.".to_string(),
        ));
    }

    // Calculate duration in hours
    let duration_millis = payload.end_time.signed_duration_since(payload.start_time).num_milliseconds();
    let duration_hours = (duration_millis as f64) / (1000.0 * 60.0 * 60.0);
    let total_amount = duration_hours * resource.hourly_rate;
    let down_payment = payload.down_payment.unwrap_or(total_amount * 0.3); // default 30% DP

    // Generate unique human-readable code
    let random_suffix = Uuid::new_v4().to_string()[..6].to_uppercase();
    let booking_code = format!("BKG-{}-{}", Utc::now().format("%Y%m%d"), random_suffix);

    let booking_id = Uuid::new_v4();
    let payment_method = payload.payment_method.unwrap_or_else(|| "qris".to_string());

    sqlx::query(
        r#"
        INSERT INTO bookings (
            id, resource_id, booking_code, customer_name, customer_phone, customer_email,
            start_time, end_time, duration_hours, total_amount, down_payment,
            booking_status, payment_status, payment_method, notes
        ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11,
            'confirmed', 'down_payment_paid', $12, $13
        )
        "#
    )
    .bind(booking_id)
    .bind(payload.resource_id)
    .bind(&booking_code)
    .bind(&payload.customer_name)
    .bind(&payload.customer_phone)
    .bind(&payload.customer_email)
    .bind(payload.start_time)
    .bind(payload.end_time)
    .bind(duration_hours)
    .bind(total_amount)
    .bind(down_payment)
    .bind(&payment_method)
    .bind(&payload.notes)
    .execute(&pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "message": "Booking berhasil terkonfirmasi! Tiket & detail reservasi otomatis aktif.",
        "booking": {
            "id": booking_id,
            "booking_code": booking_code,
            "resource_name": resource.name,
            "category": resource.category,
            "customer_name": payload.customer_name,
            "customer_phone": payload.customer_phone,
            "start_time": payload.start_time,
            "end_time": payload.end_time,
            "duration_hours": duration_hours,
            "total_amount": total_amount,
            "down_payment": down_payment,
            "status": "confirmed",
            "payment_status": "down_payment_paid",
            "whatsapp_notification_ready": true
        }
    })))
}

// 4. List All Bookings
pub async fn list_bookings(State(pool): State<PgPool>) -> Result<Json<Vec<Booking>>, AppError> {
    let bookings = sqlx::query_as::<_, Booking>(
        r#"
        SELECT id, resource_id, booking_code, customer_name, customer_phone, customer_email,
               start_time, end_time, duration_hours::FLOAT8, total_amount::FLOAT8, down_payment::FLOAT8,
               booking_status, payment_status, payment_method, notes, created_at, updated_at
        FROM bookings
        ORDER BY start_time DESC
        LIMIT 100
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(bookings))
}

// 5. Update Booking Status
pub async fn update_booking_status(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBookingStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query(
        r#"
        UPDATE bookings 
        SET 
            booking_status = COALESCE($1, booking_status),
            payment_status = COALESCE($2, payment_status),
            updated_at = NOW()
        WHERE id = $3
        RETURNING booking_code, booking_status, payment_status
        "#,
    )
    .bind(payload.booking_status)
    .bind(payload.payment_status)
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Data booking tidak ditemukan.".to_string()))?;

    use sqlx::Row;
    let code: String = row.get("booking_code");
    let b_status: String = row.get("booking_status");
    let p_status: String = row.get("payment_status");

    Ok(Json(json!({
        "success": true,
        "booking_code": code,
        "booking_status": b_status,
        "payment_status": p_status
    })))
}
