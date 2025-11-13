//# TGP Protocol Message Types
//
//**Destination Path:** `crates/tbc-core/src/tgp/messages.rs`
//
//**Implementation:** M1 - TGP Message Parsing & Basic Routing

//! TGP message types per TGP-00 specification
//!
//! This module implements the core message types for the Transaction Gateway Protocol (TGP-00).
//! Each message type corresponds to a specific section in the TGP-00 specification.
//!
//! # Message Types
//!
//! - [`QueryMessage`] - §3.1: Initiates capability or path query
//! - [`OfferMessage`] - §3.2: Suggests viable route or settlement method
//! - [`SettleMessage`] - §3.3: Reports settlement completion
//! - [`ErrorMessage`] - §3.4: Notifies of protocol failure
//!
//! # Enumerations
//!
//! - [`ZkProfile`] - §3.5: Buyer's ZK proof preference
//! - [`SettleSource`] - §3.7: Settlement reporter identity
//!
//! # Supporting Types
//!
//! - [`EconomicEnvelope`] - §3.6: Economic constraints
//! - [`TGPMessage`] - Discriminated union of all message types

use serde::{Deserialize, Serialize};

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
/// use tbc_core::tgp::messages::{TGPMessage, QueryMessage, ZkProfile};
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
///
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tbc_core::tgp::messages::{TGPMessage, ErrorMessage};
    /// let error = ErrorMessage {
    ///     id: "err-123".to_string(),
    ///     code: "TIMEOUT".to_string(),
    ///     message: "Session timed out".to_string(),
    ///     correlation_id: None,
    /// };
    /// let msg = TGPMessage::Error(error);
    /// assert_eq!(msg.id(), "err-123");
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tbc_core::tgp::messages::{TGPMessage, QueryMessage, ZkProfile};
    /// let query = QueryMessage {
    ///     id: "".to_string(), // Invalid: empty ID
    ///     from: "buyer://alice".to_string(),
    ///     to: "seller://bob".to_string(),
    ///     asset: "USDC".to_string(),
    ///     amount: 1000,
    ///     escrow_from_402: false,
    ///     escrow_contract_from_402: None,
    ///     zk_profile: ZkProfile::Optional,
    /// };
    /// let msg = TGPMessage::Query(query);
    /// assert!(msg.validate().is_err());
    /// ```
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
/// routing advice and settlement options. Typically initiated after
/// receiving an HTTP 402 response with Layer-8 metadata.
///
/// # Specification Reference
/// - TGP-00 §3.1 QUERY Message
///
/// # Fields
///
/// All fields correspond to TGP-00 §3.1 table:
///
/// | Field | Type | Required | Description |
/// |-------|------|----------|-------------|
/// | `id` | string | ✓ | Unique identifier for this query |
/// | `from` | string | ✓ | Buyer identifier |
/// | `to` | string | ✓ | Seller identifier |
/// | `asset` | string | ✓ | Asset denomination |
/// | `amount` | u64 | ✓ | Amount in smallest unit |
/// | `escrow_from_402` | boolean | ✓ | Whether 402 advertised CoreProver |
/// | `escrow_contract_from_402` | string? | optional | CoreProver contract from 402 |
/// | `zk_profile` | ZkProfile | ✓ | Buyer's ZK preference |
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::{QueryMessage, ZkProfile};
///
/// let query = QueryMessage {
///     id: "q-abc123".to_string(),
///     from: "buyer://alice.wallet".to_string(),
///     to: "seller://store.example".to_string(),
///     asset: "USDC".to_string(),
///     amount: 1_000_000, // 1 USDC (6 decimals)
///     escrow_from_402: true,
///     escrow_contract_from_402: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string()),
///     zk_profile: ZkProfile::Required,
/// };
///
/// assert!(query.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryMessage {
    /// Unique identifier for this query (client-generated)
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    ///
    /// **Format:** Alphanumeric string, typically prefixed with "q-"
    ///
    /// **Validation:** Must not be empty
    pub id: String,

    /// Buyer identifier
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    ///
    /// **Format:** URI-style identifier (e.g., `buyer://alice`, wallet address, agent URI)
    ///
    /// **Validation:** Must not be empty
    pub from: String,

    /// Seller identifier
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    ///
    /// **Format:** URI-style identifier (e.g., `seller://bob`, service endpoint)
    ///
    /// **Validation:** Must not be empty
    pub to: String,

    /// Asset denomination
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    ///
    /// **Format:** Token symbol (e.g., "USDC", "ETH", "WBTC")
    ///
    /// **Validation:** Must not be empty
    pub asset: String,

    /// Amount in smallest unit
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    ///
    /// **Format:** Base units (e.g., wei for ETH, lamports for SOL)
    ///
    /// **Validation:** Must be greater than zero
    pub amount: u64,

    /// Whether the 402 response explicitly advertised CoreProver/escrow support
    ///
    /// **Spec:** TGP-00 §3.1 - Required field
    ///
    /// **Usage:**
    /// - `true`: Seller advertised escrow in HTTP 402 headers
    /// - `false`: No 402 advertisement (Controller provisions contract)
    pub escrow_from_402: bool,

    /// CoreProver contract address from 402 `X-Escrow-Contract` header
    ///
    /// **Spec:** TGP-00 §3.1 - Optional field
    ///
    /// **Format:** Ethereum address (0x-prefixed hex string)
    ///
    /// **Present when:** `escrow_from_402 == true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escrow_contract_from_402: Option<String>,

    /// Buyer's preference for ZK/CoreProver involvement
    ///
    /// **Spec:** TGP-00 §3.1 - Required field, see §3.5 for enum values
    ///
    /// **Values:**
    /// - `NONE`: Direct x402 preferred (no escrow)
    /// - `OPTIONAL`: Willing to use escrow if recommended
    /// - `REQUIRED`: Demands escrow (high-value or untrusted)
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
    /// - `escrow_contract_from_402` must be a valid address if present
    ///
    /// # Errors
    ///
    /// Returns a descriptive error string if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tbc_core::tgp::messages::{QueryMessage, ZkProfile};
    /// let mut query = QueryMessage {
    ///     id: "q-123".to_string(),
    ///     from: "buyer://alice".to_string(),
    ///     to: "seller://bob".to_string(),
    ///     asset: "USDC".to_string(),
    ///     amount: 1000,
    ///     escrow_from_402: false,
    ///     escrow_contract_from_402: None,
    ///     zk_profile: ZkProfile::Optional,
    /// };
    ///
    /// assert!(query.validate().is_ok());
    ///
    /// query.amount = 0; // Invalid
    /// assert!(query.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("id is required and must not be empty".to_string());
        }
        if self.from.is_empty() {
            return Err("from is required and must not be empty".to_string());
        }
        if self.to.is_empty() {
            return Err("to is required and must not be empty".to_string());
        }
        if self.asset.is_empty() {
            return Err("asset is required and must not be empty".to_string());
        }
        if self.amount == 0 {
            return Err("amount must be greater than 0".to_string());
        }

        // Validate escrow contract address format if present
        if let Some(ref contract) = self.escrow_contract_from_402 {
            if !contract.starts_with("0x") || contract.len() != 42 {
                return Err(format!(
                    "escrow_contract_from_402 must be a valid Ethereum address: {}",
                    contract
                ));
            }
        }

        Ok(())
    }

    /// Create a new QUERY message with required fields
    ///
    /// This is a convenience constructor for the most common case.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tbc_core::tgp::messages::{QueryMessage, ZkProfile};
    /// let query = QueryMessage::new(
    ///     "q-123",
    ///     "buyer://alice",
    ///     "seller://bob",
    ///     "USDC",
    ///     1_000_000,
    ///     ZkProfile::Optional,
    /// );
    /// ```
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
}

