use axum::Json;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

/// Health check endpoint for load balancers and monitoring.
/// Returns JSON with service status and metadata.
pub async fn health_check() -> Json<serde_json::Value> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    
    Json(json!({
        "status": "ok",
        "service": "tbc-node",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "TGP-00 v3.2",
        "timestamp": uptime,
        "endpoints": {
            "http": "/tgp",
            "websocket": "/tgp/ws",
            "health": "/health"
        },
        "security": {
            "layers": ["L1-Registry", "L2-Signature", "L3-Bytecode", "L4-ZK", "L5-Policy"],
            "mode": "fail-closed"
        }
    }))
}