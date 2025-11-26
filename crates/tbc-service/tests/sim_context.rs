//! Simulation Context
//!
//! Provides a shared async test environment for CoreProver escrow demonstrations.
//! This context simulates EVM-like behavior with realistic delays and state management.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

// Mock types matching ethers-rs style
type H256 = [u8; 32];
type U256 = u128;
type Address = String;

/// Simulated user account with balance tracking
#[derive(Debug, Clone)]
pub struct SimUser {
    pub address: Address,
    pub balance: U256,
    pub nonce: u64,
}

impl SimUser {
    pub fn new(name: &str, initial_balance: U256) -> Self {
        Self {
            address: format!("0x{:040x}", name.as_bytes().iter().map(|&b| b as usize).sum::<usize>()),
            balance: initial_balance,
            nonce: 0,
        }
    }

    pub fn deduct(&mut self, amount: U256) -> Result<(), String> {
        if self.balance < amount {
            return Err("Insufficient balance".to_string());
        }
        self.balance -= amount;
        self.nonce += 1;
        Ok(())
    }

    pub fn credit(&mut self, amount: U256) {
        self.balance += amount;
    }
}

/// Escrow states matching the state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowState {
    Created,
    Accepted,
    Funded,
    Delivered,
    Settled,
    Disputed,
    Refunded,
}

/// Escrow record stored in the mock store
#[derive(Debug, Clone)]
pub struct EscrowRecord {
    pub id: H256,
    pub buyer: Address,
    pub seller: Address,
    pub amount: U256,
    pub state: EscrowState,
    pub created_at: u64,
    pub updated_at: u64,
    pub commitment_window: u64,
    pub claim_window: u64,
    pub allows_timed_release: bool,
    pub timed_release_delay: u64,
    pub counter_escrow_required: bool,
    pub counter_escrow_amount: U256,
}

/// In-memory escrow storage
#[derive(Debug, Clone)]
pub struct MockEscrowStore {
    escrows: Arc<RwLock<HashMap<H256, EscrowRecord>>>,
}