// ============================================================================
// OFFER Message (§3.2)
// ============================================================================

/// OFFER message - proposes viable route or settlement method
///
/// Sent by a Controller/Gateway in response to a QUERY. Contains routing
/// recommendations, settlement parameters, and economic envelope constraints.
///
/// # Specification Reference
/// - TGP-00 §3.2 OFFER Message
///
/// # Fields
///
/// All fields correspond to TGP-00 §3.2 table:
///
/// | Field | Type | Required | Description |
/// |-------|------|----------|-------------|
/// | `id` | string | ✓ | Unique identifier for this offer |
/// | `query_id` | string | ✓ | Correlation ID to originating QUERY |
/// | `asset` | string | ✓ | Asset denomination (echoed) |
/// | `amount` | u64 | ✓ | Amount in smallest unit (echoed) |
/// | `coreprover_contract` | string? | optional | CoreProver escrow contract |
/// | `session_id` | string? | optional | Session identifier for routing |
/// | `zk_required` | boolean | ✓ | ZK/CoreProver required by policy |
/// | `economic_envelope` | EconomicEnvelope | ✓ | Fee limits and constraints |
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::{OfferMessage, EconomicEnvelope};
///
/// let offer = OfferMessage {
///     id: "offer-abc123".to_string(),
///     query_id: "q-abc123".to_string(),
///     asset: "USDC".to_string(),
///     amount: 1_000_000,
///     coreprover_contract: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string()),
///     session_id: Some("sess-abc123".to_string()),
///     zk_required: true,
///     economic_envelope: EconomicEnvelope {
///         max_fees_bps: 50,
///         expiry: Some("2025-11-10T23:59:59Z".to_string()),
///     },
/// };
///
/// assert!(offer.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfferMessage {
    /// Unique identifier for this offer (controller-generated)
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    ///
    /// **Format:** Alphanumeric string, typically prefixed with "offer-"
    ///
    /// **Validation:** Must not be empty
    pub id: String,

    /// Correlation ID linking back to the originating QUERY
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    ///
    /// **Format:** Must match a valid QUERY.id
    ///
    /// **Validation:** Must not be empty
    pub query_id: String,

    /// Asset denomination (echoed from QUERY)
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    ///
    /// **Validation:** Must not be empty
    pub asset: String,

    /// Amount in smallest unit (echoed from QUERY)
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    ///
    /// **Validation:** Must be greater than zero
    pub amount: u64,

    /// CoreProver escrow contract address (if escrow path selected)
    ///
    /// **Spec:** TGP-00 §3.2 - Optional field
    ///
    /// **Format:** Ethereum address (0x-prefixed hex string)
    ///
    /// **Present when:** Controller selects escrow settlement path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreprover_contract: Option<String>,

    /// Unique session identifier for CoreProver onchain routing
    ///
    /// **Spec:** TGP-00 §3.2 - Optional field
    ///
    /// **Format:** Unique string, typically prefixed with "sess-"
    ///
    /// **Usage:** Included in Layer-8 transaction for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Whether ZK/CoreProver is required under Controller policy
    ///
    /// **Spec:** TGP-00 §3.2 - Required field
    ///
    /// **Determination:**
    /// - Based on QUERY.zk_profile and Controller policy
    /// - Overrides Buyer preference if policy demands escrow
    pub zk_required: bool,

    /// Fee limits and validity constraints
    ///
    /// **Spec:** TGP-00 §3.2 - Required field, see §3.6 for structure
    ///
    /// **Contents:**
    /// - `max_fees_bps`: Maximum acceptable total fees in basis points
    /// - `expiry`: RFC3339 timestamp after which offer is invalid
    pub economic_envelope: EconomicEnvelope,
}

