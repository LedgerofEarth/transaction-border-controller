//! # Inbound Router (Controller-Side)
//!
//! Transaction Layer for the Transaction Border Controller.
//!
//! Responsibilities:
//!   • transport parsing (codec_tx)
//!   • message classification (phase → handler)
//!   • replay protection
//!   • structural validation (TGP-00 §3)
//!   • session lookup/update (TGP-00 §4)
//!   • handler dispatch (pure functions)
//!   • unified error model (protocol.rs)
//!   • logging (JSON/ANSI-safe)

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::json;

use tbc_core::{
    codec_tx::{
        classify_message,
        encode_message,
        validate_and_classify_message,
        ReplayProtector,
        InMemoryReplayCache,
        TGPValidationResult,
    },
    protocol::{TGPMessage, make_protocol_error},
    tgp::state::{TGPSession, TGPState, SessionStore},
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
    pub replay: Arc<dyn ReplayProtector + Send + Sync>,
}

impl<S: SessionStore + Send + Sync> InboundRouter<S> {
    pub fn new(sessions: S) -> Self {
        Self {
            sessions,
            replay: Arc::new(InMemoryReplayCache::default()),
        }
    }
}

#[async_trait]
impl<S: SessionStore + Send + Sync> TGPInboundRouter for InboundRouter<S> {
    async fn route_inbound(&self, raw_json: &str) -> Result<String> {
        log_rx(raw_json);

        // ------------------------------------------------------
        // 1. Decode + classify
        // ------------------------------------------------------
        let (metadata, message) = match classify_message(raw_json) {
            Ok(v) => v,
            Err(e) => {
                let err = make_protocol_error(None, "INVALID_JSON", e);
                log_err(&err);
                return encode_message(&TGPMessage::Error(err))
                    .map_err(|e| anyhow!("encode error: {}", e));
            }
        };

        // ------------------------------------------------------
        // 2. Replay protection
        // ------------------------------------------------------
        if !self.replay.check_or_insert(&metadata.msg_id) {
            let err = make_protocol_error(
                metadata.correlation_id.clone(),
                "REPLAY_DETECTED",
                format!("Duplicate message ID: {}", metadata.msg_id),
            );
            log_err(&err);

            let encoded = encode_message(&TGPMessage::Error(err))
                .map_err(|e| anyhow!("encode error: {}", e))?;

            return Ok(encoded);
        }

        // ------------------------------------------------------
        // 3. Structural validation
        // ------------------------------------------------------
        match validate_and_classify_message(&metadata, &message) {
            TGPValidationResult::Reject(err) => {
                log_err(&err);

                let encoded = encode_message(&TGPMessage::Error(err))
                    .map_err(|e| anyhow!("encode error: {}", e))?;

                return Ok(encoded);
            }
            TGPValidationResult::Accept => { /* OK */ }
        }

        // ------------------------------------------------------
        // 4. Session lookup rules (TGP-00 §4)
        // ------------------------------------------------------
        let session = match &message {
            // Query → create + persist new session
            TGPMessage::Query(_) => {
                let s = self.sessions.create_session(metadata.msg_id.clone()).await?;
                self.sessions.put_session(&s.session_id, s.clone()).await?;
                log_session_created(&s);
                s
            }

            // Offer → must reference an existing session
            TGPMessage::Offer(o) => {
                self.sessions
                    .get_session(&o.query_id)
                    .await?
                    .ok_or_else(|| anyhow!("Unknown session for OFFER: {}", o.query_id))?
            }

            // Settle → must reference existing session
            TGPMessage::Settle(s) => {
                self.sessions
                    .get_session(&s.query_or_offer_id)
                    .await?
                    .ok_or_else(|| anyhow!("Unknown session for SETTLE: {}", s.query_or_offer_id))?
            }

            // Error → ephemeral if correlation_id missing
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
        let response = match &message {
            TGPMessage::Query(q) => {
                let span = tgp_span(&session.session_id, "QUERY");
                let _enter = span.enter();
                handle_inbound_query(&metadata, &session, q.clone()).await?
            }

            TGPMessage::Offer(o) => {
                let span = tgp_span(&session.session_id, "OFFER");
                let _enter = span.enter();
                handle_inbound_offer(&metadata, &session, o.clone()).await?
            }

            TGPMessage::Settle(s) => {
                let span = tgp_span(&session.session_id, "SETTLE");
                let _enter = span.enter();
                handle_inbound_settle(&metadata, &session, s.clone()).await?
            }

            TGPMessage::Error(e) => {
                let span = tgp_span(&session.session_id, "ERROR");
                let _enter = span.enter();
                handle_inbound_error(&metadata, &session, e.clone()).await?
            }
        };

        // ------------------------------------------------------
        // 6. State transition + persistence (TGP-00 §4)
        // ------------------------------------------------------
        let mut session = session;

        match &message {
            // --------------------------------------------------
            // Query → QuerySent
            // --------------------------------------------------
            TGPMessage::Query(_) => {
                session.transition(TGPState::QuerySent)?;
                self.sessions.update_session(&session).await?;
            }

            // --------------------------------------------------
            // Offer → OfferReceived
            // --------------------------------------------------
            TGPMessage::Offer(_) => {
                session.transition(TGPState::OfferReceived)?;
                self.sessions.update_session(&session).await?;
            }

            // --------------------------------------------------
            // Settle → Settled or Errored
            // --------------------------------------------------
            TGPMessage::Settle(s) => {
                if s.success {
                    session.transition(TGPState::Settled)?;
                } else {
                    session.transition(TGPState::Errored)?;
                }
                self.sessions.update_session(&session).await?;
            }

            // --------------------------------------------------
            // Error → Errored unless ephemeral or terminal
            // --------------------------------------------------
            TGPMessage::Error(_) => {
                if session.is_ephemeral() {
                    // Ephemeral errors are never persisted
                } else if !session.is_terminal() {
                    session.transition(TGPState::Errored)?;
                    self.sessions.update_session(&session).await?;
                }
            }
        }

        // ------------------------------------------------------
        // 7. Encode + logging
        // ------------------------------------------------------
        let response_json = encode_message(&response)
            .map_err(|e| anyhow!("encode error: {}", e))?;

        log_tx(&response_json);
        Ok(response_json)
    }
}