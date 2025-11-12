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
}

impl Default for EscrowState {
    fn default() -> Self {
        Self::None
    }
}

/// Escrow structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub buyer: Address,
    pub seller: Address,
    pub buyer_amount: U256,
    pub seller_amount: U256,
    pub state: EscrowState,
    pub created_at: u64,
}

impl Default for Escrow {
    fn default() -> Self {
        Self {
            buyer: Address::zero(),
            seller: Address::zero(),
            buyer_amount: U256::zero(),
            seller_amount: U256::zero(),
            state: EscrowState::None,
            created_at: 0,
        }
    }
}