impl OfferMessage {
    /// Validate the OFFER message structure
    ///
    /// # Validation Rules (per TGP-00 §3.2)
    ///
    /// - `id` must not be empty
    /// - `query_id` must not be empty
    /// - `asset` must not be empty
    /// - `amount` must be greater than zero
    /// - `coreprover_contract` must be a valid address if present
    /// - `economic_envelope` must be valid (see EconomicEnvelope::validate)
    ///
    /// # Errors
    ///
    /// Returns a descriptive error string if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("id is required and must not be empty".to_string());
        }
        if self.query_id.is_empty() {
            return Err("query_id is required and must not be empty".to_string());
        }
        if self.asset.is_empty() {
            return Err("asset is required and must not be empty".to_string());
        }
        if self.amount == 0 {
            return Err("amount must be greater than 0".to_string());
        }

        // Validate CoreProver contract address if present
        if let Some(ref contract) = self.coreprover_contract {
            if !contract.starts_with("0x") || contract.len() != 42 {
                return Err(format!(
                    "coreprover_contract must be a valid Ethereum address: {}",
                    contract
                ));
            }
        }

        // Validate economic envelope
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
}

// ============================================================================
// SETTLE Message (§3.3)
// ============================================================================

/// SETTLE message - reports settlement completion
///
/// Sent to notify the Controller that settlement has occurred. May be sent
/// by the Buyer, an external indexer, or synthesized by the Controller's
/// own watcher infrastructure.
///
/// # Specification Reference
/// - TGP-00 §3.3 SETTLE Message
///
/// # Fields
///
/// All fields correspond to TGP-00 §3.3 table:
///
/// | Field | Type | Required | Description |
/// |-------|------|----------|-------------|
/// | `id` | string | ✓ | Unique identifier for this report |
/// | `query_or_offer_id` | string | ✓ | Correlation ID (QUERY or OFFER) |
/// | `success` | boolean | ✓ | Whether settlement succeeded |
/// | `source` | SettleSource | ✓ | Who reported settlement |
/// | `layer8_tx` | string? | optional | Layer-8 transaction hash |
/// | `session_id` | string? | optional | CoreProver session ID |
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::{SettleMessage, SettleSource};
///
/// let settle = SettleMessage {
///     id: "settle-abc123".to_string(),
///     query_or_offer_id: "offer-abc123".to_string(),
///     success: true,
///     source: SettleSource::BuyerNotify,
///     layer8_tx: Some("0x9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e".to_string()),
///     session_id: Some("sess-abc123".to_string()),
/// };
///
/// assert!(settle.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettleMessage {
    /// Unique identifier for this settlement report
    ///
    /// **Spec:** TGP-00 §3.3 - Required field
    ///
    /// **Format:** Alphanumeric string, typically prefixed with "settle-"
    ///
    /// **Validation:** Must not be empty
    pub id: String,

    /// Correlation ID (references original QUERY or OFFER)
    ///
    /// **Spec:** TGP-00 §3.3 - Required field
    ///
    /// **Format:** Must match a valid QUERY.id or OFFER.id
    ///
    /// **Validation:** Must not be empty
    pub query_or_offer_id: String,

    /// Whether settlement completed successfully
    ///
    /// **Spec:** TGP-00 §3.3 - Required field
    ///
    /// **Values:**
    /// - `true`: Funds transferred, escrow released
    /// - `false`: Settlement failed, refund may be needed
    pub success: bool,

    /// Who reported this settlement
    ///
    /// **Spec:** TGP-00 §3.3 - Required field, see §3.7 for enum values
    ///
    /// **Values:**
    /// - `buyer-notify`: Buyer directly reporting
    /// - `controller-watcher`: Controller's indexer detected it
    /// - `coreprover-indexer`: External indexer notification
    pub source: SettleSource,

    /// Layer-8 transaction hash
    ///
    /// **Spec:** TGP-00 §3.3 - Optional field
    ///
    /// **Format:** Blockchain transaction hash (0x-prefixed hex)
    ///
    /// **Present when:** On-chain settlement occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer8_tx: Option<String>,

    /// Session ID used with CoreProver (if applicable)
    ///
    /// **Spec:** TGP-00 §3.3 - Optional field
    ///
    /// **Format:** Must match OFFER.session_id if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl SettleMessage {
    /// Validate the SETTLE message structure
    ///
    /// # Validation Rules (per TGP-00 §3.3)
    ///
    /// - `id` must not be empty
    /// - `query_or_offer_id` must not be empty
    /// - `layer8_tx` must be a valid transaction hash if present
    ///
    /// # Errors
    ///
    /// Returns a descriptive error string if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("id is required and must not be empty".to_string());
        }
        if self.query_or_offer_id.is_empty() {
            return Err("query_or_offer_id is required and must not be empty".to_string());
        }

        // Validate transaction hash format if present
        if let Some(ref tx) = self.layer8_tx {
            if !tx.starts_with("0x") || tx.len() != 66 {
                return Err(format!(
                    "layer8_tx must be a valid transaction hash (66 chars, 0x-prefixed): {}",
                    tx
                ));
            }
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
}

