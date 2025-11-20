//# TGP Message Type Definitions

//**Destination Path:** `crates/tbc-core/src/tgp/messages.rs`

//**Implementation:** M1 - TGP Message Parsing & Basic Routing
//! TGP message type definitions per TGP-00 specification §3.1-3.4
//!
//! This module defines the core message structures for the Transaction Gateway Protocol.
//! Each message type corresponds to a specific phase in the TGP session lifecycle.
//!
//! # Message Types
//!
//! - [`QueryMessage`] - §3.1: Initiates capability or path query
//! - [`OfferMessage`] - §3.2: Suggests viable route or settlement method
//! - [`SettleMessage`] - §3.3: Reports settlement completion
//! - [`ErrorMessage`] - §3.4: Notifies of protocol failureå
//! - [`TGPMessage`] - Discriminated union of all message types
//!
//! # Examples
//!
//! ```rust
//! use tbc_core::tgp::messages::{TGPMessage, QueryMessage};
//! use tbc_core::tgp::types::ZkProfile;
//!
//! let query = QueryMessage::new(
//!     "q-abc123",
//!     "buyer://alice",
//!     "seller://bob",
//!     "USDC",
//!     1_000_000,
//!     ZkProfile::Optional,
//! );
//!
//! let message = TGPMessage::Query(query);
//! let json = serde_json::to_string(&message)?;
//! # Ok::<(), serde_json::Error>(())
//! ```

use serde::{Deserialize, Serialize};

use super::types::{EconomicEnvelope, SettleSource, ZkProfile};
use super::validation::{
    validate_address, validate_non_empty, validate_positive_amount, validate_transaction_hash,
};

// ============================================================================
// Message Discriminated Union (§3.8)
// ============================================================================

/// TGP message discriminator with phase-based routing
///
/// All TGP messages are wrapped in this enum, which uses the `phase` field
/// as a discriminator for JSON serialization/deserialization.
///
/// # Specification Reference
/// - TGP-00 §3.8 Message Encoding
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::{TGPMessage, QueryMessage};
/// use tbc_core::tgp::types::ZkProfile;
///
/// let query = QueryMessage {
///     id: "q-abc123".to_string(),
///     from: "buyer://alice".to_string(),
///     to: "seller://bob".to_string(),
///     asset: "USDC".to_string(),
///     amount: 1_000_000,
///     escrow_from_402: false,
///     escrow_contract_from_402: None,
///     zk_profile: ZkProfile::Optional,
/// };
///
/// let message = TGPMessage::Query(query);
/// let json = serde_json::to_string(&message).unwrap();
/// // JSON will contain: { "phase": "QUERY", ... }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "phase")]
pub enum TGPMessage {
    /// QUERY message - initiates session
    #[serde(rename = "QUERY")]
    Query(QueryMessage),

    /// OFFER message - proposes settlement path
    #[serde(rename = "OFFER")]
    Offer(OfferMessage),

    /// SETTLE message - reports settlement outcome
    #[serde(rename = "SETTLE")]
    Settle(SettleMessage),

    /// ERROR message - signals protocol failure
    #[serde(rename = "ERROR")]
    Error(ErrorMessage),
}

impl TGPMessage {
    /// Get the message ID (present in all message types)
    pub fn id(&self) -> &str {
        match self {
            TGPMessage::Query(m) => &m.id,
            TGPMessage::Offer(m) => &m.id,
            TGPMessage::Settle(m) => &m.id,
            TGPMessage::Error(m) => &m.id,
        }
    }

    /// Get the message phase as a string
    pub fn phase(&self) -> &str {
        match self {
            TGPMessage::Query(_) => "QUERY",
            TGPMessage::Offer(_) => "OFFER",
            TGPMessage::Settle(_) => "SETTLE",
            TGPMessage::Error(_) => "ERROR",
        }
    }

    /// Validate the message structure
    ///
    /// # Errors
    ///
    /// Returns an error string if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            TGPMessage::Query(m) => m.validate(),
            TGPMessage::Offer(m) => m.validate(),
            TGPMessage::Settle(m) => m.validate(),
            TGPMessage::Error(m) => m.validate(),
        }
    }
}

