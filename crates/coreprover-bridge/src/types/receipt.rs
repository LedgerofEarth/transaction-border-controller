//! Receipt type used for off-chain escrow recordkeeping

use ethers::types::{Address, H256};
use serde::{Deserialize, Serialize};

/// Escrow receipt structure (off-chain)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
pub receipt_id: H256,
pub order_id: H256,         // Normalized to H256 for consistency
pub buyer: Address,
pub seller: Address,
pub buyer_amount: u64,
pub seller_amount: u64,
pub timestamp: u64,
pub policy_hash: H256,
}

impl Default for Receipt {
fn default() -> Self {
Self {
receipt_id: H256::zero(),
order_id: H256::zero(),
buyer: Address::zero(),
seller: Address::zero(),
buyer_amount: 0,
seller_amount: 0,
timestamp: 0,
policy_hash: H256::zero(),
}
}
}