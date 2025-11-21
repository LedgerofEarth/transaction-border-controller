//! # Inbound Router (Controller-Side)
//! (TEST–MODE version)

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

#[async_trait]
pub trait TGPInboundRouter {
    async fn route_inbound(&self, raw_json: &str) -> Result<String>;
}

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

        // =====================================================================
        // 1. DECODE JSON
        // =====================================================================
        let (metadata, message) = match classify_message(raw_json) {
            Ok(v) => v,
            Err(e) => {
                let err = make_protocol_error(None, "INVALID_JSON", e);
                log_err(&err);
                return Ok(encode_message(&TGPMessage::Error(err))?);
            }
        };

        // =====================================================================
        // 2. REPLAY PROTECTION
        // =====================================================================
        if !self.replay.check_or_insert(&metadata.msg_id) {
            let err = make_protocol_error(
                metadata.correlation_id.clone(),
                "REPLAY_DETECTED",
                format!("Duplicate message ID: {}", metadata.msg_id),
            );
            log_err(&err);
            return Ok(encode_message(&TGPMessage::Error(err))?);
        }

        // =====================================================================
        // 3. STRUCTURAL VALIDATION
        // =====================================================================
        match validate_and_classify_message(&metadata, &message) {
            TGPValidationResult::Reject(err) => {
                log_err(&err);
                return Ok(encode_message(&TGPMessage::Error(err))?);
            }
            TGPValidationResult::Accept => {}
        }

        // =====================================================================
        // 4. SESSION LOOKUP
        // =====================================================================

        let mut session: TGPSession = match &message {
            // --------------------------------------------------------------
            // QUERY → Create new session
            // --------------------------------------------------------------
            TGPMessage::Query(_) => {
                let mut s = self.sessions.create_session(metadata.msg_id.clone()).await?;
                s.query_id = Some(metadata.msg_id.clone());
                self.sessions.update_session(&s).await?;
                log_session_created(&s);
                s
            }

            // --------------------------------------------------------------
            // OFFER → Must reference existing query_id
            // --------------------------------------------------------------
            TGPMessage::Offer(o) => {
                self.sessions
                    .get_session(&o.query_id)
                    .await?
                    .ok_or_else(|| anyhow!("Unknown session for OFFER: {}", o.query_id))?
            }

            // --------------------------------------------------------------
            // SETTLE → Must reference query or offer
            // TEST-MODE CHANGE: FAIL IF NOT FOUND
            // --------------------------------------------------------------
            TGPMessage::Settle(s) => {
                // ORIGINAL:
                // if let Some(sess) = self.sessions.get_session(&s.query_or_offer_id).await? {
                //     sess
                // } else {
                //     let all = self.sessions.list_sessions(4096).await?;
                //     let maybe = all.into_iter()
                //         .find(|x| x.offer_id.as_deref() == Some(&s.query_or_offer_id));
                //     maybe.ok_or_else(|| anyhow!("Unknown session for SETTLE: {}", s.query_or_offer_id))?
                // }

                // TEST-MODE:
                match self.sessions.get_session(&s.query_or_offer_id).await? {
                    Some(sess) => sess,
                    None => return Err(anyhow!("Unknown session for SETTLE: {}", s.query_or_offer_id)),
                }
            }

            // --------------------------------------------------------------
            // ERROR → ephemeral or referenced
            // --------------------------------------------------------------
            TGPMessage::Error(e) => {
                if let Some(cid) = &e.correlation_id {
                    self.sessions
                        .get_session(cid)
                        .await?
                        .unwrap_or_else(|| TGPSession::ephemeral(cid))
                } else {
                    TGPSession::ephemeral(&metadata.msg_id)
                }
            }
        };

        // =====================================================================
        // 5. HANDLER DISPATCH (PURE)
        // =====================================================================
        let response = match &message {
            TGPMessage::Query(q) => handle_inbound_query(&metadata, &session, q.clone()).await?,
            TGPMessage::Offer(o) => handle_inbound_offer(&metadata, &session, o.clone()).await?,
            TGPMessage::Settle(s) => handle_inbound_settle(&metadata, &session, s.clone()).await?,
            TGPMessage::Error(e) => handle_inbound_error(&metadata, &session, e.clone()).await?,
        };

        // =====================================================================
        // 6. STATE TRANSITIONS (TEST-MODE)
        // =====================================================================
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
                if !session.is_ephemeral() && !session.is_terminal() {
                    session.transition(TGPState::Errored)?;
                    self.sessions.update_session(&session).await?;
                }
            }
        }

        // =====================================================================
        // 7. ENCODE + LOG
        // =====================================================================
        let outbound = encode_message(&response)?;
        log_tx(&outbound);
        Ok(outbound)
    }
}