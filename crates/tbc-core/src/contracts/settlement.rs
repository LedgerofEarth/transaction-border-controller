//! Settlement Contract Types
//!
//! Rust types mirroring SettlementContractTemplate_v0_2_5_2.sol
//!
//! The settlement contract handles dual-ZK escrow:
//! 1. Buyer commits funds with ZK proof
//! 2. Seller commits with ZK proof
//! 3. Settlement distributes funds and mints receipt

use serde::{Deserialize, Serialize};
use super::types::{Address, Bytes32, U256};

// =============================================================================
// STRUCTS (from Solidity)
// =============================================================================

/// Buyer's escrow commitment
///
/// Mirrors: struct BuyerCommit in Solidity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerCommit {
    /// ZK commitment of buyer identity (hash of ephemeral public key)
    pub pk_hash: Bytes32,
    
    /// Single-use nullifier to prevent replay
    pub nullifier: Bytes32,
    
    /// Buyer's wallet address (for refund only)
    pub buyer: Address,
    
    /// Escrowed amount
    pub amount: U256,
    
    /// Asset address (address(0) = native ETH)
    pub asset: Address,
    
    /// Proof timestamp (for TTL calculation)
    pub timestamp: u64,
    
    /// Whether this commit exists
    pub exists: bool,
}

impl Default for BuyerCommit {
    fn default() -> Self {
        Self {
            pk_hash: [0u8; 32],
            nullifier: [0u8; 32],
            buyer: [0u8; 20],
            amount: U256::ZERO,
            asset: [0u8; 20],
            timestamp: 0,
            exists: false,
        }
    }
}

/// Seller's fulfillment commitment
///
/// Mirrors: struct SellerCommit in Solidity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerCommit {
    /// ZK commitment of seller identity
    pub pk_hash: Bytes32,
    
    /// Single-use nullifier
    pub nullifier: Bytes32,
    
    /// Proof timestamp
    pub timestamp: u64,
    
    /// Whether this commit exists
    pub exists: bool,
}

impl Default for SellerCommit {
    fn default() -> Self {
        Self {
            pk_hash: [0u8; 32],
            nullifier: [0u8; 32],
            timestamp: 0,
            exists: false,
        }
    }
}

// =============================================================================
// FUNCTION CALL PARAMETERS
// =============================================================================

/// Parameters for buyerCommit() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerCommitParams {
    pub order_id: Bytes32,
    pub asset: Address,
    pub amount: U256,
    pub pk_hash: Bytes32,
    pub nullifier: Bytes32,
    pub timestamp: u64,
    pub zk_proof: Vec<u8>,
}

/// Parameters for sellerCommit() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerCommitParams {
    pub order_id: Bytes32,
    pub pk_hash: Bytes32,
    pub nullifier: Bytes32,
    pub timestamp: u64,
    pub zk_proof: Vec<u8>,
}

/// Parameters for settle() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleParams {
    pub order_id: Bytes32,
}

/// Parameters for buyerCancelExpiredCommit() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerCancelParams {
    pub order_id: Bytes32,
}

// =============================================================================
// EVENTS (from Solidity)
// =============================================================================

/// BuyerCommitted event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerCommittedEvent {
    pub order_id: Bytes32,
    pub nullifier: Bytes32,
    pub pk_hash: Bytes32,
    pub amount: U256,
    pub asset: Address,
    pub timestamp: u64,
}

/// SellerCommitted event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerCommittedEvent {
    pub order_id: Bytes32,
    pub nullifier: Bytes32,
    pub pk_hash: Bytes32,
    pub timestamp: u64,
}

/// SettlementCompleted event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementCompletedEvent {
    pub order_id: Bytes32,
    pub chain_id: u64,
    pub buyer_pk_hash: Bytes32,
    pub seller_pk_hash: Bytes32,
    pub amount: U256,
    pub asset: Address,
    pub timestamp: u64,
    pub receipt_id: U256,
}

/// BuyerRefunded event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerRefundedEvent {
    pub order_id: Bytes32,
    pub buyer_pk_hash: Bytes32,
    pub amount: U256,
    pub asset: Address,
    pub timestamp: u64,
}

