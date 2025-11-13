//! Escrow type definitions

use ethers::prelude::*;
use serde::{Deserialize, Serialize};

/// Escrow state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowState {
None,
BuyerCommitted,
SellerCommitted,
BothCommitted,
SellerClaimed,
BuyerClaimed,
BothClaimed,
Disputed,
Expired,
Settled,
Cancelled,
}

impl Default for EscrowState {
fn default() -> Self {
Self::None
}
}

/// Escrow mode enum — determines if it’s a simple purchase or mutual swap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowMode {
Purchase, // One-sided: buyer → seller
Swap,     // Two-sided: buyer ↔ seller
}

impl Default for EscrowMode {
fn default() -> Self {
Self::Purchase
}
}

/// Escrow structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
pub order_id: H256,
pub buyer: Address,
pub seller: Address,
pub buyer_amount: U256,
pub seller_amount: U256,
pub created_at: u64,
pub timestamp: u64,
pub policy_hash: H256,
pub state: EscrowState,
pub mode: EscrowMode,
}

impl Default for Escrow {
fn default() -> Self {
Self {
order_id: H256::zero(),
buyer: Address::zero(),
seller: Address::zero(),
buyer_amount: U256::zero(),
seller_amount: U256::zero(),
created_at: 0,
timestamp: 0,
policy_hash: H256::zero(),
state: EscrowState::None,
mode: EscrowMode::default(),
}
}
}