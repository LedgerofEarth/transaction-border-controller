//! High-level escrow client

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::types::receipt::Receipt;
use crate::types::escrow::{Escrow, EscrowState, EscrowMode};
use anyhow::Result;
use ethers::prelude::*;
use ethers::types::{Address, H256, U256};

/// Escrow client for interacting with CoreProverEscrow contract
pub struct EscrowClient {
provider: Arc<Provider<Http>>,
contract_address: Address,
buyer_address: Address,
// Storage for escrows
escrows: Arc<Mutex<HashMap<H256, Escrow>>>,
}

impl EscrowClient {
/// Primary constructor
pub fn new(
provider: Arc<Provider<Http>>,
contract_address: Address,
buyer_address: Address,
) -> Self {
Self {
provider,
contract_address,
buyer_address,
escrows: Arc::new(Mutex::new(HashMap::new())),
}
}

/// Convenience constructor from RPC URL
pub fn from_rpc(rpc_url: &str, contract_address: Address, buyer_address: Address) -> Result<Self> {
    let provider = Provider::<Http>::try_from(rpc_url)?;
    Ok(Self::new(Arc::new(provider), contract_address, buyer_address))
}

/// Simulate creating an escrow receipt (purchase mode)
pub fn create_purchase_receipt(&self, seller: Address, amount: u64) -> Receipt {
    let order_id = self.generate_order_id();
    let timestamp = chrono::Utc::now().timestamp() as u64;
    let policy_hash = H256::zero();
    
    // Create and store the escrow
    let escrow = Escrow {
        order_id,
        buyer: self.buyer_address,
        seller,
        buyer_amount: U256::from(amount),
        seller_amount: U256::zero(),
        created_at: timestamp,
        timestamp,
        policy_hash,
        state: EscrowState::BuyerCommitted,
        mode: EscrowMode::Purchase,
    };
    
    // Store escrow for later retrieval
    self.escrows.lock().unwrap().insert(order_id, escrow);
    
    // Return receipt
    Receipt {
        receipt_id: order_id,  // Use order_id as receipt_id
        order_id,
        buyer: self.buyer_address,
        seller,
        buyer_amount: amount,
        seller_amount: 0,
        timestamp,
        policy_hash,
    }
}

/// Simulate creating a swap receipt
pub fn create_swap_receipt(&self, seller: Address, buyer_amt: u64, seller_amt: u64) -> Receipt {
    let order_id = self.generate_order_id();
    let timestamp = chrono::Utc::now().timestamp() as u64;
    let policy_hash = H256::zero();
    
    // Create and store the escrow
    let escrow = Escrow {
        order_id,
        buyer: self.buyer_address,
        seller,
        buyer_amount: U256::from(buyer_amt),
        seller_amount: U256::from(seller_amt),
        created_at: timestamp,
        timestamp,
        policy_hash,
        state: EscrowState::BuyerCommitted,
        mode: EscrowMode::Swap,
    };
    
    // Store escrow for later retrieval
    self.escrows.lock().unwrap().insert(order_id, escrow);
    
    // Return receipt
    Receipt {
        receipt_id: order_id,  // Use order_id as receipt_id
        order_id,
        buyer: self.buyer_address,
        seller,
        buyer_amount: buyer_amt,
        seller_amount: seller_amt,
        timestamp,
        policy_hash,
    }
}

/// Create a new escrow (placeholder for contract interaction)
pub async fn create_escrow(
    &self,
    order_id: H256,
    seller: Address,
    amount: U256,
) -> Result<H256> {
    // Contract call placeholder
    Ok(H256::zero())
}

/// Simulate fetching escrow
pub async fn get_escrow(&self, order_id: H256) -> Result<Escrow> {
    // Retrieve from storage
    self.escrows
        .lock()
        .unwrap()
        .get(&order_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Escrow not found for order_id: {:?}", order_id))
}

/// Simulates basic verification
pub fn verify_receipt(&self, receipt: &Receipt) -> bool {
    receipt.buyer == self.buyer_address
}

/// Generate a unique order ID
fn generate_order_id(&self) -> H256 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    H256::from_low_u64_be((timestamp % u64::MAX as u128) as u64)
}

/// Debug logging
pub fn debug_log(&self) {
    println!(
        "Provider: {:?}, Contract: {:?}, Buyer: {:?}",
        self.provider, self.contract_address, self.buyer_address
    );
}

}