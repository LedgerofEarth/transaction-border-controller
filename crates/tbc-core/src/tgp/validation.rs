// ============================================================================
// TGP Validation Helpers -- Updated for TGP-00 v3.2
// crates/tbc-core/src/tgp/validation.rs
//
// What changed:
//   • OFFER removed entirely
//   • ACK added as a first-class message type
//   • Correlation IDs updated to allow ACK references
//   • ID prefixes updated to: q-, ack-, settle-, err-
//   • Comments rewritten for accuracy and v3.2 alignment
// ============================================================================

use serde::{Serialize, Deserialize};

// ============================================================================
// Basic Validation
// ============================================================================

pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{} is required and must not be empty", field_name));
    }
    Ok(())
}

pub fn validate_positive_amount(amount: u64, field_name: &str) -> Result<(), String> {
    if amount == 0 {
        return Err(format!("{} must be greater than 0", field_name));
    }
    Ok(())
}

// ============================================================================
// Ethereum-Specific Validation
// ============================================================================

pub fn validate_address(addr: &str, field_name: &str) -> Result<(), String> {
    if !addr.starts_with("0x") {
        return Err(format!("{} must start with 0x: {}", field_name, addr));
    }
    if addr.len() != 42 {
        return Err(format!("{} must be 42 chars (0x + 40 hex): {}", field_name, addr));
    }
    if !addr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{} contains non-hex characters: {}", field_name, addr));
    }
    Ok(())
}

pub fn validate_transaction_hash(hash: &str, field_name: &str) -> Result<(), String> {
    if !hash.starts_with("0x") {
        return Err(format!("{} must start with 0x: {}", field_name, hash));
    }
    if hash.len() != 66 {
        return Err(format!("{} must be 66 chars (0x + 64 hex): {}", field_name, hash));
    }
    if !hash[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{} contains non-hex characters: {}", field_name, hash));
    }
    Ok(())
}

// ============================================================================
// ID + Correlation Validation (TGP-00 v3.2)
// ============================================================================

/// Validate a message ID
///
/// Valid prefixes in TGP v3.2:
///   • q-
///   • ack-
///   • settle-
///   • err-
pub fn validate_id_format(id: &str, expected_prefix: Option<&str>) -> Result<(), String> {
    validate_non_empty(id, "id")?;

    if let Some(prefix) = expected_prefix {
        let prefix_str = match prefix {
            "QUERY"  => "q-",
            "ACK"    => "ack-",
            "SETTLE" => "settle-",
            "ERROR"  => "err-",
            other => return Err(format!("Unknown message type prefix: {}", other)),
        };

        if !id.starts_with(prefix_str) {
            return Err(format!(
                "id must start with '{}': {}",
                prefix_str, id
            ));
        }

        if id.len() <= prefix_str.len() {
            return Err(format!(
                "id must contain characters after '{}': {}",
                prefix_str, id
            ));
        }
    }

    Ok(())
}

/// Validate correlation IDs for QUERY/ACK/SETTLE/ERROR
pub fn validate_correlation_id(
    correlation_id: &str,
    expected_phase: Option<&str>,
) -> Result<(), String> {
    validate_non_empty(correlation_id, "correlation_id")?;

    if let Some(phase) = expected_phase {
        let prefix = match phase {
            "QUERY"  => "q-",
            "ACK"    => "ack-",
            "SETTLE" => "settle-",
            "ERROR"  => "err-",
            other => return Err(format!("Unknown correlation phase: {}", other)),
        };

        if !correlation_id.starts_with(prefix) {
            return Err(format!(
                "correlation_id must reference a {} message (starts with '{}'): {}",
                phase, prefix, correlation_id
            ));
        }
    }

    Ok(())
}

// ============================================================================
// URL + Timestamp Validation (Optional)
// ============================================================================

pub fn validate_url_format(url: &str, field_name: &str) -> Result<(), String> {
    validate_non_empty(url, field_name)?;

    const VALID: [&str; 4] = ["http://", "https://", "ipfs://", "ar://"];

    if !VALID.iter().any(|p| url.starts_with(p)) {
        return Err(format!(
            "{} must start with a valid scheme (http/https/ipfs/ar): {}",
            field_name, url
        ));
    }

    Ok(())
}

pub fn validate_rfc3339_format(timestamp: &str, field_name: &str) -> Result<(), String> {
    validate_non_empty(timestamp, field_name)?;

    if !timestamp.contains('T') {
        return Err(format!(
            "{} must be RFC3339 (missing 'T'): {}",
            field_name, timestamp
        ));
    }

    let has_tz =
        timestamp.ends_with('Z') ||
        timestamp.contains('+') ||
        timestamp.matches('-').count() > 2;

    if !has_tz {
        return Err(format!(
            "{} must include timezone indicator: {}",
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
    fn test_validate_id_prefixes() {
        assert!(validate_id_format("q-123", Some("QUERY")).is_ok());
        assert!(validate_id_format("ack-xyz", Some("ACK")).is_ok());
        assert!(validate_id_format("settle-1", Some("SETTLE")).is_ok());
        assert!(validate_id_format("err-a", Some("ERROR")).is_ok());
    }

    #[test]
    fn test_invalid_prefix() {
        assert!(validate_id_format("offer-123", Some("QUERY")).is_err());
    }
}