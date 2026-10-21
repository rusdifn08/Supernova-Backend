use axum::{
    http::{header, HeaderValue, Method},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::{modules, state::AppState};

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "supernova-backend-engine",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub fn create_router(state: AppState) -> Router {
    let cors = if state.config.demo_mode {
        CorsLayer::permissive()
    } else {
        let origins: Vec<HeaderValue> = state
            .config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::ORIGIN,
            ])
            .allow_origin(origins)
    };

    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", modules::auth::router())
        .nest("/tenants", modules::tenant::router())
        .nest("/booking", modules::booking::router())
        .nest("/pos", modules::pos::router())
        .nest("/crm", modules::crm::router())
        .nest("/erp", modules::erp::router())
        .nest("/finance", modules::finance::router())
        .nest("/resto", modules::resto::router())
        .nest("/public", modules::public::router());

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