// ============================================================================
// QUERY Message (§3.1)
// ============================================================================

/// QUERY message - initiates capability or path query
///
/// Sent by a Buyer (or Buyer Agent) to a Controller/Gateway to request
/// routing advice and settlement options.
///
/// # Specification Reference
/// - TGP-00 §3.1 QUERY Message
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::QueryMessage;
/// use tbc_core::tgp::types::ZkProfile;
///
/// let query = QueryMessage::new(
///     "q-abc123",
///     "buyer://alice",
///     "seller://bob",
///     "USDC",
///     1_000_000,
///     ZkProfile::Required,
/// );
///
/// assert!(query.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryMessage {
    /// Unique identifier for this query (client-generated)
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    pub id: String,

    /// Buyer identifier
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    pub from: String,

    /// Seller identifier
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    pub to: String,

    /// Asset denomination (e.g., "USDC", "ETH")
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    pub asset: String,

    /// Amount in smallest unit (e.g., wei, lamports)
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    pub amount: u64,

    /// Whether the 402 response advertised CoreProver
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    pub escrow_from_402: bool,

    /// CoreProver contract address from 402 header
    ///
    /// **Spec:** TGP-00 §3.1 - Optional field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escrow_contract_from_402: Option<String>,

    /// Buyer's preference for ZK/CoreProver involvement
    ///
    /// **Spec:** TGP-00 §3.1 - Required field (see §3.5)
    pub zk_profile: ZkProfile,
}

impl QueryMessage {
    /// Validate the QUERY message structure
    ///
    /// # Validation Rules (per TGP-00 §3.1)
    ///
    /// - `id` must not be empty
    /// - `from` must not be empty
    /// - `to` must not be empty
    /// - `asset` must not be empty
    /// - `amount` must be greater than zero
    /// - `escrow_contract_from_402` must be valid address if present
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.from, "from")?;
        validate_non_empty(&self.to, "to")?;
        validate_non_empty(&self.asset, "asset")?;
        validate_positive_amount(self.amount, "amount")?;

        if let Some(ref contract) = self.escrow_contract_from_402 {
            validate_address(contract, "escrow_contract_from_402")?;
        }

        Ok(())
    }

    /// Create a new QUERY message with required fields
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        asset: impl Into<String>,
        amount: u64,
        zk_profile: ZkProfile,
    ) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            asset: asset.into(),
            amount,
            escrow_from_402: false,
            escrow_contract_from_402: None,
            zk_profile,
        }
    }

    /// Create a QUERY with x402 escrow metadata
    pub fn with_escrow_from_402(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        asset: impl Into<String>,
        amount: u64,
        contract: impl Into<String>,
        zk_profile: ZkProfile,
    ) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            asset: asset.into(),
            amount,
            escrow_from_402: true,
            escrow_contract_from_402: Some(contract.into()),
            zk_profile,
        }
    }
}

// ============================================================================
// OFFER Message (§3.2)
// ============================================================================

/// OFFER message - proposes viable route or settlement method
///
/// Sent by a Controller/Gateway in response to a QUERY.
///
/// # Specification Reference
/// - TGP-00 §3.2 OFFER Message
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::OfferMessage;
/// use tbc_core::tgp::types::EconomicEnvelope;
///
/// let offer = OfferMessage::new(
///     "offer-abc123",
///     "q-abc123",
///     "USDC",
///     1_000_000,
///     true,
///     EconomicEnvelope::new(50),
/// );
///
/// assert!(offer.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfferMessage {
    /// Unique identifier for this offer
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    pub id: String,

    /// Correlation ID to originating QUERY
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    pub query_id: String,

    /// Asset denomination (echoed from QUERY)
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    pub asset: String,

    /// Amount in smallest unit (echoed from QUERY)
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    pub amount: u64,

    /// CoreProver escrow contract address
    ///
    /// **Spec:** TGP-00 §3.2 - Optional field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreprover_contract: Option<String>,

    /// Session identifier for CoreProver routing
    ///
    /// **Spec:** TGP-00 §3.2 - Optional field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Whether ZK/CoreProver is required by policy
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    pub zk_required: bool,

    /// Fee limits and validity constraints
    ///
    /// **Spec:** TGP-00 §3.2 - Required field (see §3.6)
    pub economic_envelope: EconomicEnvelope,
}

