//! Common Contract Types
//!
//! Base types used across all CoreProve contracts.

use serde::{Deserialize, Serialize};

/// 32-byte hash type (bytes32 in Solidity)
pub type Bytes32 = [u8; 32];

/// 20-byte address type (address in Solidity)
pub type Address = [u8; 20];

/// Ethereum-compatible U256 (for amounts, timestamps)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct U256(pub [u64; 4]);

impl U256 {
    pub const ZERO: U256 = U256([0, 0, 0, 0]);
    
    pub fn from_u64(val: u64) -> Self {
        U256([val, 0, 0, 0])
    }
    
    pub fn from_u128(val: u128) -> Self {
        U256([val as u64, (val >> 64) as u64, 0, 0])
    }
    
    /// Convert to big-endian bytes
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for (i, &limb) in self.0.iter().rev().enumerate() {
            let limb_bytes = limb.to_be_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb_bytes);
        }
        bytes
    }
    
    /// Create from big-endian bytes
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let start = (3 - i) * 8;
            limbs[i] = u64::from_be_bytes(bytes[start..start + 8].try_into().unwrap());
        }
        U256(limbs)
    }
}

impl From<u64> for U256 {
    fn from(val: u64) -> Self {
        U256::from_u64(val)
    }
}

impl From<u128> for U256 {
    fn from(val: u128) -> Self {
        U256::from_u128(val)
    }
}

/// Convert hex string to Bytes32
pub fn hex_to_bytes32(hex: &str) -> Result<Bytes32, &'static str> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != 64 {
        return Err("Hex string must be 64 characters for bytes32");
    }
    let bytes = hex::decode(hex).map_err(|_| "Invalid hex")?;
    bytes.try_into().map_err(|_| "Invalid length")
}

/// Convert hex string to Address
pub fn hex_to_address(hex: &str) -> Result<Address, &'static str> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != 40 {
        return Err("Hex string must be 40 characters for address");
    }
    let bytes = hex::decode(hex).map_err(|_| "Invalid hex")?;
    bytes.try_into().map_err(|_| "Invalid length")
}

/// Convert Bytes32 to hex string
pub fn bytes32_to_hex(bytes: &Bytes32) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Convert Address to hex string (checksummed)
pub fn address_to_hex(addr: &Address) -> String {
    format!("0x{}", hex::encode(addr))
}

/// Native ETH sentinel address (address(0) in Solidity)
pub const NATIVE_ETH: Address = [0u8; 20];

/// Check if address is native ETH
pub fn is_native_eth(addr: &Address) -> bool {
    *addr == NATIVE_ETH
}

