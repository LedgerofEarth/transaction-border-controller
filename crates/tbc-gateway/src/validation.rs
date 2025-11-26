//! Gateway-Level Validation (TGP-00 v3.2)
//!
//! These validators perform *gateway-local* checks permitted under the
//! updated Transaction Gateway Protocol. The TBC gateway MUST remain
//! stateless and MUST NOT perform:
//!     • ZK proof validation
//!     • Session-key or policy enforcement
//!     • Merchant attestation
//!     • CoreProve or MCP logic
//!     • OFFER validation (removed from protocol)
//!
//! Allowed validation scope (TGP-00 v3.2):
//!     • Layer-8 transaction hash format checking
//!     • Soft timestamp sanity checking
//!     • Chain-ID consistency
//!
//! All economic, ZK, and policy logic occurs in the Execution Engine,
//! CoreProve extension, or merchant policy engine -- never inside the TBC.

use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static::lazy_static! {
    /// Matches 0x-prefixed 32-byte hex (Layer-8 tx hash)
    static ref HEX_32_BYTES: Regex = Regex::new(r"^0x[0-9a-fA-F]{64}$").unwrap();

    /// Matches 0x-prefixed 16-byte hex (optional for tracing)
    static ref HEX_16_BYTES: Regex = Regex::new(r"^0x[0-9a-fA-F]{32}$").unwrap();
}

/// Validate a Layer-8 transaction hash (0x + 64 hex chars)
pub fn validate_l8_tx_hash(tx: &str) -> bool {
    HEX_32_BYTES.is_match(tx)
}

/// Optional helper for future envelope correlation
pub fn validate_hex_16(value: &str) -> bool {
    HEX_16_BYTES.is_match(value)
}

/// Validate UNIX timestamp (±5 minute drift allowed)
///
/// Gateways do NOT enforce strict replay prevention.
/// This is simply sanity checking.
pub fn validate_timestamp(ts: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    const MAX_DRIFT: u64 = 300; // 5 minutes
    ts >= now - MAX_DRIFT && ts <= now + MAX_DRIFT
}

/// Chain ID must match the configured chain for this gateway instance.
pub fn validate_chain_id(chain_id: u64, expected: u64) -> bool {
    chain_id == expected
}