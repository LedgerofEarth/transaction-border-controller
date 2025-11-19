//! OFFER Handler
//!
//! Receives:  OFFER
//! Returns:   ERROR (usually) or a policy echo
//!
//! Generally, Buyers should *not* send OFFERs. However, TGP-00 allows
//! merchant controllers and advanced flows to use OFFER messages for:
//!   • cross-domain mid-path negotiation,
//!   • merchant-provided settlement suggestions,
//!   • override signals.
//!
//! The default controller behavior is to validate and either accept
//! or reject the incoming OFFER.

use anyhow::Result;
use tbc_core::tgp::{
    protocol::{OfferMessage, TGPMessage, make_protocol_error},
    state::TGPSession,
    validation::*,
};
use crate::logging::*;

pub async fn handle_inbound_offer(
    meta: &crate::TGPMetadata,
    session: &TGPSession,
    o: OfferMessage,
) -> Result<TGPMessage> {

    log_handler("OFFER");

    // -----------------------------------------
    // Reject OFFERs from buyers unless policy allows
    // -----------------------------------------
    if !policy_allow_inbound_offer() {
        let err = make_protocol_error(
            Some(o.id.clone()),
            "POLICY_VIOLATION",
            "Inbound OFFER not allowed for this controller",
        );
        return Ok(TGPMessage::Error(err));
    }

    // -----------------------------------------
    // Validate OFFER
    // -----------------------------------------
    if let Err(e) = o.validate() {
        let err = make_protocol_error(
            Some(o.id.clone()),
            "INVALID_OFFER",
            e,
        );
        return Ok(TGPMessage::Error(err));
    }

    // -----------------------------------------
    // Default behavior: accept but do nothing
    // -----------------------------------------
    Ok(TGPMessage::Offer(o))
}

fn policy_allow_inbound_offer() -> bool {
    false
}