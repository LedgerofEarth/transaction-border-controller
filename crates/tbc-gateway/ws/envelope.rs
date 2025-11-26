use serde::{Serialize, Deserialize};
use tbc_core::tgp::messages::TGPMessage;

/// The WebSocket transport envelope used by TGP-00 v3.2
/// for raw message passing over full-duplex connections.
///
/// This is NOT TxIP. It is a minimal envelope:
///
/// {
///     "msg_id": "ws-123",
///     "tgp": { ... TGPMessage ... }
/// }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEnvelope {
    /// Client-generated UUID or monotonic ID.
    pub msg_id: String,

    /// The embedded TGP message.
    pub tgp: TGPMessage,
}

impl WsEnvelope {
    /// Validate the envelope + embedded TGP message.
    pub fn validate(&self) -> Result<(), String> {
        if self.msg_id.trim().is_empty() {
            return Err("msg_id must not be empty".into());
        }
        self.tgp.validate()
    }
}