use anyhow::Result;
use axum::extract::ws::Message;
use tbc_core::codec_tx::{classify_message, encode_message};
use tbc_core::protocol::TGPMessage;

/// Dispatches a WS JSON string → TGP router → encoded output.
pub async fn route_ws_message(json: &str) -> Result<String> {
    // Classify inbound JSON into a TGPMessage.
    let (_meta, msg) = classify_message(json)?;

    // Echo semantics (same as HTTP path)
    let encoded = encode_message(&msg)?;

    Ok(encoded)
}