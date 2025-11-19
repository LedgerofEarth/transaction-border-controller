// crates/tbc-core/src/tgp/decision.rs
//! TGP Decision & Enforcement Layer
//!
//! These enums model the output of TGP validation, including
//! allow/deny decisions, anomaly classifications, and rejection reasons.

use serde::{Deserialize, Serialize};

/// High-level gateway decision (TGP-SEC-00 §12)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TBCDecision {
    /// Fully approved
    Approve,

    /// Must use escrow (CoreProver) before proceeding
    EscrowRequired,

    /// Requires manual or automated escalation
    Escalate,

    /// Rejected due to policy or validation failure
    Reject(RejectReason),

    /// Approved but with anomaly indicators
    Anomaly(AnomalyType),
}

/// Reasons for rejection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RejectReason {
    InvalidMessage,
    UnauthorizedMerchant,
    PolicyViolation,
    ZkRequiredButMissing,
    RiskTooHigh,
    EconomicEnvelopeExceeded,
    ExpiredOffer,
    Timeout,
    InvalidState,
    InternalError(String),
}

/// Anomaly types per TGP-SEC-00 §9
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    ReplayDetected,
    ConflictingOffer,
    DoubleAccept,
    DoubleSettle,
    InvalidStateTransition,
    SuspiciousSettlementSource,
}

/// Result of policy+state+message validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TGPValidationResult {
    pub decision: TBCDecision,
    pub details: Option<String>,
}

impl TGPValidationResult {
    pub fn ok() -> Self {
        Self { decision: TBCDecision::Approve, details: None }
    }

    pub fn reject(reason: RejectReason) -> Self {
        Self { decision: TBCDecision::Reject(reason), details: None }
    }

    pub fn anomaly(a: AnomalyType) -> Self {
        Self { decision: TBCDecision::Anomaly(a), details: None }
    }
}