use axum::{extract::State, Json};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::pos_erp::{CreatePosOrderRequest, Customer, Employee, FinanceSummary, PosOrder, Product},
};

// 1. Get Products with Live Stock
pub async fn get_products(State(pool): State<PgPool>) -> Result<Json<Vec<Product>>, AppError> {
    let products = sqlx::query_as::<_, Product>(
        r#"
        SELECT id, outlet_id, sku, name, category_id, unit, buy_price::FLOAT8, sell_price::FLOAT8,
               stock_quantity, min_stock_alert, is_active, created_at, updated_at
        FROM erp_products
        WHERE is_active = TRUE
        ORDER BY category_id ASC, name ASC
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(products))
}

// 2. Process Cashier POS Order (Atomic Transaction)
pub async fn create_pos_order(
    State(pool): State<PgPool>,
    Json(payload): Json<CreatePosOrderRequest>,
) -> Result<Json<Value>, AppError> {
    if payload.items.is_empty() {
        return Err(AppError::ValidationError("Keranjang transaksi kasir tidak boleh kosong.".to_string()));
    }

    let mut tx = pool.begin().await?;

    let mut subtotal: f64 = 0.0;
    let mut total_cogs: f64 = 0.0;
    let mut item_records = Vec::new();

    // Verify products and deduct stock
    for item in &payload.items {
        if item.quantity <= 0 {
            return Err(AppError::ValidationError("Kuantitas barang harus lebih besar dari 0.".to_string()));
        }

        let product = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, outlet_id, sku, name, category_id, unit, buy_price::FLOAT8, sell_price::FLOAT8,
                   stock_quantity, min_stock_alert, is_active, created_at, updated_at
            FROM erp_products
            WHERE id = $1
            FOR UPDATE
            "#
        )
        .bind(item.product_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Produk ID {} tidak ditemukan.", item.product_id)))?;

        if product.stock_quantity < item.quantity {
            return Err(AppError::BadRequest(format!(
                "Stok untuk produk '{}' tidak mencukupi (Tersedia: {}, Diminta: {}).",
                product.name, product.stock_quantity, item.quantity
            )));
        }

        let item_subtotal = product.sell_price * (item.quantity as f64);
        let item_cogs = product.buy_price * (item.quantity as f64);
        subtotal += item_subtotal;
        total_cogs += item_cogs;

        let new_stock = product.stock_quantity - item.quantity;

        // Deduct inventory
        sqlx::query("UPDATE erp_products SET stock_quantity = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_stock)
            .bind(product.id)
            .execute(&mut *tx)
            .await?;

        // Log ERP inventory audit
        sqlx::query(
            r#"
            INSERT INTO erp_inventory_logs (product_id, change_type, quantity_changed, previous_stock, new_stock, notes)
            VALUES ($1, 'sale', $2, $3, $4, 'Penjualan Kasir POS')
            "#
        )
        .bind(product.id)
        .bind(-item.quantity)
        .bind(product.stock_quantity)
        .bind(new_stock)
        .execute(&mut *tx)
        .await?;

        item_records.push((product.id, product.name, item.quantity, product.sell_price, product.buy_price, item_subtotal));
    }

    let tax = subtotal * 0.11; // 11% PPN standar
    let total_amount = subtotal + tax;

    let invoice_number = format!("INV-{}-{}", Utc::now().format("%Y%m%d%H%M"), &Uuid::new_v4().to_string()[..4].to_uppercase());
    let order_id = Uuid::new_v4();

    // Insert POS Order
    sqlx::query(
        r#"
        INSERT INTO pos_orders (
            id, invoice_number, customer_id, cashier_id, subtotal, discount_amount,
            tax_amount, total_amount, payment_method, payment_status, notes
        ) VALUES ($1, $2, $3, $4, $5, 0, $6, $7, $8, 'completed', $9)
        "#
    )
    .bind(order_id)
    .bind(&invoice_number)
    .bind(payload.customer_id)
    .bind(payload.cashier_id)
    .bind(subtotal)
    .bind(tax)
    .bind(total_amount)
    .bind(&payload.payment_method)
    .bind(&payload.notes)
    .execute(&mut *tx)
    .await?;

    // Insert Order Items
    for (pid, pname, qty, sell, buy, item_sub) in item_records {
        sqlx::query(
            r#"
            INSERT INTO pos_order_items (order_id, product_id, product_name, quantity, unit_price, unit_buy_price, subtotal)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(order_id)
        .bind(pid)
        .bind(&pname)
        .bind(qty)
        .bind(sell)
        .bind(buy)
        .bind(item_sub)
        .execute(&mut *tx)
        .await?;
    }

    // Record Income in Finance Journal
    sqlx::query(
        r#"
        INSERT INTO finance_journals (entry_type, category, amount, reference_order_id, description)
        VALUES ('income', 'Penjualan Kasir POS', $1, $2, $3)
        "#
    )
    .bind(total_amount)
    .bind(order_id)
    .bind(format!("Pendapatan Penjualan Invoice {}", invoice_number))
    .execute(&mut *tx)
    .await?;

    // If Customer is registered in CRM, add points and total spent
    if let Some(cust_id) = payload.customer_id {
        let points_earned = (total_amount / 10000.0).floor() as i32; // 1 point per 10k IDR
        sqlx::query(
            r#"
            UPDATE crm_customers
            SET loyalty_points = loyalty_points + $1,
                total_spent = total_spent + $2,
                last_visit = NOW(),
                updated_at = NOW()
            WHERE id = $3
            "#
        )
        .bind(points_earned)
        .bind(total_amount)
        .bind(cust_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(json!({
        "success": true,
        "message": "Transaksi Kasir berhasil! Stok terpotong dan jurnal laba rugi otomatis terupdate.",
        "invoice_number": invoice_number,
        "order_id": order_id,
        "subtotal": subtotal,
        "tax_11_pct": tax,
        "total_amount": total_amount,
        "gross_profit_estimated": subtotal - total_cogs,
        "payment_method": payload.payment_method,
        "status": "completed"
    })))
}

// 3. List POS Orders
pub async fn list_pos_orders(State(pool): State<PgPool>) -> Result<Json<Vec<PosOrder>>, AppError> {
    let orders = sqlx::query_as::<_, PosOrder>(
        r#"
        SELECT id, invoice_number, subtotal::FLOAT8, discount_amount::FLOAT8,
               tax_amount::FLOAT8, total_amount::FLOAT8, payment_method,
               payment_status, notes, created_at
        FROM pos_orders
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(orders))
}

// 4. Get CRM Customers
pub async fn get_customers(State(pool): State<PgPool>) -> Result<Json<Vec<Customer>>, AppError> {
    let customers = sqlx::query_as::<_, Customer>(
        r#"
        SELECT id, name, phone, email, tier, loyalty_points, total_spent::FLOAT8, last_visit, created_at
        FROM crm_customers
        ORDER BY total_spent DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(customers))
}

// 5. Get Employees (Shift & Attendance Monitoring)
pub async fn get_employees(State(pool): State<PgPool>) -> Result<Json<Vec<Employee>>, AppError> {
    let employees = sqlx::query_as::<_, Employee>(
        r#"
        SELECT id, name, role, phone, status, shift, attendance_status, created_at
        FROM erp_employees
        ORDER BY role ASC, name ASC
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(employees))
}

// 6. Comprehensive Financial Summary
pub async fn get_finance_summary(State(pool): State<PgPool>) -> Result<Json<FinanceSummary>, AppError> {
    let order_stats = sqlx::query(
        r#"
        SELECT COALESCE(SUM(total_amount), 0)::FLOAT8 AS total_rev,
               COUNT(*)::INT8 AS total_count
        FROM pos_orders
        WHERE payment_status = 'completed'
        "#
    )
    .fetch_one(&pool)
    .await?;

    let total_revenue: f64 = order_stats.get("total_rev");
    let total_orders_count: i64 = order_stats.get("total_count");

    let cogs_row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(quantity * unit_buy_price), 0)::FLOAT8 AS total_cogs
        FROM pos_order_items
        "#
    )
    .fetch_one(&pool)
    .await?;

    let total_cogs: f64 = cogs_row.get("total_cogs");
    let gross_profit = total_revenue - total_cogs;
    let profit_margin_pct = if total_revenue > 0.0 {
        (gross_profit / total_revenue) * 100.0
    } else {
        0.0
    };

    let low_stock_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM erp_products WHERE stock_quantity <= min_stock_alert AND is_active = TRUE"
    )
    .fetch_one(&pool)
    .await?;

    let active_emp_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM erp_employees WHERE status = 'active' AND attendance_status = 'present'"
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(FinanceSummary {
        total_revenue,
        total_cogs,
        gross_profit,
        profit_margin_pct,
        total_orders_count,
        low_stock_items_count: low_stock_count,
        active_employees_count: active_emp_count,
    }))
}
