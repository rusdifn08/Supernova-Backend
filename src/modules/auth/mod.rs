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
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/demo", post(handlers::demo))
        .route("/me", get(handlers::me))
}