// ============================================================================
// ERROR Message (§3.4)
// ============================================================================

/// ERROR message - signals protocol-level failure
///
/// Signals a protocol-level failure or policy violation during
/// QUERY/OFFER/SETTLE processing.
///
/// # Specification Reference
/// - TGP-00 §3.4 ERROR Message
///
/// # Fields
///
/// All fields correspond to TGP-00 §3.4 table:
///
/// | Field | Type | Required | Description |
/// |-------|------|----------|-------------|
/// | `id` | string | ✓ | Unique identifier for this error |
/// | `code` | string | ✓ | Machine-readable error code |
/// | `message` | string | ✓ | Human-readable description |
/// | `correlation_id` | string? | optional | ID of triggering message |
///
/// # Standard Error Codes (per TGP-00 §3.4)
///
/// - `INVALID_QUERY` - QUERY message failed validation
/// - `UNSUPPORTED_ASSET` - Asset not supported by Controller
/// - `POLICY_VIOLATION` - Request violates domain policy
/// - `CONTRACT_BLACKLISTED` - CoreProver contract is blacklisted
/// - `INSUFFICIENT_FUNDS` - Buyer has insufficient balance
/// - `TIMEOUT` - Session or operation timed out
/// - `SETTLEMENT_FAILED` - Layer-8 transaction failed
/// - `INVALID_STATE` - Operation not allowed in current state
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::ErrorMessage;
///
/// let error = ErrorMessage {
///     id: "err-abc123".to_string(),
///     code: "UNSUPPORTED_ASSET".to_string(),
///     message: "Asset DOGE not supported in this jurisdiction".to_string(),
///     correlation_id: Some("q-abc123".to_string()),
/// };
///
/// assert!(error.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    /// Unique identifier for this error report
    ///
    /// **Spec:** TGP-00 §3.4 - Required field
    ///
    /// **Format:** Alphanumeric string, typically prefixed with "err-"
    ///
    /// **Validation:** Must not be empty
    pub id: String,

    /// Machine-readable error code
    ///
    /// **Spec:** TGP-00 §3.4 - Required field
    ///
    /// **Format:** SCREAMING_SNAKE_CASE (e.g., "POLICY_VIOLATION")
    ///
    /// **Validation:** Must not be empty
    ///
    /// **Standard Codes:** See TGP-00 §3.4 table
    pub code: String,

    /// Human-readable error description
    ///
    /// **Spec:** TGP-00 §3.4 - Required field
    ///
    /// **Format:** Plain English description
    ///
    /// **Validation:** Must not be empty
    pub message: String,

    /// ID of the message that triggered this error
    ///
    /// **Spec:** TGP-00 §3.4 - Optional field
    ///
    /// **Format:** QUERY/OFFER/SETTLE message ID
    ///
    /// **Present when:** Error relates to a specific message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl ErrorMessage {
    /// Validate the ERROR message structure
    ///
    /// # Validation Rules (per TGP-00 §3.4)
    ///
    /// - `id` must not be empty
    /// - `code` must not be empty
    /// - `message` must not be empty
    ///
    /// # Errors
    ///
    /// Returns a descriptive error string if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("id is required and must not be empty".to_string());
        }
        if self.code.is_empty() {
            return Err("code is required and must not be empty".to_string());
        }
        if self.message.is_empty() {
            return Err("message is required and must not be empty".to_string());
        }
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
// ZkProfile Enumeration (§3.5)
// ============================================================================

