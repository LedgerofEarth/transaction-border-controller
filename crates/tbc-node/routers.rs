use std::sync::Arc;
use axum::{
    Router,
    Extension,
    routing::{post, get},
    extract::State,
};

use crate::{
    app_state::AppState,
    health::health_check,
};
use tbc_gateway::{InboundRouter, TGPInboundRouter, WsState};

pub fn build_routes(state: AppState) -> Router {
    // Create WebSocket state (stateless per TGP-TBC-SEC-00)
    let ws_state = Arc::new(WsState {
        tbc_id: state.cfg.tbc_id.clone().unwrap_or_else(|| "tbc-default".to_string()),
    });
    
    Router::new()
        .route("/health", get(health_check))

        // ---------------------------------------------------
        // TGP inbound routes (HTTP POST + WebSocket)
        // SECURITY: Both use same InboundRouter verification
        // Per TGP-TBC-SEC-00 §10.2: No bypass paths allowed
        // ---------------------------------------------------
        .route("/tgp", post(tgp_inbound))
        .route("/tgp/ws", get(ws_handler))

        .layer(Extension(ws_state))
        .with_state(state)
}

/// HTTP POST handler for TGP messages
/// 
/// SECURITY: Routes through full L1-L6 verification pipeline
async fn tgp_inbound(
    State(_state): State<AppState>,
    body: String,
) -> String {
    let router = InboundRouter::new();
    router.route_inbound(&body).await.unwrap_or_else(|e| {
        // Fail-closed: return structured ERROR
        format!(r#"{{"type":"ERROR","code":"TBC_HTTP_DISPATCH_ERROR","layer_failed":0,"message":"{}"}}"#, e)
    })
}

/// WebSocket upgrade handler for TGP messages
/// 
/// SECURITY: Uses same InboundRouter as HTTP endpoint
/// Per TGP-TBC-SEC-00: identical security guarantees
async fn ws_handler(
    Extension(ws_state): Extension<Arc<WsState>>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| tbc_gateway::ws::handler::handle_ws_public(socket, ws_state))
}