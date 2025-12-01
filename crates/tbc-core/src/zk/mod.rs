//! Zero-Knowledge Proof Module
//!
//! Rust types for ZK proofs per TGP-EXT-ZK-00 specification.
//!
//! Proof Types:
//! - ZKB01: Buyer Deposit Proof
//! - ZKS01: Seller Fulfillment Proof
//! - ZKM01: Merchant Policy Proof (TBC-only)
//!
//! Flow: Extension → TBC → (ZK Verify) → Contract-Safe Rewrite

pub mod proofs;
pub mod inputs;
pub mod verifier;
pub mod errors;

pub use proofs::*;
pub use inputs::*;
pub use verifier::*;
pub use errors::*;

