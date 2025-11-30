use serde::{Serialize, Deserialize};
use tbc_core::protocol::TGPMessage;

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
    /// Per TGP-TBC-SEC-00: fail-closed on any validation error.
    pub fn validate(&self) -> Result<(), String> {
        if self.msg_id.trim().is_empty() {
            return Err("msg_id must not be empty".into());
        }
        
        // Validate inner message based on type
        match &self.tgp {
            TGPMessage::Query(q) => q.validate(),
            TGPMessage::Ack(a) => a.validate(),
            TGPMessage::Settle(s) => s.validate(),
            TGPMessage::Error(e) => e.validate(),
        }
    }
}