impl OfferMessage {
    /// Validate the OFFER message structure
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.query_id, "query_id")?;
        validate_non_empty(&self.asset, "asset")?;
        validate_positive_amount(self.amount, "amount")?;

        if let Some(ref contract) = self.coreprover_contract {
            validate_address(contract, "coreprover_contract")?;
        }

        self.economic_envelope.validate()?;

        Ok(())
    }

    /// Create a new OFFER message with required fields
    pub fn new(
        id: impl Into<String>,
        query_id: impl Into<String>,
        asset: impl Into<String>,
        amount: u64,
        zk_required: bool,
        economic_envelope: EconomicEnvelope,
    ) -> Self {
        Self {
            id: id.into(),
            query_id: query_id.into(),
            asset: asset.into(),
            amount,
            coreprover_contract: None,
            session_id: None,
            zk_required,
            economic_envelope,
        }
    }

    /// Builder method to set CoreProver contract
    pub fn with_coreprover(mut self, contract: impl Into<String>) -> Self {
        self.coreprover_contract = Some(contract.into());
        self
    }

    /// Builder method to set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

// ============================================================================
// SETTLE Message (§3.3)
// ============================================================================

/// SETTLE message - reports settlement completion
///
/// Sent to notify the Controller that settlement has occurred.
///
/// # Specification Reference
/// - TGP-00 §3.3 SETTLE Message
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::SettleMessage;
/// use tbc_core::tgp::types::SettleSource;
///
/// let settle = SettleMessage::new(
///     "settle-abc123",
///     "offer-abc123",
///     true,
///     SettleSource::BuyerNotify,
/// );
///
/// assert!(settle.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettleMessage {
    /// Unique identifier for this settlement report
    ///
    /// **Spec:** TGP-00 §3.3 - Required field
    pub id: String,

    /// Correlation ID (references QUERY or OFFER)
    ///
    /// **Spec:** TGP-00 §3.3 - Required field
    pub query_or_offer_id: String,

    /// Whether settlement completed successfully
    ///
    /// **Spec:** TGP-00 §3.3 - Required field
    pub success: bool,

    /// Who reported this settlement
    ///
    /// **Spec:** TGP-00 §3.3 - Required field (see §3.7)
    pub source: SettleSource,

    /// Layer-8 transaction hash
    ///
    /// **Spec:** TGP-00 §3.3 - Optional field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer8_tx: Option<String>,

    /// Session ID used with CoreProver
    ///
    /// **Spec:** TGP-00 §3.3 - Optional field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl SettleMessage {
    /// Validate the SETTLE message structure
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.query_or_offer_id, "query_or_offer_id")?;

        if let Some(ref tx) = self.layer8_tx {
            validate_transaction_hash(tx, "layer8_tx")?;
        }

        Ok(())
    }

    /// Create a new SETTLE message with required fields
    pub fn new(
        id: impl Into<String>,
        query_or_offer_id: impl Into<String>,
        success: bool,
        source: SettleSource,
    ) -> Self {
        Self {
            id: id.into(),
            query_or_offer_id: query_or_offer_id.into(),
            success,
            source,
            layer8_tx: None,
            session_id: None,
        }
    }

    /// Builder method to set transaction hash
    pub fn with_tx(mut self, tx: impl Into<String>) -> Self {
        self.layer8_tx = Some(tx.into());
        self
    }

    /// Builder method to set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

// ============================================================================
// ERROR Message (§3.4)
// ============================================================================

