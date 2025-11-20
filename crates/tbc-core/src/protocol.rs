//! # TGP Protocol Definitions (Spec-Pure)
//!
//! **Destination:** `crates/tbc-core/src/tgp/protocols.rs`
//!
//! This module contains *only* the core Transaction Gateway Protocol (TGP-00)
//! message definitions, semantic validation rules, and enumerations.
//!
//! ## Architectural Note (RFC 3261 Alignment)
//!
//! Following the SIP architecture from RFC 3261:
//!
//! - **Core protocol grammar** lives in the SIP spec (RFC 3261 §7, §20)
//! - **Transaction/state logic** lives separately (RFC 3261 §17)
//! - **Transport encode/decode** lives separately (RFC 3261 §18)
//!
//! TGP mirrors this separation:
//!
//! | SIP (RFC 3261) Section | Equivalent TGP Component |
//! |------------------------|--------------------------|
//! | Core message syntax (§7, §20) | `protocols.rs` (this file) |
//! | Transaction layer (§17) | `state.rs` |
//! | Transport/encoding (§18) | `codec_tx.rs` |
//!
//! Thus, no routing, envelope, metadata, codec, or session binding logic
//! appears here. This file is deliberately "spec-pure."

use serde::{Deserialize, Serialize};
use crate::protocol::make_protocol_error;
use crate::protocol::ErrorMessage;
use crate::tgp::types::{ZkProfile, EconomicEnvelope, SettleSource};
use crate::tgp::validation::{
    validate_non_empty,
    validate_positive_amount,
    validate_address,
    validate_transaction_hash,
};

//
// ============================================================================
// TGPMessage Discriminated Union (§3.8)
// ============================================================================
//

/// TGP message discriminator (JSON `phase` tag).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "phase")]
pub enum TGPMessage {
    #[serde(rename = "QUERY")]
    Query(QueryMessage),

    #[serde(rename = "OFFER")]
    Offer(OfferMessage),

    #[serde(rename = "SETTLE")]
    Settle(SettleMessage),

    #[serde(rename = "ERROR")]
    Error(ErrorMessage),
}

impl TGPMessage {
    /// Get the canonical message ID, required across all phases.
    pub fn id(&self) -> &str {
        match self {
            TGPMessage::Query(m) => &m.id,
            TGPMessage::Offer(m) => &m.id,
            TGPMessage::Settle(m) => &m.id,
            TGPMessage::Error(m) => &m.id,
        }
    }

    /// Validate the underlying message (semantic rules).
    pub fn validate(&self) -> Result<(), String> {
        match self {
            TGPMessage::Query(m) => m.validate(),
            TGPMessage::Offer(m) => m.validate(),
            TGPMessage::Settle(m) => m.validate(),
            TGPMessage::Error(m) => m.validate(),
        }
    }

    /// Convenience: "QUERY", "OFFER", "SETTLE", "ERROR".
    pub fn phase(&self) -> &str {
        match self {
            TGPMessage::Query(_) => "QUERY",
            TGPMessage::Offer(_) => "OFFER",
            TGPMessage::Settle(_) => "SETTLE",
            TGPMessage::Error(_) => "ERROR",
        }
    }
}

//
// ============================================================================
// QUERY Message (§3.1)
// ============================================================================
//

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub asset: String,
    pub amount: u64,

    pub escrow_from_402: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escrow_contract_from_402: Option<String>,

    pub zk_profile: ZkProfile,
}

impl QueryMessage {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.from, "from")?;
        validate_non_empty(&self.to, "to")?;
        validate_non_empty(&self.asset, "asset")?;
        validate_positive_amount(self.amount, "amount")?;

        if let Some(ref addr) = self.escrow_contract_from_402 {
            validate_address(addr, "escrow_contract_from_402")?;
        }

        Ok(())
    }

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

//
// ============================================================================
// OFFER Message (§3.2)
// ============================================================================
//

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfferMessage {
    pub id: String,
    pub query_id: String,
    pub asset: String,
    pub amount: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreprover_contract: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    pub zk_required: bool,
    pub economic_envelope: EconomicEnvelope,
}

