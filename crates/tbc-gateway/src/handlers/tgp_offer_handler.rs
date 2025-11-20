//! OFFER Handler (Pure Function)
//!
//! Receives:  OFFER
//! Returns:   OFFER echo or ERROR
//!
//! Buyers generally should NOT send OFFERs. Controllers may:
//!   • validate vendor-provided mid-path negotiation
//!   • return policy violation errors
//!
//! This handler is intentionally simple and stateless.

use anyhow::Result;

use tbc_core::{
    protocol::{OfferMessage, TGPMessage, make_protocol_error},
    codec_tx::TGPMetadata,
    tgp::state::TGPSession,
};

use crate::logging::*;

/// Handle inbound OFFER → returns OFFER or ERROR
pub async fn handle_inbound_offer(
    meta: &TGPMetadata,
    _session: &tbc_core::tgp::state::TGPSession,
    o: OfferMessage,
) -> Result<TGPMessage> {

    log_handler("OFFER");

    // ----------------------------------------------------------
    // 1. OFFERs from buyers are normally prohibited
    // ----------------------------------------------------------
    if !policy_allow_inbound_offer() {
        let err = make_protocol_error(
            Some(o.id.clone()),
            "POLICY_VIOLATION",
            "Inbound OFFER is not allowed for this controller",
        );
        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 2. Validate OFFER structure
    // ----------------------------------------------------------
    if let Err(e) = o.validate() {
        let err = make_protocol_error(
            Some(o.id.clone()),
            "INVALID_OFFER",
            format!("Invalid OFFER: {}", e),
        );
        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 3. Echo back -- controllers rarely transform OFFERs
    // ----------------------------------------------------------
    Ok(TGPMessage::Offer(o))
}

fn policy_allow_inbound_offer() -> bool {
    false // default: disallow buyer → controller OFFER messages
}