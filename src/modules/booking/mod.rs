pub mod dto;
pub mod handlers;
pub mod service;

use axum::{
    routing::{get, patch},
    Router,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/resources", get(handlers::list_resources).post(handlers::create_resource))
        .route("/reservations", get(handlers::list_reservations).post(handlers::create_reservation))
        .route("/reservations/{id}", patch(handlers::update_reservation))
        .route("/stats", get(handlers::get_stats))
}
