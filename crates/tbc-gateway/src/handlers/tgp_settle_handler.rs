//! SETTLE Handler (Pure Function)
//!
//! Receives:  SETTLE
//! Returns:   SETTLE echo (or ERROR)
//!
//! Responsibilities:
//!   • verify semantic correctness of a SETTLE message
//!   • check tx-hash provided when required
//!   • no chain verification (delegated to MCP agent)
//!   • no state mutation (router performs transitions)
//!
//! SESSION TRANSITIONS:
//!   The Router (not this handler) will:
//!     • move OfferReceived → Finalizing
//!     • Finalizing → Settled/Errored
//!
//! This ensures purity + SIP-style separation.

use anyhow::Result;

use tbc_core::tgp::{
    protocol::{SettleMessage, TGPMessage, make_protocol_error},
    types::SettleSource,
    codec_tx::TGPMetadata,
};

use crate::logging::*;


pub async fn handle_inbound_settle(
    meta: &TGPMetadata,
    _session: &tbc_core::tgp::state::TGPSession,
    s: SettleMessage,
) -> Result<TGPMessage> {

    log_handler("SETTLE");

    // ----------------------------------------------------------
    // 1. Structural validation
    // ----------------------------------------------------------
    if let Err(e) = s.validate() {
        let err = make_protocol_error(
            Some(s.id.clone()),
            "INVALID_SETTLE",
            e,
        );
        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 2. tx-hash verification requirement
    // ----------------------------------------------------------
    if s.source.requires_verification() && s.layer8_tx.is_none() {
        let err = make_protocol_error(
            Some(s.id.clone()),
            "SETTLEMENT_UNVERIFIED",
            "SETTLE requires tx-hash for this reporter type",
        );
        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 3. No chain verification here (delegated to MCP agent)
    // ----------------------------------------------------------

    // ----------------------------------------------------------
    // 4. Echo SETTLE back to caller
    // ----------------------------------------------------------
    Ok(TGPMessage::Settle(s))
}