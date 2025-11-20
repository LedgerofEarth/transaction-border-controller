use anyhow::Result;

use tbc_core::{
    codec_tx::TGPMetadata,
    protocol::{ErrorMessage, TGPMessage},
    tgp::state::TGPSession,
};

// Macros exported at crate root
use crate::{log_error, log_info};

// Logging functions
use crate::logging::{log_handler, log_err};

/// Handle inbound ERROR message → echo + logging
pub async fn handle_inbound_error(
    _meta: &TGPMetadata,
    session: &TGPSession,
    e: ErrorMessage,
) -> Result<TGPMessage>
{
    log_handler("ERROR");

    // ------------------------------------------------------
    // 1. Structural validation (defensive)
    // ------------------------------------------------------
    if let Err(v) = e.validate() {
        log_error!(
            target: "tgp.error",
            {
                "id": e.id.clone(),
                "validation_error": v,
                "correlation": e.correlation_id
            },
            "Inbound ERROR failed structural validation"
        );

        return Ok(TGPMessage::Error(e));
    }

    // ------------------------------------------------------
    // 2. Log ERROR metadata
    // ------------------------------------------------------
    log_err(&e);

    log_info!(
        target: "tgp.error",
        {
            "id": e.id.clone(),
            "code": e.code.clone(),
            "correlation": e.correlation_id.clone()
        },
        "Inbound ERROR recorded"
    );

    // ------------------------------------------------------
    // 3. Optional correlation chain linking
    // ------------------------------------------------------
    if let Some(cid) = &e.correlation_id {
        log_info!(
            target: "tgp.error",
            {
                "id": e.id.clone(),
                "correlation": cid,
                "session_id": session.session_id.clone()
            },
            "Correlation chain linked"
        );
    }

    // ------------------------------------------------------
    // 4. Echo ERROR back
    // ------------------------------------------------------
    Ok(TGPMessage::Error(e))
}