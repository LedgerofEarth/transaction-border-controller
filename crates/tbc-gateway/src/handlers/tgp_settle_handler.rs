//! SETTLE Handler (Pure Function)
//!
//! Receives:  SETTLE
//! Returns:   SETTLE echo (or ERROR)
//!
//! Responsibilities:
//!   • verify semantic correctness of a SETTLE message
//!   • require tx-hash for unverified settlement sources
//!   • purity: no session mutation (router handles transitions)

use anyhow::Result;

use tbc_core::{
    protocol::{SettleMessage, TGPMessage, make_protocol_error},
    codec_tx::TGPMetadata,
};

use tbc_core::tgp::state::TGPSession;
use tbc_core::tgp::types::SettleSource;

// macros exported at crate root
use crate::log_info;

// functions that live inside logging.rs
use crate::logging::{log_handler, log_err};

/// Handle inbound SETTLE → returns SETTLE or ERROR
pub async fn handle_inbound_settle(
    _meta: &TGPMetadata,
    _session: &TGPSession,
    s: SettleMessage,
) -> Result<TGPMessage>
{
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
        log_info!(
            target: "tgp.settle",
            {
                "id": s.id.clone(),
                "source": s.source.to_string(),
                "reason": "missing-layer8-tx"
            },
            "Inbound SETTLE requires tx-hash for this source"
        );

        let err = make_protocol_error(
            Some(s.id.clone()),
            "SETTLEMENT_UNVERIFIED",
            "SETTLE requires tx-hash for this reporter type",
        );
        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 3. No on-chain verification here (delegated to MCP agent)
    // ----------------------------------------------------------

    // ----------------------------------------------------------
    // 4. Echo SETTLE back (SIP-style)
    // ----------------------------------------------------------
    log_info!(
        target: "tgp.settle",
        {
            "id": s.id.clone(),
            "source": s.source.to_string()
        },
        "Inbound SETTLE accepted and echoed"
    );

    Ok(TGPMessage::Settle(s))
}