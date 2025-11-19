//! SETTLE Handler
//!
//! Receives:  SETTLE
//! Returns:   SETTLE echo (or ERROR)
//!
//! Controller responsibilities:
//!   • trust-level evaluation
//!   • check settlement validity
//!   • session closure or continuation
//!   • verify tx-hash / watcher data
//!   • apply "ZK required" policy if applicable

use anyhow::Result;
use tbc_core::tgp::{
    protocol::{SettleMessage, TGPMessage, make_protocol_error},
    state::TGPSession,
    validation::*,
    types::SettleSource,
};
use crate::logging::*;

pub async fn handle_inbound_settle(
    meta: &crate::TGPMetadata,
    session: &TGPSession,
    s: SettleMessage,
) -> Result<TGPMessage> {

    log_handler("SETTLE");

    // ------------------------------------------
    // Validate structural rules
    // ------------------------------------------
    if let Err(e) = s.validate() {
        let err = make_protocol_error(
            Some(s.id.clone()),
            "INVALID_SETTLE",
            e,
        );
        return Ok(TGPMessage::Error(err));
    }

    // ------------------------------------------
    // Verify trust requirements
    // ------------------------------------------
    if s.source.requires_verification() && s.layer8_tx.is_none() {
        let err = make_protocol_error(
            Some(s.id.clone()),
            "SETTLEMENT_UNVERIFIED",
            "Non-watcher SETTLE must include tx hash",
        );
        return Ok(TGPMessage::Error(err));
    }

    // ------------------------------------------
    // Here you would verify tx-hash with watcher/chain
    // ------------------------------------------
    // placeholder: accept

    // ------------------------------------------
    // Return SETTLE echo (controller ack)
    // ------------------------------------------
    Ok(TGPMessage::Settle(s))
}