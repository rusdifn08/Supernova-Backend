use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::qr_order::{
        KitchenTicket, KitchenTicketItem, PlaceTableOrderRequest, RestoMenuItem, RestoTable,
        RestoTableOrder, UpdateOrderStatusRequest,
    },
};

// 1. Get Table & Menu by Scanning QR Code
pub async fn scan_table_qr(
    State(pool): State<PgPool>,
    Path(qr_token): Path<String>,
) -> Result<Json<Value>, AppError> {
    let table = sqlx::query_as::<_, RestoTable>(
        r#"
        SELECT id, table_number, qr_token, section_area, capacity, status, is_active, created_at
        FROM resto_tables
        WHERE qr_token = $1 AND is_active = TRUE
        "#
    )
    .bind(&qr_token)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("QR Code Meja tidak valid atau meja tidak aktif.".to_string()))?;

    let menu_items = sqlx::query_as::<_, RestoMenuItem>(
        r#"
        SELECT id, category_id, name, description, price::FLOAT8, image_url,
               available_variants, is_available, created_at
        FROM resto_menu_items
        WHERE is_available = TRUE
        ORDER BY category_id ASC, name ASC
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "table": {
            "id": table.id,
            "table_number": table.table_number,
            "section": table.section_area,
            "capacity": table.capacity,
            "current_status": table.status
        },
        "menu_catalog": menu_items,
        "payment_options": ["qris", "pay_at_cashier", "ewallet"]
    })))
}