/// Buyer's preference for zero-knowledge proof and CoreProver escrow involvement
///
/// Indicates whether the Buyer wants to use CoreProver escrow settlement
/// or prefers direct x402 payment.
///
/// # Specification Reference
/// - TGP-00 §3.5 ZkProfile Enumeration
///
/// # Values
///
/// | Value | Meaning |
/// |-------|---------|
/// | `NONE` | Buyer does not want CoreProver escrow (direct x402 preferred) |
/// | `OPTIONAL` | Buyer is willing to use CoreProver if Controller recommends it |
/// | `REQUIRED` | Buyer demands CoreProver escrow (high-value or untrusted seller) |
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::ZkProfile;
///
/// let profile = ZkProfile::Required;
/// assert_eq!(serde_json::to_string(&profile).unwrap(), r#""REQUIRED""#);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ZkProfile {
    /// Buyer does not want CoreProver escrow (direct x402 preferred)
    ///
    /// **Use Case:** Low-value transactions with trusted sellers
    #[serde(rename = "NONE")]
    None,

    /// Buyer is willing to use CoreProver if Controller recommends it
    ///
    /// **Use Case:** Medium-value transactions, defer to Controller policy
    #[serde(rename = "OPTIONAL")]
    Optional,

    /// Buyer demands CoreProver escrow
    ///
    /// **Use Case:** High-value or untrusted counterparties
    #[serde(rename = "REQUIRED")]
    Required,
}

impl ZkProfile {
    /// Check if this profile allows escrow settlement
    pub fn allows_escrow(&self) -> bool {
        matches!(self, ZkProfile::Optional | ZkProfile::Required)
    }

    /// Check if this profile requires escrow settlement
    pub fn requires_escrow(&self) -> bool {
        matches!(self, ZkProfile::Required)
    }
}

impl Default for ZkProfile {
    fn default() -> Self {
        ZkProfile::Optional
    }
}

