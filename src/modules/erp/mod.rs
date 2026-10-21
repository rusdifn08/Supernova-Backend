pub mod dto;
pub mod handlers;

use axum::{
    routing::get,
    Router,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/employees", get(handlers::list_employees).post(handlers::create_employee))
        .route("/inventory/logs", get(handlers::list_inventory_logs))
}
