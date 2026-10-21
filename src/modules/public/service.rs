use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use super::dto::*;

pub struct PublicService;

impl PublicService {
    pub async fn get_tenant(pool: &PgPool, slug: &str) -> Result<(Uuid, PublicTenantDto), AppError> {
        let row = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, Option<String>)>(
            r#"
            SELECT id, slug, code, name, business_type, city, phone
            FROM tenants
            WHERE slug = $1 AND is_active = TRUE
            "#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Bisnis dengan slug '{}' tidak ditemukan", slug)))?;

        let tenant_id = row.0;
        let dto = PublicTenantDto {
            slug: row.1,
            code: row.2,
            name: row.3,
            business_type: row.4,
            city: row.5,
            phone: row.6,
        };

        Ok((tenant_id, dto))
    }

    pub async fn get_booking_availability(
        pool: &PgPool,
        slug: &str,
        date: NaiveDate,
    ) -> Result<Vec<PublicResourceAvailabilityDto>, AppError> {
        let (tenant_id, _) = Self::get_tenant(pool, slug).await?;

        let resources = sqlx::query_as::<_, (Uuid, String, String, f64, i32)>(
            r#"
            SELECT id, name, category, hourly_price::FLOAT8, capacity
            FROM booking_resources
            WHERE tenant_id = $1 AND is_active = TRUE
            ORDER BY name ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        let mut results = Vec::with_capacity(resources.len());

        for (id, name, category, hourly_price, capacity) in resources {
            let booked = sqlx::query_as::<_, (chrono::NaiveTime, chrono::NaiveTime)>(
                r#"
                SELECT start_time, end_time
                FROM bookings
                WHERE resource_id = $1 AND booking_date = $2 AND status NOT IN ('cancelled')
                ORDER BY start_time ASC
                "#,
            )
            .bind(id)
            .bind(date)
            .fetch_all(pool)
            .await?;

            let booked_slots = booked
                .into_iter()
                .map(|(s, e)| format!("{}-{}", s.format("%H:%M"), e.format("%H:%M")))
                .collect();

            results.push(PublicResourceAvailabilityDto {
                resource: PublicResourceDto {
                    id,
                    name,
                    category,
                    hourly_price,
                    capacity,
                },
                booked_slots,
            });
        }

        Ok(results)
    }

    pub async fn reserve_booking(
        pool: &PgPool,
        slug: &str,
        req: PublicReserveRequest,
    ) -> Result<PublicReserveResponse, AppError> {
        let (tenant_id, _) = Self::get_tenant(pool, slug).await?;

        if req.duration_hours <= 0 || req.duration_hours > 12 {
            return Err(AppError::BadRequest("Durasi sewa minimal 1 jam dan maksimal 12 jam".into()));
        }

        let end_time = req.start_time + Duration::hours(req.duration_hours as i64);

        // Fetch resource
        let (res_name, hourly_price): (String, f64) = sqlx::query_as(
            "SELECT name, hourly_price::FLOAT8 FROM booking_resources WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE",
        )
        .bind(req.resource_id)
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Fasilitas atau lapangan tidak ditemukan".into()))?;

        // Collision check
        let (conflict_count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM bookings
            WHERE resource_id = $1
              AND booking_date = $2
              AND status NOT IN ('cancelled')
              AND start_time < $4
              AND end_time > $3
            "#,
        )
        .bind(req.resource_id)
        .bind(req.booking_date)
        .bind(req.start_time)
        .bind(end_time)
        .fetch_one(pool)
        .await?;

        if conflict_count > 0 {
            return Err(AppError::Conflict("Jadwal pada jam tersebut sudah dipesan orang lain. Silakan pilih jam lain.".into()));
        }

        let total_amount = hourly_price * (req.duration_hours as f64);
        let down_payment = total_amount * 0.50; // 50% DP
        let booking_id = Uuid::new_v4();
        let booking_code = format!("BK-{}-{}", Utc::now().format("%d%m"), booking_id.to_string()[..4].to_uppercase());
        let payment_method = req.payment_method.unwrap_or_else(|| "qris".to_string());

        sqlx::query(
            r#"
            INSERT INTO bookings (
                id, tenant_id, booking_code, resource_id, customer_name, customer_phone, customer_email,
                booking_date, start_time, end_time, duration_hours, total_amount, down_payment,
                status, payment_method, notes
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'confirmed', $14, $15
            )
            "#,
        )
        .bind(booking_id)
        .bind(tenant_id)
        .bind(&booking_code)
        .bind(req.resource_id)
        .bind(&req.customer_name)
        .bind(&req.customer_phone)
        .bind(req.customer_email)
        .bind(req.booking_date)
        .bind(req.start_time)
        .bind(end_time)
        .bind(req.duration_hours)
        .bind(total_amount)
        .bind(down_payment)
        .bind(&payment_method)
        .bind(req.notes)
        .execute(pool)
        .await?;

        Ok(PublicReserveResponse {
            booking_code,
            resource_name: res_name,
            booking_date: req.booking_date,
            start_time: req.start_time,
            end_time,
            total_amount,
            down_payment,
            status: "confirmed".to_string(),
        })
    }

    pub async fn get_pos_catalog(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Vec<PublicProductDto>, AppError> {
        let (tenant_id, _) = Self::get_tenant(pool, slug).await?;

        let rows = sqlx::query_as::<_, (Uuid, String, String, String, f64, i32)>(
            r#"
            SELECT p.id, p.name, p.sku, COALESCE(c.name, 'Umum'), p.sell_price::FLOAT8, p.stock
            FROM erp_products p
            LEFT JOIN erp_categories c ON c.id = p.category_id
            WHERE p.tenant_id = $1 AND p.is_active = TRUE AND p.stock > 0
            ORDER BY p.name ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| PublicProductDto {
                id: r.0,
                name: r.1,
                sku: r.2,
                category_name: r.3,
                price: r.4,
                stock: r.5,
            })
            .collect())
    }

    pub async fn get_resto_table_and_menu(
        pool: &PgPool,
        slug: &str,
        qr_token: &str,
    ) -> Result<PublicRestoTableDto, AppError> {
        let (tenant_id, _) = Self::get_tenant(pool, slug).await?;

        let table = sqlx::query_as::<_, (Uuid, String, String)>(
            r#"
            SELECT id, table_number, section_area
            FROM resto_tables
            WHERE tenant_id = $1 AND qr_token = $2 AND is_active = TRUE
            "#,
        )
        .bind(tenant_id)
        .bind(qr_token)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("QR Meja tidak valid atau meja dinonaktifkan".into()))?;

        let menu_rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, f64, bool)>(
            r#"
            SELECT id, category_id, name, description, price::FLOAT8, is_available
            FROM resto_menu_items
            WHERE tenant_id = $1 AND is_available = TRUE
            ORDER BY category_id, name ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        let menu = menu_rows
            .into_iter()
            .map(|r| PublicMenuItemDto {
                id: r.0,
                category_id: r.1,
                name: r.2,
                description: r.3,
                price: r.4,
                is_available: r.5,
            })
            .collect();

        Ok(PublicRestoTableDto {
            table_id: table.0,
            table_number: table.1,
            section_area: table.2,
            menu,
        })
    }

    pub async fn create_resto_order(
        pool: &PgPool,
        slug: &str,
        req: PublicRestoOrderRequest,
    ) -> Result<PublicRestoOrderResponse, AppError> {
        let (tenant_id, _) = Self::get_tenant(pool, slug).await?;

        if req.items.is_empty() {
            return Err(AppError::BadRequest("Pesanan harus memiliki minimal 1 item".into()));
        }

        let table = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, table_number FROM resto_tables WHERE tenant_id = $1 AND qr_token = $2",
        )
        .bind(tenant_id)
        .bind(&req.qr_token)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Meja tidak valid".into()))?;

        let mut tx = pool.begin().await?;

        let mut subtotal = 0.0;
        let mut order_items_data = Vec::new();

        for item_req in &req.items {
            let (name, price): (String, f64) = sqlx::query_as(
                "SELECT name, price::FLOAT8 FROM resto_menu_items WHERE id = $1 AND tenant_id = $2 AND is_available = TRUE",
            )
            .bind(item_req.menu_item_id)
            .bind(tenant_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("Menu item {} tidak tersedia", item_req.menu_item_id)))?;

            let item_subtotal = price * (item_req.quantity as f64);
            subtotal += item_subtotal;
            order_items_data.push((item_req.menu_item_id, name, item_req.quantity, price, item_subtotal, &item_req.variant, &item_req.notes));
        }

        let tax_service = subtotal * 0.15; // 10% tax + 5% service
        let total_amount = subtotal + tax_service;
        let order_id = Uuid::new_v4();
        let order_code = format!("RST-{}", order_id.to_string()[..6].to_uppercase());
        let cust_name = req.customer_name.unwrap_or_else(|| "Guest".to_string());
        let pay_method = req.payment_method.unwrap_or_else(|| "qris".to_string());

        sqlx::query(
            r#"
            INSERT INTO resto_table_orders (
                id, tenant_id, order_code, table_id, customer_name, status,
                payment_method, payment_status, subtotal, tax_service, total_amount, customer_notes
            ) VALUES (
                $1, $2, $3, $4, $5, 'pending', $6, 'paid', $7, $8, $9, $10
            )
            "#,
        )
        .bind(order_id)
        .bind(tenant_id)
        .bind(&order_code)
        .bind(table.0)
        .bind(&cust_name)
        .bind(&pay_method)
        .bind(subtotal)
        .bind(tax_service)
        .bind(total_amount)
        .bind(req.notes)
        .execute(&mut *tx)
        .await?;

        for (mid, name, qty, price, line_subtotal, variant, notes) in order_items_data {
            sqlx::query(
                r#"
                INSERT INTO resto_table_order_items (
                    id, order_id, menu_item_id, item_name, quantity, unit_price, selected_variant, special_notes, subtotal
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(order_id)
            .bind(mid)
            .bind(name)
            .bind(qty)
            .bind(price)
            .bind(variant)
            .bind(notes)
            .bind(line_subtotal)
            .execute(&mut *tx)
            .await?;
        }

        // Set table status to ordered
        sqlx::query("UPDATE resto_tables SET status = 'ordered' WHERE id = $1")
            .bind(table.0)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(PublicRestoOrderResponse {
            order_code,
            table_number: table.1,
            subtotal,
            tax_service,
            total_amount,
            status: "pending".to_string(),
        })
    }

    pub async fn call_waiter(
        pool: &PgPool,
        slug: &str,
        req: CallWaiterRequest,
    ) -> Result<(), AppError> {
        let (tenant_id, _) = Self::get_tenant(pool, slug).await?;

        let table = sqlx::query_as::<_, (Uuid, String, String)>(
            "SELECT id, table_number, section_area FROM resto_tables WHERE tenant_id = $1 AND qr_token = $2",
        )
        .bind(tenant_id)
        .bind(&req.qr_token)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Meja tidak valid".into()))?;

        sqlx::query(
            r#"
            INSERT INTO resto_service_requests (id, tenant_id, table_id, table_number, section_area, request_type, note, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'open')
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(table.0)
        .bind(&table.1)
        .bind(&table.2)
        .bind(&req.request_type)
        .bind(req.note)
        .execute(pool)
        .await?;

        let _ = sqlx::query("UPDATE resto_tables SET status = 'calling_waiter' WHERE id = $1")
            .bind(table.0)
            .execute(pool)
            .await;

        Ok(())
    }
}