// ============================================================================
// EconomicEnvelope Structure (§3.6)
// ============================================================================

/// Economic constraints for an OFFER
///
/// Encodes fee limits and validity constraints that the Buyer must
/// accept when proceeding with the offered settlement path.
///
/// # Specification Reference
/// - TGP-00 §3.6 EconomicEnvelope Structure
///
/// # Fields
///
/// | Field | Type | Required | Description |
/// |-------|------|----------|-------------|
/// | `max_fees_bps` | u32 | ✓ | Max fees in basis points (e.g., 50 = 0.50%) |
/// | `expiry` | string? | optional | RFC3339 timestamp for offer expiry |
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::EconomicEnvelope;
///
/// let envelope = EconomicEnvelope {
///     max_fees_bps: 50, // 0.50% max fees
///     expiry: Some("2025-11-10T23:59:59Z".to_string()),
/// };
///
/// assert!(envelope.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EconomicEnvelope {
    /// Maximum acceptable total fees in basis points
    ///
    /// **Spec:** TGP-00 §3.6 - Required field
    ///
    /// **Format:** Basis points (1 bps = 0.01%)
    ///
    /// **Example:** 50 = 0.50%, 100 = 1.00%
    ///
    /// **Validation:** Must not exceed 10000 (100%)
    pub max_fees_bps: u32,

    /// RFC3339 timestamp after which the offer is invalid
    ///
    /// **Spec:** TGP-00 §3.6 - Optional field
    ///
    /// **Format:** RFC3339 (e.g., "2025-11-10T23:59:59Z")
    ///
    /// **Validation:** Must be valid RFC3339 and in the future
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
}

impl EconomicEnvelope {
    /// Validate the economic envelope
    ///
    /// # Validation Rules (per TGP-00 §3.6)
    ///
    /// - `max_fees_bps` must not exceed 10000 (100%)
    /// - `expiry` must be valid RFC3339 format if present
    ///
    /// # Errors
    ///
    /// Returns a descriptive error string if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_fees_bps > 10000 {
            return Err(format!(
                "max_fees_bps cannot exceed 10000 (100%), got: {}",
                self.max_fees_bps
            ));
        }

        // Validate RFC3339 format if expiry is present
        if let Some(ref expiry) = self.expiry {
            // Simple format check (full validation would require chrono)
            if !expiry.contains('T') || !expiry.ends_with('Z') {
                return Err(format!(
                    "expiry must be in RFC3339 format (e.g., 2025-11-10T23:59:59Z): {}",
                    expiry
                ));
            }
        }

        Ok(())
    }

    /// Create a new EconomicEnvelope with required fields
    pub fn new(max_fees_bps: u32) -> Self {
        Self {
            max_fees_bps,
            expiry: None,
        }
    }

    /// Create an EconomicEnvelope with expiry
    pub fn with_expiry(max_fees_bps: u32, expiry: impl Into<String>) -> Self {
        Self {
            max_fees_bps,
            expiry: Some(expiry.into()),
        }
    }
}

// ============================================================================
// SettleSource Enumeration (§3.7)
// ============================================================================

/// Indicates who is notifying the Controller about settlement
///
/// Used in SETTLE messages to identify the source of the settlement
/// notification for audit and trust purposes.
///
/// # Specification Reference
/// - TGP-00 §3.7 SettleSource Enumeration
///
/// # Values
///
/// | Value | Meaning |
/// |-------|---------|
/// | `buyer-notify` | Buyer (or Buyer Agent) directly reporting |
/// | `controller-watcher` | Controller's indexer observed transaction |
/// | `coreprover-indexer` | External indexer sent notification |
///
/// # Examples
///
/// ```rust
/// use tbc_core::tgp::messages::SettleSource;
///
/// let source = SettleSource::BuyerNotify;
/// assert_eq!(serde_json::to_string(&source).unwrap(), r#""buyer-notify""#);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SettleSource {
    /// Buyer (or Buyer Agent) directly reporting settlement
    ///
    /// **Trust Level:** Lowest (requires verification)
    BuyerNotify,

    /// Controller's own CoreProver indexer/watcher observed the transaction
    ///
    /// **Trust Level:** Highest (Controller verified)
    ControllerWatcher,

    /// External third-party CoreProver indexer sent notification
    ///
    /// **Trust Level:** Medium (requires verification against Controller's view)
    CoreproverIndexer,
}

