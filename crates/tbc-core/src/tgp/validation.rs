//# TGP Validation Helpers

//**Destination Path:** `crates/tbc-core/src/tgp/validation.rs`

//**Implementation:** M1 - TGP Message Parsing & Basic Routing

//! TGP message validation helpers
//!
//! This module provides reusable validation functions for TGP message fields.
//! These functions enforce the validation rules specified in TGP-00 §3.1-3.4.
//!
//! # Validation Functions
//!
//! - [`validate_non_empty`] - Check that strings are not empty
//! - [`validate_positive_amount`] - Check that amounts are greater than zero
//! - [`validate_address`] - Check Ethereum address format
//! - [`validate_transaction_hash`] - Check transaction hash format
//! - [`validate_id_format`] - Check message ID format (optional)
//!
//! # Examples
//!
//! ```rust
//! use tbc_core::tgp::validation::*;
//!
//! // Validate a non-empty string
//! validate_non_empty("q-abc123", "id")?;
//!
//! // Validate an amount
//! validate_positive_amount(1_000_000, "amount")?;
//!
//! // Validate an Ethereum address
//! validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0", "contract")?;
//! # Ok::<(), String>(())
//! ```

// ============================================================================
// Basic Validation Functions
// ============================================================================

/// Validate that a string field is not empty
///
/// # Arguments
///
/// * `value` - The string value to validate
/// * `field_name` - Name of the field (for error messages)
///
/// # Errors
///
/// Returns an error if the string is empty.
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_non_empty;
/// assert!(validate_non_empty("q-123", "id").is_ok());
/// assert!(validate_non_empty("", "id").is_err());
/// ```
pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{} is required and must not be empty", field_name));
    }
    Ok(())
}

/// Validate that an amount is greater than zero
///
/// # Arguments
///
/// * `amount` - The amount value to validate
/// * `field_name` - Name of the field (for error messages)
///
/// # Errors
///
/// Returns an error if the amount is zero.
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_positive_amount;
/// assert!(validate_positive_amount(1000, "amount").is_ok());
/// assert!(validate_positive_amount(0, "amount").is_err());
/// ```
pub fn validate_positive_amount(amount: u64, field_name: &str) -> Result<(), String> {
    if amount == 0 {
        return Err(format!("{} must be greater than 0", field_name));
    }
    Ok(())
}

// ============================================================================
// Ethereum-Specific Validation
// ============================================================================

/// Validate an Ethereum address format
///
/// Checks that the address:
/// - Starts with "0x"
/// - Is exactly 42 characters long (0x + 40 hex chars)
/// - Contains only hexadecimal characters after "0x"
///
/// # Arguments
///
/// * `address` - The address string to validate
/// * `field_name` - Name of the field (for error messages)
///
/// # Errors
///
/// Returns an error if the address format is invalid.
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_address;
/// let valid = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
/// assert!(validate_address(valid, "contract").is_ok());
///
/// let invalid = "0x123"; // Too short
/// assert!(validate_address(invalid, "contract").is_err());
/// ```
pub fn validate_address(address: &str, field_name: &str) -> Result<(), String> {
    // Check prefix
    if !address.starts_with("0x") {
        return Err(format!(
            "{} must be a valid Ethereum address starting with 0x: {}",
            field_name, address
        ));
    }

    // Check length (0x + 40 hex chars = 42 total)
    if address.len() != 42 {
        return Err(format!(
            "{} must be 42 characters long (0x + 40 hex chars): {}",
            field_name, address
        ));
    }

    // Check that all characters after 0x are hexadecimal
    let hex_part = &address[2..];
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "{} must contain only hexadecimal characters after 0x: {}",
            field_name, address
        ));
    }

    Ok(())
}

/// Validate a transaction hash format
///
/// Checks that the transaction hash:
/// - Starts with "0x"
/// - Is exactly 66 characters long (0x + 64 hex chars)
/// - Contains only hexadecimal characters after "0x"
///
/// # Arguments
///
/// * `hash` - The transaction hash to validate
/// * `field_name` - Name of the field (for error messages)
///
/// # Errors
///
/// Returns an error if the hash format is invalid.
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_transaction_hash;
/// let valid = "0x9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e";
/// assert!(validate_transaction_hash(valid, "layer8_tx").is_ok());
///
/// let invalid = "0x123"; // Too short
/// assert!(validate_transaction_hash(invalid, "layer8_tx").is_err());
/// ```
pub fn validate_transaction_hash(hash: &str, field_name: &str) -> Result<(), String> {
    // Check prefix
    if !hash.starts_with("0x") {
        return Err(format!(
            "{} must be a valid transaction hash starting with 0x: {}",
            field_name, hash
        ));
    }

    // Check length (0x + 64 hex chars = 66 total)
    if hash.len() != 66 {
        return Err(format!(
            "{} must be 66 characters long (0x + 64 hex chars): {}",
            field_name, hash
        ));
    }

    // Check that all characters after 0x are hexadecimal
    let hex_part = &hash[2..];
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "{} must contain only hexadecimal characters after 0x: {}",
            field_name, hash
        ));
    }

    Ok(())
}

