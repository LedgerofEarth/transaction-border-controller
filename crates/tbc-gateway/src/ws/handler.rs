//! WebSocket Handler for TGP-00 v3.2
//!
//! SECURITY NOTES (per TGP-TBC-SEC-00):
//! - Uses same InboundRouter as HTTP endpoint (no bypass)
//! - Fail-closed: all errors result in ERROR response
//! - Stateless: no session persistence in handler
//! - All messages routed through full L1-L6 verification

use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{StreamExt, SinkExt};
use crate::ws::state::WsState;
use crate::ws::router::route_ws_message;
use crate::logging::log_rx;

/// WebSocket upgrade endpoint
/// 
/// SECURITY: Upgrade is stateless. No session created.
pub async fn ws_upgrade(
    State(state): State<Arc<WsState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_public(socket, state))
}

/// Main WebSocket loop
/// 
/// SECURITY INVARIANTS:
/// - Every message goes through full TGP verification pipeline
/// - No message is processed without InboundRouter validation
/// - All errors are logged and returned as TGP ERROR messages
/// - Binary frames rejected (attack surface reduction)
pub async fn handle_ws_public(mut socket: WebSocket, state: Arc<WsState>) {
    tracing::info!("WebSocket connection established for TBC: {}", state.tbc_id);
    
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(body) => {
                // Log inbound for audit trail
                log_rx(&body);
                
                // Route through SAME verification pipeline as HTTP
                // Per TGP-TBC-SEC-00 §10.2: No bypass paths allowed
                match route_ws_message(&body).await {
                    Ok(resp) => {
                        let _ = socket.send(Message::Text(resp)).await;
                    }
                    Err(e) => {
                        // Fail-closed: return structured ERROR
                        // Per TGP-TBC-SEC-00 §9.1: All errors must be deterministic
                        let err_msg = format!(
                            r#"{{"type":"ERROR","code":"TBC_WS_DISPATCH_ERROR","layer_failed":0,"message":"{}"}}"#,
                            e.to_string().replace('"', "'")
                        );
                        tracing::error!("WebSocket dispatch error: {}", e);
                        let _ = socket.send(Message::Text(err_msg)).await;
                    }
                }
            }

            Message::Close(_) => {
                tracing::info!("WebSocket connection closed");
                break;
            }
            
            Message::Ping(p) => {
                let _ = socket.send(Message::Pong(p)).await;
            }
            
            Message::Pong(_) => {}
            
            Message::Binary(_) => {
                // SECURITY: Reject binary frames (attack surface reduction)
                // Only JSON text frames are valid TGP messages
                let err = r#"{"type":"ERROR","code":"TBC_WS_BINARY_REJECTED","layer_failed":0,"message":"Binary frames not supported. Use JSON text."}"#;
                tracing::warn!("Rejected binary WebSocket frame");
                let _ = socket.send(Message::Text(err.into())).await;
            }
        }
    }
}