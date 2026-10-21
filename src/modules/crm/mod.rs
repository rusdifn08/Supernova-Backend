pub mod dto;
pub mod handlers;

use axum::{
    routing::get,
    Router,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/customers", get(handlers::list_customers).post(handlers::create_customer))
}
