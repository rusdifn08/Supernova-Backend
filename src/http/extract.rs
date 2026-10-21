use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    #[allow(dead_code)]
    pub role: String,
}

impl AuthContext {
    pub async fn ensure_active_trial(&self, pool: &sqlx::PgPool) -> Result<(), AppError> {
        let is_active = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT (is_active AND trial_expires_at > NOW())
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(self.tenant_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false);

        if !is_active {
            return Err(AppError::TrialExpired(
                "Masa lisensi uji coba 30 hari telah berakhir. Silakan hubungi tim Supernova Labs untuk aktivasi lisensi operasional.".to_string(),
            ));
        }

        Ok(())
    }
}

impl FromRequestParts<AppState> for AuthContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| {
                AppError::Unauthorized("Header Authorization (Bearer token) diperlukan untuk mengakses data ini.".to_string())
            })?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Format token tidak valid. Gunakan: Bearer <token>".to_string()));
        }

        let token = &auth_header[7..].trim();
        let claims = state.jwt_keys.verify_token(token)?;

        Ok(AuthContext {
            user_id: claims.sub,
            tenant_id: claims.tid,
            role: claims.role,
        })
    }
}
