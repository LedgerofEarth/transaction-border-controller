//! Contract Types Module
//!
//! Rust types that mirror the CoreProve Solidity contracts:
//! - SettlementContractTemplate_v0_2_5_2
//! - MerchantContractFactory_v0_4_2  
//! - ReceiptVault_2025_26_v0_2_6
//!
//! These types are used for:
//! - Encoding calldata for contract interactions
//! - Decoding events and return values
//! - Type-safe contract binding in TBC

pub mod settlement;
pub mod receipt_vault;
pub mod factory;
pub mod types;

pub use settlement::*;
pub use receipt_vault::*;
pub use factory::*;
pub use types::*;

