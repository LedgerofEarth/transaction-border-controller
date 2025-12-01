//! Receipt Vault Types
//!
//! Rust types mirroring ReceiptVault_2025_26_v0_2_6.sol
//!
//! The ReceiptVault stores non-transferable NFT receipts for settled transactions.
//! Receipts can be verified via ZK proofs without revealing wallet addresses.

use serde::{Deserialize, Serialize};
use super::types::{Address, Bytes32, U256};

// =============================================================================
// STRUCTS (from Solidity)
// =============================================================================

/// Receipt data stored in vault
///
/// Mirrors: struct ReceiptData in Solidity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptData {
    /// Chain ID where receipt was minted
    pub chain_id: u64,
    
    /// Session identifier (unified with order_id)
    pub session_id: Bytes32,
    
    /// Order identifier
    pub order_id: Bytes32,
    
    /// ZK commitment of buyer identity
    pub buyer_pk_hash: Bytes32,
    
    /// ZK commitment of seller identity
    pub seller_pk_hash: Bytes32,
    
    /// Merchant settlement contract address
    pub merchant_contract: Address,
    
    /// Settlement timestamp
    pub timestamp: u64,
    
    /// Settlement amount
    pub amount: U256,
    
    /// Asset address (address(0) = native ETH)
    pub asset: Address,
}

impl Default for ReceiptData {
    fn default() -> Self {
        Self {
            chain_id: 0,
            session_id: [0u8; 32],
            order_id: [0u8; 32],
            buyer_pk_hash: [0u8; 32],
            seller_pk_hash: [0u8; 32],
            merchant_contract: [0u8; 20],
            timestamp: 0,
            amount: U256::ZERO,
            asset: [0u8; 20],
        }
    }
}

// =============================================================================
// FUNCTION CALL PARAMETERS
// =============================================================================

/// Parameters for mintReceipt() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintReceiptParams {
    pub session_id: Bytes32,
    pub order_id: Bytes32,
    pub buyer_pk_hash: Bytes32,
    pub seller_pk_hash: Bytes32,
    pub amount: U256,
    pub asset: Address,
    pub merchant_contract: Address,
}

/// Parameters for authorizeSettlementContract() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeSettlementParams {
    pub settlement: Address,
}

/// Parameters for revokeSettlementContract() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeSettlementParams {
    pub settlement: Address,
}

/// Parameters for verifyReceiptOwnership() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReceiptOwnershipParams {
    pub receipt_id: U256,
    pub proof: Vec<u8>,
    pub public_signals: Vec<U256>,
}

/// Parameters for proveReceiptExists() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveReceiptExistsParams {
    pub order_id: Bytes32,
    pub proof: Vec<u8>,
    pub public_signals: Vec<U256>,
}

// =============================================================================
// EVENTS (from Solidity)
// =============================================================================

/// ReceiptMinted event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMintedEvent {
    pub receipt_id: U256,
    pub order_id: Bytes32,
    pub session_id: Bytes32,
    pub merchant_contract: Address,
    pub chain_id: u64,
    pub amount: U256,
    pub asset: Address,
    pub timestamp: u64,
}

/// SettlementContractAuthorized event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementContractAuthorizedEvent {
    pub settlement: Address,
}

/// SettlementContractRevoked event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementContractRevokedEvent {
    pub settlement: Address,
}

// =============================================================================
// ERRORS (from Solidity)
// =============================================================================

/// Receipt vault errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptVaultError {
    NotFactory,
    UnauthorizedSettlementContract,
    InvalidReceiptId,
    ReceiptNotFound,
    NonTransferable,
}

impl std::fmt::Display for ReceiptVaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFactory => write!(f, "NOT_FACTORY"),
            Self::UnauthorizedSettlementContract => write!(f, "UNAUTHORIZED_SETTLEMENT_CONTRACT"),
            Self::InvalidReceiptId => write!(f, "INVALID_RECEIPT_ID"),
            Self::ReceiptNotFound => write!(f, "RECEIPT_NOT_FOUND"),
            Self::NonTransferable => write!(f, "NON_TRANSFERABLE"),
        }
    }
}

// =============================================================================
// RECEIPT METADATA (on-chain JSON)
// =============================================================================

/// On-chain receipt metadata (from tokenURI)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptMetadata {
    pub receipt_id: String,
    pub order_id: String,
    pub session_id: String,
    pub buyer_pk_hash: String,
    pub seller_pk_hash: String,
    pub chain_id: String,
    pub merchant_contract: String,
    pub timestamp: String,
    pub amount: String,
    pub asset: String,
    pub epoch: String,
}

impl From<&ReceiptData> for ReceiptMetadata {
    fn from(data: &ReceiptData) -> Self {
        use super::types::{bytes32_to_hex, address_to_hex};
        
        Self {
            receipt_id: "0".to_string(), // Set separately
            order_id: bytes32_to_hex(&data.order_id),
            session_id: bytes32_to_hex(&data.session_id),
            buyer_pk_hash: bytes32_to_hex(&data.buyer_pk_hash),
            seller_pk_hash: bytes32_to_hex(&data.seller_pk_hash),
            chain_id: data.chain_id.to_string(),
            merchant_contract: address_to_hex(&data.merchant_contract),
            timestamp: data.timestamp.to_string(),
            amount: format!("{:?}", data.amount), // TODO: proper U256 formatting
            asset: address_to_hex(&data.asset),
            epoch: "2025_26".to_string(),
        }
    }
}

// =============================================================================
// RECEIPT QUERY RESULTS
// =============================================================================

/// Result of getReceiptByOrderId query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptQueryResult {
    pub found: bool,
    pub receipt_id: Option<U256>,
    pub data: Option<ReceiptData>,
}

impl ReceiptQueryResult {
    pub fn not_found() -> Self {
        Self {
            found: false,
            receipt_id: None,
            data: None,
        }
    }
    
    pub fn found(receipt_id: U256, data: ReceiptData) -> Self {
        Self {
            found: true,
            receipt_id: Some(receipt_id),
            data: Some(data),
        }
    }
}

