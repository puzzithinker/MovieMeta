//! REST API server for Movie Data Capture
//!
//! This crate provides a REST API and WebSocket interface for the MDC application.

pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod websocket;

pub use error::{ApiError, ApiResult};

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// Build the API router with all routes
pub fn create_router() -> Router {
    // Restrict CORS: allow any origin in development, but only same-origin in production
    // Override with CORS_ORIGIN env var for specific domains (comma-separated)
    let cors = match std::env::var("CORS_ORIGIN") {
        Ok(origins) if !origins.is_empty() => {
            let allowed: Vec<_> = origins
                .split(',')
                .filter_map(|o| o.trim().parse().ok())
                .collect();
            if allowed.is_empty() {
                CorsLayer::permissive()
            } else {
                CorsLayer::new()
                    .allow_origin(allowed)
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
        }
        _ => CorsLayer::permissive(),
    };

    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Job management
        .route("/api/jobs", post(handlers::create_job))
        .route("/api/jobs", get(handlers::list_jobs))
        .route("/api/jobs/:id", get(handlers::get_job))
        .route("/api/jobs/:id/cancel", post(handlers::cancel_job))
        .route("/api/jobs/:id/retry", post(handlers::retry_job))
        // File scanning
        .route("/api/scan", post(handlers::scan_folder))
        // Configuration
        .route("/api/config", get(handlers::get_config))
        .route("/api/config", post(handlers::update_config))
        // Statistics
        .route("/api/stats", get(handlers::get_stats))
        // WebSocket for progress updates
        .route("/ws/progress", get(websocket::ws_handler))
        // Add CORS middleware
        .layer(cors)
}

/// Start the API server
pub async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    tracing::info!("Starting API server on {}", addr);

    let app = create_router();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