/// MerchantActiveChanged event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantActiveChangedEvent {
    pub active: bool,
}

// =============================================================================
// ERRORS (from Solidity)
// =============================================================================

/// Settlement contract errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementError {
    NotMerchantAdmin,
    NotAuthorized,
    InvalidTimestamp,
    BuyerTTLExpired,
    SellerTTLExpired,
    DuplicateBuyerCommit,
    DuplicateSellerCommit,
    ZeroAmount,
    UnsupportedAsset,
    FeeOverflow,
    NoBuyerCommit,
    NotBuyer,
    SettlementFailed,
    ZKInvalid,
    MerchantInactive,
}

impl std::fmt::Display for SettlementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMerchantAdmin => write!(f, "NOT_MERCHANT_ADMIN"),
            Self::NotAuthorized => write!(f, "NOT_AUTHORIZED"),
            Self::InvalidTimestamp => write!(f, "INVALID_TIMESTAMP"),
            Self::BuyerTTLExpired => write!(f, "BUYER_TTL_EXPIRED"),
            Self::SellerTTLExpired => write!(f, "SELLER_TTL_EXPIRED"),
            Self::DuplicateBuyerCommit => write!(f, "DUPLICATE_BUYER_COMMIT"),
            Self::DuplicateSellerCommit => write!(f, "DUPLICATE_SELLER_COMMIT"),
            Self::ZeroAmount => write!(f, "ZERO_AMOUNT"),
            Self::UnsupportedAsset => write!(f, "UNSUPPORTED_ASSET"),
            Self::FeeOverflow => write!(f, "FEE_OVERFLOW"),
            Self::NoBuyerCommit => write!(f, "NO_BUYER_COMMIT"),
            Self::NotBuyer => write!(f, "NOT_BUYER"),
            Self::SettlementFailed => write!(f, "SETTLEMENT_FAILED"),
            Self::ZKInvalid => write!(f, "ZK_INVALID"),
            Self::MerchantInactive => write!(f, "MERCHANT_INACTIVE"),
        }
    }
}

// =============================================================================
// CONSTANTS (from Solidity)
// =============================================================================

/// Maximum fee in basis points (100%)
pub const MAX_FEE_BPS: u64 = 10_000;

/// Minimum TTL in seconds (5 minutes)
pub const MIN_TTL: u64 = 5 * 60;

/// Maximum TTL in seconds (30 days)
pub const MAX_TTL: u64 = 30 * 24 * 60 * 60;

/// Freshness window for ZK proofs (1 hour)
pub const FRESHNESS_WINDOW: u64 = 60 * 60;

// =============================================================================
// ZK PUBLIC SIGNALS
// =============================================================================

/// Public signals for buyer ZK proof verification
/// Format: [pkHash, nullifier, timestamp, amount]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerZKPublicSignals {
    pub pk_hash: U256,
    pub nullifier: U256,
    pub timestamp: U256,
    pub amount: U256,
}

impl BuyerZKPublicSignals {
    /// Convert to fixed-size array for contract call
    pub fn to_array(&self) -> [U256; 4] {
        [self.pk_hash, self.nullifier, self.timestamp, self.amount]
    }
}

/// Public signals for seller ZK proof verification
/// Format: [pkHash, nullifier, timestamp, orderHash]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerZKPublicSignals {
    pub pk_hash: U256,
    pub nullifier: U256,
    pub timestamp: U256,
    pub order_hash: U256,
}

impl SellerZKPublicSignals {
    /// Convert to fixed-size array for contract call
    pub fn to_array(&self) -> [U256; 4] {
        [self.pk_hash, self.nullifier, self.timestamp, self.order_hash]
    }
    
    /// Compute order hash as done in contract
    /// orderHash = keccak256(abi.encodePacked(orderId, amount, asset, pkHash))
    pub fn compute_order_hash(
        order_id: &Bytes32,
        amount: &U256,
        asset: &Address,
        pk_hash: &Bytes32,
    ) -> Bytes32 {
        use sha3::{Digest, Keccak256};
        
        let mut hasher = Keccak256::new();
        hasher.update(order_id);
        hasher.update(&amount.to_be_bytes());
        hasher.update(asset);
        hasher.update(pk_hash);
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

