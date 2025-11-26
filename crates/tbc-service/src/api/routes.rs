//! API routes

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use super::handlers;

/// Create the API router
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/escrow/:order_id", get(handlers::get_escrow))
        .route("/escrow", post(handlers::create_escrow))
        .route("/events", get(handlers::query_events))
        .layer(TraceLayer::new_for_http())
}