impl MockEscrowStore {
    pub fn new() -> Self {
        Self {
            escrows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_escrow(&self, record: EscrowRecord) -> Result<H256, String> {
        // Simulate network delay
        sleep(Duration::from_millis(10)).await;
        
        let mut store = self.escrows.write().unwrap();
        let id = record.id;
        store.insert(id, record);
        Ok(id)
    }

    pub async fn get_escrow(&self, id: &H256) -> Result<EscrowRecord, String> {
        sleep(Duration::from_millis(5)).await;
        
        let store = self.escrows.read().unwrap();
        store.get(id)
            .cloned()
            .ok_or_else(|| "Escrow not found".to_string())
    }

    pub async fn update_state(&self, id: &H256, new_state: EscrowState) -> Result<(), String> {
        sleep(Duration::from_millis(10)).await;
        
        let mut store = self.escrows.write().unwrap();
        let record = store.get_mut(id)
            .ok_or_else(|| "Escrow not found".to_string())?;
        
        record.state = new_state;
        record.updated_at = current_timestamp();
        Ok(())
    }

    pub async fn list_escrows(&self) -> Vec<EscrowRecord> {
        let store = self.escrows.read().unwrap();
        store.values().cloned().collect()
    }
}

/// Receipt structure for the vault
#[derive(Debug, Clone)]
pub struct Receipt {
    pub receipt_id: u64,
    pub escrow_id: H256,
    pub buyer: Address,
    pub seller: Address,
    pub amount: U256,
    pub timestamp: u64,
    pub proof_hash: H256,
    pub metadata: String,
}

/// In-memory receipt vault
#[derive(Debug, Clone)]
pub struct MockReceiptVault {
    receipts: Arc<RwLock<HashMap<u64, Receipt>>>,
    next_id: Arc<RwLock<u64>>,
}

impl MockReceiptVault {
    pub fn new() -> Self {
        Self {
            receipts: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn store_receipt(&self, receipt: Receipt) -> Result<u64, String> {
        sleep(Duration::from_millis(10)).await;
        
        let mut receipts = self.receipts.write().unwrap();
        let mut next_id = self.next_id.write().unwrap();
        
        let id = *next_id;
        *next_id += 1;
        
        let mut receipt = receipt;
        receipt.receipt_id = id;
        receipts.insert(id, receipt);
        
        Ok(id)
    }

    pub async fn get_receipt(&self, id: u64) -> Result<Receipt, String> {
        sleep(Duration::from_millis(5)).await;
        
        let receipts = self.receipts.read().unwrap();
        receipts.get(&id)
            .cloned()
            .ok_or_else(|| "Receipt not found".to_string())
    }

    pub async fn list_receipts(&self) -> Vec<Receipt> {
        let receipts = self.receipts.read().unwrap();
        receipts.values().cloned().collect()
    }
}

/// Mock ZK prover that generates deterministic proofs
#[derive(Debug, Clone)]
pub struct MockProver {
    proof_delay_ms: u64,
}

impl MockProver {
    pub fn new() -> Self {
        Self {
            proof_delay_ms: 50, // Simulate proof generation time
        }
    }

    pub async fn generate_proof(&self, data: &[u8]) -> Result<H256, String> {
        println!("  🔐 Generating ZK proof...");
        sleep(Duration::from_millis(self.proof_delay_ms)).await;
        
        // Generate deterministic hash from input
        let mut proof = [0u8; 32];
        for (i, byte) in data.iter().enumerate() {
            proof[i % 32] ^= byte;
        }
        
        println!("  ✅ Proof generated: 0x{}", hex::encode(&proof[..8]));
        Ok(proof)
    }

    pub async fn verify_proof(&self, proof: &H256, expected_data: &[u8]) -> Result<bool, String> {
        println!("  🔍 Verifying ZK proof...");
        sleep(Duration::from_millis(20)).await;
        
        let expected_proof = {
            let mut p = [0u8; 32];
            for (i, byte) in expected_data.iter().enumerate() {
                p[i % 32] ^= byte;
            }
            p
        };
        
        let valid = proof == &expected_proof;
        println!("  {} Proof verification", if valid { "✅" } else { "❌" });
        Ok(valid)
    }
}

/// High-level escrow processor wrapping the core logic
#[derive(Debug, Clone)]
pub struct EscrowProcessor {
    store: MockEscrowStore,
    vault: MockReceiptVault,
    prover: MockProver,
}

impl EscrowProcessor {
    pub fn new(store: MockEscrowStore, vault: MockReceiptVault, prover: MockProver) -> Self {
        Self { store, vault, prover }
    }

    pub async fn create_escrow(
        &self,
        buyer: &SimUser,
        seller: &SimUser,
        amount: U256,
        commitment_window: u64,
        claim_window: u64,
        allows_timed_release: bool,
        timed_release_delay: u64,
        counter_escrow_required: bool,
    ) -> Result<H256, String> {
        let escrow_id = generate_escrow_id(&buyer.address, &seller.address, amount);
        
        let record = EscrowRecord {
            id: escrow_id,
            buyer: buyer.address.clone(),
            seller: seller.address.clone(),
            amount,
            state: EscrowState::Created,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            commitment_window,
            claim_window,
            allows_timed_release,
            timed_release_delay,
            counter_escrow_required,
            counter_escrow_amount: if counter_escrow_required { amount } else { 0 },
        };

        self.store.create_escrow(record).await
    }

    pub async fn seller_accept(&self, escrow_id: &H256) -> Result<(), String> {
        let escrow = self.store.get_escrow(escrow_id).await?;
        
        if escrow.state != EscrowState::Created {
            return Err("Invalid state for acceptance".to_string());
        }

        self.store.update_state(escrow_id, EscrowState::Accepted).await
    }

    pub async fn buyer_fund(&self, escrow_id: &H256, buyer: &mut SimUser) -> Result<(), String> {
        let escrow = self.store.get_escrow(escrow_id).await?;
        
        if escrow.state != EscrowState::Accepted {
            return Err("Invalid state for funding".to_string());
        }

        buyer.deduct(escrow.amount)?;
        self.store.update_state(escrow_id, EscrowState::Funded).await
    }

    pub async fn mark_delivered(&self, escrow_id: &H256) -> Result<(), String> {
        let escrow = self.store.get_escrow(escrow_id).await?;
        
        if escrow.state != EscrowState::Funded {
            return Err("Invalid state for delivery".to_string());
        }

        self.store.update_state(escrow_id, EscrowState::Delivered).await
    }

    pub async fn settle(
        &self,
        escrow_id: &H256,
        seller: &mut SimUser,
    ) -> Result<u64, String> {
        let escrow = self.store.get_escrow(escrow_id).await?;
        
        if escrow.state != EscrowState::Delivered {
            return Err("Invalid state for settlement".to_string());
        }

        // Generate proof
        let proof_data = format!("{:?}{:?}{}", escrow.buyer, escrow.seller, escrow.amount);
        let proof_hash = self.prover.generate_proof(proof_data.as_bytes()).await?;

        // Credit seller
        seller.credit(escrow.amount);

        // Update state
        self.store.update_state(escrow_id, EscrowState::Settled).await?;

        // Store receipt
        let receipt = Receipt {
            receipt_id: 0, // Will be set by vault
            escrow_id: *escrow_id,
            buyer: escrow.buyer,
            seller: escrow.seller,
            amount: escrow.amount,
            timestamp: current_timestamp(),
            proof_hash,
            metadata: "Escrow settled successfully".to_string(),
        };

        self.vault.store_receipt(receipt).await
    }

    pub async fn get_escrow_state(&self, escrow_id: &H256) -> Result<EscrowState, String> {
        let escrow = self.store.get_escrow(escrow_id).await?;
        Ok(escrow.state)
    }
}

/// Main simulation context
pub struct SimContext {
    pub buyer: SimUser,
    pub seller: SimUser,
    pub escrow_store: MockEscrowStore,
    pub vault: MockReceiptVault,
    pub prover: MockProver,
    pub processor: EscrowProcessor,
}

impl SimContext {
    pub fn new() -> Self {
        let buyer = SimUser::new("alice", 1_000_000_000); // 1 billion wei
        let seller = SimUser::new("bob", 500_000_000);    // 500 million wei

        let escrow_store = MockEscrowStore::new();
        let vault = MockReceiptVault::new();
        let prover = MockProver::new();
        let processor = EscrowProcessor::new(
            escrow_store.clone(),
            vault.clone(),
            prover.clone(),
        );

        Self {
            buyer,
            seller,
            escrow_store,
            vault,
            prover,
            processor,
        }
    }

    pub fn reset(&mut self) {
        self.buyer = SimUser::new("alice", 1_000_000_000);
        self.seller = SimUser::new("bob", 500_000_000);
        self.escrow_store = MockEscrowStore::new();
        self.vault = MockReceiptVault::new();
        self.prover = MockProver::new();
        self.processor = EscrowProcessor::new(
            self.escrow_store.clone(),
            self.vault.clone(),
            self.prover.clone(),
        );
    }

    pub async fn print_status(&self) {
        println!("\n📊 Current Status:");
        println!("   💰 Buyer balance: {} wei", self.buyer.balance);
        println!("   💰 Seller balance: {} wei", self.seller.balance);
        
        let escrows = self.escrow_store.list_escrows().await;
        println!("   📦 Active escrows: {}", escrows.len());
        
        let receipts = self.vault.list_receipts().await;
        println!("   🧾 Total receipts: {}", receipts.len());
    }
}

// Helper functions
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn generate_escrow_id(buyer: &str, seller: &str, amount: U256) -> H256 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    buyer.hash(&mut hasher);
    seller.hash(&mut hasher);
    amount.hash(&mut hasher);
    
    let hash = hasher.finish();
    let mut id = [0u8; 32];
    id[..8].copy_from_slice(&hash.to_le_bytes());
    id
}

// Hex encoding helper
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
