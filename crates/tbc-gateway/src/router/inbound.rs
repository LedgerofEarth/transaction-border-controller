//! # Inbound Router (Controller-Side)
//!
//! This is the SIP-style Transaction Layer for the Transaction Border Controller.
//!
//! Responsibilities:
//!   • transport parsing      (via codec_tx)
//!   • message classification (phase → handler)
//!   • replay protection      (via codec_tx::ReplayProtector)
//!   • structural validation  (TGP-00 §3)
//!   • state lookup/update    (TGP-00 §4 session lifecycle)
//!   • handler dispatch       (pure functions in handlers/*)
//!   • unified error model    (protocol.rs::make_protocol_error)
//!   • logging                (JSON or ANSI-safe)
//!
//! NOT responsible for application logic (handlers are pure).
//! NOT responsible for settlement verification (MCP agent handles this externally).
//!
//! Mirrors SIP RFC3261 Transaction Layer boundaries:
//!   • Parsing separated (codec_tx)
//!   • Transport separated (gateway.rs)
//!   • Application logic isolated from router (handlers/*)

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use tbc_core::tgp::{
    codec_tx::{
        classify_message,
        encode_message,
        validate_and_classify_message,
        ReplayProtector,
    },
    protocol::TGPMessage,
    state::{TGPSession, SessionStore},
};

use crate::handlers::{
    handle_inbound_query,
    handle_inbound_offer,
    handle_inbound_settle,
    handle_inbound_error,
};

use crate::logging::*;

/// Inbound Router Interface
#[async_trait]
pub trait TGPInboundRouter {
    async fn route_inbound(&self, raw_json: &str) -> Result<String>;
}

/// Default Router Implementation
pub struct InboundRouter<S: SessionStore + Send + Sync> {
    pub sessions: S,
    pub replay: ReplayProtector,
}

impl<S: SessionStore + Send + Sync> InboundRouter<S> {
    pub fn new(sessions: S) -> Self {
        Self {
            sessions,
            replay: ReplayProtector::default(),
        }
    }
}

#[async_trait]
impl<S: SessionStore + Send + Sync> TGPInboundRouter for InboundRouter<S> {
    async fn route_inbound(&self, raw_json: &str) -> Result<String> {
        log_rx(raw_json);

        // ------------------------------------------------------
        // 1. Decode + classify (via codec_tx)
        // ------------------------------------------------------
        let (metadata, message) = match classify_message(raw_json) {
            Ok(v) => v,
            Err(e) => {
                let err = codec_tx::make_protocol_error(None, "INVALID_JSON", e);
                log_err(&err);
                return Ok(encode_message(&TGPMessage::Error(err))?);
            }
        };

        // ------------------------------------------------------
        // 2. Replay protection
        // ------------------------------------------------------
        if !self.replay.check_or_insert(&metadata.msg_id) {
            let err = codec_tx::make_protocol_error(
                metadata.correlation_id.clone(),
                "REPLAY_DETECTED",
                format!("Duplicate message ID: {}", metadata.msg_id),
            );
            log_err(&err);
            return Ok(encode_message(&TGPMessage::Error(err))?);
        }

        // ------------------------------------------------------
        // 3. Message validation (pure structural)
        // ------------------------------------------------------
        match validate_and_classify_message(&metadata, &message) {
            codec_tx::TGPValidationResult::Reject(err) => {
                log_err(&err);
                return Ok(encode_message(&TGPMessage::Error(err))?);
            }
            codec_tx::TGPValidationResult::Accept => {}
        }

        // ------------------------------------------------------
        // 4. Session lookup rules (TGP-00 §4)
        // ------------------------------------------------------
        let session = match &message {
            TGPMessage::Query(_) => {
                // lazy new session
                let s = self.sessions.create_session(metadata.msg_id.clone()).await?;
                log_session_created(&s);
                s
            }

            TGPMessage::Offer(o) => {
                self.sessions
                    .get_session(&o.query_id)
                    .await?
                    .ok_or_else(|| anyhow!("Unknown session for OFFER: {}", o.query_id))?
            }

            TGPMessage::Settle(s) => {
                self.sessions
                    .get_session(&s.query_or_offer_id)
                    .await?
                    .ok_or_else(|| anyhow!("Unknown session for SETTLE: {}", s.query_or_offer_id))?
            }

            TGPMessage::Error(e) => {
                if let Some(cid) = &e.correlation_id {
                    self.sessions.get_session(cid).await?
                } else {
                    None
                }
                .unwrap_or_else(|| TGPSession::ephemeral(&metadata.msg_id))
            }
        };

        // ------------------------------------------------------
        // 5. Dispatch to pure handlers
        // ------------------------------------------------------
        let response = match message {
            TGPMessage::Query(q) => {
                let span = tgp_span(&session.session_id, "QUERY");
                let _enter = span.enter();
                handle_inbound_query(&metadata, &session, q).await?
            }

            TGPMessage::Offer(o) => {
                let span = tgp_span(&session.session_id, "OFFER");
                let _enter = span.enter();
                handle_inbound_offer(&metadata, &session, o).await?
            }

            TGPMessage::Settle(s) => {
                let span = tgp_span(&session.session_id, "SETTLE");
                let _enter = span.enter();
                handle_inbound_settle(&metadata, &session, s).await?
            }

            TGPMessage::Error(e) => {
                let span = tgp_span(&session.session_id, "ERROR");
                let _enter = span.enter();
                handle_inbound_error(&metadata, &session, e).await?
            }
        };

        // ------------------------------------------------------
        // 6. Encode + logging
        // ------------------------------------------------------
        let response_json = encode_message(&response)?;
        log_tx(&response_json);
        Ok(response_json)
    }
}
