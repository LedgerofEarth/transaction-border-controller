// # TGP Validation Helpers
//
// **Destination Path:** `crates/tbc-core/src/tgp/validation.rs`
// **Implementation:** M1 - TGP Message Parsing & Basic Routing
//
// This module contains *pure* validation helpers used throughout the
// Transaction Gateway Protocol (TGP) message model. These functions implement
// the structural and syntactic validation rules defined in:
//
//     • TGP-00 §3.1  Required Fields
//     • TGP-00 §3.2  Amounts & Asset Identifiers
//     • TGP-00 §3.3  Addresses, Contract Identifiers, Hashes
//     • TGP-00 §3.4  Identifiers (Message IDs, Correlation IDs)
//
// **Importantly:**  
// This file now contains **ONLY validation**.  
// All policy enforcement, domain rules, scoring layers, and decision logic
// are handled in the TBC gateway (`tbc-gateway/`).  
//
// This preserves TGP-core as a deterministic, policy-neutral parsing layer,
// suitable for:
//     • x402 handlers
//     • TGP-00 conformity tests
//     • TGP-MGMT-00 serialization logic
//     • Message-model unit tests
//
// Validation functions return a simple `Result<(), String>` for ease of
// embedding directly into protocol constructors and handlers.

use serde::{Serialize, Deserialize};

// ============================================================================
// Basic Validation
// ============================================================================

/// Validate that a string field is non-empty
///
/// TGP-00 §3.1 requires that `id`, `session_id`, and other identifiers must
/// never be empty.
///
/// Example:
/// ```rust
/// validate_non_empty("q-123", "id")?;
/// ```
pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{} is required and must not be empty", field_name));
    }
    Ok(())
}

/// Validate that an amount > 0
///
/// TGP-00 §3.2 amount fields (e.g., `amount` in QUERY) must be strictly
/// greater than zero.
///
/// Example:
/// ```rust
/// validate_positive_amount(1000, "amount")?;
/// ```
pub fn validate_positive_amount(amount: u64, field_name: &str) -> Result<(), String> {
    if amount == 0 {
        return Err(format!("{} must be greater than 0", field_name));
    }
    Ok(())
}

// ============================================================================
// Ethereum-Specific Validation (TGP-00 §3.3)
// ============================================================================

/// Validate an Ethereum-style address (0x + 40 hex chars)
///
/// Used for:
///     • Controller contract fields
///     • Settlement contract fields
///     • Any payment-profile address
pub fn validate_address(addr: &str, field_name: &str) -> Result<(), String> {
    if !addr.starts_with("0x") {
        return Err(format!(
            "{} must start with 0x: {}",
            field_name, addr
        ));
    }

    if addr.len() != 42 {
        return Err(format!(
            "{} must be 42 characters (0x + 40 hex chars): {}",
            field_name, addr
        ));
    }

    if !addr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "{} contains non-hexadecimal characters: {}",
            field_name, addr
        ));
    }

    Ok(())
}

/// Validate a transaction hash (0x + 64 hex chars)
///
/// Required for SETTLE messages when `source != controller-watcher`.
pub fn validate_transaction_hash(hash: &str, field_name: &str) -> Result<(), String> {
    if !hash.starts_with("0x") {
        return Err(format!(
            "{} must start with 0x: {}",
            field_name, hash
        ));
    }

    if hash.len() != 66 {
        return Err(format!(
            "{} must be 66 characters (0x + 64 hex chars): {}",
            field_name, hash
        ));
    }

    if !hash[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "{} contains non-hex characters: {}",
            field_name, hash
        ));
    }

    Ok(())
}

// ============================================================================
// ID + Correlation Validation (TGP-00 §3.4)
// ============================================================================

/// Validate a message ID format
///
/// Requirements:
///     • Non-empty
///     • If `expected_prefix` provided, must begin with `${prefix}-`
///
/// Example valid IDs:
///     • `q-abc123`
///     • `offer-xyz789`
///     • `settle-1234abcd`
pub fn validate_id_format(id: &str, expected_prefix: Option<&str>) -> Result<(), String> {
    validate_non_empty(id, "id")?;

    if let Some(prefix) = expected_prefix {
        let expected = format!("{}-", prefix);
        if !id.starts_with(&expected) {
            return Err(format!(
                "id must start with '{}': {}",
                expected, id
            ));
        }
        if id.len() <= expected.len() {
            return Err(format!(
                "id must contain characters after '{}': {}",
                expected, id
            ));
        }
    }

    Ok(())
}

