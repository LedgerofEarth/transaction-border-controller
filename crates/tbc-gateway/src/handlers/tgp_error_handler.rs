//! tgp_error_handlers.rs -- TGP-00 v3.2
//! Stateless ERROR passthrough handler
//! -----------------------------------
//! In TGP v3.2, inbound ERROR messages:
//!   • MUST NOT alter gateway state
//!   • MUST NOT create or modify sessions
//!   • MUST be logged
//!   • MUST be echoed back to caller
//! 
//! This module implements that behavior.

use anyhow::Result;

use tbc_core::{
    codec_tx::TGPMetadata,
    protocol::{ErrorMessage, TGPMessage},
};

// Logging utilities from crate
use crate::logging::{log_handler, log_err, log_info};


/// -----------------------------------------------------------------------
/// ERROR Handler -- Stateless Passthrough
/// -----------------------------------------------------------------------
pub async fn handle_inbound_error(
    meta: &TGPMetadata,
    err: ErrorMessage,
) -> Result<TGPMessage>
{
    log_handler("ERROR");

    // -------------------------------------------------------------------
    // 1. Validate ErrorMessage (defensive)
    // -------------------------------------------------------------------
    if let Err(v) = err.validate() {
        log_err(&err);
        log_info!(
            target: "tgp.error",
            {
                "id": err.id.clone(),
                "correlation": err.correlation_id.clone(),
                "validation_error": v,
            },
            "Inbound ERROR failed structural validation"
        );

        // Still echo back; TGP never suppresses an inbound ERROR.
        return Ok(TGPMessage::Error(err));
    }

    // -------------------------------------------------------------------
    // 2. Log inbound ERROR
    // -------------------------------------------------------------------
    log_err(&err);

    log_info!(
        target: "tgp.error",
        {
            "id": err.id.clone(),
            "code": err.code.clone(),
            "correlation": err.correlation_id.clone(),
            "source_msg_id": meta.msg_id.clone(),
        },
        "Inbound ERROR accepted"
    );

    // -------------------------------------------------------------------
    // NOTE: Under TGP v3.2, correlation IDs are purely informational.
    // Gateways DO NOT:
    //   • Lookup sessions
    //   • Mutate any local state
    //   • Imply transactional linkage
    //
    // Correlation is logged only for observability.
    // -------------------------------------------------------------------

    if let Some(cid) = &err.correlation_id {
        log_info!(
            target: "tgp.error",
            {
                "id": err.id.clone(),
                "correlation": cid,
            },
            "Correlation observed"
        );
    }

    // -------------------------------------------------------------------
    // 3. Stateless passthrough
    // -------------------------------------------------------------------
    Ok(TGPMessage::Error(err))
}