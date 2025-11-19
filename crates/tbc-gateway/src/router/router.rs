//! TGP Message Router (Controller-Side)
//!
//! This module implements the inbound message router for all TGP-00 traffic.
//! It performs:
//!   • JSON parsing
//!   • message classification
//!   • structural validation
//!   • policy enforcement
//!   • state lookup
//!   • routing to correct handler
//!   • envelope construction
//!   • standardized error responses
//!
//! This is the controller brain of the Transaction Border Controller (TBC).

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use tbc_core::tgp::{
    protocol::{
        TGPMessage, TGPMetadata, TGPValidationResult, classify_message,
        validate_and_classify_message, encode_message, make_protocol_error
    },
    state::{TGPSession, SessionStore},
    validation::*,
};

use crate::handlers::{
    handle_inbound_query,
    handle_inbound_offer,
    handle_inbound_settle,
    handle_inbound_error,
};

use crate::logging::*;


/// Primary Router Interface
#[async_trait]
pub trait TGPInboundRouter {
    async fn route_inbound(&self, json: &str) -> Result<String>;
}


/// Default Router Implementation
pub struct InboundRouter<S: SessionStore + Send + Sync> {
    pub sessions: S,
}

impl<S: SessionStore + Send + Sync> InboundRouter<S> {
    pub fn new(sessions: S) -> Self {
        Self { sessions }
    }
}

#[async_trait]
impl<S: SessionStore + Send + Sync> TGPInboundRouter for InboundRouter<S> {
    /// Top-level router entry point
    async fn route_inbound(&self, json: &str) -> Result<String> {
        log_rx(json);

        // ------------------------------------------------------
        // 1. Parse + classify
        // ------------------------------------------------------
        let (metadata, message) = match classify_message(json) {
            Ok(v) => v,
            Err(e) => {
                let err = make_protocol_error(None, "INVALID_JSON", e);
                log_err(&err);
                return Ok(encode_message(&TGPMessage::Error(err))?);
            }
        };

        // ------------------------------------------------------
        // 2. Structural Validation (JSON-level)
        // ------------------------------------------------------
        let structural = validate_and_classify_message(&metadata, &message);
        if let TGPValidationResult::Reject(error_msg) = structural {
            log_err(&error_msg);
            return Ok(encode_message(&TGPMessage::Error(error_msg))?);
        }

        // ------------------------------------------------------
        // 3. Session Lookup (QUERY lazily creates)
        // ------------------------------------------------------
        let session = match &message {
            TGPMessage::Query(_) => {
                // Lazy session start -- allowed only for QUERY
                let sess = self.sessions.create_session(metadata.msg_id.clone()).await?;
                log_session_created(&sess);
                sess
            }

            TGPMessage::Offer(o) => {
                self.sessions.get_session(&o.query_id).await?
                    .ok_or_else(|| anyhow!("Unknown session: {}", o.query_id))?
            }

            TGPMessage::Settle(s) => {
                self.sessions.get_session(&s.query_or_offer_id).await?
                    .ok_or_else(|| anyhow!("Unknown session: {}", s.query_or_offer_id))?
            }

            TGPMessage::Error(e) => {
                if let Some(cid) = &e.correlation_id {
                    self.sessions.get_session(cid).await?
                } else {
                    None
                }
                .unwrap_or(TGPSession::ephemeral(&metadata.msg_id))
            }
        };

        // ------------------------------------------------------
        // 4. Dispatch by Message Type
        // ------------------------------------------------------
        let response = match message {
            TGPMessage::Query(q) => {
                handle_inbound_query(&metadata, &session, q).await?
            }

            TGPMessage::Offer(o) => {
                handle_inbound_offer(&metadata, &session, o).await?
            }

            TGPMessage::Settle(s) => {
                handle_inbound_settle(&metadata, &session, s).await?
            }

            TGPMessage::Error(e) => {
                handle_inbound_error(&metadata, &session, e).await?
            }
        };

        // ------------------------------------------------------
        // 5. Encode + return
        // ------------------------------------------------------
        log_tx(&response);
        Ok(encode_message(&response)?)
    }
}