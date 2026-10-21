use sha2::{Digest, Sha256};
use supernova_backend::auth::{
    jwt::JwtKeys,
    password::{hash_password, verify_password, VerifyResult},
};
use uuid::Uuid;

#[test]
fn test_password_argon2_hashing_and_legacy_sha256_upgrade() {
    let plain = "SuperSecretPass2026!";

    // 1. Argon2 hashing
    let hash = hash_password(plain).expect("Hash must succeed");
    assert!(hash.starts_with("$argon2"), "Must produce argon2 hash");
    assert_eq!(verify_password(plain, &hash), VerifyResult::Valid);
    assert_eq!(verify_password("wrong-pass", &hash), VerifyResult::Invalid);

    // 2. Legacy SHA-256 hash upgrade detection
    let mut hasher = Sha256::new();
    hasher.update(plain.as_bytes());
    let legacy_hex = hex::encode(hasher.finalize());

    assert_eq!(
        verify_password(plain, &legacy_hex),
        VerifyResult::ValidNeedUpgrade,
        "Legacy SHA-256 must verify and flag for upgrade"
    );
    assert_eq!(
        verify_password("wrong-pass", &legacy_hex),
        VerifyResult::Invalid
    );
}

#[test]
fn test_jwt_token_roundtrip_and_tampering() {
    let keys = JwtKeys::new("test_secret_key_antigravity_supernova_2026");
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let role = "owner";

    let token = keys
        .generate_token(user_id, tenant_id, role)
        .expect("JWT token generation failed");

    let claims = keys.verify_token(&token).expect("JWT verification failed");
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.tid, tenant_id);
    assert_eq!(claims.role, role);

    // Tampered token test
    let tampered = format!("{}tampered", token);
    assert!(keys.verify_token(&tampered).is_err(), "Tampered token must fail verification");
}

#[test]
fn test_slug_sanitization() {
    fn generate_slug(name: &str, tenant_id: Uuid) -> String {
        let base_slug = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let clean = base_slug.trim_matches('-').to_string();
        if clean.is_empty() {
            format!("biz-{}", &tenant_id.to_string()[..6])
        } else {
            format!("{}-{}", clean, &tenant_id.to_string()[..4])
        }
    }

    let tid = Uuid::parse_str("a29513ec-4fca-432a-af51-d7ff7dd23f26").unwrap();
    let slug = generate_slug("Supernova Badminton Arena & Cafe!", tid);
    assert_eq!(slug, "supernova-badminton-arena---cafe-a295");
}

#[test]
fn test_pricing_and_tax_computations() {
    // POS: 11% PPN
    let pos_subtotal = 100_000.0;
    let pos_discount = 10_000.0;
    let after_discount = pos_subtotal - pos_discount;
    let pos_tax = after_discount * 0.11;
    let pos_total = after_discount + pos_tax;

    assert_eq!(after_discount, 90_000.0);
    assert_eq!(pos_tax, 9_900.0);
    assert_eq!(pos_total, 99_900.0);

    // Resto: 10% tax + 5% service = 15%
    let resto_subtotal = 200_000.0;
    let resto_tax_service = resto_subtotal * 0.15;
    let resto_total = resto_subtotal + resto_tax_service;

    assert_eq!(resto_tax_service, 30_000.0);
    assert_eq!(resto_total, 230_000.0);

    // Booking: 3.5 hours * 120,000/hr, 50% DP
    let hourly_rate = 120_000.0;
    let duration_hours = 3.5;
    let booking_total = hourly_rate * duration_hours;
    let booking_dp = booking_total * 0.50;

    assert_eq!(booking_total, 420_000.0);
    assert_eq!(booking_dp, 210_000.0);
}

#[test]
fn test_resto_order_status_transitions() {
    fn is_valid_transition(current: &str, next: &str) -> bool {
        match (current, next) {
            ("pending", "cooking") | ("pending", "cancelled") => true,
            ("cooking", "served") | ("cooking", "cancelled") => true,
            ("served", "completed") => true,
            (s, target) if s == target => true,
            _ => false,
        }
    }

    assert!(is_valid_transition("pending", "cooking"));
    assert!(is_valid_transition("pending", "cancelled"));
    assert!(is_valid_transition("cooking", "served"));
    assert!(is_valid_transition("served", "completed"));

    // Invalid transitions
    assert!(!is_valid_transition("pending", "served"));
    assert!(!is_valid_transition("pending", "completed"));
    assert!(!is_valid_transition("completed", "cooking"));
    assert!(!is_valid_transition("cancelled", "served"));
}
