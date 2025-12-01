//! Merchant Contract Factory Types
//!
//! Rust types mirroring MerchantContractFactory_v0_4_2.sol
//!
//! The factory deploys merchant settlement contracts using CREATE2
//! with deterministic addresses and bytecode verification.

use serde::{Deserialize, Serialize};
use super::types::{Address, Bytes32, U256};

// =============================================================================
// ENUMS (from Solidity)
// =============================================================================

/// Template stability flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilityFlag {
    Experimental = 0,
    Stable = 1,
    Deprecated = 2,
}

impl From<u8> for StabilityFlag {
    fn from(val: u8) -> Self {
        match val {
            0 => StabilityFlag::Experimental,
            1 => StabilityFlag::Stable,
            2 => StabilityFlag::Deprecated,
            _ => StabilityFlag::Experimental,
        }
    }
}

// =============================================================================
// STRUCTS (from Solidity)
// =============================================================================

/// Registered template information
///
/// Mirrors: struct TemplateInfo in Solidity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template contract address
    pub template_address: Address,
    
    /// keccak256(runtime bytecode)
    pub code_hash: Bytes32,
    
    /// Stability status
    pub stability: StabilityFlag,
    
    /// Whether template exists
    pub exists: bool,
}

impl Default for TemplateInfo {
    fn default() -> Self {
        Self {
            template_address: [0u8; 20],
            code_hash: [0u8; 32],
            stability: StabilityFlag::Experimental,
            exists: false,
        }
    }
}

// =============================================================================
// FUNCTION CALL PARAMETERS
// =============================================================================

/// Parameters for registerTemplate() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterTemplateParams {
    pub version: u64,
    pub template_address: Address,
    pub stability: StabilityFlag,
}

/// Parameters for setTemplateStability() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTemplateStabilityParams {
    pub version: u64,
    pub new_stability: StabilityFlag,
}

/// Parameters for deployMerchant() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployMerchantParams {
    /// Template version to use
    pub version: u64,
    
    /// Merchant admin address
    pub merchant_admin: Address,
    
    /// TBC relay address (can call settle)
    pub tbc_relay_address: Address,
    
    /// ZK verifier contract address
    pub zk_verifier: Address,
    
    /// TBC fee recipient address
    pub tbc_fee_recipient: Address,
    
    /// ZK relay fee recipient address
    pub zk_fee_recipient: Address,
    
    /// Merchant fee recipient address
    pub merchant_fee_recipient: Address,
    
    /// TBC fee in basis points
    pub tbc_fee_bps: u64,
    
    /// ZK fee in basis points
    pub zk_fee_bps: u64,
    
    /// TTL in seconds
    pub ttl_seconds: u64,
    
    /// Initially supported assets
    pub initial_supported_assets: Vec<Address>,
    
    /// Deployment salt for CREATE2
    pub salt: Bytes32,
}

impl DeployMerchantParams {
    /// Validate parameters before deployment
    pub fn validate(&self) -> Result<(), FactoryError> {
        if self.merchant_admin == [0u8; 20] {
            return Err(FactoryError::InvalidAdmin);
        }
        if self.tbc_relay_address == [0u8; 20] {
            return Err(FactoryError::InvalidTbcRelay);
        }
        if self.zk_verifier == [0u8; 20] {
            return Err(FactoryError::InvalidVerifier);
        }
        if self.tbc_fee_bps + self.zk_fee_bps > 10_000 {
            return Err(FactoryError::InvalidFeeBps);
        }
        if self.ttl_seconds < 5 * 60 {
            return Err(FactoryError::TtlTooShort);
        }
        if self.ttl_seconds > 30 * 24 * 60 * 60 {
            return Err(FactoryError::TtlTooLong);
        }
        Ok(())
    }
    
