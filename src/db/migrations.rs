use sqlx::PgPool;
use tracing::info;

use crate::error::AppError;

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    info!("Running database schema migration & multi-tenant isolation setup...");

    // 1. Run base schema first (idempotent CREATE TABLE IF NOT EXISTS)
    let schema_sql = include_str!("schema.sql");
    sqlx::raw_sql(schema_sql).execute(pool).await?;

    // 2. Run multi-tenant migration
    let migration_sql = r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id UUID PRIMARY KEY,
            slug VARCHAR(100) NOT NULL UNIQUE,
            code VARCHAR(50) NOT NULL UNIQUE,
            name VARCHAR(255) NOT NULL,
            business_type VARCHAR(100) NOT NULL,
            city VARCHAR(100),
            phone VARCHAR(50),
            service_plan VARCHAR(50) NOT NULL DEFAULT 'trial_1_month',
            trial_starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            trial_expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            full_name VARCHAR(255) NOT NULL,
            role VARCHAR(50) NOT NULL DEFAULT 'owner',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        -- Add tenant_id to domain tables if not present
        ALTER TABLE booking_resources ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE bookings ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE pos_outlets ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE erp_categories ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE erp_products ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE erp_inventory_logs ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE crm_customers ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE erp_employees ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE pos_orders ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE finance_journals ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE resto_tables ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE resto_menu_categories ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE resto_menu_items ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
        ALTER TABLE resto_table_orders ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

        -- Create service requests table for QR Resto waiter call / bill request
        CREATE TABLE IF NOT EXISTS resto_service_requests (
            id UUID PRIMARY KEY,
            tenant_id UUID REFERENCES tenants(id),
            table_id UUID REFERENCES resto_tables(id),
            table_number VARCHAR(50) NOT NULL,
            section_area VARCHAR(50) NOT NULL DEFAULT 'indoor',
            request_type VARCHAR(50) NOT NULL,
            note TEXT,
            status VARCHAR(50) NOT NULL DEFAULT 'open',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            resolved_at TIMESTAMPTZ
        );

        -- Seed Demo Tenant
        INSERT INTO tenants (
            id, slug, code, name, business_type, city, phone, service_plan, trial_starts_at, trial_expires_at, is_active
        ) VALUES (
            '00000000-0000-0000-0000-000000000001',
            'demo',
            '#SNV-DEMO-2026',
            'Supernova Multiplatform Demo',
            'all_in_one',
            'Jakarta Selatan',
            '081234567890',
            'enterprise',
            NOW(),
            NOW() + INTERVAL '365 days',
            TRUE
        ) ON CONFLICT (slug) DO NOTHING;

        -- Seed Demo User (owner@supernovasports.id / Supernova123!)
        INSERT INTO users (
            id, tenant_id, email, password_hash, full_name, role
        ) VALUES (
            '00000000-0000-0000-0000-000000000002',
            '00000000-0000-0000-0000-000000000001',
            'owner@supernovasports.id',
            'ef51a37c449339739502b48981467493e81a32a6a57519cf2e259e86ba6f98ef',
            'Chef & General Manager Supernova',
            'owner'
        ) ON CONFLICT (email) DO NOTHING;

        -- Backfill existing rows without tenant_id to demo tenant
        UPDATE booking_resources SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE bookings SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE erp_products SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE crm_customers SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE erp_employees SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE pos_orders SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE finance_journals SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE resto_tables SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE resto_menu_items SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;
        UPDATE resto_table_orders SET tenant_id = '00000000-0000-0000-0000-000000000001' WHERE tenant_id IS NULL;

        -- Migrate legacy business_users to tenants & users if any
        INSERT INTO tenants (id, slug, code, name, business_type, city, phone, trial_starts_at, trial_expires_at, is_active)
        SELECT 
            id, 
            LOWER(REGEXP_REPLACE(business_name, '[^a-zA-Z0-9]+', '-', 'g')),
            '#SNV-' || SUBSTRING(id::text, 1, 4),
            business_name,
            business_type,
            city,
            phone_number,
            trial_starts_at,
            trial_expires_at,
            is_active
        FROM business_users
        ON CONFLICT (id) DO NOTHING;

        INSERT INTO users (id, tenant_id, email, password_hash, full_name, role)
        SELECT 
            id,
            id,
            email,
            password_hash,
            full_name,
            'owner'
        FROM business_users
        ON CONFLICT (email) DO NOTHING;
    "#;

    sqlx::raw_sql(migration_sql).execute(pool).await?;

    info!("Database multi-tenant schema ready.");
    Ok(())
}
