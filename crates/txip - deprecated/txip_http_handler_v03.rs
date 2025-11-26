// crates/tbc-gateway/src/txip/txip_http_handler_v03.rs
// FINAL - TxIP HTTP Handler with CoreProver v0.3 Compatibility
//
// HTTP handler that accepts engine-provided timestamps.
// NO timing logic - all timestamps come from TimestampProvider.
// NO Instant, Duration, or SystemTime usage.

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::timestamp_types_v03::TimestampProvider;
use super::txip_session_v03::SessionManager;
use super::txip_types_v03::*;

/// Shared HTTP handler state
#[derive(Clone)]
pub struct HttpHandlerState<T: TimestampProvider> {
    pub session_manager: Arc<SessionManager<T>>,
    pub tbc_id: String,
}

/// HTTP response for successful message acceptance
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageAcceptedResponse {
    pub status: String,
    pub msg_id: String,
}

/// Handle incoming TxIP message via HTTP POST
/// 
/// Timestamps are validated from the envelope (client-provided)
/// but session timing uses engine's TimestampProvider
pub async fn handle_txip_message<T: TimestampProvider + Send + Sync + 'static>(
    State(state): State<Arc<HttpHandlerState<T>>>,
    Json(envelope): Json<TxipEnvelope>,
) -> Response {
    // Validate envelope structure
    if let Err(e) = envelope.validate() {
        return error_response(
            &state,
            &envelope.session_id,
            ErrorCode::TxipInvalidEnvelope,
            400,
            Some(envelope.msg_id.clone()),
            e,
            false,
        );
    }

    // Check for duplicate message (idempotency)
    if state.session_manager.is_duplicate_message(&envelope.session_id, &envelope.msg_id) {
        // Return success for idempotent request
        return accepted_response(&envelope.msg_id);
    }

    // Route based on message type
    match &envelope.message_type {
        MessageType::Control => handle_control_message(state, envelope).await,
        MessageType::Tgp => handle_tgp_message(state, envelope).await,
        MessageType::Error => {
            // Clients shouldn't send ERROR messages to us, but we can log it
            tracing::warn!("Received ERROR message from client: {:?}", envelope);
            accepted_response(&envelope.msg_id)
        }
    }
}

/// Handle CONTROL messages
async fn handle_control_message<T: TimestampProvider + Send + Sync + 'static>(
    state: Arc<HttpHandlerState<T>>,
    envelope: TxipEnvelope,
) -> Response {
    match &envelope.payload {
        Payload::Control(control) => match control {
            ControlPayload::Hello(hello) => {
                handle_hello(state, envelope.session_id, envelope.msg_id, hello, envelope.role)
            }
            ControlPayload::Heartbeat(_heartbeat) => {
                handle_heartbeat(state, envelope.session_id, envelope.msg_id)
            }
            ControlPayload::Close(close) => {
                handle_close(state, envelope.session_id, close)
            }
            ControlPayload::Welcome(_) => {
                // Clients shouldn't send WELCOME to us
                error_response(
                    &state,
                    &envelope.session_id,
                    ErrorCode::TxipInvalidEnvelope,
                    400,
                    Some(envelope.msg_id),
                    "WELCOME messages are sent by TBC, not received".to_string(),
                    false,
                )
            }
        },
        _ => error_response(
            &state,
            &envelope.session_id,
            ErrorCode::TxipInvalidEnvelope,
            400,
            Some(envelope.msg_id),
            "Invalid payload for CONTROL message".to_string(),
            false,
        ),
    }
}

/// Handle HELLO message
fn handle_hello<T: TimestampProvider + Send + Sync + 'static>(
    state: Arc<HttpHandlerState<T>>,
    session_id: String,
    msg_id: String,
    hello: &HelloPayload,
    role: Role,
) -> Response {
    // Create or update session (uses engine timestamp internally)
    match state.session_manager.handle_hello(hello, session_id.clone(), role) {
        Ok(session_info) => {
            // Record message
            let _ = state.session_manager.record_message(&session_id, &msg_id);

            // Get current timestamp from engine for WELCOME response
            let now = state.session_manager.now();

            // Create WELCOME response
            let welcome = TxipEnvelope::welcome(
                generate_msg_id(),
                session_id,
                state.tbc_id.clone(),
                session_info.negotiated_tgp_version,
                session_info.negotiated_chains,
                session_info.features,
                state.session_manager.heartbeat_interval_sec(),
                now,
            );

            Json(welcome).into_response()
        }
        Err(e) => error_response(
            &state,
            &session_id,
            ErrorCode::TxipInternalError,
            500,
            Some(msg_id),
            format!("Failed to create session: {}", e),
            true,
        ),
    }
}

/// Handle HEARTBEAT message
fn handle_heartbeat<T: TimestampProvider + Send + Sync + 'static>(
    state: Arc<HttpHandlerState<T>>,
    session_id: String,
    msg_id: String,
) -> Response {
    // Update session activity (uses engine timestamp internally)
    if let Err(e) = state.session_manager.touch_session(&session_id) {
        return error_response(
            &state,
            &session_id,
            ErrorCode::TxipInternalError,
            500,
            Some(msg_id.clone()),
            format!("Failed to update session: {}", e),
            true,
        );
    }

    // Record message
    let _ = state.session_manager.record_message(&session_id, &msg_id);

    accepted_response(&msg_id)
}

/// Handle CLOSE message
fn handle_close<T: TimestampProvider + Send + Sync + 'static>(
    state: Arc<HttpHandlerState<T>>,
    session_id: String,
    _close: &ClosePayload,
) -> Response {
    // Close session
    let _ = state.session_manager.close_session(&session_id);

    // Return 200 with no body
    StatusCode::OK.into_response()
}

