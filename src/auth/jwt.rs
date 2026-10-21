use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,       // user_id
    pub tid: Uuid,       // tenant_id
    pub role: String,    // "owner" | "staff"
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(&self, user_id: Uuid, tenant_id: Uuid, role: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = (now + Duration::days(1)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            tid: tenant_id,
            role: role.to_string(),
            exp,
            iat,
        };

        encode(&Header::default(), &claims, &self.encoding)
            .map_err(|e| AppError::InternalError(format!("Gagal membuat token autentikasi: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(token, &self.decoding, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("Sesi login Anda tidak valid atau telah kedaluwarsa. Silakan masuk kembali.".to_string()))
    }
}
