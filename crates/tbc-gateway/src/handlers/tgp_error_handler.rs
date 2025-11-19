//! ERROR Handler (Pure Function with Rich Telemetry)
//! ------------------------------------------------------------
//! Receives:  ERROR
//! Returns:   ERROR (echo)
//!
//! Controller behavior for inbound ERRORs:
//!   • defensively validate ERROR structure
//!   • log error details, correlation chain, and session info
//!   • echo unchanged (TGP-00 forbids ERROR→ERROR synthesis)
//!
//! NOTE: This handler performs NO session mutation.

use anyhow::Result;

use tbc_core::tgp::{
    protocol::{ErrorMessage, TGPMessage},
    codec_tx::TGPMetadata,
    state::TGPSession,
};

use crate::logging::*;

/// Handle inbound ERROR message (echo + logging)
pub async fn handle_inbound_error(
    meta: &TGPMetadata,
    session: &TGPSession,
    e: ErrorMessage,
) -> Result<TGPMessage>
{
    log_handler("ERROR");

    // ------------------------------------------------------
    // 1. Defensive structural validation
    // ------------------------------------------------------
    if let Err(v) = e.validate() {
        log_error!(
            target: "tgp.error",
            {
                "id": e.id,
                "validation_error": v,
                "correlation": e.correlation_id
            },
            "Inbound ERROR failed structural validation"
        );

        // TGP-00: MUST still return the malformed ERROR unchanged.
        return Ok(TGPMessage::Error(e));
    }

    // ------------------------------------------------------
    // 2. Log primary ERROR info
    // ------------------------------------------------------
    log_err(&e);

    log_info!(
        target: "tgp.error",
        {
            "id": e.id,
            "code": e.code,
            "correlation": e.correlation_id
        },
        "Inbound ERROR recorded"
    );

    // ------------------------------------------------------
    // 3. Optional correlation chain logging
    // ------------------------------------------------------
    if let Some(cid) = &e.correlation_id {
        log_info!(
            target: "tgp.error",
            {
                "id": e.id,
                "correlation": cid,
                "session_id": session.session_id
            },
            "Correlation chain linked"
        );
    }

    // ------------------------------------------------------
    // 4. Echo ERROR back (SIP-style)
    // ------------------------------------------------------
    Ok(TGPMessage::Error(e))
}
