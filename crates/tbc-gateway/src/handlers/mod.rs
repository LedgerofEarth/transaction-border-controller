//! handlers/mod.rs -- TGP-00 v3.2 Handler Layer
//! --------------------------------------------------
//! Stateless, deterministic handlers invoked by inbound.rs
//!
//! Equivalent to SIP Transaction User layer (TU).
//!
//! QUERY  → run state machine → ACK/ERROR
//! ACK    → passthrough
//! SETTLE → passthrough
//! ERROR  → passthrough

use tbc_core::protocol::{
    QueryMessage,
    ErrorMessage,
    AckMessage,
    SettleMessage,
    TGPMessage,
};
use tbc_core::codec_tx::TGPMetadata;
use anyhow::Result;


// ------------------------------------------------------------
// QUERY → ACK / ERROR
// ------------------------------------------------------------
pub async fn handle_inbound_query(
    _meta: &TGPMetadata,
    q: QueryMessage,
) -> Result<TGPMessage> {
    // TODO: Implement full state machine when tgp::state is ready
    // For now, return a simple ACK
    let ack = AckMessage::offer_for(&q);
    Ok(TGPMessage::Ack(ack))
}


// ------------------------------------------------------------
// ACK passthrough
// ------------------------------------------------------------
pub async fn handle_inbound_ack(
    _meta: &TGPMetadata,
    a: AckMessage,
) -> Result<TGPMessage> {
    // Gateway passes through ACKs
    Ok(TGPMessage::Ack(a))
}


// ------------------------------------------------------------
// SETTLE passthrough
// ------------------------------------------------------------
pub async fn handle_inbound_settle(
    _meta: &TGPMetadata,
    s: SettleMessage,
) -> Result<TGPMessage> {
    // Gateway does NOT modify terminal settlement states
    Ok(TGPMessage::Settle(s))
}


// ------------------------------------------------------------
// ERROR passthrough
// ------------------------------------------------------------
pub async fn handle_inbound_error(
    _meta: &TGPMetadata,
    e: ErrorMessage,
) -> Result<TGPMessage> {
    Ok(TGPMessage::Error(e))
}