// ============================================================================
// Optional Advanced Validation
// ============================================================================

/// Validate message ID format (optional - implementation-specific)
///
/// Checks that the ID:
/// - Is not empty
/// - Follows a recommended format (prefix + hyphen + alphanumeric)
/// - Examples: "q-abc123", "offer-xyz789", "settle-123456"
///
/// This is an optional validation function. Message IDs can be any non-empty
/// string, but following a consistent format improves debugging and logging.
///
/// # Arguments
///
/// * `id` - The message ID to validate
/// * `expected_prefix` - Optional expected prefix (e.g., "q-", "offer-")
///
/// # Errors
///
/// Returns an error if the ID format doesn't match expectations.
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_id_format;
/// assert!(validate_id_format("q-abc123", Some("q")).is_ok());
/// assert!(validate_id_format("offer-xyz", Some("offer")).is_ok());
/// assert!(validate_id_format("invalid", Some("q")).is_err());
/// ```
pub fn validate_id_format(id: &str, expected_prefix: Option<&str>) -> Result<(), String> {
    validate_non_empty(id, "id")?;

    if let Some(prefix) = expected_prefix {
        let expected = format!("{}-", prefix);
        if !id.starts_with(&expected) {
            return Err(format!(
                "id should start with '{}': {}",
                expected, id
            ));
        }

        // Check that there's content after the prefix
        if id.len() <= expected.len() {
            return Err(format!(
                "id must have content after '{}': {}",
                expected, id
            ));
        }
    }

    Ok(())
}

/// Validate URL format (basic check)
///
/// Performs a basic check that a string looks like a URL.
/// This is NOT a comprehensive URL validation - use a proper URL parsing
/// library for production validation.
///
/// # Arguments
///
/// * `url` - The URL string to validate
/// * `field_name` - Name of the field (for error messages)
///
/// # Errors
///
/// Returns an error if the URL format is obviously invalid.
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_url_format;
/// assert!(validate_url_format("https://example.com", "metadata_uri").is_ok());
/// assert!(validate_url_format("ipfs://Qm...", "metadata_uri").is_ok());
/// assert!(validate_url_format("not a url", "metadata_uri").is_err());
/// ```
pub fn validate_url_format(url: &str, field_name: &str) -> Result<(), String> {
    validate_non_empty(url, field_name)?;

    // Check for common URL schemes
    let valid_schemes = ["http://", "https://", "ipfs://", "ar://"];
    let has_valid_scheme = valid_schemes.iter().any(|scheme| url.starts_with(scheme));

    if !has_valid_scheme {
        return Err(format!(
            "{} must start with a valid scheme (http://, https://, ipfs://, ar://): {}",
            field_name, url
        ));
    }

    Ok(())
}

/// Validate RFC3339 timestamp format (basic check)
///
/// Performs a basic format check for RFC3339 timestamps.
/// For production use, parse with chrono or time crate for full validation.
///
/// # Arguments
///
/// * `timestamp` - The timestamp string to validate
/// * `field_name` - Name of the field (for error messages)
///
/// # Errors
///
/// Returns an error if the format is obviously wrong.
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_rfc3339_format;
/// assert!(validate_rfc3339_format("2025-11-10T23:59:59Z", "expiry").is_ok());
/// assert!(validate_rfc3339_format("2025-11-10T23:59:59+00:00", "expiry").is_ok());
/// assert!(validate_rfc3339_format("invalid", "expiry").is_err());
/// ```
pub fn validate_rfc3339_format(timestamp: &str, field_name: &str) -> Result<(), String> {
    validate_non_empty(timestamp, field_name)?;

    // Basic format check: must contain 'T' separator
    if !timestamp.contains('T') {
        return Err(format!(
            "{} must be in RFC3339 format (e.g., 2025-11-10T23:59:59Z): {}",
            field_name, timestamp
        ));
    }

    // Check for timezone indicator (Z, +, or -)
    let has_timezone = timestamp.ends_with('Z')
        || timestamp.contains('+')
        || timestamp.matches('-').count() > 2; // More than 2 hyphens means timezone offset

    if !has_timezone {
        return Err(format!(
            "{} must include timezone (Z or offset like +00:00): {}",
            field_name, timestamp
        ));
    }

    Ok(())
}

