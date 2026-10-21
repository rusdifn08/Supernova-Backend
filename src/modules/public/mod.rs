pub mod dto;
pub mod handlers;
pub mod service;

use axum::{
    routing::{get, post},
    Router,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tenants/{slug}", get(handlers::get_tenant))
        .route("/booking/{slug}/availability", get(handlers::get_booking_availability))
        .route("/booking/{slug}/reserve", post(handlers::reserve_booking))
        .route("/pos/{slug}/catalog", get(handlers::get_pos_catalog))
        .route("/resto/{slug}/table/{qr_token}", get(handlers::get_resto_table))
        .route("/resto/{slug}/order", post(handlers::create_resto_order))
        .route("/resto/{slug}/call-waiter", post(handlers::call_waiter))
}
