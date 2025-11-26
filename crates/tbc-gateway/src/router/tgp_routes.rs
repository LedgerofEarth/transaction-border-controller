//! tgp_routes.rs -- TGP-00 v3.2 Routing Layer
//! -----------------------------------------
//! All TGP traffic flows through a single endpoint.
//!
//! The router delegates to the inbound TGP router (controller)
//! which handles:
//!   • decode → validate → classify
//!   • QUERY → state engine
//!   • SETTLE passthrough
//!   • ERROR passthrough
//!
//! CoreProve ZK envelopes (ZKB01/ZKS01/ZKM01) are NOT exposed here;
//! they are executed inside tx_builder during QUERY processing.

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};

use crate::AppState;
use crate::tgp_inbound::InboundRouter;


/// ---------------------------------------------------------------------------
/// Build canonical TGP routes
/// ---------------------------------------------------------------------------
pub fn build_tgp_routes() -> Router<AppState> {
    Router::new()
        .route("/tgp", post(handle_tgp_message))
}


/// ---------------------------------------------------------------------------
/// Handle inbound TGP JSON
/// ---------------------------------------------------------------------------
async fn handle_tgp_message(
    State(app): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {

    let inbound = InboundRouter::new();

    // Convert JSON to string for router
    let raw = payload.to_string();

    match inbound.route_inbound(&raw).await {
        Ok(outbound_raw) => {
            // Convert back into JSON for axum
            let parsed: serde_json::Value = serde_json::from_str(&outbound_raw)
                .unwrap_or_else(|_| json!({"type": "ERROR", "code": "TGP_GATEWAY_ENCODING_ERROR"}));

            Json(parsed)
        }
        Err(e) => {
            Json(json!({
                "type": "ERROR",
                "code": "TGP_GATEWAY_ROUTING_FAILURE",
                "message": e.to_string()
            }))
        }
    }
}