// ============================================================================
// Composite Validation Functions
// ============================================================================

/// Validate that a correlation ID references a valid message
///
/// This is a placeholder for more sophisticated validation that would
/// check against a database or message store.
///
/// # Arguments
///
/// * `correlation_id` - The ID to validate
/// * `expected_phase` - Optional expected message phase (e.g., "QUERY", "OFFER")
///
/// # Examples
///
/// ```rust
/// # use tbc_core::tgp::validation::validate_correlation_id;
/// assert!(validate_correlation_id("q-abc123", Some("QUERY")).is_ok());
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
                "correlation_id should reference a {} message (start with '{}'): {}",
                phase, prefix, correlation_id
            ));
        }
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
        assert!(validate_non_empty("test", "field").is_ok());
        assert!(validate_non_empty("", "field").is_err());
    }

    #[test]
    fn test_validate_positive_amount() {
        assert!(validate_positive_amount(1, "amount").is_ok());
        assert!(validate_positive_amount(1000, "amount").is_ok());
        assert!(validate_positive_amount(0, "amount").is_err());
    }

    #[test]
    fn test_validate_address() {
        // Valid addresses
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0", "contract").is_ok());
        assert!(validate_address("0x0000000000000000000000000000000000000000", "contract").is_ok());

        // Invalid addresses
        assert!(validate_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0", "contract").is_err()); // No 0x
        assert!(validate_address("0x123", "contract").is_err()); // Too short
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbXX", "contract").is_err()); // Too long
        assert!(validate_address("0xGGGd35Cc6634C0532925a3b844Bc9e7595f0bEb0", "contract").is_err()); // Invalid hex
    }

    #[test]
    fn test_validate_transaction_hash() {
        // Valid hash
        let valid = "0x9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e";
        assert!(validate_transaction_hash(valid, "tx").is_ok());

        // Invalid hashes
        assert!(validate_transaction_hash("0x123", "tx").is_err()); // Too short
        assert!(validate_transaction_hash("9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e", "tx").is_err()); // No 0x
    }

    #[test]
    fn test_validate_id_format() {
        // Valid IDs with prefix
        assert!(validate_id_format("q-abc123", Some("q")).is_ok());
        assert!(validate_id_format("offer-xyz789", Some("offer")).is_ok());

        // Invalid IDs
        assert!(validate_id_format("", Some("q")).is_err()); // Empty
        assert!(validate_id_format("invalid", Some("q")).is_err()); // Wrong prefix
        assert!(validate_id_format("q-", Some("q")).is_err()); // No content after prefix
    }

    #[test]
    fn test_validate_url_format() {
        // Valid URLs
        assert!(validate_url_format("https://example.com", "uri").is_ok());
        assert!(validate_url_format("http://localhost:3000", "uri").is_ok());
        assert!(validate_url_format("ipfs://QmHash", "uri").is_ok());
        assert!(validate_url_format("ar://TxId", "uri").is_ok());

        // Invalid URLs
        assert!(validate_url_format("", "uri").is_err()); // Empty
        assert!(validate_url_format("not a url", "uri").is_err()); // No scheme
        assert!(validate_url_format("ftp://example.com", "uri").is_err()); // Invalid scheme
    }

    #[test]
    fn test_validate_rfc3339_format() {
        // Valid timestamps
        assert!(validate_rfc3339_format("2025-11-10T23:59:59Z", "expiry").is_ok());
        assert!(validate_rfc3339_format("2025-11-10T23:59:59+00:00", "expiry").is_ok());
        assert!(validate_rfc3339_format("2025-11-10T23:59:59-05:00", "expiry").is_ok());

        // Invalid timestamps
        assert!(validate_rfc3339_format("", "expiry").is_err()); // Empty
        assert!(validate_rfc3339_format("2025-11-10", "expiry").is_err()); // No time
        assert!(validate_rfc3339_format("2025-11-10 23:59:59", "expiry").is_err()); // Space instead of T
        assert!(validate_rfc3339_format("2025-11-10T23:59:59", "expiry").is_err()); // No timezone
    }

    #[test]
    fn test_validate_correlation_id() {
        // Valid correlation IDs
        assert!(validate_correlation_id("q-abc123", Some("QUERY")).is_ok());
        assert!(validate_correlation_id("offer-xyz", Some("OFFER")).is_ok());

        // Invalid correlation IDs
        assert!(validate_correlation_id("", Some("QUERY")).is_err()); // Empty
        assert!(validate_correlation_id("invalid", Some("QUERY")).is_err()); // Wrong prefix
    }
}
//============================================================================
// TGP Enforcement Bridge (Hybrid in Mem Integration Layer)
// ============================================================================
//
// This layer connects low-level input validation (this file)
// with the high-level policy + decision modules:
//
//   types.rs     → EconomicEnvelope, ZkProfile
//   policy.rs    → BuyerPolicy, VendorPolicy, SessionKeyPolicy
//   decision.rs  → TBCDecision, RejectReason, TGPValidationResult
//
// This keeps validation.rs fully backward-compatible while enabling
// TGP-SEC-00 and MCP-AUTO-PAY-00 rule enforcement.

