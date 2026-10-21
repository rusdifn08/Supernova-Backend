use chrono::{Duration, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    auth::{
        jwt::JwtKeys,
        password::{hash_password, verify_password, VerifyResult},
    },
    error::AppError,
    modules::auth::dto::{LoginRequest, RegisterRequest, SessionDto, TenantDto, UserDto},
};

#[derive(FromRow)]
struct UserRow {
    id: Uuid,
    tenant_id: Uuid,
    email: String,
    password_hash: String,
    full_name: String,
    role: String,
}

#[derive(FromRow)]
struct TenantRow {
    id: Uuid,
    slug: String,
    code: String,
    name: String,
    business_type: String,
    city: Option<String>,
    phone: Option<String>,
    service_plan: String,
    trial_expires_at: chrono::DateTime<Utc>,
    is_active: bool,
}

pub async fn register(pool: &PgPool, jwt_keys: &JwtKeys, req: RegisterRequest) -> Result<SessionDto, AppError> {
    if req.email.trim().is_empty() || !req.email.contains('@') {
        return Err(AppError::ValidationError("Email tidak valid.".to_string()));
    }
    if req.password.trim().len() < 6 {
        return Err(AppError::ValidationError("Password minimal 6 karakter.".to_string()));
    }
    if req.business_name.trim().is_empty() {
        return Err(AppError::ValidationError("Nama usaha wajib diisi.".to_string()));
    }

    let email_clean = req.email.trim().to_lowercase();

    // Check if email exists
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE LOWER(email) = $1")
        .bind(&email_clean)
        .fetch_one(pool)
        .await?;

    if count > 0 {
        return Err(AppError::Conflict("Email ini sudah terdaftar. Silakan login.".to_string()));
    }

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let password_hash = hash_password(req.password.trim())?;

    // Slug generation
    let base_slug = req.business_name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let clean_slug = base_slug.trim_matches('-').to_string();
    let slug = if clean_slug.is_empty() {
        format!("biz-{}", &tenant_id.to_string()[..6])
    } else {
        format!("{}-{}", clean_slug, &tenant_id.to_string()[..4])
    };

    let code = format!("#SNV-{}", tenant_id.to_string()[..4].to_uppercase());
    let now = Utc::now();
    let expires = now + Duration::days(30);

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO tenants (id, slug, code, name, business_type, city, phone, service_plan, trial_starts_at, trial_expires_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'trial_1_month', $8, $9, TRUE)
        "#,
    )
    .bind(tenant_id)
    .bind(&slug)
    .bind(&code)
    .bind(req.business_name.trim())
    .bind(req.business_type.trim())
    .bind(&req.city)
    .bind(req.phone.trim())
    .bind(now)
    .bind(expires)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, tenant_id, email, password_hash, full_name, role)
        VALUES ($1, $2, $3, $4, $5, 'owner')
        "#,
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(&email_clean)
    .bind(&password_hash)
    .bind(req.full_name.trim())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let token = jwt_keys.generate_token(user_id, tenant_id, "owner")?;
    let days_remaining = (expires - now).num_days().max(0);

    Ok(SessionDto {
        token,
        user: UserDto {
            id: user_id,
            email: email_clean,
            full_name: req.full_name.trim().to_string(),
            role: "owner".to_string(),
        },
        tenant: TenantDto {
            id: tenant_id,
            slug,
            code,
            name: req.business_name.trim().to_string(),
            business_type: req.business_type.trim().to_string(),
            city: req.city,
            phone: Some(req.phone.trim().to_string()),
            plan: "trial_1_month".to_string(),
            trial_expires_at: expires,
            days_remaining,
            is_trial_active: true,
        },
    })
}

pub async fn login(pool: &PgPool, jwt_keys: &JwtKeys, req: LoginRequest) -> Result<SessionDto, AppError> {
    let email_clean = req.email.trim().to_lowercase();

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, tenant_id, email, password_hash, full_name, role
        FROM users
        WHERE LOWER(email) = $1
        "#,
    )
    .bind(&email_clean)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Email atau password yang Anda masukkan salah.".to_string()))?;

    match verify_password(req.password.trim(), &user.password_hash) {
        VerifyResult::Valid => {}
        VerifyResult::ValidNeedUpgrade => {
            if let Ok(new_hash) = hash_password(req.password.trim()) {
                let _ = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
                    .bind(new_hash)
                    .bind(user.id)
                    .execute(pool)
                    .await;
            }
        }
        VerifyResult::Invalid => {
            return Err(AppError::Unauthorized("Email atau password yang Anda masukkan salah.".to_string()));
        }
    }

    let tenant = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT id, slug, code, name, business_type, city, phone, service_plan, trial_expires_at, is_active
        FROM tenants
        WHERE id = $1
        "#,
    )
    .bind(user.tenant_id)
    .fetch_one(pool)
    .await?;

    let now = Utc::now();
    let days_remaining = (tenant.trial_expires_at - now).num_days().max(0);
    let is_trial_active = tenant.is_active && tenant.trial_expires_at > now;

    let token = jwt_keys.generate_token(user.id, tenant.id, &user.role)?;

    Ok(SessionDto {
        token,
        user: UserDto {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
        },
        tenant: TenantDto {
            id: tenant.id,
            slug: tenant.slug,
            code: tenant.code,
            name: tenant.name,
            business_type: tenant.business_type,
            city: tenant.city,
            phone: tenant.phone,
            plan: tenant.service_plan,
            trial_expires_at: tenant.trial_expires_at,
            days_remaining,
            is_trial_active,
        },
    })
}

pub async fn demo_login(pool: &PgPool, jwt_keys: &JwtKeys) -> Result<SessionDto, AppError> {
    let demo_tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let demo_user = sqlx::query_as::<_, UserRow>(
        "SELECT id, tenant_id, email, password_hash, full_name, role FROM users WHERE tenant_id = $1 LIMIT 1",
    )
    .bind(demo_tenant_id)
    .fetch_one(pool)
    .await?;

    let tenant = sqlx::query_as::<_, TenantRow>(
        "SELECT id, slug, code, name, business_type, city, phone, service_plan, trial_expires_at, is_active FROM tenants WHERE id = $1",
    )
    .bind(demo_tenant_id)
    .fetch_one(pool)
    .await?;

    let now = Utc::now();
    let days_remaining = (tenant.trial_expires_at - now).num_days().max(0);
    let token = jwt_keys.generate_token(demo_user.id, demo_tenant_id, &demo_user.role)?;

    Ok(SessionDto {
        token,
        user: UserDto {
            id: demo_user.id,
            email: demo_user.email,
            full_name: demo_user.full_name,
            role: demo_user.role,
        },
        tenant: TenantDto {
            id: tenant.id,
            slug: tenant.slug,
            code: tenant.code,
            name: tenant.name,
            business_type: tenant.business_type,
            city: tenant.city,
            phone: tenant.phone,
            plan: tenant.service_plan,
            trial_expires_at: tenant.trial_expires_at,
            days_remaining,
            is_trial_active: true,
        },
    })
}
