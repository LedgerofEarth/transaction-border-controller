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

/// Escrow mode enum — determines if it's a simple purchase or mutual swap
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
    pub order_id: [u8; 32],          // External system reference
    pub buyer: Address,
    pub seller: Address,
    pub buyer_amount: U256,
    pub seller_amount: U256,
    pub state: EscrowState,
    pub mode: EscrowMode,
    pub created_at: u64,
}

impl Default for Escrow {
    fn default() -> Self {
        Self {
            order_id: [0u8; 32],
            buyer: Address::zero(),
            seller: Address::zero(),
            buyer_amount: U256::zero(),
            seller_amount: U256::zero(),
            state: EscrowState::None,
            mode: EscrowMode::default(),
            created_at: 0,
        }
    }
}