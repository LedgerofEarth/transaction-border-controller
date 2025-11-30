use axum::{
    Router,
    routing::{post, get},
    extract::State,
};

use crate::{
    app_state::AppState,
    health::health_check,
};
use tbc_gateway::{InboundRouter, TGPInboundRouter};

pub fn build_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))

        // ---------------------------------------------------
        // TGP inbound route (SBC-like message handler)
        // ---------------------------------------------------
        .route("/tgp", post(tgp_inbound))

        .with_state(state)
}

async fn tgp_inbound(
    State(_state): State<AppState>,
    body: String,
) -> String {
    let router = InboundRouter::new();
    router.route_inbound(&body).await.unwrap_or_else(|e| {
        format!(r#"{{"type":"ERROR","code":"RUNTIME_FAILURE","message":"{}"}}"#, e)
    })
}