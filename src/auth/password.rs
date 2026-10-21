use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Digest, Sha256};

use crate::error::AppError;

#[derive(Debug, PartialEq, Eq)]
pub enum VerifyResult {
    Valid,
    ValidNeedUpgrade,
    Invalid,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::InternalError(format!("Gagal hashing password: {}", e)))
}

pub fn verify_password(password: &str, stored_hash: &str) -> VerifyResult {
    // 1. Try Argon2 verify
    if let Ok(parsed_hash) = PasswordHash::new(stored_hash) {
        if Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok() {
            return VerifyResult::Valid;
        }
    }

    // 2. Try Legacy Unsalted SHA-256 verify (for migrating legacy users)
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let sha_hex = hex::encode(hasher.finalize());
    if sha_hex.eq_ignore_ascii_case(stored_hash) {
        return VerifyResult::ValidNeedUpgrade;
    }

    VerifyResult::Invalid
}