/// ERROR message - signals protocol-level failure
///
/// Signals a protocol-level failure or policy violation.
///
/// # Specification Reference
/// - TGP-00 §3.4 ERROR Message
///
/// # Standard Error Codes
///
/// - `INVALID_QUERY` - QUERY message failed validation
/// - `UNSUPPORTED_ASSET` - Asset not supported
/// - `POLICY_VIOLATION` - Request violates policy
/// - `CONTRACT_BLACKLISTED` - Contract is blacklisted
/// - `INSUFFICIENT_FUNDS` - Insufficient balance
/// - `TIMEOUT` - Session timed out
/// - `SETTLEMENT_FAILED` - Transaction failed
/// - `INVALID_STATE` - Invalid state transition
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::ErrorMessage;
///
/// let error = ErrorMessage::new(
///     "err-abc123",
///     "UNSUPPORTED_ASSET",
///     "Asset DOGE not supported",
/// );
///
/// assert!(error.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    /// Unique identifier for this error
    ///
    /// **Spec:** TGP-00 §3.4 - Required field
    pub id: String,

    /// Machine-readable error code
    ///
    /// **Spec:** TGP-00 §3.4 - Required field
    pub code: String,

    /// Human-readable error description
    ///
    /// **Spec:** TGP-00 §3.4 - Required field
    pub message: String,

    /// ID of the triggering message
    ///
    /// **Spec:** TGP-00 §3.4 - Optional field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl ErrorMessage {
    /// Validate the ERROR message structure
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.code, "code")?;
        validate_non_empty(&self.message, "message")?;
        Ok(())
    }

    /// Create a new ERROR message
    pub fn new(
        id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            message: message.into(),
            correlation_id: None,
        }
    }

    /// Create an ERROR message with correlation
    pub fn with_correlation(
        id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        correlation_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            message: message.into(),
            correlation_id: Some(correlation_id.into()),
        }
    }
}

// ============================================================================
// Standard Error Codes
// ============================================================================

/// Standard TGP error codes (per TGP-00 §3.4)
pub mod error_codes {
    pub const INVALID_QUERY: &str = "INVALID_QUERY";
    pub const UNSUPPORTED_ASSET: &str = "UNSUPPORTED_ASSET";
    pub const POLICY_VIOLATION: &str = "POLICY_VIOLATION";
    pub const CONTRACT_BLACKLISTED: &str = "CONTRACT_BLACKLISTED";
    pub const INSUFFICIENT_FUNDS: &str = "INSUFFICIENT_FUNDS";
    pub const TIMEOUT: &str = "TIMEOUT";
    pub const SETTLEMENT_FAILED: &str = "SETTLEMENT_FAILED";
    pub const INVALID_STATE: &str = "INVALID_STATE";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_message_validation() {
        let valid = QueryMessage::new(
            "q-123",
            "buyer://alice",
            "seller://bob",
            "USDC",
            1000,
            ZkProfile::Optional,
        );
        assert!(valid.validate().is_ok());

        let mut invalid = valid.clone();
        invalid.id = String::new();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_offer_message_validation() {
        let valid = OfferMessage::new(
            "offer-123",
            "q-123",
            "USDC",
            1000,
            true,
            EconomicEnvelope::new(50),
        );
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_settle_message_validation() {
        let valid = SettleMessage::new(
            "settle-123",
            "offer-123",
            true,
            SettleSource::BuyerNotify,
        );
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_error_message_validation() {
        let valid = ErrorMessage::new("err-123", "TIMEOUT", "Session timed out");
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_message_serialization() {
        let query = QueryMessage::new(
            "q-123",
            "buyer://alice",
            "seller://bob",
            "USDC",
            1000,
            ZkProfile::Optional,
        );
        let message = TGPMessage::Query(query);
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains(r#""phase":"QUERY""#));

        let parsed: TGPMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(message, parsed);
    }

    #[test]
    fn test_builder_methods() {
        let offer = OfferMessage::new(
            "offer-123",
            "q-123",
            "USDC",
            1000,
            true,
            EconomicEnvelope::new(50),
        )
        .with_coreprover("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb")
        .with_session("sess-123");

        assert!(offer.coreprover_contract.is_some());
        assert!(offer.session_id.is_some());
    }
}