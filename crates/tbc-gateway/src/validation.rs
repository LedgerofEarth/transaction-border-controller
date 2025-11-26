//! Gateway-Level Validation (TGP-00 v3.2 Compatible)
//!
//! These validators perform *gateway-local* checks that are not part of
//! the TGP core spec but are required before the TBC node forwards
//! settlement or commit messages to the Execution Engine.
//!
//! IMPORTANT:
//!     • No ZK validation here (belongs to CoreProve)
//!     • No nullifier logic
//!     • No session-key logic
//!     • No OFFER validation (removed from spec)
//!     • No signature or domain attestation
//!
//! This layer performs ONLY:
//!     • chain consistency checks
//!     • basic format checks for envelopes
//!     • timestamp sanity checks

use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static::lazy_static! {
    static ref HEX_66: Regex = Regex::new(r"^0x[0-9a-fA-F]{64}$").unwrap();
}

/// Validate a Layer-8 transaction hash (0x + 64 hex chars)
pub fn validate_l8_tx_hash(tx: &str) -> bool {
    HEX_66.is_match(tx)
}

/// Validate UNIX timestamp (must be within ±5 minutes)
pub fn validate_timestamp(ts: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // allow slight drift
    let max_drift = 300;
    ts > now - max_drift && ts < now + max_drift
}

/// Chain ID must match configured chain
pub fn validate_chain_id(chain_id: u64, expected: u64) -> bool {
    chain_id == expected
}