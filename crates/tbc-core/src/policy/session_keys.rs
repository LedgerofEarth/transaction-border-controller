//! Represents a session key as registered by CoreProve.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    pub public_key: String,
    pub key_id: String,
}