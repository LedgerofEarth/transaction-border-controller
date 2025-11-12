//! Receipt type used for off-chain escrow recordkeeping

use ethers::types::{Address, H256};
use serde::{Deserialize, Serialize};

/// Escrow receipt structure (off-chain)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: H256,       // Unique identifier (matches escrow receipt or off-chain ref)
    pub order_id: [u8; 32],     // Optional: used to link with legacy systems
    pub buyer: Address,
    pub seller: Address,
    pub buyer_amount: u64,      // Optional: used in swap mode
    pub seller_amount: u64,     // Optional: used in swap mode
    pub timestamp: u64,
    pub policy_hash: H256,      // Represents the agreed settlement or compliance policy
}

impl Default for Receipt {
    fn default() -> Self {
        Self {
            receipt_id: H256::zero(),
            order_id: [0u8; 32],
            buyer: Address::zero(),
            seller: Address::zero(),
            buyer_amount: 0,
            seller_amount: 0,
            timestamp: 0,
            policy_hash: H256::zero(),
        }
    }
}