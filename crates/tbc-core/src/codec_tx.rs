//! # TGP Codec + Transport Layer (codec_tx.rs)
//!
//! This module corresponds to SIP’s separation of:
//!   • RFC3261 §7  – *Message Encoding / Decoding*
//!   • RFC3261 §8  – *Transaction Layer*
//!
//! In TGP-00, this file is the *pure transport codec*:
//!   • Parse JSON into typed TGPMessage
//!   • Serialize typed TGPMessage to JSON
//!   • Construct metadata (TGPMetadata)
//!   • Provide MessageEnvelope<T> for router-level framing
//!   • Provide replay-window protection (in-memory ring buffer)
//!
//! Handlers remain **pure** and take typed structures only.
//! Router remains controller-side logic.
//!
//! No policy logic lives here. No state transitions. No handler mutation.
//!
//! This module is intentionally "dumb," just like SIP's parser layer,
//! ensuring correctness, inspectability, and security by simplicity.

use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

use crate::tgp::{
    protocol::TGPMessage,
    protocol::{ErrorMessage, make_protocol_error},
};

// ================================================================================================
// Message Envelope (Controller-side framing)
// ================================================================================================

/// Controller-side envelope similar to SIP’s transaction-layer transport wrapper.
///
/// Encapsulates:
///   • session_id (if established)
///   • correlation_id
///   • rx timestamp (ms)
///   • body (typed TGPMessage)
///
/// This is what the router operates on before dispatch.
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
            received_at: Utc::now().timestamp_millis() as u64,
            body,
        }
    }
}

// ================================================================================================
// Metadata (transport header reflection)
// ================================================================================================

/// The router inspects this before handler dispatch.
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
    /// SIP-style classification pass: parse → reflect info.
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

/// JSON → typed TGPMessage (pure function)
pub fn parse_message(json: &str) -> Result<TGPMessage, String> {
    serde_json::from_str::<TGPMessage>(json)
        .map_err(|e| format!("Failed to parse TGPMessage: {}", e))
}

/// Parse + metadata classification
pub fn classify_message(json: &str) -> Result<(TGPMetadata, TGPMessage), String> {
    let msg = parse_message(json)?;
    let metadata = TGPMetadata::from_message(json.to_string(), &msg);
    Ok((metadata, msg))
}

/// typed TGPMessage → JSON
pub fn encode_message(message: &TGPMessage) -> Result<String, String> {
    serde_json::to_string(message)
        .map_err(|e| format!("Failed to encode TGPMessage: {}", e))
}

// ================================================================================================
// Replay Protection (Option B: 8192 sliding window)
// ================================================================================================

/// Generic trait to allow redis, dynamo, in-mem, or sharded cache.
pub trait ReplayProtector: Send + Sync {
    /// Returns true if this msg_id is fresh.
    /// Returns false if replayed.
    fn check_or_insert(&self, msg_id: &str) -> bool;
}

/// In-memory ring buffer (lock-free under RwLock)
///
/// Memory use:
///   • 8192 entries * 48 bytes ≈ 0.39 MB
/// Fast. Perfect for a single TBC instance. Replaceable.
use std::sync::{RwLock};

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
        Self::new(8192)   // Option B -- fixed replay window
    }
}

impl ReplayProtector for InMemoryReplayCache {
    fn check_or_insert(&self, msg_id: &str) -> bool {
        let mut idx = self.index.write().unwrap();
        let mut buffer = self.buffer.write().unwrap();

        // Check for presence
        if buffer.contains(&msg_id.to_string()) {
            return false; // replay
        }

        // Insert in ring
        buffer[*idx] = msg_id.to_string();
        *idx = (*idx + 1) % self.window;

        true
    }
}

// ================================================================================================
// Error Builders -- spec compliant
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
// Transport Validation Entry Point
// ================================================================================================

/// SIP-style "initial validity" check (message-structure only)
///
/// Handlers are pure. Router applies policy. Codec is structure-only.
#[derive(Debug)]
pub enum TGPValidationResult {
    Accept,
    Reject(ErrorMessage),
}

pub fn validate_and_classify_message(
    metadata: &TGPMetadata,
    message: &TGPMessage,
) -> TGPValidationResult {

    // 1. Built-in structural validation (Message.validate())
    if let Err(e) = message.validate() {
        return TGPValidationResult::Reject(error_from_validation(metadata, e));
    }

    // 2. No policy here -- router does it.
    // 3. No settlement logic here -- handlers do it.
    // 4. No state logic here -- state.rs does it.

    TGPValidationResult::Accept
}