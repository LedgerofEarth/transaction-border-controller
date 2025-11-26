//! tgp_ack_handlers.rs -- TGP-00 v3.2 ACK Handler
//! ------------------------------------------------
//! ACK replaces the legacy OFFER message type.
//!
//! ACK semantics in TGP-00 v3.2:
//!   • status = "offer" → preview only, MUST NOT include tx envelope
//!   • status = "allow" → MUST include full Economic Envelope
//!   • status = "deny"  → MUST NOT include tx envelope
//!   • status = "revise"→ MAY include hints, MUST NOT include tx envelope
//!
//! ACK is stateless and MUST be echoed back unless invalid.

use anyhow::Result;

use tbc_core::{
    protocol::{AckMessage, TGPMessage, make_protocol_error},
    codec_tx::TGPMetadata,
};

use crate::logging::{log_handler, log_info, log_err};


/// ---------------------------------------------------------------------------
/// ACK Handler -- Pure, Stateless, Deterministic
/// ---------------------------------------------------------------------------
pub async fn handle_inbound_ack(
    meta: &TGPMetadata,
    ack: AckMessage,
) -> Result<TGPMessage>
{
    log_handler("ACK");

    // =======================================================================
    // 1. STRUCTURAL VALIDATION  (basic schema rules)
    // =======================================================================
    if let Err(e) = ack.validate() {
        let err = make_protocol_error(
            Some(ack.id.clone()),
            "INVALID_ACK",
            format!("structural validation failed: {}", e),
        );
        log_err(&err);
        return Ok(TGPMessage::Error(err));
    }

    // =======================================================================
    // 2. SEMANTIC VALIDATION (status rules)
    // =======================================================================
    match ack.status.as_str() {

        // ---------------------------------------------------------------
        // OFFER  (preview)
        // ---------------------------------------------------------------
        "offer" => {
            if ack.tx.is_some() {
                let err = make_protocol_error(
                    Some(ack.id.clone()),
                    "INVALID_ACK_OFFER",
                    "status='offer' MUST NOT contain executable tx",
                );
                log_err(&err);
                return Ok(TGPMessage::Error(err));
            }
        }

        // ---------------------------------------------------------------
        // ALLOW (final executable envelope)
        // ---------------------------------------------------------------
        "allow" => {
            if ack.tx.is_none() {
                let err = make_protocol_error(
                    Some(ack.id.clone()),
                    "INVALID_ACK_ALLOW",
                    "status='allow' MUST include an EconomicEnvelope",
                );
                log_err(&err);
                return Ok(TGPMessage::Error(err));
            }

            // Deep validate envelope
            if let Some(env) = &ack.tx {
                if let Err(e) = env.validate() {
                    let err = make_protocol_error(
                        Some(ack.id.clone()),
                        "INVALID_ECON_ENVELOPE",
                        format!("EconomicEnvelope invalid: {}", e),
                    );
                    log_err(&err);
                    return Ok(TGPMessage::Error(err));
                }
            }
        }

        // ---------------------------------------------------------------
        // DENY
        // ---------------------------------------------------------------
        "deny" => {
            if ack.tx.is_some() {
                let err = make_protocol_error(
                    Some(ack.id.clone()),
                    "INVALID_ACK_DENY",
                    "status='deny' MUST NOT include EconomicEnvelope",
                );
                log_err(&err);
                return Ok(TGPMessage::Error(err));
            }
        }

        // ---------------------------------------------------------------
        // REVISE
        // ---------------------------------------------------------------
        "revise" => {
            if ack.tx.is_some() {
                let err = make_protocol_error(
                    Some(ack.id.clone()),
                    "INVALID_ACK_REVISE",
                    "status='revise' MUST NOT include EconomicEnvelope",
                );
                log_err(&err);
                return Ok(TGPMessage::Error(err));
            }
        }

        // ---------------------------------------------------------------
        // UNKNOWN
        // ---------------------------------------------------------------
        other => {
            let err = make_protocol_error(
                Some(ack.id.clone()),
                "INVALID_ACK_STATUS",
                format!("Unknown ACK.status '{}'", other),
            );
            log_err(&err);
            return Ok(TGPMessage::Error(err));
        }
    }

    // =======================================================================
    // 3. OBSERVABILITY (log everything)
    // =======================================================================
    log_info!(
        target: "tgp.ack",
        {
            "id": ack.id.clone(),
            "status": ack.status.clone(),
            "msg_id": meta.msg_id.clone(),
        },
        "Inbound ACK accepted"
    );

    // =======================================================================
    // 4. ECHO BACK (SIP-style)
    // =======================================================================
    Ok(TGPMessage::Ack(ack))
}