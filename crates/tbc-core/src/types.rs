//! Core type definitions

use serde::{Deserialize, Serialize};

/// Order identifier
pub type OrderId = String;

/// Address type (EVM-compatible)
pub type Address = String;

/// Transaction hash
pub type TxHash = String;

/// Order structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub buyer: Address,
    pub seller: Address,
    pub amount: u128,
    pub created_at: u64,
}

/// Route information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub order_id: OrderId,
    pub seller_address: Address,
    pub agent_id: String,
}