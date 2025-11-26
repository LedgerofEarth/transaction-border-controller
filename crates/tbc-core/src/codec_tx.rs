//! # codec_tx.rs -- TGP-00 v3.2 Transport Codec
//!
//! Responsibilities:
//!   • Raw JSON parsing
//!   • Message-type classification
//!   • Structural validation
//!   • Replay metadata extraction
//!
//! Supported types per TGP-00 v3.2:
//!   • QUERY
//!   • ACK
//!   • ERROR
//!   • SETTLE
//!
//! Forbidden:
//!   • OFFER (removed in v3.2)

use serde_json::{Value, json};
use anyhow::{Result, anyhow};
use uuid::Uuid;

use crate::protocol::{
    TGPMessage,
    QueryMessage,
    AckMessage, AckStatus,
    ErrorMessage,
    SettleMessage,
};

/// Metadata extracted during parse/classify stage.
#[derive(Debug, Clone)]
pub struct TGPMetadata {
    pub msg_id: String,
    pub msg_type: String,
    pub correlation_id: Option<String>,
}

/// Replay protection trait.
pub trait ReplayProtector {
    fn check_or_insert(&self, msg_id: &str) -> bool;
}

/// In-memory replay cache for dev/tests.
#[derive(Default)]
pub struct InMemoryReplayCache(std::sync::Mutex<std::collections::HashSet<String>>);

impl ReplayProtector for InMemoryReplayCache {
    fn check_or_insert(&self, msg_id: &str) -> bool {
        let mut guard = self.0.lock().unwrap();
        guard.insert(msg_id.to_string())
    }
}

/// ===========================================================================
/// classify_message(raw_json)
/// → (TGPMetadata, TGPMessage)
/// ===========================================================================
pub fn classify_message(raw: &str) -> Result<(TGPMetadata, TGPMessage)> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| anyhow!("JSON parse error: {}", e))?;

    // -----------------------------------------------------------------------
    // 1. Extract "type"
    // -----------------------------------------------------------------------
    let typ = v.get("type")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("Missing required field: type"))?
        .to_uppercase();

    // -----------------------------------------------------------------------
    // 2. Extract or generate message ID
    // -----------------------------------------------------------------------
    let msg_id = match v.get("id").and_then(|x| x.as_str()) {
        Some(id) => id.to_string(),
        None => Uuid::new_v4().to_string(),
    };

    let correlation_id = v.get("correlation_id")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());

    let metadata = TGPMetadata {
        msg_id: msg_id.clone(),
        msg_type: typ.clone(),
        correlation_id,
    };

    // -----------------------------------------------------------------------
    // 3. Dispatch by type
    // -----------------------------------------------------------------------
    let tgp_msg = match typ.as_str() {

        // ---------------------------------------------------------------
        // QUERY
        // ---------------------------------------------------------------
        "QUERY" => {
            let q: QueryMessage = serde_json::from_value(v.clone())
                .map_err(|e| anyhow!("Invalid QUERY: {}", e))?;
            TGPMessage::Query(q)
        }

        // ---------------------------------------------------------------
        // ACK  (new in v3.2, replaces OFFER)
        // ---------------------------------------------------------------
        "ACK" => {
            let status = v.get("status")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow!("ACK missing `status` field"))?
                .to_lowercase();

            let status_enum = match status.as_str() {
                "offer" =>  AckStatus::Offer,
                "allow" =>  AckStatus::Allow,
                "deny"  =>  AckStatus::Deny,
                "revise"=>  AckStatus::Revise,
                _ => return Err(anyhow!("Unknown ACK.status: {}", status)),
            };

            // Deserialize full ACK body
            let mut ack: AckMessage = serde_json::from_value(v.clone())
                .map_err(|e| anyhow!("Invalid ACK: {}", e))?;

            ack.status = status_enum;
            TGPMessage::Ack(ack)
        }

        // ---------------------------------------------------------------
        // SETTLE
        // ---------------------------------------------------------------
        "SETTLE" => {
            let s: SettleMessage = serde_json::from_value(v.clone())
                .map_err(|e| anyhow!("Invalid SETTLE: {}", e))?;
            TGPMessage::Settle(s)
        }

        // ---------------------------------------------------------------
        // ERROR
        // ---------------------------------------------------------------
        "ERROR" => {
            let e: ErrorMessage = serde_json::from_value(v.clone())
                .map_err(|e| anyhow!("Invalid ERROR: {}", e))?;
            TGPMessage::Error(e)
        }

        // ---------------------------------------------------------------
        // Unsupported types
        // ---------------------------------------------------------------
        other => {
            return Err(anyhow!("Unsupported TGP message type: {}", other));
        }
    };

    Ok((metadata, tgp_msg))
}

/// ===========================================================================
/// validate_and_classify_message(metadata, message)
/// Simplified for v3.2 -- each message performs its own validate().
/// ===========================================================================
pub enum TGPValidationResult {
    Accept,
    Reject(ErrorMessage),
}

pub fn validate_and_classify_message(
    meta: &TGPMetadata,
    msg: &TGPMessage,
) -> TGPValidationResult {

    let res = match msg {
        TGPMessage::Query(m)  => m.validate(),
        TGPMessage::Ack(m)    => m.validate(),
        TGPMessage::Settle(m) => m.validate(),
        TGPMessage::Error(m)  => m.validate(),
    };

    match res {
        Ok(_) => TGPValidationResult::Accept,
        Err(e) => {
            let err = ErrorMessage::new(
                meta.msg_id.clone(),
                "INVALID_MESSAGE",
                format!("{}", e)
            );
            TGPValidationResult::Reject(err)
        }
    }
}

/// ===========================================================================
/// encode_message(msg)
/// ===========================================================================
pub fn encode_message(msg: &TGPMessage) -> Result<String> {
    serde_json::to_string(msg)
        .map_err(|e| anyhow!("encode error: {}", e))
}