use crate::tgp::policy::{BuyerPolicy, VendorPolicy, SessionKeyPolicy};
use crate::tgp::decision::{TBCDecision, RejectReason, TGPValidationResult};

/// Validate an EconomicEnvelope against Buyer + Vendor policy
pub fn enforce_economic_policy(
    envelope: &super::types::EconomicEnvelope,
    buyer: &BuyerPolicy,
    vendor: &VendorPolicy,
) -> TGPValidationResult {
    // Validate envelope itself
    if let Err(e) = envelope.validate() {
        return TGPValidationResult::reject(RejectReason::EconomicEnvelopeExceeded)
            .with_details(e);
    }

    // Check vendor's own maximum
    if envelope.max_fees_bps > vendor.economic.max_fees_bps {
        return TGPValidationResult::reject(RejectReason::PolicyViolation)
            .with_details(format!(
                "Vendor max fees {}bps < envelope {}bps",
                vendor.economic.max_fees_bps, envelope.max_fees_bps
            ));
    }

    // All good
    TGPValidationResult::ok()
}

/// Validate whether escrow is required (Buyer + Vendor + SessionKey)
pub fn enforce_escrow_policy(
    buyer: &BuyerPolicy,
    vendor: &VendorPolicy,
    key: Option<&SessionKeyPolicy>,
    amount: u64,
) -> TGPValidationResult {
    // Buyer explicitly requires escrow
    if buyer.zk_profile.requires_escrow() {
        return TGPValidationResult {
            decision: TBCDecision::EscrowRequired,
            details: Some("Buyer requires escrow".into()),
        };
    }

    // Vendor explicitly requires escrow (untrusted seller)
    if vendor.require_escrow {
        return TGPValidationResult {
            decision: TBCDecision::EscrowRequired,
            details: Some("Vendor requires escrow".into()),
        };
    }

    // Buyer allows direct payment only below a threshold
    if amount > buyer.max_direct_amount {
        return TGPValidationResult {
            decision: TBCDecision::EscrowRequired,
            details: Some("Direct payment exceeds buyer threshold".into()),
        };
    }

    // Session key constraints
    if let Some(k) = key {
        if k.escrow_required {
            return TGPValidationResult {
                decision: TBCDecision::EscrowRequired,
                details: Some("SessionKey requires escrow".into()),
            };
        }
    }

    // Everything permits direct payment
    TGPValidationResult::ok()
}

/// Validate an amount against Buyer and Vendor maximums
pub fn enforce_amount_policy(
    amount: u64,
    buyer: &BuyerPolicy,
    vendor: &VendorPolicy,
) -> TGPValidationResult {
    if amount > buyer.max_total_amount {
        return TGPValidationResult::reject(RejectReason::PolicyViolation)
            .with_details("Amount exceeds buyer maximum");
    }

    if amount > vendor.max_offer_amount {
        return TGPValidationResult::reject(RejectReason::PolicyViolation)
            .with_details("Amount exceeds vendor maximum");
    }

    TGPValidationResult::ok()
}

// Extend TGPValidationResult with convenience builder
impl TGPValidationResult {
    pub fn with_details(mut self, msg: impl Into<String>) -> Self {
        self.details = Some(msg.into());
        self
    }
}