    /// Compute deterministic salt as done in contract
    /// finalSalt = keccak256(abi.encodePacked(merchantAdmin, version, salt))
    pub fn compute_final_salt(&self) -> Bytes32 {
        use sha3::{Digest, Keccak256};
        
        let mut hasher = Keccak256::new();
        hasher.update(&self.merchant_admin);
        hasher.update(&self.version.to_be_bytes());
        hasher.update(&self.salt);
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// Parameters for revokeMerchant() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeMerchantParams {
    pub merchant_contract: Address,
}

// =============================================================================
// EVENTS (from Solidity)
// =============================================================================

/// TemplateRegistered event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRegisteredEvent {
    pub version: u64,
    pub template: Address,
    pub code_hash: Bytes32,
    pub stability: StabilityFlag,
}

/// TemplateStabilityUpdated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStabilityUpdatedEvent {
    pub version: u64,
    pub new_stability: StabilityFlag,
}

/// MerchantDeployed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantDeployedEvent {
    pub version: u64,
    pub merchant_contract: Address,
    pub merchant_admin: Address,
}

/// MerchantRevoked event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantRevokedEvent {
    pub merchant_contract: Address,
}

/// MerchantCodeHashMismatch event (indicates bytecode tampering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantCodeHashMismatchEvent {
    pub deployed_contract: Address,
    pub expected_hash: Bytes32,
    pub actual_hash: Bytes32,
}

// =============================================================================
// ERRORS (from Solidity)
// =============================================================================

/// Factory errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactoryError {
    NotOwner,
    InvalidTemplate,
    VersionExists,
    NoTemplate,
    InvalidVersion,
    TemplateDeprecated,
    InvalidAdmin,
    InvalidTbcRelay,
    InvalidVerifier,
    InvalidTbcFeeRecipient,
    InvalidZkFeeRecipient,
    InvalidMerchantFeeRecipient,
    InvalidFeeBps,
    TtlTooShort,
    TtlTooLong,
    Create2Failed,
    CodeHashMismatch,
    InvalidMerchant,
    InvalidVault,
}

impl std::fmt::Display for FactoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOwner => write!(f, "NOT_OWNER"),
            Self::InvalidTemplate => write!(f, "INVALID_TEMPLATE"),
            Self::VersionExists => write!(f, "VERSION_EXISTS"),
            Self::NoTemplate => write!(f, "NO_TEMPLATE"),
            Self::InvalidVersion => write!(f, "INVALID_VERSION"),
            Self::TemplateDeprecated => write!(f, "TEMPLATE_DEPRECATED"),
            Self::InvalidAdmin => write!(f, "INVALID_ADMIN"),
            Self::InvalidTbcRelay => write!(f, "INVALID_TBC_RELAY"),
            Self::InvalidVerifier => write!(f, "INVALID_VERIFIER"),
            Self::InvalidTbcFeeRecipient => write!(f, "INVALID_TBC_FEE_RECIPIENT"),
            Self::InvalidZkFeeRecipient => write!(f, "INVALID_ZK_FEE_RECIPIENT"),
            Self::InvalidMerchantFeeRecipient => write!(f, "INVALID_MERCHANT_FEE_RECIPIENT"),
            Self::InvalidFeeBps => write!(f, "INVALID_FEE_BPS"),
            Self::TtlTooShort => write!(f, "TTL_TOO_SHORT"),
            Self::TtlTooLong => write!(f, "TTL_TOO_LONG"),
            Self::Create2Failed => write!(f, "CREATE2_FAILED"),
            Self::CodeHashMismatch => write!(f, "CODE_HASH_MISMATCH"),
            Self::InvalidMerchant => write!(f, "INVALID_MERCHANT"),
            Self::InvalidVault => write!(f, "INVALID_VAULT"),
        }
    }
}

// =============================================================================
// CREATE2 ADDRESS PREDICTION
// =============================================================================

/// Predict CREATE2 deployment address
pub fn predict_create2_address(
    factory: &Address,
    salt: &Bytes32,
    init_code_hash: &Bytes32,
) -> Address {
    use sha3::{Digest, Keccak256};
    
    let mut hasher = Keccak256::new();
    hasher.update(&[0xff]);
    hasher.update(factory);
    hasher.update(salt);
    hasher.update(init_code_hash);
    
    let result = hasher.finalize();
    let mut address = [0u8; 20];
    address.copy_from_slice(&result[12..32]);
    address
}

