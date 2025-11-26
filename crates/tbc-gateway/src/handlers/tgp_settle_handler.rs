//! tgp_settle_handlers.rs -- TGP-00 v3.2 SETTLE Handler
//! ---------------------------------------------------
//! SETTLE messages represent a terminal lifecycle event emitted by a
//! settlement engine (gateway or contract observer).
//!
//! TGP v3.2 rules:
//!   • SETTLE MUST NOT create or modify session state (gateways are stateless)
//!   • SETTLE MUST be validated structurally
//!   • SETTLE MUST be echoed back as-is
//!   • Gateways MUST reject unverifiable settlement reports
//!   • Gateways MUST NOT include executable transactions
//!   • SETTLE is terminal -- no follow-up ACK allowed
//!
//! This handler is pure and deterministic.

use anyhow::Result;

use tbc_core::{
    codec_tx::TGPMetadata,
    protocol::{SettleMessage, TGPMessage, make_protocol_error},
    tgp::types::SettleSource,
};

// Logging
use crate::logging::{log_handler, log_info, log_err};


/// ---------------------------------------------------------------------------
/// SETTLE Handler -- Pure Echo with Validation
/// ---------------------------------------------------------------------------
pub async fn handle_inbound_settle(
    meta: &TGPMetadata,
    s: SettleMessage,
) -> Result<TGPMessage>
{
    log_handler("SETTLE");

    // =======================================================================
    // 1. STRUCTURAL VALIDATION (TGP-00 v3.2 §5.4)
    // =======================================================================
    if let Err(e) = s.validate() {
        let err = make_protocol_error(
            Some(s.id.clone()),
            "INVALID_SETTLE",
            format!("structural validation failed: {}", e),
        );

        log_err(&err);
        return Ok(TGPMessage::Error(err));
    }

    // =======================================================================
    // 2. VALIDATE REQUIRED FIELDS BASED ON REPORTER TYPE
    // =======================================================================
    //
    // Settlement sources:
    //   • LAYER8_EVENT       → MUST include layer8_tx
    //   • CONTRACT_LOG       → OPTIONAL
    //   • GATEWAY_EMITTED    → OPTIONAL
    //   • MANUAL_REPORT      → MUST include layer8_tx
    //
    // Deterministic: If required → reject if missing.
    //
    if s.source.requires_verification() && s.layer8_tx.is_none() {
        let err = make_protocol_error(
            Some(s.id.clone()),
            "SETTLEMENT_UNVERIFIED",
            "SETTLE requires layer8_tx for this source type",
        );

        log_err(&err);

        return Ok(TGPMessage::Error(err));
    }

    // =======================================================================
    // 3. OBSERVABILITY (Logging Only)
    // =======================================================================
    log_info!(
        target: "tgp.settle",
        {
            "id": s.id.clone(),
            "source": s.source.to_string(),
            "result": s.result.final_status.clone(),
            "msg_id": meta.msg_id.clone(),
        },
        "Inbound SETTLE accepted"
    );

    // =======================================================================
    // 4. SETTLE IS TERMINAL -- Echo response
    // =======================================================================
    //
    // Gateways MAY forward SETTLE to the client, but MUST NOT alter state.
    // No ACK is permitted. SETTLE is its own final response.
    //
    Ok(TGPMessage::Settle(s))
}