/// Validate a correlation ID (referencing QUERY/OFFER/SETTLE)
///
/// Unlike message IDs, correlation IDs *may* reference any prior phase.
/// This validator ensures:
///     • Non-empty
///     • Optional prefix-phase check
///
/// Example:
/// ```rust
/// validate_correlation_id("q-abc", Some("QUERY"))?;
/// ```
pub fn validate_correlation_id(
    correlation_id: &str,
    expected_phase: Option<&str>,
) -> Result<(), String> {
    validate_non_empty(correlation_id, "correlation_id")?;

    if let Some(phase) = expected_phase {
        let prefix = match phase {
            "QUERY" => "q-",
            "OFFER" => "offer-",
            "SETTLE" => "settle-",
            "ERROR" => "err-",
            _ => return Err(format!("Unknown phase: {}", phase)),
        };

        if !correlation_id.starts_with(prefix) {
            return Err(format!(
                "correlation_id should reference a {} message (start '{}'): {}",
                phase, prefix, correlation_id
            ));
        }
    }

    Ok(())
}

// ============================================================================
// URL + Timestamp Validation (Optional, but useful)
// ============================================================================

/// Validate URL scheme (not a full URL parser)
///
/// Allowed schemes:
///     • http://
///     • https://
///     • ipfs://
///     • ar://
pub fn validate_url_format(url: &str, field_name: &str) -> Result<(), String> {
    validate_non_empty(url, field_name)?;

    const VALID: [&str; 4] = ["http://", "https://", "ipfs://", "ar://"];

    if !VALID.iter().any(|p| url.starts_with(p)) {
        return Err(format!(
            "{} must start with a valid URL scheme (http://, https://, ipfs://, ar://): {}",
            field_name, url
        ));
    }

    Ok(())
}

/// Validate RFC3339 timestamp (simple check for `T` + timezone)
pub fn validate_rfc3339_format(timestamp: &str, field_name: &str) -> Result<(), String> {
    validate_non_empty(timestamp, field_name)?;

    if !timestamp.contains('T') {
        return Err(format!(
            "{} must be RFC3339 (missing 'T'): {}",
            field_name, timestamp
        ));
    }

    let has_timezone =
        timestamp.ends_with('Z') ||
        timestamp.contains('+') ||
        timestamp.matches('-').count() > 2;

    if !has_timezone {
        return Err(format!(
            "{} must include timezone indicator (Z or +offset): {}",
            field_name, timestamp
        ));
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert!(validate_non_empty("ok", "x").is_ok());
        assert!(validate_non_empty("", "x").is_err());
    }

    #[test]
    fn test_validate_positive_amount() {
        assert!(validate_positive_amount(1, "amount").is_ok());
        assert!(validate_positive_amount(0, "amount").is_err());
    }

    #[test]
    fn test_validate_address() {
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0", "addr").is_ok());
        assert!(validate_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0", "addr").is_err());
    }

    #[test]
    fn test_validate_tx_hash() {
        let valid = "0x9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e";
        assert!(validate_transaction_hash(valid, "tx").is_ok());
        assert!(validate_transaction_hash("0x123", "tx").is_err());
    }

    #[test]
    fn test_validate_id_format() {
        assert!(validate_id_format("q-abc", Some("q")).is_ok());
        assert!(validate_id_format("invalid", Some("q")).is_err());
    }

    #[test]
    fn test_validate_correlation_id() {
        assert!(validate_correlation_id("q-abc", Some("QUERY")).is_ok());
        assert!(validate_correlation_id("", Some("QUERY")).is_err());
    }

    #[test]
    fn test_validate_url_format() {
        assert!(validate_url_format("https://example.com", "uri").is_ok());
        assert!(validate_url_format("ftp://example.com", "uri").is_err());
    }

    #[test]
    fn test_validate_rfc3339_format() {
        assert!(validate_rfc3339_format("2025-11-10T23:59:59Z", "ts").is_ok());
        assert!(validate_rfc3339_format("2025-11-10", "ts").is_err());
    }
}