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
//!     • Finalizing → Settled / Errored
//!
//! This ensures purity + SIP-style separation.

use anyhow::Result;

use tbc_core::codec_tx::{
    TGPMetadata,
    classify_message,
    encode_message,
    validate_and_classify_message,
    ReplayProtector,
    TGPValidationResult,
};

use tbc_core::tgp::types::SettleSource;
use tbc_core::tgp::state::TGPSession;

use crate::logging::{log_handler, log_info, log_err};


/// Handle inbound SETTLE → returns SETTLE or ERROR
pub async fn handle_inbound_settle(
    _meta: &TGPMetadata,
    _session: &TGPSession,
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
    // 2. If reporter requires verification → require tx-hash
    // ----------------------------------------------------------
    if s.source.requires_verification() && s.layer8_tx.is_none() {
        let err = make_protocol_error(
            Some(s.id.clone()),
            "SETTLEMENT_UNVERIFIED",
            "SETTLE requires tx-hash for this reporter type",
        );

        log_info!(
            {
                "id": s.id.clone(),
                "source": s.source.to_string(),
                "reason": "tx-hash missing"
            },
            "Inbound SETTLE missing required tx-hash"
        );

        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 3. CoreProver chain verification is delegated to MCP agent
    // ----------------------------------------------------------

    // ----------------------------------------------------------
    // 4. Echo SETTLE back (SIP-style)
    // ----------------------------------------------------------
    log_info!(
        {
            "id": s.id.clone(),
            "source": s.source.to_string()
        },
        "Inbound SETTLE accepted and echoed"
    );

    Ok(TGPMessage::Settle(s))
}