/// Handle TGP messages
async fn handle_tgp_message<T: TimestampProvider + Send + Sync + 'static>(
    state: Arc<HttpHandlerState<T>>,
    envelope: TxipEnvelope,
) -> Response {
    // Verify session exists
    if state.session_manager.get_session(&envelope.session_id).is_none() {
        return error_response(
            &state,
            &envelope.session_id,
            ErrorCode::TxipUnauthenticated,
            401,
            Some(envelope.msg_id.clone()),
            "Session not found. Send HELLO first.".to_string(),
            false,
        );
    }

    // Update session activity (uses engine timestamp internally)
    let _ = state.session_manager.touch_session(&envelope.session_id);

    // Record message
    let _ = state.session_manager.record_message(&envelope.session_id, &envelope.msg_id);

    // Extract TGP payload
    match &envelope.payload {
        Payload::Tgp(tgp_payload) => {
            // TODO: Forward to TGP routing layer
            // The routing layer will receive:
            // - session_id
            // - origin_chain_id (if present)
            // - tgp_type (event type if present)
            // - Full TGP message payload
            // - Envelope timestamp for ordering
            
            tracing::info!(
                "Received TGP message: phase={:?}, session={}, msg_id={}, chain={:?}",
                envelope.tgp_phase,
                envelope.session_id,
                envelope.msg_id,
                envelope.origin_chain_id,
            );

            // In production, this would be:
            // tgp_router::route_message(
            //     &envelope.session_id,
            //     envelope.origin_chain_id,
            //     &envelope.tgp_phase,
            //     envelope.tgp_type.as_deref(),
            //     &tgp_payload.tgp,
            //     envelope.timestamp(),
            // ).await?;

            accepted_response(&envelope.msg_id)
        }
        _ => error_response(
            &state,
            &envelope.session_id,
            ErrorCode::TxipInvalidEnvelope,
            400,
            Some(envelope.msg_id),
            "Invalid payload for TGP message".to_string(),
            false,
        ),
    }
}

/// Create a success response
fn accepted_response(msg_id: &str) -> Response {
    let response = MessageAcceptedResponse {
        status: "accepted".to_string(),
        msg_id: msg_id.to_string(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Create an error response with engine timestamp
fn error_response<T: TimestampProvider + Send + Sync + 'static>(
    state: &Arc<HttpHandlerState<T>>,
    session_id: &str,
    error_code: ErrorCode,
    http_status: u16,
    related_msg_id: Option<String>,
    details: String,
    retryable: bool,
) -> Response {
    let now = state.session_manager.now();
    
    let envelope = TxipEnvelope::error(
        generate_msg_id(),
        session_id.to_string(),
        Direction::TbcToClient,
        error_code,
        http_status,
        related_msg_id,
        details,
        retryable,
        now,
    );

    let status = StatusCode::from_u16(http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    (status, Json(envelope)).into_response()
}

/// Generate a message ID (UUID v4)
fn generate_msg_id() -> String {
    // In production, use uuid crate
    // For now, return a simple placeholder
    format!("msg-{}", rand::random::<u64>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txip::timestamp_types_v03::TripleTimestamp;

    /// Test timestamp provider for unit tests
    struct TestTimestampProvider {
        current_mono: std::sync::atomic::AtomicU64,
        current_unix: std::sync::atomic::AtomicU64,
    }

    impl TestTimestampProvider {
        fn new(mono: u64, unix: u64) -> Self {
            Self {
                current_mono: std::sync::atomic::AtomicU64::new(mono),
                current_unix: std::sync::atomic::AtomicU64::new(unix),
            }
        }
    }

    impl TimestampProvider for TestTimestampProvider {
        fn now(&self) -> TripleTimestamp {
            let mono = self.current_mono.load(std::sync::atomic::Ordering::SeqCst);
            let unix = self.current_unix.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(
                mono,
                unix,
                format!("2024-11-14T12:00:{}Z", mono % 60),
            )
        }

        fn at_unix(&self, unix: u64) -> TripleTimestamp {
            let mono = self.current_mono.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(mono, unix, "2024-11-14T12:00:00Z".to_string())
        }

        fn at_mono(&self, mono: u64) -> TripleTimestamp {
            let unix = self.current_unix.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(mono, unix, "2024-11-14T12:00:00Z".to_string())
        }
    }

    fn create_test_state() -> Arc<HttpHandlerState<TestTimestampProvider>> {
        let provider = Arc::new(TestTimestampProvider::new(1000, 1731600000));
        let session_manager = Arc::new(SessionManager::new(Default::default(), provider));
        
        Arc::new(HttpHandlerState {
            session_manager,
            tbc_id: "tbc://test".to_string(),
        })
    }

    #[test]
    fn test_hello_response() {
        let state = create_test_state();
        
        let hello_payload = HelloPayload {
            agent_id: "buyer://alice".to_string(),
            supported_tgp_versions: vec!["2.0".to_string()],
            supported_transports: vec!["HTTP".to_string()],
            supported_chains: vec![1, 369],
            supported_assets: vec!["USDC".to_string()],
            features: Features {
                zk_discount_proofs: true,
                receipt_ownership_proofs: true,
                late_discount_support: true,
                cross_chain_support: true,
            },
            auth: AuthInfo {
                scheme: AuthScheme::None,
                token: None,
            },
        };

        let response = handle_hello(
            state.clone(),
            "sess-456".to_string(),
            "msg-123".to_string(),
            &hello_payload,
            Role::BuyerAgent,
        );

        // Verify WELCOME was returned (status 200)
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_accepted_response() {
        let response = accepted_response("msg-123");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
