//! ZK Error Types
//!
//! Error codes per TGP-EXT-ZK-00 §7.

use serde::{Deserialize, Serialize};

/// ZK-specific error codes
///
/// Per TGP-EXT-ZK-00 error taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZkErrorCode {
    /// Proof verification failed (cryptographic failure)
    #[serde(rename = "ZK_INVALID_PROOF")]
    InvalidProof,
    
    /// Proof timestamp outside valid window
    #[serde(rename = "ZK_EXPIRED_PROOF")]
    ExpiredProof,
    
    /// Nullifier has already been used
    #[serde(rename = "ZK_REPLAY")]
    Replay,
    
    /// Session key does not match proof
    #[serde(rename = "ZK_PK_MISMATCH")]
    PkMismatch,
    
    /// Circuit version not supported
    #[serde(rename = "ZK_UNSUPPORTED_VERSION")]
    UnsupportedVersion,
    
    /// Proof type not recognized
    #[serde(rename = "ZK_UNKNOWN_TYPE")]
    UnknownType,
    
    /// Input validation failed
    #[serde(rename = "ZK_INVALID_INPUTS")]
    InvalidInputs,
    
    /// Device commitment mismatch
    #[serde(rename = "ZK_DEVICE_MISMATCH")]
    DeviceMismatch,
    
    /// Order ID mismatch between proof and session
    #[serde(rename = "ZK_ORDER_MISMATCH")]
    OrderMismatch,
    
    /// Chain ID mismatch
    #[serde(rename = "ZK_CHAIN_MISMATCH")]
    ChainMismatch,
    
    /// Internal verification error
    #[serde(rename = "ZK_INTERNAL_ERROR")]
    InternalError,
}

impl std::fmt::Display for ZkErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProof => write!(f, "ZK_INVALID_PROOF"),
            Self::ExpiredProof => write!(f, "ZK_EXPIRED_PROOF"),
            Self::Replay => write!(f, "ZK_REPLAY"),
            Self::PkMismatch => write!(f, "ZK_PK_MISMATCH"),
            Self::UnsupportedVersion => write!(f, "ZK_UNSUPPORTED_VERSION"),
            Self::UnknownType => write!(f, "ZK_UNKNOWN_TYPE"),
            Self::InvalidInputs => write!(f, "ZK_INVALID_INPUTS"),
            Self::DeviceMismatch => write!(f, "ZK_DEVICE_MISMATCH"),
            Self::OrderMismatch => write!(f, "ZK_ORDER_MISMATCH"),
            Self::ChainMismatch => write!(f, "ZK_CHAIN_MISMATCH"),
            Self::InternalError => write!(f, "ZK_INTERNAL_ERROR"),
        }
    }
}

impl ZkErrorCode {
    /// Get human-readable error message
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidProof => "Cryptographic proof verification failed",
            Self::ExpiredProof => "Proof timestamp is outside the valid window",
            Self::Replay => "This proof has already been used",
            Self::PkMismatch => "Session key does not match proof",
            Self::UnsupportedVersion => "This proof version is not supported",
            Self::UnknownType => "Unknown proof type",
            Self::InvalidInputs => "Proof inputs failed validation",
            Self::DeviceMismatch => "Device commitment does not match",
            Self::OrderMismatch => "Order ID in proof does not match session",
            Self::ChainMismatch => "Chain ID in proof does not match expected chain",
            Self::InternalError => "Internal ZK verification error",
        }
    }
    
    /// Is this error recoverable? (can retry with different proof)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ExpiredProof | Self::PkMismatch | Self::DeviceMismatch
        )
    }
    
    /// Should this error be logged as suspicious activity?
    pub fn is_suspicious(&self) -> bool {
        matches!(
            self,
            Self::Replay | Self::InvalidProof | Self::UnsupportedVersion
        )
    }
}

/// Full ZK error with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkError {
    /// Error code
    pub code: ZkErrorCode,
    
    /// Human-readable message
    pub message: String,
    
    /// Proof type that failed (if known)
    pub proof_type: Option<super::ZkProofType>,
    
    /// Session ID (if known)
    pub session_id: Option<String>,
    
    /// Additional context
    pub context: Option<String>,
}

impl ZkError {
    pub fn new(code: ZkErrorCode) -> Self {
        Self {
            code,
            message: code.message().to_string(),
            proof_type: None,
            session_id: None,
            context: None,
        }
    }
    
    pub fn with_message(code: ZkErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            proof_type: None,
            session_id: None,
            context: None,
        }
    }
    
    pub fn with_proof_type(mut self, pt: super::ZkProofType) -> Self {
        self.proof_type = Some(pt);
        self
    }
    
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
    
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }
}

impl std::fmt::Display for ZkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(pt) = &self.proof_type {
            write!(f, " (type: {})", pt)?;
        }
        if let Some(ctx) = &self.context {
            write!(f, " [{}]", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for ZkError {}

impl From<ZkErrorCode> for ZkError {
    fn from(code: ZkErrorCode) -> Self {
        ZkError::new(code)
    }
}

