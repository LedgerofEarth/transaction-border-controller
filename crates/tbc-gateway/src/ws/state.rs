use serde::{Serialize, Deserialize};

/// Stateless WebSocket-layer state.
/// Contains only the TBC identifier -- no session tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsState {
    pub tbc_id: String,
}