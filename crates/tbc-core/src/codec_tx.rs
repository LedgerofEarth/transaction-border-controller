//! # TGP Codec + Transport Layer (codec_tx.rs)
//!
//! Pure JSON codec + transport metadata for TGP-00.
//! No policy, no MGMT, no session logic, no chrono, no platform-unsafe deps.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    protocol::{TGPMessage, ErrorMessage, make_protocol_error},
};

// ================================================================================================
// Timestamp Helper (WASM-safe, no chrono)
// ================================================================================================

/// Deterministic, portable millisecond timestamp.
///
/// Avoids `chrono` because tbc-core must run in WASM, MCP agents,
/// gateways, and minimal embedded validators.
///
/// Not strictly RFC3339; it's metadata only.
fn now_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ================================================================================================
// Message Envelope (Controller-side framing)
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub session_id: Option<String>,
    pub correlation_id: Option<String>,
    pub received_at: u64,
    pub body: T,
}

impl<T> MessageEnvelope<T> {
    pub fn new(session_id: Option<String>, correlation_id: Option<String>, body: T) -> Self {
        Self {
            session_id,
            correlation_id,
            received_at: now_millis(),
            body,
        }
    }
}

// ================================================================================================
// Metadata (transport header reflection)
// ================================================================================================

#[derive(Debug, Clone)]
pub struct TGPMetadata {
    pub msg_type: TGPMessageType,
    pub msg_id: String,
    pub correlation_id: Option<String>,
    pub origin: String,
    pub raw_json: String,
}

#[derive(Debug, Clone)]
pub enum TGPMessageType {
    Query,
    Offer,
    Settle,
    Error,
}

impl TGPMetadata {
    pub fn from_message(raw_json: String, message: &TGPMessage) -> Self {
        let msg_type = match message {
            TGPMessage::Query(_)  => TGPMessageType::Query,
            TGPMessage::Offer(_)  => TGPMessageType::Offer,
            TGPMessage::Settle(_) => TGPMessageType::Settle,
            TGPMessage::Error(_)  => TGPMessageType::Error,
        };

        let correlation_id = match message {
            TGPMessage::Query(_) => None,
            TGPMessage::Offer(o) => Some(o.query_id.clone()),
            TGPMessage::Settle(s) => Some(s.query_or_offer_id.clone()),
            TGPMessage::Error(e) => e.correlation_id.clone(),
        };

        Self {
            msg_type,
            msg_id: message.id().to_string(),
            correlation_id,
            origin: "unknown".into(),
            raw_json,
        }
    }
}

// ================================================================================================
// JSON Parsing / Encoding Layer (pure SIP-style codec)
// ================================================================================================

pub fn parse_message(json: &str) -> Result<TGPMessage, String> {
    serde_json::from_str::<TGPMessage>(json)
        .map_err(|e| format!("Failed to parse TGPMessage: {}", e))
}

pub fn classify_message(json: &str) -> Result<(TGPMetadata, TGPMessage), String> {
    let msg = parse_message(json)?;
    let metadata = TGPMetadata::from_message(json.to_string(), &msg);
    Ok((metadata, msg))
}

pub fn encode_message(message: &TGPMessage) -> Result<String, String> {
    serde_json::to_string(message)
        .map_err(|e| format!("Failed to encode TGPMessage: {}", e))
}

// ================================================================================================
// Replay Protection
// ================================================================================================

pub trait ReplayProtector: Send + Sync {
    fn check_or_insert(&self, msg_id: &str) -> bool;
}

use std::sync::RwLock;

pub struct InMemoryReplayCache {
    window: usize,
    buffer: RwLock<Vec<String>>,
    index: RwLock<usize>,
}

impl InMemoryReplayCache {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            buffer: RwLock::new(vec![String::new(); window]),
            index: RwLock::new(0),
        }
    }
}

impl Default for InMemoryReplayCache {
    fn default() -> Self {
        Self::new(8192)
    }
}

impl ReplayProtector for InMemoryReplayCache {
    fn check_or_insert(&self, msg_id: &str) -> bool {
        let mut idx = self.index.write().unwrap();
        let mut buffer = self.buffer.write().unwrap();

        if buffer.contains(&msg_id.to_string()) {
            return false;
        }

        buffer[*idx] = msg_id.to_string();
        *idx = (*idx + 1) % self.window;

        true
    }
}

// ================================================================================================
// Error Builders
// ================================================================================================

pub fn error_from_validation(
    metadata: &TGPMetadata,
    reason: impl Into<String>,
) -> ErrorMessage {
    make_protocol_error(
        metadata.correlation_id.clone(),
        "POLICY_VIOLATION",
        reason.into(),
    )
}

pub fn error_from_exception(msg: impl Into<String>) -> ErrorMessage {
    make_protocol_error(None, "INTERNAL_ERROR", msg.into())
}

// ================================================================================================
// Validation
// ================================================================================================

#[derive(Debug)]
pub enum TGPValidationResult {
    Accept,
    Reject(ErrorMessage),
}

pub fn validate_and_classify_message(
    metadata: &TGPMetadata,
    message: &TGPMessage,
) -> TGPValidationResult {

    if let Err(e) = message.validate() {
        return TGPValidationResult::Reject(error_from_validation(metadata, e));
    }

    TGPValidationResult::Accept
}