impl OfferMessage {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.query_id, "query_id")?;
        validate_non_empty(&self.asset, "asset")?;
        validate_positive_amount(self.amount, "amount")?;

        if let Some(ref addr) = self.coreprover_contract {
            validate_address(addr, "coreprover_contract")?;
        }

        self.economic_envelope.validate()?;
        Ok(())
    }

    pub fn new(
        id: impl Into<String>,
        query_id: impl Into<String>,
        asset: impl Into<String>,
        amount: u64,
        zk_required: bool,
        econ: EconomicEnvelope,
    ) -> Self {
        Self {
            id: id.into(),
            query_id: query_id.into(),
            asset: asset.into(),
            amount,
            coreprover_contract: None,
            session_id: None,
            zk_required,
            economic_envelope: econ,
        }
    }
}

//
// ============================================================================
// SETTLE Message (§3.3)
// ============================================================================
//

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettleMessage {
    pub id: String,
    pub query_or_offer_id: String,
    pub success: bool,
    pub source: SettleSource,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer8_tx: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl SettleMessage {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.query_or_offer_id, "query_or_offer_id")?;

        if let Some(ref tx) = self.layer8_tx {
            validate_transaction_hash(tx, "layer8_tx")?;
        }

        Ok(())
    }

    pub fn new(
        id: impl Into<String>,
        q_or_o: impl Into<String>,
        success: bool,
        source: SettleSource,
    ) -> Self {
        Self {
            id: id.into(),
            query_or_offer_id: q_or_o.into(),
            success,
            source,
            layer8_tx: None,
            session_id: None,
        }
    }
}

//
// ============================================================================
// ERROR Message (§3.4)
// ============================================================================
//

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    pub id: String,
    pub code: String,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl ErrorMessage {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.code, "code")?;
        validate_non_empty(&self.message, "message")?;
        Ok(())
    }

    pub fn new(id: impl Into<String>, code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            message: msg.into(),
            correlation_id: None,
        }
    }

    pub fn with_correlation(
        id: impl Into<String>,
        code: impl Into<String>,
        msg: impl Into<String>,
        corr: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            message: msg.into(),
            correlation_id: Some(corr.into()),
        }
    }
}
// ============================================================================
// Protocol Error Construction Helper (§3.4)
// ============================================================================
//
// This helper produces fully-formed, protocol-compliant ErrorMessage structs.
//
// It is used by:
//   • codec_tx.rs   -- JSON parsing errors, replay detection, decoding failures
//   • inbound router -- session lookup failures, handler dispatch failures
//   • handlers/*    -- validation failures, policy violations, settlement failures
//
// Behavior:
//   • Always generates a fresh UUIDv4 for the error ID
//   • Automatically applies correlation_id when provided
//   • Produces errors that pass ErrorMessage::validate()
//   • Keeps ErrorMessage pure and deterministic
//
// This function is intentionally placed in protocol.rs so the codec, router,
// and handler layers can depend on it without creating a circular dependency.

use uuid::Uuid;

/// Create a TGP protocol error with automatic UUID generation.
///
/// # Examples
///
/// ```rust
/// // Standalone error
/// let e = make_protocol_error(None,
///                             "INVALID_JSON",
///                             "Malformed payload");
///
/// // Correlated error
/// let e = make_protocol_error(
///         Some("q-123".to_string()),
///         "UNSUPPORTED_ASSET",
///         "DOGE not supported");
/// ```
pub fn make_protocol_error(
    correlation_id: Option<String>,
    code: impl Into<String>,
    message: impl Into<String>,
) -> ErrorMessage {
    let error_id = Uuid::new_v4().to_string();

    match correlation_id {
        Some(cid) => ErrorMessage::with_correlation(
            error_id,
            code.into(),
            message.into(),
            cid,
        ),
        None => ErrorMessage::new(
            error_id,
            code.into(),
            message.into(),
        ),
    }
}