//! handlers/mod.rs -- TGP-00 v3.2 Handler Layer
//! --------------------------------------------------
//! Stateless, deterministic handlers invoked by inbound.rs
//!
//! Equivalent to SIP Transaction User layer (TU).
//!
//! QUERY  → run state machine → ACK/ERROR
//! SETTLE → passthrough
//! ERROR  → passthrough

use tbc_core::tgp::protocol::{
    QueryMessage,
    ErrorMessage,
    AckMessage,
    SettleMessage,
};
use tbc_core::tgp::state::{
    handle_query, 
    TGPStateResult,
};
use anyhow::Result;


// ------------------------------------------------------------
// QUERY → ACK / ERROR
// ------------------------------------------------------------
pub async fn handle_inbound_query(
    q: QueryMessage,
) -> Result<serde_json::Value> {

    match handle_query(q).await {
        TGPStateResult::Ack(ack) => Ok(serde_json::to_value(ack)?),
        TGPStateResult::Error(err) => Ok(serde_json::to_value(err)?),
        TGPStateResult::Settle(s) => Ok(serde_json::to_value(s)?), // edge case
    }
}


// ------------------------------------------------------------
// SETTLE passthrough
// ------------------------------------------------------------
pub async fn handle_inbound_settle(
    s: SettleMessage,
) -> Result<serde_json::Value> {

    // Gateway does NOT modify terminal settlement states
    Ok(serde_json::to_value(s)?)
}


// ------------------------------------------------------------
// ERROR passthrough
// ------------------------------------------------------------
pub async fn handle_inbound_error(
    e: ErrorMessage,
) -> Result<serde_json::Value> {

    Ok(serde_json::to_value(e)?)
}