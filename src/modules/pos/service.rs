use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    error::AppError,
    modules::pos::dto::{
        CategoryDto, CreatePosOrderRequest, CreateProductRequest, PosOrderDto, ProductDto,
        StockAdjustmentRequest,
    },
};

#[derive(FromRow)]
struct PosOrderRow {
    id: Uuid,
    invoice_number: String,
    subtotal: f64,
    discount_amount: f64,
    tax_amount: f64,
    total_amount: f64,
    payment_method: String,
    created_at: DateTime<Utc>,
}

pub async fn list_categories(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<CategoryDto>, AppError> {
    let rows = sqlx::query_as::<_, CategoryDto>(
        "SELECT id, name FROM erp_categories WHERE tenant_id = $1 OR tenant_id IS NULL ORDER BY name ASC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn list_products(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<ProductDto>, AppError> {
    let rows = sqlx::query_as::<_, ProductDto>(
        r#"
        SELECT id, sku, name, category_id, unit, buy_price::FLOAT8, sell_price::FLOAT8, stock_quantity, min_stock_alert, is_active
        FROM erp_products
        WHERE tenant_id = $1
        ORDER BY name ASC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn create_product(
    pool: &PgPool,
    tenant_id: Uuid,
    req: CreateProductRequest,
) -> Result<ProductDto, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::ValidationError("Nama produk wajib diisi.".to_string()));
    }

    let id = Uuid::new_v4();
    let unit = req.unit.unwrap_or_else(|| "pcs".to_string());
    let min_stock = req.min_stock_alert.unwrap_or(5);

    let row = sqlx::query_as::<_, ProductDto>(
        r#"
        INSERT INTO erp_products (id, tenant_id, sku, name, category_id, unit, buy_price, sell_price, stock_quantity, min_stock_alert, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, TRUE)
        RETURNING id, sku, name, category_id, unit, buy_price::FLOAT8, sell_price::FLOAT8, stock_quantity, min_stock_alert, is_active
        "#,
    )
    .bind(id)
    .bind(tenant_id)
    .bind(req.sku.trim())
    .bind(req.name.trim())
    .bind(&req.category_id)
    .bind(&unit)
    .bind(req.buy_price)
    .bind(req.sell_price)
    .bind(req.stock_quantity)
    .bind(min_stock)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn adjust_stock(
    pool: &PgPool,
    tenant_id: Uuid,
    product_id: Uuid,
    req: StockAdjustmentRequest,
) -> Result<ProductDto, AppError> {
    let mut tx = pool.begin().await?;

    let (current_stock,): (i32,) = sqlx::query_as(
        "SELECT stock_quantity FROM erp_products WHERE id = $1 AND tenant_id = $2 FOR UPDATE",
    )
    .bind(product_id)
    .bind(tenant_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Produk tidak ditemukan.".to_string()))?;

    let new_stock = current_stock + req.quantity_delta;
    if new_stock < 0 {
        return Err(AppError::Conflict("Pengurangan stok melebihi stok yang tersedia saat ini.".to_string()));
    }

    let updated = sqlx::query_as::<_, ProductDto>(
        r#"
        UPDATE erp_products
        SET stock_quantity = $1, updated_at = NOW()
        WHERE id = $2 AND tenant_id = $3
        RETURNING id, sku, name, category_id, unit, buy_price::FLOAT8, sell_price::FLOAT8, stock_quantity, min_stock_alert, is_active
        "#,
    )
    .bind(new_stock)
    .bind(product_id)
    .bind(tenant_id)
    .fetch_one(&mut *tx)
    .await?;

    let log_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO erp_inventory_logs (id, tenant_id, product_id, change_type, quantity_changed, previous_stock, new_stock, notes)
        VALUES ($1, $2, $3, 'adjustment', $4, $5, $6, $7)
        "#,
    )
    .bind(log_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(req.quantity_delta)
    .bind(current_stock)
    .bind(new_stock)
    .bind(&req.reason)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(updated)
}

pub async fn create_pos_order(
    pool: &PgPool,
    tenant_id: Uuid,
    req: CreatePosOrderRequest,
) -> Result<PosOrderDto, AppError> {
    if req.items.is_empty() {
        return Err(AppError::ValidationError("Keranjang belanja kosong.".to_string()));
    }

    let mut tx = pool.begin().await?;
    let mut subtotal: f64 = 0.0;

    for item in &req.items {
        let (name, price, stock): (String, f64, i32) = sqlx::query_as(
            "SELECT name, sell_price::FLOAT8, stock_quantity FROM erp_products WHERE id = $1 AND tenant_id = $2 FOR UPDATE",
        )
        .bind(item.product_id)
        .bind(tenant_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Produk ID {} tidak ditemukan.", item.product_id)))?;

        if stock < item.quantity {
            return Err(AppError::Conflict(format!(
                "Stok untuk produk '{}' tidak mencukupi (Tersisa: {}, Diminta: {}).",
                name, stock, item.quantity
            )));
        }

        subtotal += price * (item.quantity as f64);
        let new_stock = stock - item.quantity;

        sqlx::query(
            "UPDATE erp_products SET stock_quantity = $1, updated_at = NOW() WHERE id = $2 AND tenant_id = $3",
        )
        .bind(new_stock)
        .bind(item.product_id)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        let log_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO erp_inventory_logs (id, tenant_id, product_id, change_type, quantity_changed, previous_stock, new_stock, notes)
            VALUES ($1, $2, $3, 'sale', $4, $5, $6, 'Penjualan Kasir POS')
            "#,
        )
        .bind(log_id)
        .bind(tenant_id)
        .bind(item.product_id)
        .bind(-item.quantity)
        .bind(stock)
        .bind(new_stock)
        .execute(&mut *tx)
        .await?;
    }

    let discount = req.discount_amount.unwrap_or(0.0);
    let after_discount = (subtotal - discount).max(0.0);
    let tax = after_discount * 0.11; // 11% PPN standard
    let total_amount = after_discount + tax;

    let amount_paid = req.amount_paid.unwrap_or(total_amount);
    let change_amount = (amount_paid - total_amount).max(0.0);

    let order_id = Uuid::new_v4();
    let invoice_num = format!("INV-{}", order_id.to_string()[..8].to_uppercase());
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO pos_orders (
            id, tenant_id, invoice_number, subtotal, discount_amount, tax_amount, total_amount,
            payment_method, payment_status, notes, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, 'completed', $9, $10
        )
        "#,
    )
    .bind(order_id)
    .bind(tenant_id)
    .bind(&invoice_num)
    .bind(subtotal)
    .bind(discount)
    .bind(tax)
    .bind(total_amount)
    .bind(&req.payment_method)
    .bind(&req.notes)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    let journal_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO finance_journals (id, tenant_id, transaction_type, category, amount, description, reference_code)
        VALUES ($1, $2, 'income', 'POS Sales', $3, 'Penjualan Kasir POS', $4)
        "#,
    )
    .bind(journal_id)
    .bind(tenant_id)
    .bind(total_amount)
    .bind(&invoice_num)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(PosOrderDto {
        id: order_id,
        invoice_number: invoice_num,
        subtotal,
        discount_amount: discount,
        tax_amount: tax,
        total_amount,
        amount_paid,
        change_amount,
        payment_method: req.payment_method,
        customer_id: req.customer_id,
        customer_name: None,
        created_at: now,
    })
}

pub async fn list_pos_orders(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<PosOrderDto>, AppError> {
    let rows = sqlx::query_as::<_, PosOrderRow>(
        r#"
        SELECT id, invoice_number, subtotal::FLOAT8, discount_amount::FLOAT8, tax_amount::FLOAT8, total_amount::FLOAT8,
               payment_method, created_at
        FROM pos_orders
        WHERE tenant_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    let list = rows
        .into_iter()
        .map(|r| PosOrderDto {
            id: r.id,
            invoice_number: r.invoice_number,
            subtotal: r.subtotal,
            discount_amount: r.discount_amount,
            tax_amount: r.tax_amount,
            total_amount: r.total_amount,
            amount_paid: r.total_amount,
            change_amount: 0.0,
            payment_method: r.payment_method,
            customer_id: None,
            customer_name: None,
            created_at: r.created_at,
        })
        .collect();

    Ok(list)
}
