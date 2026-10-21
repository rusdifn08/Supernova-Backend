pub mod dto;
pub mod handlers;

use axum::{routing::patch, Router};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", patch(handlers::update_tenant))
}
