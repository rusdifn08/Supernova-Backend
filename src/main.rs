use axum::{
    routing::{get, patch, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod models;
mod routes;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,supernova_backend=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Supernova Labs Studio - Multiplatform Backend Engine...");

    let config = Config::from_env();

    // Initialize Database Pool
    let pool = db::create_pool(&config.database_url).await?;

    // Auto-run Database Schema & Seed Checks
    db::run_migrations(&pool).await?;

    // Configure Secure CORS Layer
    tracing::info!("Allowed CORS Origins: {:?}", config.cors_origins);
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allows frontend clients during development/staging
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ]);

    // Build Axum Router
    let app = Router::new()
        // Health Checks
        .route("/health", get(routes::health::health_check))
        .route("/api/v1/health", get(routes::health::health_check))
        // Domain 1: Booking System
        .route("/api/v1/booking/types", get(routes::booking::get_business_types))
        .route("/api/v1/booking/resources", get(routes::booking::get_resources))
        .route("/api/v1/booking/reserve", post(routes::booking::create_booking))
        .route("/api/v1/booking/reservations", get(routes::booking::list_bookings))
        .route("/api/v1/booking/reservations/{id}/status", patch(routes::booking::update_booking_status))
        // Domain 2: POS, ERP, CRM & Keuangan
        .route("/api/v1/pos/products", get(routes::pos_erp::get_products))
        .route("/api/v1/pos/orders", post(routes::pos_erp::create_pos_order).get(routes::pos_erp::list_pos_orders))
        .route("/api/v1/crm/customers", get(routes::pos_erp::get_customers))
        .route("/api/v1/erp/employees", get(routes::pos_erp::get_employees))
        .route("/api/v1/finance/summary", get(routes::pos_erp::get_finance_summary))
        // Domain 3: Ordering in Table with QR Code
        .route("/api/v1/qr-order/table/{qr_token}", get(routes::qr_order::scan_table_qr))
        .route("/api/v1/qr-order/order", post(routes::qr_order::place_table_order))
        .route("/api/v1/qr-order/kitchen-orders", get(routes::qr_order::get_kitchen_orders))
        .route("/api/v1/qr-order/orders/{id}/status", patch(routes::qr_order::update_order_status))
        .route("/api/v1/qr-order/orders", get(routes::qr_order::list_table_orders))
        .route("/api/v1/qr-order/tables", get(routes::qr_order::list_resto_tables))
        // Domain 4: Demo / Trial Inquiries
        .route("/api/v1/trial/inquire", post(routes::trial::submit_trial_inquiry).get(routes::trial::list_trial_inquiries))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("Supernova Backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
