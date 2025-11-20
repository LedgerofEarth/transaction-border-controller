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

use tbc_core::codec_tx::{
    TGPMetadata,
    classify_message,
    encode_message,
    validate_and_classify_message,
    ReplayProtector,
    TGPValidationResult,
};

use tbc_core::tgp::state::TGPSession;

use crate::logging::{log_handler, log_err, log_info};

/// Handle inbound ERROR message (echo + logging)
pub async fn handle_inbound_error(
    _meta: &TGPMetadata,
    session: &TGPSession,
    e: ErrorMessage,
) -> Result<TGPMessage>
{
    log_handler("ERROR");

    // ------------------------------------------------------
    // 1. Defensive structural validation
    // ------------------------------------------------------
    if let Err(v) = e.validate() {
        log_info!(
            {
                "id": e.id.clone(),
                "validation_error": v,
                "correlation": e.correlation_id.clone()
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
        {
            "id": e.id.clone(),
            "code": e.code.clone(),
            "correlation": e.correlation_id.clone()
        },
        "Inbound ERROR recorded"
    );

    // ------------------------------------------------------
    // 3. Correlation chain logging
    // ------------------------------------------------------
    if let Some(cid) = &e.correlation_id {
        log_info!(
            {
                "id": e.id.clone(),
                "correlation": cid.clone(),
                "session_id": session.session_id.clone()
            },
            "Correlation chain linked"
        );
    }

    // ------------------------------------------------------
    // 4. Echo ERROR back (SIP-style)
    // ------------------------------------------------------
    Ok(TGPMessage::Error(e))
}