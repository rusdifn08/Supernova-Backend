use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use super::dto::*;

pub struct RestoService;

impl RestoService {
    pub async fn list_tables(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<TableDto>, AppError> {
        let rows = sqlx::query_as::<_, TableDto>(
            r#"
            SELECT id, table_number, qr_token, section_area, capacity, status, is_active
            FROM resto_tables
            WHERE tenant_id = $1
            ORDER BY table_number ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn create_table(
        pool: &PgPool,
        tenant_id: Uuid,
        req: CreateTableRequest,
    ) -> Result<TableDto, AppError> {
        let id = Uuid::new_v4();
        let qr_token = format!("QR-{}-{}", req.table_number.to_uppercase(), &Uuid::new_v4().to_string()[..8]);
        let section = req.section_area.unwrap_or_else(|| "indoor".to_string());
        let cap = req.capacity.unwrap_or(4);

        let row = sqlx::query_as::<_, TableDto>(
            r#"
            INSERT INTO resto_tables (id, tenant_id, table_number, qr_token, section_area, capacity, status, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, 'empty', TRUE)
            RETURNING id, table_number, qr_token, section_area, capacity, status, is_active
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(&req.table_number)
        .bind(&qr_token)
        .bind(&section)
        .bind(cap)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn list_menu(
        pool: &PgPool,
        tenant_id: Uuid,
        category: Option<String>,
    ) -> Result<Vec<MenuItemDto>, AppError> {
        let rows = match category {
            Some(cat) => {
                sqlx::query_as::<_, (Uuid, String, String, Option<String>, f64, bool)>(
                    r#"
                    SELECT id, category_id, name, description, price::FLOAT8, is_available
                    FROM resto_menu_items
                    WHERE tenant_id = $1 AND category_id = $2
                    ORDER BY name ASC
                    "#,
                )
                .bind(tenant_id)
                .bind(cat)
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, (Uuid, String, String, Option<String>, f64, bool)>(
                    r#"
                    SELECT id, category_id, name, description, price::FLOAT8, is_available
                    FROM resto_menu_items
                    WHERE tenant_id = $1
                    ORDER BY category_id, name ASC
                    "#,
                )
                .bind(tenant_id)
                .fetch_all(pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| MenuItemDto {
                id: r.0,
                category_id: r.1,
                name: r.2,
                description: r.3,
                price: r.4,
                is_available: r.5,
            })
            .collect())
    }

    pub async fn create_menu_item(
        pool: &PgPool,
        tenant_id: Uuid,
        req: CreateMenuItemRequest,
    ) -> Result<MenuItemDto, AppError> {
        let id = Uuid::new_v4();
        let (mid, cat, name, desc, price, avail) = sqlx::query_as::<_, (Uuid, String, String, Option<String>, f64, bool)>(
            r#"
            INSERT INTO resto_menu_items (id, tenant_id, category_id, name, description, price, is_available)
            VALUES ($1, $2, $3, $4, $5, $6, TRUE)
            RETURNING id, category_id, name, description, price::FLOAT8, is_available
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(&req.category_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.price)
        .fetch_one(pool)
        .await?;

        Ok(MenuItemDto {
            id: mid,
            category_id: cat,
            name,
            description: desc,
            price,
            is_available: avail,
        })
    }

    pub async fn list_kitchen_tickets(
        pool: &PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<KitchenTicketDto>, AppError> {
        let orders = sqlx::query_as::<_, (Uuid, String, String, String, String, chrono::DateTime<Utc>)>(
            r#"
            SELECT o.id, o.order_code, t.table_number, t.section_area, o.status, o.created_at
            FROM resto_table_orders o
            JOIN resto_tables t ON t.id = o.table_id
            WHERE o.tenant_id = $1 AND o.status IN ('pending', 'cooking')
            ORDER BY o.created_at ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        let mut tickets = Vec::with_capacity(orders.len());
        for (oid, code, table, section, status, created_at) in orders {
            let items = sqlx::query_as::<_, (String, i32, Option<String>, Option<String>)>(
                r#"
                SELECT item_name, quantity, selected_variant, special_notes
                FROM resto_table_order_items
                WHERE order_id = $1
                "#,
            )
            .bind(oid)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|i| KitchenTicketItemDto {
                name: i.0,
                quantity: i.1,
                variant: i.2,
                notes: i.3,
            })
            .collect();

            let elapsed_mins = (Utc::now() - created_at).num_minutes();
            let time_elapsed = format!("{} m lalu", elapsed_mins);

            tickets.push(KitchenTicketDto {
                id: oid,
                order_code: code,
                table_number: table,
                section_area: section,
                status,
                time_elapsed,
                items,
                created_at,
            });
        }

        Ok(tickets)
    }

    pub async fn update_order_status(
        pool: &PgPool,
        tenant_id: Uuid,
        order_id: Uuid,
        new_status: &str,
    ) -> Result<(), AppError> {
        let (current_status, table_id): (String, Uuid) = sqlx::query_as(
            "SELECT status, table_id FROM resto_table_orders WHERE id = $1 AND tenant_id = $2",
        )
        .bind(order_id)
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Order resto tidak ditemukan".into()))?;

        // State transition validation
        let valid = match (current_status.as_str(), new_status) {
            ("pending", "cooking") | ("pending", "cancelled") => true,
            ("cooking", "served") | ("cooking", "cancelled") => true,
            ("served", "completed") => true,
            (s, target) if s == target => true,
            _ => false,
        };

        if !valid {
            return Err(AppError::BadRequest(format!(
                "Transisi status dari '{}' ke '{}' tidak diperbolehkan",
                current_status, new_status
            )));
        }

        sqlx::query(
            "UPDATE resto_table_orders SET status = $1, updated_at = NOW() WHERE id = $2 AND tenant_id = $3",
        )
        .bind(new_status)
        .bind(order_id)
        .bind(tenant_id)
        .execute(pool)
        .await?;

        // Update table status accordingly
        if new_status == "completed" {
            let _ = sqlx::query("UPDATE resto_tables SET status = 'empty' WHERE id = $1")
                .bind(table_id)
                .execute(pool)
                .await;
        }

        Ok(())
    }

    pub async fn list_service_requests(
        pool: &PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<ServiceRequestDto>, AppError> {
        let rows = sqlx::query_as::<_, ServiceRequestDto>(
            r#"
            SELECT id, table_number, section_area, request_type, note, status, created_at
            FROM resto_service_requests
            WHERE tenant_id = $1 AND status = 'open'
            ORDER BY created_at ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn resolve_service_request(
        pool: &PgPool,
        tenant_id: Uuid,
        request_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE resto_service_requests SET status = 'resolved', resolved_at = NOW() WHERE id = $1 AND tenant_id = $2",
        )
        .bind(request_id)
        .bind(tenant_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_stats(pool: &PgPool, tenant_id: Uuid) -> Result<RestoStatsDto, AppError> {
        let (revenue,): (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(total_amount), 0)::FLOAT8
            FROM resto_table_orders
            WHERE tenant_id = $1 AND status = 'completed' AND created_at >= CURRENT_DATE
            "#,
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await?;

        let (active_tables,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resto_tables WHERE tenant_id = $1 AND status IN ('occupied', 'ordered', 'calling_waiter')",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await?;

        let (total_tables,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resto_tables WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await?;

        let (open_requests,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM resto_service_requests WHERE tenant_id = $1 AND status = 'open'",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await?;

        Ok(RestoStatsDto {
            revenue_today: revenue,
            active_tables,
            total_tables,
            avg_cook_minutes: 12.5,
            open_requests,
        })
    }
}
