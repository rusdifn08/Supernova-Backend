use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    error::AppError,
    modules::booking::dto::{
        BookingStatsDto, CreateReservationRequest, CreateResourceRequest, ReservationDto,
        ResourceDto, UpdateReservationRequest,
    },
};

#[derive(FromRow)]
struct ReservationRow {
    id: Uuid,
    booking_code: String,
    resource_id: Uuid,
    resource_name: String,
    customer_name: String,
    customer_phone: String,
    customer_email: Option<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration_hours: f64,
    total_amount: f64,
    down_payment: f64,
    booking_status: String,
    payment_status: String,
    payment_method: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn list_resources(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<ResourceDto>, AppError> {
    let rows = sqlx::query_as::<_, ResourceDto>(
        r#"
        SELECT id, name, category, hourly_rate::FLOAT8 as hourly_rate, capacity, description, is_active
        FROM booking_resources
        WHERE tenant_id = $1 AND is_active = TRUE
        ORDER BY name ASC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn create_resource(
    pool: &PgPool,
    tenant_id: Uuid,
    req: CreateResourceRequest,
) -> Result<ResourceDto, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::ValidationError("Nama sarana/fasilitas wajib diisi.".to_string()));
    }

    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, ResourceDto>(
        r#"
        INSERT INTO booking_resources (id, tenant_id, business_type_id, name, category, hourly_rate, capacity, description, is_active)
        VALUES ($1, $2, 'custom', $3, $4, $5, $6, $7, TRUE)
        RETURNING id, name, category, hourly_rate::FLOAT8 as hourly_rate, capacity, description, is_active
        "#,
    )
    .bind(id)
    .bind(tenant_id)
    .bind(req.name.trim())
    .bind(req.category.trim())
    .bind(req.hourly_rate)
    .bind(req.capacity)
    .bind(req.description)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_reservations(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<ReservationDto>, AppError> {
    let rows = sqlx::query_as::<_, ReservationRow>(
        r#"
        SELECT 
            b.id, b.booking_code, b.resource_id, COALESCE(r.name, 'Sarana Terhapus') as resource_name,
            b.customer_name, b.customer_phone, b.customer_email,
            b.start_time, b.end_time, b.duration_hours::FLOAT8, b.total_amount::FLOAT8, b.down_payment::FLOAT8,
            b.status as booking_status, b.status as payment_status, b.payment_method, b.notes, b.created_at
        FROM bookings b
        LEFT JOIN booking_resources r ON b.resource_id = r.id
        WHERE b.tenant_id = $1
        ORDER BY b.start_time DESC
        LIMIT 100
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    let list = rows
        .into_iter()
        .map(|r| ReservationDto {
            id: r.id,
            code: r.booking_code,
            resource_id: r.resource_id,
            resource_name: r.resource_name,
            customer_name: r.customer_name,
            customer_phone: r.customer_phone,
            customer_email: r.customer_email,
            start_time: r.start_time,
            end_time: r.end_time,
            duration_hours: r.duration_hours,
            total_amount: r.total_amount,
            down_payment: r.down_payment,
            status: r.booking_status,
            payment_status: r.payment_status,
            payment_method: r.payment_method,
            notes: r.notes,
            created_at: r.created_at,
        })
        .collect();

    Ok(list)
}

pub async fn create_reservation(
    pool: &PgPool,
    tenant_id: Uuid,
    req: CreateReservationRequest,
) -> Result<ReservationDto, AppError> {
    if req.end_time <= req.start_time {
        return Err(AppError::ValidationError("Waktu selesai booking harus lebih besar dari waktu mulai.".to_string()));
    }

    let (res_name, hourly_rate) = sqlx::query_as::<_, (String, f64)>(
        "SELECT name, hourly_rate::FLOAT8 FROM booking_resources WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE",
    )
    .bind(req.resource_id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Sarana yang dipilih tidak ditemukan.".to_string()))?;

    let overlap = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM bookings
        WHERE tenant_id = $1
          AND resource_id = $2
          AND status != 'cancelled'
          AND (start_time < $4 AND end_time > $3)
        "#,
    )
    .bind(tenant_id)
    .bind(req.resource_id)
    .bind(req.start_time)
    .bind(req.end_time)
    .fetch_one(pool)
    .await?;

    if overlap > 0 {
        return Err(AppError::Conflict("Jadwal pada jam dan sarana tersebut sudah terisi (Bentrok Jadwal). Silakan pilih slot lain.".to_string()));
    }

    let duration_hours = (req.end_time - req.start_time).num_minutes() as f64 / 60.0;
    let total_amount = duration_hours * hourly_rate;
    let down_payment = req.down_payment.unwrap_or(0.0);

    let booking_id = Uuid::new_v4();
    let booking_code = format!("BKG-{}", booking_id.to_string()[..6].to_uppercase());
    let booking_date = req.start_time.date_naive();
    let start_time_naive = req.start_time.time();
    let end_time_naive = req.end_time.time();

    let (created_at,): (DateTime<Utc>,) = sqlx::query_as(
        r#"
        INSERT INTO bookings (
            id, tenant_id, resource_id, booking_code, customer_name, customer_phone, customer_email,
            booking_date, start_time, end_time, duration_hours, total_amount, down_payment, status,
            payment_method, notes
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'confirmed', $14, $15
        )
        RETURNING created_at
        "#,
    )
    .bind(booking_id)
    .bind(tenant_id)
    .bind(req.resource_id)
    .bind(&booking_code)
    .bind(req.customer_name.trim())
    .bind(req.customer_phone.trim())
    .bind(req.customer_email.as_ref())
    .bind(booking_date)
    .bind(start_time_naive)
    .bind(end_time_naive)
    .bind(duration_hours as i32)
    .bind(total_amount)
    .bind(down_payment)
    .bind(req.payment_method.as_deref().unwrap_or("cash"))
    .bind(req.notes.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(ReservationDto {
        id: booking_id,
        code: booking_code,
        resource_id: req.resource_id,
        resource_name: res_name,
        customer_name: req.customer_name,
        customer_phone: req.customer_phone,
        customer_email: req.customer_email,
        start_time: req.start_time,
        end_time: req.end_time,
        duration_hours,
        total_amount,
        down_payment,
        status: "confirmed".to_string(),
        payment_status: "paid".to_string(),
        payment_method: req.payment_method,
        notes: req.notes,
        created_at,
    })
}

pub async fn update_reservation_status(
    pool: &PgPool,
    tenant_id: Uuid,
    id: Uuid,
    req: UpdateReservationRequest,
) -> Result<ReservationDto, AppError> {
    sqlx::query(
        r#"
        UPDATE bookings
        SET status = COALESCE($1, status),
            updated_at = NOW()
        WHERE id = $2 AND tenant_id = $3
        "#,
    )
    .bind(req.status.as_deref())
    .bind(id)
    .bind(tenant_id)
    .execute(pool)
    .await?;

    let r = sqlx::query_as::<_, ReservationRow>(
        r#"
        SELECT 
            b.id, b.booking_code, b.resource_id, COALESCE(r.name, 'Sarana') as resource_name,
            b.customer_name, b.customer_phone, b.customer_email,
            b.start_time, b.end_time, b.duration_hours::FLOAT8, b.total_amount::FLOAT8, b.down_payment::FLOAT8,
            b.status as booking_status, b.status as payment_status, b.payment_method, b.notes, b.created_at
        FROM bookings b
        LEFT JOIN booking_resources r ON b.resource_id = r.id
        WHERE b.id = $1 AND b.tenant_id = $2
        "#,
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Data booking tidak ditemukan.".to_string()))?;

    Ok(ReservationDto {
        id: r.id,
        code: r.booking_code,
        resource_id: r.resource_id,
        resource_name: r.resource_name,
        customer_name: r.customer_name,
        customer_phone: r.customer_phone,
        customer_email: r.customer_email,
        start_time: r.start_time,
        end_time: r.end_time,
        duration_hours: r.duration_hours,
        total_amount: r.total_amount,
        down_payment: r.down_payment,
        status: r.booking_status,
        payment_status: r.payment_status,
        payment_method: r.payment_method,
        notes: r.notes,
        created_at: r.created_at,
    })
}

pub async fn get_stats(pool: &PgPool, tenant_id: Uuid) -> Result<BookingStatsDto, AppError> {
    let revenue = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT SUM(total_amount)::FLOAT8 FROM bookings WHERE tenant_id = $1 AND booking_date >= CURRENT_DATE AND status != 'cancelled'",
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?
    .unwrap_or(0.0);

    let count_today = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM bookings WHERE tenant_id = $1 AND booking_date >= CURRENT_DATE AND status != 'cancelled'",
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;

    let upcoming = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM bookings WHERE tenant_id = $1 AND booking_date >= CURRENT_DATE AND status = 'confirmed'",
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;

    let pending_pay = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM bookings WHERE tenant_id = $1 AND status = 'pending'",
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;

    Ok(BookingStatsDto {
        revenue_today: revenue,
        reservations_today: count_today,
        occupancy_pct: 68.5,
        upcoming_count: upcoming,
        pending_payment_count: pending_pay,
    })
}
