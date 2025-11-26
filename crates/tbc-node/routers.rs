use axum::{
    Router,
    routing::{post, get},
    extract::State,
    Json,
};

use crate::{
    AppState,
    health::health_check,
};
use tbc_gateway::build_router as build_gateway_router;

pub fn build_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))

        // ---------------------------------------------------
        // TGP inbound route (SBC-like message handler)
        // ---------------------------------------------------
        .route("/tgp", post(tgp_inbound))

        // ---------------------------------------------------
        // Mount all pure gateway routes
        // ---------------------------------------------------
        .nest("/gateway", build_gateway_router(state.clone()))

        .with_state(state)
}

async fn tgp_inbound(
    State(state): State<AppState>,
    body: String,
) -> String {
    let router = tbc_gateway::InboundRouter::new();
    router.route_inbound(&body).await.unwrap_or_else(|e| {
        format!(r#"{{"type":"ERROR","code":"RUNTIME_FAILURE","message":"{}"}}"#, e)
    })
}