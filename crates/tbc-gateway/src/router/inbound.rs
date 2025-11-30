//! # Inbound Router -- TGP-00 v3.2
//!
//! Fully stateless routing layer.
//! SESSIONSTORE IS NOW OPTIONAL -- Gateway does not mutate client sessions.
//!
//! Message flow:
//!   classify → validate → dispatch → encode
//!
//! Supported:
//!   • QUERY
//!   • ACK        (replaces OFFER)
//!   • SETTLE
//!   • ERROR
//!
//! Forbidden (legacy):
//!   • OFFER (removed)

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;

use tbc_core::{
    codec_tx::{
        classify_message,
        encode_message,
        validate_and_classify_message,
        ReplayProtector,
        InMemoryReplayCache,
        TGPValidationResult,
    },
    protocol::{TGPMessage, make_protocol_error, ErrorMessage},
};

use crate::handlers::{
    handle_inbound_query,
    handle_inbound_ack,
    handle_inbound_settle,
    handle_inbound_error,
};

use crate::logging::*;

/// ---------------------------------------------------------------------------
/// Trait Definition
/// ---------------------------------------------------------------------------
#[async_trait]
pub trait TGPInboundRouter {
    async fn route_inbound(&self, raw_json: &str) -> Result<String>;
}

/// ---------------------------------------------------------------------------
/// Stateless Router
/// ---------------------------------------------------------------------------
pub struct InboundRouter {
    pub replay: Arc<dyn ReplayProtector + Send + Sync>,
}

impl InboundRouter {
    pub fn new() -> Self {
        Self {
            replay: Arc::new(InMemoryReplayCache::default()),
        }
    }
}

/// ---------------------------------------------------------------------------
/// Router Implementation
/// ---------------------------------------------------------------------------
#[async_trait]
impl TGPInboundRouter for InboundRouter {
    async fn route_inbound(&self, raw_json: &str) -> Result<String> {
        log_rx(raw_json);

        // ====================================================================
        // 1. CLASSIFY JSON → (metadata, TGPMessage)
        // ====================================================================
        let (metadata, message) = match classify_message(raw_json) {
            Ok(pair) => pair,
            Err(e) => {
                let err = make_protocol_error(0, "INVALID_JSON", e.to_string());
                log_err(&err);
                return Ok(encode_message(&TGPMessage::Error(err))?);
            }
        };

        // ====================================================================
        // 2. REPLAY PROTECTION
        // ====================================================================
        if !self.replay.check_or_insert(&metadata.msg_id) {
            let err = make_protocol_error(
                0,
                "REPLAY_DETECTED",
                format!("Duplicate message ID: {}", metadata.msg_id),
            );
            log_err(&err);
            return Ok(encode_message(&TGPMessage::Error(err))?);
        }

        // ====================================================================
        // 3. STRUCTURAL + SEMANTIC VALIDATION
        // ====================================================================
        match validate_and_classify_message(&metadata, &message) {
            TGPValidationResult::Reject(err) => {
                log_err(&err);
                return Ok(encode_message(&TGPMessage::Error(err))?);
            }
            TGPValidationResult::Accept => {}
        }

        // ====================================================================
        // 4. DISPATCH TO HANDLERS (stateless, RFC-style)
        // ====================================================================
        let out_msg: TGPMessage = match &message {

            //----------------------------------------------------------
            // QUERY Handler
            //----------------------------------------------------------
            TGPMessage::Query(q) => {
                handle_inbound_query(&metadata, q.clone()).await?
            }

            //----------------------------------------------------------
            // ACK Handler  (replaces OFFER)
            //----------------------------------------------------------
            TGPMessage::Ack(a) => {
                handle_inbound_ack(&metadata, a.clone()).await?
            }

            //----------------------------------------------------------
            // SETTLE Handler
            //----------------------------------------------------------
            TGPMessage::Settle(s) => {
                handle_inbound_settle(&metadata, s.clone()).await?
            }

            //----------------------------------------------------------
            // ERROR Handler
            //----------------------------------------------------------
            TGPMessage::Error(e) => {
                handle_inbound_error(&metadata, e.clone()).await?
            }
        };

        // ====================================================================
        // 5. ENCODE OUTBOUND (SIP-style echo semantics)
        // ====================================================================
        let outbound = encode_message(&out_msg)
            .map_err(|e| anyhow!("encode error: {}", e))?;

        log_tx(&outbound);
        Ok(outbound)
    }
}