impl SettleSource {
    /// Check if this source requires verification
    pub fn requires_verification(&self) -> bool {
        !matches!(self, SettleSource::ControllerWatcher)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_message_validation() {
        let valid_query = QueryMessage {
            id: "q-123".to_string(),
            from: "buyer://alice".to_string(),
            to: "seller://bob".to_string(),
            asset: "USDC".to_string(),
            amount: 1000,
            escrow_from_402: false,
            escrow_contract_from_402: None,
            zk_profile: ZkProfile::Optional,
        };

        assert!(valid_query.validate().is_ok());

        // Test empty id
        let mut invalid = valid_query.clone();
        invalid.id = "".to_string();
        assert!(invalid.validate().is_err());

        // Test zero amount
        let mut invalid = valid_query.clone();
        invalid.amount = 0;
        assert!(invalid.validate().is_err());

        // Test invalid contract address
        let mut invalid = valid_query.clone();
        invalid.escrow_contract_from_402 = Some("invalid".to_string());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_offer_message_validation() {
        let valid_offer = OfferMessage {
            id: "offer-123".to_string(),
            query_id: "q-123".to_string(),
            asset: "USDC".to_string(),
            amount: 1000,
            coreprover_contract: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
            session_id: Some("sess-123".to_string()),
            zk_required: true,
            economic_envelope: EconomicEnvelope {
                max_fees_bps: 50,
                expiry: Some("2025-11-10T23:59:59Z".to_string()),
            },
        };

        assert!(valid_offer.validate().is_ok());

        // Test invalid economic envelope
        let mut invalid = valid_offer.clone();
        invalid.economic_envelope.max_fees_bps = 20000;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_settle_message_validation() {
        let valid_settle = SettleMessage {
            id: "settle-123".to_string(),
            query_or_offer_id: "offer-123".to_string(),
            success: true,
            source: SettleSource::BuyerNotify,
            layer8_tx: Some("0x9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e".to_string()),
            session_id: Some("sess-123".to_string()),
        };

        assert!(valid_settle.validate().is_ok());

        // Test invalid tx hash
        let mut invalid = valid_settle.clone();
        invalid.layer8_tx = Some("invalid".to_string());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_error_message_validation() {
        let valid_error = ErrorMessage {
            id: "err-123".to_string(),
            code: "TIMEOUT".to_string(),
            message: "Session timed out".to_string(),
            correlation_id: Some("q-123".to_string()),
        };

        assert!(valid_error.validate().is_ok());

        // Test empty code
        let mut invalid = valid_error.clone();
        invalid.code = "".to_string();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_tgp_message_serialization() {
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

        // Should contain phase field
        assert!(json.contains(r#""phase":"QUERY""#));

        // Round-trip test
        let parsed: TGPMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(message, parsed);
    }

    #[test]
    fn test_zk_profile_serialization() {
        assert_eq!(
            serde_json::to_string(&ZkProfile::None).unwrap(),
            r#""NONE""#
        );
        assert_eq!(
            serde_json::to_string(&ZkProfile::Optional).unwrap(),
            r#""OPTIONAL""#
        );
        assert_eq!(
            serde_json::to_string(&ZkProfile::Required).unwrap(),
            r#""REQUIRED""#
        );
    }

    #[test]
    fn test_settle_source_serialization() {
        assert_eq!(
            serde_json::to_string(&SettleSource::BuyerNotify).unwrap(),
            r#""buyer-notify""#
        );
        assert_eq!(
            serde_json::to_string(&SettleSource::ControllerWatcher).unwrap(),
            r#""controller-watcher""#
        );
        assert_eq!(
            serde_json::to_string(&SettleSource::CoreproverIndexer).unwrap(),
            r#""coreprover-indexer""#
        );
    }

    #[test]
    fn test_economic_envelope_validation() {
        let valid = EconomicEnvelope {
            max_fees_bps: 50,
            expiry: Some("2025-11-10T23:59:59Z".to_string()),
        };
        assert!(valid.validate().is_ok());

        // Test excessive fees
        let invalid = EconomicEnvelope {
            max_fees_bps: 20000,
            expiry: None,
        };
        assert!(invalid.validate().is_err());

        // Test invalid expiry format
        let invalid = EconomicEnvelope {
            max_fees_bps: 50,
            expiry: Some("invalid-date".to_string()),
        };
        assert!(invalid.validate().is_err());
    }
}