// 2. Place Customer Order from Table
pub async fn place_table_order(
    State(pool): State<PgPool>,
    Json(payload): Json<PlaceTableOrderRequest>,
) -> Result<Json<Value>, AppError> {
    if payload.items.is_empty() {
        return Err(AppError::ValidationError("Daftar pesanan menu tidak boleh kosong.".to_string()));
    }

    let mut tx = pool.begin().await?;

    // Verify table
    let table = sqlx::query_as::<_, RestoTable>(
        r#"
        SELECT id, table_number, qr_token, section_area, capacity, status, is_active, created_at
        FROM resto_tables
        WHERE qr_token = $1 AND is_active = TRUE
        FOR UPDATE
        "#
    )
    .bind(&payload.table_qr_token)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("QR Code Meja tidak ditemukan.".to_string()))?;

    let mut subtotal: f64 = 0.0;
    let mut verified_items = Vec::new();

    for item in &payload.items {
        if item.quantity <= 0 {
            return Err(AppError::ValidationError("Jumlah item harus lebih dari 0.".to_string()));
        }

        let menu = sqlx::query_as::<_, RestoMenuItem>(
            r#"
            SELECT id, category_id, name, description, price::FLOAT8, image_url,
                   available_variants, is_available, created_at
            FROM resto_menu_items
            WHERE id = $1 AND is_available = TRUE
            "#
        )
        .bind(item.menu_item_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Menu item ID {} tidak tersedia.", item.menu_item_id)))?;

        let item_sub = menu.price * (item.quantity as f64);
        subtotal += item_sub;
        verified_items.push((menu.id, menu.name, item.quantity, menu.price, item_sub, item.selected_variant.clone(), item.special_notes.clone()));
    }

    let tax_service = subtotal * 0.10; // 10% resto PB1
    let total_amount = subtotal + tax_service;

    let order_code = format!("ORD-TBL{}-{}", table.table_number, &Uuid::new_v4().to_string()[..4].to_uppercase());
    let order_id = Uuid::new_v4();
    let cust_name = payload.customer_name.unwrap_or_else(|| format!("Tamu Meja {}", table.table_number));

    // Insert Table Order
    sqlx::query(
        r#"
        INSERT INTO resto_table_orders (
            id, order_code, table_id, customer_name, status, payment_method,
            payment_status, subtotal, tax_service, total_amount, customer_notes
        ) VALUES ($1, $2, $3, $4, 'pending', $5, 'unpaid', $6, $7, $8, $9)
        "#
    )
    .bind(order_id)
    .bind(&order_code)
    .bind(table.id)
    .bind(&cust_name)
    .bind(&payload.payment_method)
    .bind(subtotal)
    .bind(tax_service)
    .bind(total_amount)
    .bind(&payload.customer_notes)
    .execute(&mut *tx)
    .await?;

    // Insert Order Items
    for (mid, mname, qty, price, isub, variant, notes) in verified_items {
        sqlx::query(
            r#"
            INSERT INTO resto_table_order_items (
                order_id, menu_item_id, item_name, quantity, unit_price,
                selected_variant, special_notes, subtotal
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(order_id)
        .bind(mid)
        .bind(&mname)
        .bind(qty)
        .bind(price)
        .bind(variant)
        .bind(notes)
        .bind(isub)
        .execute(&mut *tx)
        .await?;
    }

    // Update table status to occupied
    sqlx::query("UPDATE resto_tables SET status = 'ordered' WHERE id = $1")
        .bind(table.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(json!({
        "success": true,
        "message": "Pesanan Anda berhasil terkirim langsung ke Dapur!",
        "order_code": order_code,
        "table_number": table.table_number,
        "status": "pending_kitchen",
        "subtotal": subtotal,
        "tax_service": tax_service,
        "total_amount": total_amount,
        "payment_method": payload.payment_method
    })))
}

// 3. Kitchen Display System (KDS): Live Active Kitchen Orders
pub async fn get_kitchen_orders(State(pool): State<PgPool>) -> Result<Json<Vec<KitchenTicket>>, AppError> {
    let orders = sqlx::query(
        r#"
        SELECT o.id, o.order_code, t.table_number, t.section_area, o.customer_name,
               o.status, o.customer_notes, o.created_at
        FROM resto_table_orders o
        JOIN resto_tables t ON t.id = o.table_id
        WHERE o.status IN ('pending', 'cooking')
        ORDER BY o.created_at ASC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let mut tickets = Vec::new();

    for row in orders {
        let order_id: Uuid = row.get("id");
        let order_code: String = row.get("order_code");
        let table_number: String = row.get("table_number");
        let section_area: String = row.get("section_area");
        let customer_name: String = row.get("customer_name");
        let status: String = row.get("status");
        let customer_notes: Option<String> = row.get("customer_notes");
        let created_at = row.get("created_at");

        let items_rows = sqlx::query(
            r#"
            SELECT item_name, quantity, selected_variant, special_notes
            FROM resto_table_order_items
            WHERE order_id = $1
            "#
        )
        .bind(order_id)
        .fetch_all(&pool)
        .await?;

        let items = items_rows
            .into_iter()
            .map(|ir| KitchenTicketItem {
                item_name: ir.get("item_name"),
                quantity: ir.get("quantity"),
                selected_variant: ir.get("selected_variant"),
                special_notes: ir.get("special_notes"),
            })
            .collect();

        tickets.push(KitchenTicket {
            order_id,
            order_code,
            table_number,
            section_area,
            customer_name,
            status,
            created_at,
            items,
            customer_notes,
        });
    }

    Ok(Json(tickets))
}

// 4. Update Order Status (e.g. from kitchen: cooking -> served -> completed)
pub async fn update_order_status(
    State(pool): State<PgPool>,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<UpdateOrderStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let valid_statuses = ["pending", "cooking", "served", "completed", "cancelled"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err(AppError::ValidationError("Status order tidak valid.".to_string()));
    }

    let result = sqlx::query(
        "UPDATE resto_table_orders SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING order_code, status"
    )
    .bind(&payload.status)
    .bind(order_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Order ID tidak ditemukan.".to_string()))?;

    let code: String = result.get("order_code");
    let new_status: String = result.get("status");

    Ok(Json(json!({
        "success": true,
        "order_code": code,
        "new_status": new_status,
        "message": format!("Status pesanan berhasil diperbarui menjadi '{}'", new_status)
    })))
}

// 5. List All Resto Tables
pub async fn list_resto_tables(State(pool): State<PgPool>) -> Result<Json<Vec<RestoTable>>, AppError> {
    let tables = sqlx::query_as::<_, RestoTable>(
        r#"
        SELECT id, table_number, qr_token, section_area, capacity, status, is_active, created_at
        FROM resto_tables
        WHERE is_active = TRUE
        ORDER BY table_number ASC
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(tables))
}

// 6. List All Table Orders
pub async fn list_table_orders(State(pool): State<PgPool>) -> Result<Json<Vec<RestoTableOrder>>, AppError> {
    let orders = sqlx::query_as::<_, RestoTableOrder>(
        r#"
        SELECT id, order_code, table_id, customer_name, status, payment_method,
               payment_status, subtotal::FLOAT8, tax_service::FLOAT8, total_amount::FLOAT8,
               customer_notes, created_at
        FROM resto_table_orders
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(orders))
}

// 7. Call Waiter from Table
pub async fn call_waiter(
    State(pool): State<PgPool>,
    Path(qr_token): Path<String>,
) -> Result<Json<Value>, AppError> {
    let table = sqlx::query_as::<_, RestoTable>(
        "SELECT id, table_number, qr_token, section_area, capacity, status, is_active, created_at FROM resto_tables WHERE qr_token = $1 AND is_active = TRUE"
    )
    .bind(&qr_token)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("QR Code Meja tidak valid.".to_string()))?;

    sqlx::query("UPDATE resto_tables SET status = 'calling_waiter' WHERE id = $1")
        .bind(table.id)
        .execute(&pool)
        .await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Pelayan telah diberitahu untuk menuju ke Meja {} ({})!", table.table_number, table.section_area),
        "table_number": table.table_number,
        "status": "calling_waiter",
        "timestamp": chrono::Utc::now()
    })))
}

// 8. Request Bill from Table
pub async fn request_bill(
    State(pool): State<PgPool>,
    Path(qr_token): Path<String>,
) -> Result<Json<Value>, AppError> {
    let table = sqlx::query_as::<_, RestoTable>(
        "SELECT id, table_number, qr_token, section_area, capacity, status, is_active, created_at FROM resto_tables WHERE qr_token = $1 AND is_active = TRUE"
    )
    .bind(&qr_token)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("QR Code Meja tidak valid.".to_string()))?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Permintaan tagihan Meja {} telah dikirim ke Kasir.", table.table_number),
        "table_number": table.table_number,
        "timestamp": chrono::Utc::now()
    })))
}
