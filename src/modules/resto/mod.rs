pub mod dto;
pub mod handlers;
pub mod service;

use axum::{
    routing::{get, patch, post},
    Router,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tables", get(handlers::list_tables).post(handlers::create_table))
        .route("/menu", get(handlers::list_menu).post(handlers::create_menu_item))
        .route("/kitchen", get(handlers::list_kitchen_tickets))
        .route("/kitchen/{id}/status", patch(handlers::update_order_status))
        .route("/service-requests", get(handlers::list_service_requests))
        .route("/service-requests/{id}/resolve", post(handlers::resolve_service_request))
        .route("/stats", get(handlers::get_stats))
}
