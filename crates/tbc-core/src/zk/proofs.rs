//! ZK Proof Envelope Types
//!
//! Per TGP-EXT-ZK-00 v0.1 specification.

use serde::{Deserialize, Serialize};

/// ZK Proof type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZkProofType {
    /// Buyer Deposit Proof
    #[serde(rename = "ZKB01")]
    ZKB01,
    
    /// Seller Fulfillment Proof
    #[serde(rename = "ZKS01")]
    ZKS01,
    
    /// Merchant Policy Proof (TBC-only)
    #[serde(rename = "ZKM01")]
    ZKM01,
}

impl std::fmt::Display for ZkProofType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZKB01 => write!(f, "ZKB01"),
            Self::ZKS01 => write!(f, "ZKS01"),
            Self::ZKM01 => write!(f, "ZKM01"),
        }
    }
}

/// Full ZK Proof Envelope
///
/// This is the raw message the extension sends through TGP-EXT.
/// Per TGP-EXT-ZK-00 §3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofEnvelope {
    /// Message type identifier
    #[serde(rename = "type")]
    pub message_type: String,
    
    /// Proof payload
    pub payload: ZkProofPayload,
}

impl ZkProofEnvelope {
    pub const MESSAGE_TYPE: &'static str = "TGP_ZK_PROOF";
    
    pub fn new(payload: ZkProofPayload) -> Self {
        Self {
            message_type: Self::MESSAGE_TYPE.to_string(),
            payload,
        }
    }
    
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.message_type != Self::MESSAGE_TYPE {
            return Err("Invalid message type");
        }
        self.payload.validate()
    }
}

/// ZK Proof Payload
///
/// Contains all proof data and public inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofPayload {
    /// Proof type (ZKB01, ZKS01, ZKM01)
    pub zk_type: ZkProofType,
    
    /// Raw SNARK proof (base64url encoded)
    /// NEVER goes on-chain — TBC verifies and discards
    pub zk_proof: String,
    
    /// Public inputs (type-specific)
    pub zk_inputs: serde_json::Value,
    
    /// Single-use nullifier (0x<32-byte hex>)
    pub zk_nullifier: String,
    
    /// Proof timestamp (unix seconds)
    pub zk_timestamp: u64,
    
    /// Compressed ephemeral session public key (0x<33-byte>)
    pub session_pubkey: String,
    
    /// Device commitment hash (anti-theft/anti-malware)
    pub device_commitment: String,
    
    /// Circuit version number
    pub proof_version: u32,
    
    /// Session identifier (0x<32-byte>)
    pub session_id: String,
    
    /// Order identifier (0x<32-byte>)
    pub order_id: String,
    
    /// Payment profile hash (0x<32-byte>)
    pub profile_hash: String,
    
    /// Target chain ID
    pub chain_id: u64,
}

impl ZkProofPayload {
    /// Validate payload structure
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.zk_proof.is_empty() {
            return Err("Missing zk_proof");
        }
        if self.zk_nullifier.is_empty() {
            return Err("Missing zk_nullifier");
        }
        if self.session_pubkey.is_empty() {
            return Err("Missing session_pubkey");
        }
        if self.session_id.is_empty() {
            return Err("Missing session_id");
        }
        if self.order_id.is_empty() {
            return Err("Missing order_id");
        }
        if self.chain_id == 0 {
            return Err("Invalid chain_id");
        }
        Ok(())
    }
    
    /// Check if proof timestamp is within TTL
    pub fn is_timestamp_valid(&self, current_time: u64, ttl_seconds: u64) -> bool {
        if self.zk_timestamp > current_time {
            return false; // Future timestamp
        }
        let age = current_time - self.zk_timestamp;
        age <= ttl_seconds
    }
    
    /// Decode base64url proof bytes
    pub fn decode_proof(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        URL_SAFE_NO_PAD.decode(&self.zk_proof)
    }
}

/// Contract-safe proof output
///
/// After TBC verification, the proof is rewritten into this form.
/// No proof bytes are forwarded. No witness. No private data.
/// Per TGP-EXT-ZK-00 §6.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSafeProof {
    /// Buyer outputs (for buyerCommit)
    pub buyer: Option<ContractBuyerOutput>,
    
    /// Seller outputs (for sellerCommit)
    pub seller: Option<ContractSellerOutput>,
}

/// Buyer outputs after TBC verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractBuyerOutput {
    /// Hash of buyer's ephemeral public key
    pub pk_hash: String,
    
    /// Used nullifier (burned after use)
    pub nullifier: String,
    
    /// Proof timestamp
    pub timestamp: String,
    
    /// Deposit amount
    pub amount: String,
}

/// Seller outputs after TBC verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSellerOutput {
    /// Hash of seller's ephemeral public key
    pub pk_hash: String,
    
    /// Used nullifier (burned after use)
    pub nullifier: String,
    
    /// Proof timestamp
    pub timestamp: String,
    
    /// Order hash
    pub order_hash: String,
}

// =============================================================================
// PROOF VERSION CONSTANTS
// =============================================================================

/// Current supported proof version
pub const CURRENT_PROOF_VERSION: u32 = 1;

/// Proof TTL in seconds (5 minutes)
pub const PROOF_TTL_SECONDS: u64 = 300;

/// Maximum timestamp drift allowed (30 seconds)
pub const MAX_TIMESTAMP_DRIFT_SECONDS: u64 = 30;

