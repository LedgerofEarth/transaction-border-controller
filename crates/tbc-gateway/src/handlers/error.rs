//! ERROR Handler
//!
//! Receives:  ERROR
//! Returns:   ERROR echo
//!
//! This ensures the error is logged and reflected back to the peer.

use anyhow::Result;
use tbc_core::tgp::{
    protocol::{ErrorMessage, TGPMessage},
    state::TGPSession,
};
use crate::logging::*;

pub async fn handle_inbound_error(
    meta: &crate::TGPMetadata,
    session: &TGPSession,
    e: ErrorMessage,
) -> Result<TGPMessage> {

    log_handler("ERROR");
    log_err(&e);

    // Controller doesn't alter inbound errors.
    Ok(TGPMessage::Error(e))
}