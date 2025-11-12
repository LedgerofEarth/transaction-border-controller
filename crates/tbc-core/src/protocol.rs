//! Protocol definitions and state machines

use serde::{Deserialize, Serialize};

/// TBC protocol version
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    OrderCreate,
    OrderRoute,
    OrderUpdate,
    OrderComplete,
}

/// Protocol state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolState {
    Initialized,
    Routing,
    Processing,
    Completed,
    Failed,
}

/// Protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub version: String,
    pub message_type: MessageType,
    pub state: ProtocolState,
}

impl Default for Protocol {
    fn default() -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            message_type: MessageType::OrderCreate,
            state: ProtocolState::Initialized,
        }
    }
}