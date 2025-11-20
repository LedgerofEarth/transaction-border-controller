//! # Inbound Router (Controller-Side)
//!
//! Transaction Layer controller for the Transaction Border Controller.
//!
//! Responsibilities:
//!   • transport parsing (codec_tx)
//!   • structural validation (TGP-00 §3)
//!   • replay protection
//!   • session creation/lookup (TGP-00 §4)
//!   • handler dispatch (pure functions)
//!   • state transitions (router-owned, handlers are pure)
//!   • unified error model
//!   • logging
//!

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

        // ============================================================
        // 1. Decode JSON → (metadata, TGPMessage)
        // ============================================================
        let (metadata, message) = match classify_message(raw_json) {
            Ok(v) => v,
            Err(e) => {
                let err = make_protocol_error(None, "INVALID_JSON", e);
                log_err(&err);
                return encode_message(&TGPMessage::Error(err))
                    .map_err(|e| anyhow!("encode error: {}", e));
            }
        };

        // ============================================================
        // 2. Replay protection
        // ============================================================
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

        // ============================================================
        // 3. Structural TGP validation (TGP-00 §3)
        // ============================================================
        match validate_and_classify_message(&metadata, &message) {
            TGPValidationResult::Reject(err) => {
                log_err(&err);
                let encoded = encode_message(&TGPMessage::Error(err))
    .map_err(|e| anyhow!("encode error: {}", e))?;
return Ok(encoded);
            }
            TGPValidationResult::Accept => { /* OK */ }
        }

        // ============================================================
        // 4. SESSION LOOKUP (TGP-00 §4)
        // ============================================================

        // ------------ Query → creates a session ------------
        let mut session = match &message {
            TGPMessage::Query(_) => {
                let mut s = self.sessions.create_session(metadata.msg_id.clone()).await?;

                // Bind the query_id to the session
                s.query_id = Some(metadata.msg_id.clone());

                self.sessions.update_session(&s).await?;
                log_session_created(&s);
                s
            }

            // ------------ Offer → must reference existing Query ------------
            TGPMessage::Offer(o) => {
                // Look up by query_id
                let mut s = self.sessions
                    .get_session(&o.query_id)
                    .await?
                    .ok_or_else(|| anyhow!("Unknown session for OFFER: {}", o.query_id))?;

                // Bind an offer_id to the session
                s.offer_id = Some(o.id.clone());
                self.sessions.update_session(&s).await?;

                s
            }

            // ------------ Settle → may reference offer_id or query_id ------------
            TGPMessage::Settle(s) => {
                // First try exact match
                if let Some(sess) = self.sessions.get_session(&s.query_or_offer_id).await? {
                    sess
                } else {
                    // Otherwise search for a session whose offer_id matches
                    let all = self.sessions.list_sessions(4096).await?;
                    let maybe = all.into_iter()
                        .find(|x| x.offer_id.as_deref() == Some(&s.query_or_offer_id));

                    maybe.ok_or_else(|| {
                        anyhow!("Unknown session for SETTLE: {}", s.query_or_offer_id)
                    })?
                }
            }

            // ------------ Error → ephemeral or referenced ------------
            TGPMessage::Error(e) => {
                if let Some(cid) = &e.correlation_id {
                    // Referenced session OR ephemeral
                    self.sessions
                        .get_session(cid)
                        .await?
                        .unwrap_or_else(|| TGPSession::ephemeral(cid))
                } else {
                    // Pure ephemeral
                    TGPSession::ephemeral(&metadata.msg_id)
                }
            }
        };

        // ============================================================
        // 5. HANDLER DISPATCH (pure functions)
        // ============================================================
        let response = match &message {
            TGPMessage::Query(q) => {
                let span = tgp_span(&session.session_id, "QUERY");
                let _e = span.enter();
                handle_inbound_query(&metadata, &session, q.clone()).await?
            }

            TGPMessage::Offer(o) => {
                let span = tgp_span(&session.session_id, "OFFER");
                let _e = span.enter();
                handle_inbound_offer(&metadata, &session, o.clone()).await?
            }

            TGPMessage::Settle(s) => {
                let span = tgp_span(&session.session_id, "SETTLE");
                let _e = span.enter();
                handle_inbound_settle(&metadata, &session, s.clone()).await?
            }

            TGPMessage::Error(e) => {
                let span = tgp_span(&session.session_id, "ERROR");
                let _e = span.enter();
                handle_inbound_error(&metadata, &session, e.clone()).await?
            }
        };

        // ============================================================
        // 6. STATE TRANSITIONS (router-owned)
        // ============================================================
        match &message {
            TGPMessage::Query(_) => {
                session.transition(TGPState::QuerySent)?;
                self.sessions.update_session(&session).await?;
            }

            TGPMessage::Offer(_) => {
                session.transition(TGPState::OfferReceived)?;
                self.sessions.update_session(&session).await?;
            }

            TGPMessage::Settle(s) => {
                if s.success {
                    session.transition(TGPState::Settled)?;
                } else {
                    session.transition(TGPState::Errored)?;
                }
                self.sessions.update_session(&session).await?;
            }

            TGPMessage::Error(_) => {
                // ephemeral errors are never persisted
                if session.is_ephemeral() {
                    // no-op
                } else if !session.is_terminal() {
                    session.transition(TGPState::Errored)?;
                    self.sessions.update_session(&session).await?;
                }
            }
        }

        // ============================================================
        // 7. ENCODE OUTBOUND + LOG
        // ============================================================
        let outbound = encode_message(&response)
    .map_err(|e| anyhow!("encode error: {}", e))?;
        log_tx(&outbound);
        Ok(outbound)
    }
}