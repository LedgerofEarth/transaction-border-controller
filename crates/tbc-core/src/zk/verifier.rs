//! ZK Verifier Interface
//!
//! Traits and types for ZK proof verification in TBC.
//! Per TGP-EXT-ZK-00 §5.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{ZkProofPayload, ZkProofType, ZkInputs};

/// Result of ZK proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the proof is valid
    pub valid: bool,
    
    /// Proof type that was verified
    pub proof_type: ZkProofType,
    
    /// Nullifier from the proof (for replay tracking)
    pub nullifier: String,
    
    /// Timestamp from the proof
    pub timestamp: u64,
    
    /// Error message if verification failed
    pub error: Option<String>,
}

impl VerificationResult {
    pub fn valid(proof_type: ZkProofType, nullifier: String, timestamp: u64) -> Self {
        Self {
            valid: true,
            proof_type,
            nullifier,
            timestamp,
            error: None,
        }
    }
    
    pub fn invalid(proof_type: ZkProofType, error: impl Into<String>) -> Self {
        Self {
            valid: false,
            proof_type,
            nullifier: String::new(),
            timestamp: 0,
            error: Some(error.into()),
        }
    }
}

/// ZK Verifier trait
///
/// Implement this for different verification backends:
/// - Groth16 (snarkjs-compatible)
/// - PLONK
/// - In-memory mock (for testing)
#[async_trait]
pub trait ZkVerifier: Send + Sync {
    /// Verify a ZK proof
    ///
    /// Returns verification result with nullifier for replay tracking.
    async fn verify(&self, payload: &ZkProofPayload) -> VerificationResult;
    
    /// Get supported proof types
    fn supported_types(&self) -> Vec<ZkProofType>;
    
    /// Check if a nullifier has been used
    async fn is_nullifier_used(&self, nullifier: &str) -> bool;
    
    /// Mark a nullifier as used
    async fn mark_nullifier_used(&self, nullifier: &str) -> Result<(), String>;
}

/// Nullifier storage trait
///
/// Separate trait for nullifier management (can be implemented by Redis, etc.)
#[async_trait]
pub trait NullifierStore: Send + Sync {
    /// Check if nullifier exists
    async fn exists(&self, nullifier: &str) -> bool;
    
    /// Insert nullifier (returns false if already exists)
    async fn insert(&self, nullifier: &str, timestamp: u64) -> bool;
    
    /// Get count of stored nullifiers
    async fn count(&self) -> usize;
}

/// In-memory nullifier store for testing
#[derive(Default)]
pub struct MemoryNullifierStore {
    nullifiers: std::sync::RwLock<std::collections::HashSet<String>>,
}

#[async_trait]
impl NullifierStore for MemoryNullifierStore {
    async fn exists(&self, nullifier: &str) -> bool {
        self.nullifiers.read().unwrap().contains(nullifier)
    }
    
    async fn insert(&self, nullifier: &str, _timestamp: u64) -> bool {
        self.nullifiers.write().unwrap().insert(nullifier.to_string())
    }
    
    async fn count(&self) -> usize {
        self.nullifiers.read().unwrap().len()
    }
}

/// Mock verifier for testing
///
/// Always returns valid unless proof contains "INVALID"
pub struct MockZkVerifier {
    nullifier_store: MemoryNullifierStore,
}

impl Default for MockZkVerifier {
    fn default() -> Self {
        Self {
            nullifier_store: MemoryNullifierStore::default(),
        }
    }
}

#[async_trait]
impl ZkVerifier for MockZkVerifier {
    async fn verify(&self, payload: &ZkProofPayload) -> VerificationResult {
        // Simulate proof verification
        if payload.zk_proof.contains("INVALID") {
            return VerificationResult::invalid(payload.zk_type, "Mock: Invalid proof marker");
        }
        
        // Check nullifier replay
        if self.is_nullifier_used(&payload.zk_nullifier).await {
            return VerificationResult::invalid(payload.zk_type, "Nullifier already used");
        }
        
        // Check timestamp freshness
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if payload.zk_timestamp > now + super::MAX_TIMESTAMP_DRIFT_SECONDS {
            return VerificationResult::invalid(payload.zk_type, "Future timestamp");
        }
        
        if payload.zk_timestamp < now.saturating_sub(super::PROOF_TTL_SECONDS) {
            return VerificationResult::invalid(payload.zk_type, "Proof expired");
        }
        
        VerificationResult::valid(
            payload.zk_type,
            payload.zk_nullifier.clone(),
            payload.zk_timestamp,
        )
    }
    
    fn supported_types(&self) -> Vec<ZkProofType> {
        vec![ZkProofType::ZKB01, ZkProofType::ZKS01, ZkProofType::ZKM01]
    }
    
    async fn is_nullifier_used(&self, nullifier: &str) -> bool {
        self.nullifier_store.exists(nullifier).await
    }
    
    async fn mark_nullifier_used(&self, nullifier: &str) -> Result<(), String> {
        if self.nullifier_store.insert(nullifier, 0).await {
            Ok(())
        } else {
            Err("Nullifier already exists".to_string())
        }
    }
}

/// Groth16 verification parameters (from snarkjs trusted setup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groth16VerificationKey {
    /// Protocol identifier
    pub protocol: String,
    
    /// Curve name (e.g., "bn128")
    pub curve: String,
    
    /// Number of public inputs
    pub n_public: usize,
    
    /// Verification key points (hex encoded)
    pub vk_alpha_1: Vec<String>,
    pub vk_beta_2: Vec<Vec<String>>,
    pub vk_gamma_2: Vec<Vec<String>>,
    pub vk_delta_2: Vec<Vec<String>>,
    
    /// IC points for public inputs
    #[serde(rename = "IC")]
    pub ic: Vec<Vec<String>>,
}

/// Groth16 proof structure (from snarkjs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groth16Proof {
    pub pi_a: Vec<String>,
    pub pi_b: Vec<Vec<String>>,
    pub pi_c: Vec<String>,
    pub protocol: String,
    pub curve: String,
}

