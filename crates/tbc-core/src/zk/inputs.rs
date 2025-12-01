//! ZK Proof Input Types
//!
//! Type-specific public inputs for each proof circuit.
//! Per TGP-EXT-ZK-00 §4.

use serde::{Deserialize, Serialize};

/// Buyer Deposit Proof inputs (ZKB01)
///
/// Proves: "I have deposited X to escrow E for session S"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkBuyerInputs {
    /// Escrow contract address (0x...)
    pub escrow_address: String,
    
    /// Deposit amount (wei as string)
    pub amount: String,
    
    /// Hash of buyer's ephemeral public key
    pub pk_hash: String,
    
    /// Single-use nullifier
    pub nullifier: String,
    
    /// Proof timestamp (unix seconds as string)
    pub timestamp: String,
    
    /// Session public key (compressed, 0x<33-byte>)
    pub session_pubkey: String,
    
    /// Deposit transaction hash
    pub deposit_tx_hash: String,
    
    /// Chain ID
    pub chain_id: u64,
}

impl ZkBuyerInputs {
    /// Convert to array of field elements for snarkjs
    pub fn to_public_signals(&self) -> Vec<String> {
        vec![
            self.pk_hash.clone(),
            self.nullifier.clone(),
            self.timestamp.clone(),
            self.amount.clone(),
        ]
    }
}

/// Seller Fulfillment Proof inputs (ZKS01)
///
/// Proves: "I am fulfilling order O as the authorized seller"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkSellerInputs {
    /// Order hash (keccak256 of order details)
    pub order_hash: String,
    
    /// Hash of seller's ephemeral public key
    pub pk_hash: String,
    
    /// Single-use nullifier
    pub nullifier: String,
    
    /// Proof timestamp (unix seconds as string)
    pub timestamp: String,
    
    /// Session public key (compressed)
    pub session_pubkey: String,
    
    /// Chain ID
    pub chain_id: u64,
}

impl ZkSellerInputs {
    /// Convert to array of field elements for snarkjs
    pub fn to_public_signals(&self) -> Vec<String> {
        vec![
            self.pk_hash.clone(),
            self.nullifier.clone(),
            self.timestamp.clone(),
            self.order_hash.clone(),
        ]
    }
}

/// Merchant Policy Proof inputs (ZKM01)
///
/// TBC-only: Proves merchant policy compliance without exposing policy details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkMerchantInputs {
    /// Policy contract address
    pub policy_address: String,
    
    /// Hash of policy parameters
    pub policy_hash: String,
    
    /// Hash of policy bytecode
    pub bytecode_hash: String,
    
    /// Proof timestamp
    pub timestamp: String,
    
    /// Single-use nullifier
    pub nullifier: String,
    
    /// Chain ID
    pub chain_id: u64,
}

impl ZkMerchantInputs {
    /// Convert to array of field elements for snarkjs
    pub fn to_public_signals(&self) -> Vec<String> {
        vec![
            self.policy_hash.clone(),
            self.nullifier.clone(),
            self.timestamp.clone(),
            self.bytecode_hash.clone(),
        ]
    }
}

/// Union type for typed ZK inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ZkInputs {
    Buyer(ZkBuyerInputs),
    Seller(ZkSellerInputs),
    Merchant(ZkMerchantInputs),
}

impl ZkInputs {
    /// Get nullifier from any input type
    pub fn nullifier(&self) -> &str {
        match self {
            ZkInputs::Buyer(b) => &b.nullifier,
            ZkInputs::Seller(s) => &s.nullifier,
            ZkInputs::Merchant(m) => &m.nullifier,
        }
    }
    
    /// Get timestamp from any input type
    pub fn timestamp(&self) -> &str {
        match self {
            ZkInputs::Buyer(b) => &b.timestamp,
            ZkInputs::Seller(s) => &s.timestamp,
            ZkInputs::Merchant(m) => &m.timestamp,
        }
    }
    
    /// Get pk_hash from any input type (merchant doesn't have one)
    pub fn pk_hash(&self) -> Option<&str> {
        match self {
            ZkInputs::Buyer(b) => Some(&b.pk_hash),
            ZkInputs::Seller(s) => Some(&s.pk_hash),
            ZkInputs::Merchant(_) => None,
        }
    }
}

// =============================================================================
// CIRCUIT PUBLIC INPUT INDICES
// =============================================================================

/// Public input indices for ZKB01 circuit
pub mod zkb01 {
    pub const PK_HASH: usize = 0;
    pub const NULLIFIER: usize = 1;
    pub const TIMESTAMP: usize = 2;
    pub const AMOUNT: usize = 3;
    pub const NUM_PUBLIC: usize = 4;
}

/// Public input indices for ZKS01 circuit
pub mod zks01 {
    pub const PK_HASH: usize = 0;
    pub const NULLIFIER: usize = 1;
    pub const TIMESTAMP: usize = 2;
    pub const ORDER_HASH: usize = 3;
    pub const NUM_PUBLIC: usize = 4;
}

/// Public input indices for ZKM01 circuit
pub mod zkm01 {
    pub const POLICY_HASH: usize = 0;
    pub const NULLIFIER: usize = 1;
    pub const TIMESTAMP: usize = 2;
    pub const BYTECODE_HASH: usize = 3;
    pub const NUM_PUBLIC: usize = 4;
}

