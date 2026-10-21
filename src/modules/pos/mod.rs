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
        .route("/categories", get(handlers::list_categories))
        .route("/products", get(handlers::list_products).post(handlers::create_product))
        .route("/products/{id}/stock-adjustments", post(handlers::adjust_stock))
        .route("/orders", get(handlers::list_orders).post(handlers::create_order))
}
