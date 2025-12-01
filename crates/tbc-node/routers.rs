use std::sync::Arc;
use axum::{
    Router,
    Extension,
    routing::{post, get},
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    app_state::AppState,
    health::health_check,
    admin::auth::SignedRequest,
    admin::commands::{AdminCommand, CommandResult},
};
use tbc_gateway::{InboundRouter, TGPInboundRouter, WsState};

pub fn build_routes(state: AppState) -> Router {
    // Create WebSocket state (stateless per TGP-TBC-SEC-00)
    let ws_state = Arc::new(WsState {
        tbc_id: state.cfg.tbc_id.clone().unwrap_or_else(|| "tbc-default".to_string()),
    });
    
    // Log admin key status
    let admin_count = state.admin.auth.key_store().list_admins().len();
    if admin_count > 0 {
        tracing::info!("Admin API: {} admin keys loaded", admin_count);
    } else {
        println!("Admin API: No admin keys configured (set TBC_ADMIN_KEYS)");
    }
    
    Router::new()
        .route("/health", get(health_check))

        // ---------------------------------------------------
        // TGP inbound routes (HTTP POST + WebSocket)
        // SECURITY: Both use same InboundRouter verification
        // Per TGP-TBC-SEC-00 §10.2: No bypass paths allowed
        // ---------------------------------------------------
        .route("/tgp", post(tgp_inbound))
        .route("/tgp/ws", get(ws_handler))
        
        // ---------------------------------------------------
        // Admin routes (Ed25519 authenticated)
        // ---------------------------------------------------
        .route("/admin/health", get(admin_health))
        .route("/admin/exec", post(admin_exec))

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

/// Public admin health endpoint (no auth required)
async fn admin_health(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let uptime = state.admin.start_time.elapsed().as_secs();
    let admin_count = state.admin.auth.key_store().list_admins().len();
    
    Json(json!({
        "status": "ok",
        "service": "tbc-admin",
        "uptime_seconds": uptime,
        "tbc_id": state.cfg.tbc_id,
        "admin_keys_loaded": admin_count,
    }))
}

/// Execute authenticated admin command
async fn admin_exec(
    State(state): State<AppState>,
    Json(request): Json<SignedRequest>,
) -> impl IntoResponse {
    // Verify authentication
    let admin = match state.admin.auth.verify_request(&request) {
        Ok(admin) => admin,
        Err(e) => {
            tracing::warn!(
                pubkey = %request.public_key,
                command = %request.command,
                error = %e,
                "Admin auth failed"
            );
            return (
                StatusCode::UNAUTHORIZED,
                Json(CommandResult::err(&request.command, format!("Auth failed: {}", e))),
            );
        }
    };

    // Parse command
    let cmd: AdminCommand = match serde_json::from_value(json!({
        "cmd": request.command,
        "args": request.args,
    })) {
        Ok(cmd) => cmd,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CommandResult::err(&request.command, format!("Invalid command: {}", e))),
            );
        }
    };

    // Check role
    if let Err(e) = state.admin.auth.check_role(&admin, cmd.required_role()) {
        tracing::warn!(
            admin = %admin.name,
            command = %request.command,
            error = %e,
            "Admin permission denied"
        );
        return (
            StatusCode::FORBIDDEN,
            Json(CommandResult::err(&request.command, e.to_string())),
        );
    }

    // Execute command
    let result = crate::admin::run_admin_command(&state.admin, &admin, cmd).await;
    
    let status = if result.success {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, Json(result))
}