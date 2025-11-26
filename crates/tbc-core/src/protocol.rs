//! TGP-00 v3.2 -- Integrated Runtime-Aware Protocol Engine
//! ------------------------------------------------------
//! This file contains:
//!   • Canonical message definitions (spec-pure)
//!   • Deterministic runtime validation
//!   • Routing normalization
//!   • ACK construction helpers (offer / allow / deny / revise)
//!   • Terminal SETTLE message construction
//!   • Protocol-compliant error builder
//!
//! Mirrors SIP separation of concerns but includes runtime validation
//! because TBC must produce deterministic ACKs and SETTLE events.

use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::tgp::types::{EconomicEnvelope};
use crate::tgp::validation::{
    validate_non_empty,
    validate_address,
    validate_positive_amount,
};

// -----------------------------------------------------------------------------
// 0. Canonical Enumerations
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TGPVerb {
    COMMIT,
    PAY,
    CLAIM,
    WITHDRAW,
    QUOTE,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TGPParty {
    BUYER,
    SELLER,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TGPMODE {
    DIRECT,
    SHIELDED,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AckStatus {
    /// Preview: not executable
    Offer,
    /// Executable envelope included
    Allow,
    /// Rejected
    Deny,
    /// Needs modification
    Revise,
}

// -----------------------------------------------------------------------------
// 1. Intent Structure
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Intent {
    pub verb: TGPVerb,
    pub party: TGPParty,
    pub mode: TGPMODE,
}

// -----------------------------------------------------------------------------
// 2. Routing Metadata
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_area_id: Option<String>,

    #[serde(default)]
    pub path: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_gateway: Option<String>,
}

// -----------------------------------------------------------------------------
// 3. QUERY -- Transaction Intent
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryMessage {
    #[serde(rename = "type")]
    pub msg_type: String,

    pub tgp_version: String,
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegated_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<serde_json::Value>,

    #[serde(default)]
    pub routing: RoutingMetadata,

    pub intent: Intent,

    pub payment_profile: String,
    pub amount: u64,
    pub chain_id: u64,

    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl QueryMessage {
    pub fn validate(&self) -> Result<(), String> {
        if self.msg_type != "QUERY" {
            return Err("QUERY.type must equal \"QUERY\"".into());
        }
        if self.tgp_version != "3.2" {
            return Err("Unsupported TGP version (must be 3.2)".into());
        }

        validate_non_empty(&self.id, "id")?;
        validate_address(&self.payment_profile, "payment_profile")?;
        validate_positive_amount(self.amount, "amount")?;

        Ok(())
    }

    /// Normalizes routing.path by appending TAID if missing.
    pub fn normalize_routing(&mut self) {
        if let Some(ref taid) = self.routing.transaction_area_id {
            if self.routing.path.is_empty() {
                self.routing.path = vec![taid.clone()];
            } else if self.routing.path.last() != Some(taid) {
                self.routing.path.push(taid.clone());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// 4. ACK -- Deterministic Gateway Response
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AckMessage {
    #[serde(rename = "type")]
    pub msg_type: String,

    pub status: AckStatus,
    pub id: String,

    pub intent: Intent,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx: Option<EconomicEnvelope>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

// ----------------------------
// ACK Constructors
// ----------------------------

impl AckMessage {
    pub fn offer_for(query: &QueryMessage) -> Self {
        AckMessage {
            msg_type: "ACK".into(),
            status: AckStatus::Offer,
            id: query.id.clone(),
            intent: query.intent.clone(),
            routing: Some(query.routing.clone()),
            tx: None,
            expires_at: None,
        }
    }

    pub fn allow_for(
        query: &QueryMessage,
        envelope: EconomicEnvelope,
        expires_at: String,
    ) -> Self {
        AckMessage {
            msg_type: "ACK".into(),
            status: AckStatus::Allow,
            id: query.id.clone(),
            intent: query.intent.clone(),
            routing: Some(query.routing.clone()),
            tx: Some(envelope),
            expires_at: Some(expires_at),
        }
    }

    pub fn deny_for(query: &QueryMessage, reason: &str) -> Self {
        AckMessage {
            msg_type: "ACK".into(),
            status: AckStatus::Deny,
            id: query.id.clone(),
            intent: query.intent.clone(),
            routing: Some(query.routing.clone()),
            tx: None,
            expires_at: None,
        }
    }

    pub fn revise_for(query: &QueryMessage, reason: &str) -> Self {
        AckMessage {
            msg_type: "ACK".into(),
            status: AckStatus::Revise,
            id: query.id.clone(),
            intent: query.intent.clone(),
            routing: Some(query.routing.clone()),
            tx: None,
            expires_at: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.msg_type != "ACK" {
            return Err("ACK.type must equal \"ACK\"".into());
        }

        if self.status == AckStatus::Allow && self.tx.is_none() {
            return Err("ACK.status=allow requires tx".into());
        }

        if self.status == AckStatus::Offer && self.tx.is_some() {
            return Err("ACK.status=offer MUST NOT include tx".into());
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 5. ERROR -- Protocol Failure
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    #[serde(rename = "type")]
    pub msg_type: String,

    pub id: String,
    pub code: String,
    pub layer_failed: u8,
    pub message: String,
}

pub fn make_protocol_error(
    layer: u8,
    code: impl Into<String>,
    message: impl Into<String>,
) -> ErrorMessage {
    ErrorMessage {
        msg_type: "ERROR".into(),
        id: Uuid::new_v4().to_string(),
        code: code.into(),
        layer_failed: layer,
        message: message.into(),
    }
}

// -----------------------------------------------------------------------------
// 6. SETTLE -- Terminal State
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettleResult {
    pub final_status: String,
    pub escrow_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettleMessage {
    #[serde(rename = "type")]
    pub msg_type: String,

    pub id: String,
    pub result: SettleResult,

    pub timestamp: String,
}

impl SettleMessage {
    pub fn terminal(
        id: impl Into<String>,
        final_status: impl Into<String>,
        escrow_id: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> Self {
        SettleMessage {
            msg_type: "SETTLE".into(),
            id: id.into(),
            result: SettleResult {
                final_status: final_status.into(),
                escrow_id: escrow_id.into(),
            },
            timestamp: timestamp.into(),
        }
    }
}