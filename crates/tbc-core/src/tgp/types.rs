// ============================================================================
// TGP Supporting Types
// crates/tbc-core/src/tgp/types.rs
//
// Updated for:
//   • zk_must_verify (state.rs integration)
//   • TGP-00 v01.1 trust scoring
//   • anomaly scaffolding (future risk engine)
//   • direct compatibility with updated handlers
//   • settlement trust weighting
//
// This file intentionally contains *no logging* and no external deps
// beyond serde. Deterministic, WASM-safe, portable.
// ============================================================================

use serde::{Deserialize, Serialize};

// ============================================================================
// ZkProfile Enumeration (§3.5)
// ============================================================================

/// Buyer's zero-knowledge / CoreProver preference.
///
/// Controllers map this into:
//   • zk_must_verify (immutable per-session)
///   • settlement path selection (escrow vs direct)
///
/// NOTE: The *session* stores the actual `zk_must_verify` based on
/// policy + OFFER, not the buyer preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ZkProfile {
    #[serde(rename = "NONE")]
    None,

    #[serde(rename = "OPTIONAL")]
    Optional,

    #[serde(rename = "REQUIRED")]
    Required,
}

impl ZkProfile {
    pub fn allows_escrow(&self) -> bool {
        matches!(self, ZkProfile::Optional | ZkProfile::Required)
    }

    pub fn requires_escrow(&self) -> bool {
        matches!(self, ZkProfile::Required)
    }

    pub fn description(&self) -> &'static str {
        match self {
            ZkProfile::None => "Buyer prefers direct L8 settlement",
            ZkProfile::Optional => "Buyer defers to Controller policy",
            ZkProfile::Required => "Buyer demands CoreProver escrow",
        }
    }
}

impl Default for ZkProfile {
    fn default() -> Self {
        ZkProfile::Optional
    }
}

impl std::fmt::Display for ZkProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkProfile::None => write!(f, "NONE"),
            ZkProfile::Optional => write!(f, "OPTIONAL"),
            ZkProfile::Required => write!(f, "REQUIRED"),
        }
    }
}

// ============================================================================
// EconomicEnvelope (§3.6)
// ============================================================================

/// Defines fee + validity constraints for an OFFER.
///
/// Controllers MUST validate these before sending.
///
/// Consumers (Buyer, CoreProver, Controller) use this to validate responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EconomicEnvelope {
    pub max_fees_bps: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
}

impl EconomicEnvelope {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_fees_bps > 10000 {
            return Err(format!("max_fees_bps cannot exceed 10000, got {}", self.max_fees_bps));
        }

        if let Some(ref expiry) = self.expiry {
            if !expiry.contains('T') {
                return Err("expiry must be RFC3339 (missing 'T')".into());
            }
            if !(expiry.ends_with('Z') || expiry.contains('+') || expiry.contains('-')) {
                return Err("expiry must be RFC3339 (timezone missing)".into());
            }
        }

        Ok(())
    }

    pub fn new(max_fees_bps: u32) -> Self {
        Self {
            max_fees_bps,
            expiry: None,
        }
    }

    pub fn with_expiry(max_fees_bps: u32, expiry: impl Into<String>) -> Self {
        Self {
            max_fees_bps,
            expiry: Some(expiry.into()),
        }
    }

    pub fn max_fee_percentage(&self) -> f64 {
        self.max_fees_bps as f64 / 100.0
    }

    pub fn calculate_max_fee(&self, amount: u64) -> u64 {
        ((amount as u128 * self.max_fees_bps as u128) / 10000) as u64
    }

    pub fn is_expired(&self, now_rfc3339: &str) -> bool {
        match &self.expiry {
            Some(exp) => now_rfc3339 > exp,
            None => false,
        }
    }
}

// ============================================================================
// SettleSource (§3.7)
// ============================================================================

/// Settlement reporter identity.
///
/// Used for trust-level weighting + anomaly scoring in handlers.
///
/// ControllerWatcher = highest trust (chain-observed)
/// BuyerNotify = lowest trust (self-reported)
/// CoreproverIndexer = intermediate trust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SettleSource {
    BuyerNotify,
    ControllerWatcher,
    CoreproverIndexer,
}

impl SettleSource {
    /// Whether this settlement source requires validation.
    pub fn requires_verification(&self) -> bool {
        !matches!(self, SettleSource::ControllerWatcher)
    }

    /// Trust weighting (0–100).
    ///
    /// This is used by:
    ///   - settle handler
    ///   - anomaly scoring engine (future)
    ///   - audit/trust-report exporters (future)
    pub fn trust_level(&self) -> u8 {
        match self {
            SettleSource::ControllerWatcher => 100,
            SettleSource::CoreproverIndexer => 60,
            SettleSource::BuyerNotify => 30,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SettleSource::BuyerNotify => "Buyer-reported (unverified)",
            SettleSource::ControllerWatcher => "Controller watcher (verified)",
            SettleSource::CoreproverIndexer => "External CP indexer (partially trusted)",
        }
    }
}

impl std::fmt::Display for SettleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettleSource::BuyerNotify => write!(f, "buyer-notify"),
            SettleSource::ControllerWatcher => write!(f, "controller-watcher"),
            SettleSource::CoreproverIndexer => write!(f, "coreprover-indexer"),
        }
    }
}

// ============================================================================
// Anomaly & Trust Scaffolding (NEW)
// ============================================================================

/// Reasons a session might accumulate anomaly points.
///
/// These do *not* represent errors; they help power future MGMT APIs.
///
/// Examples:
///   • Buyer provides inconsistent domains
///   • SETTLE without tx hash from a low-trust source
///   • Significant fee deviations
///   • Conflicts with controller policy hashes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalyKind {
    MissingTxHash,
    SuspiciousTxSource,
    ExpiredEnvelope,
    DomainMismatch,
    PolicyMismatch,
    UnexpectedOffer,
}

/// Lightweight anomaly record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalyEvent {
    pub kind: AnomalyKind,
    pub weight: u8,
    pub message: String,
}

/// Output for handlers that want to return anomaly information.
/// Handlers may return `anomaly_score` inside TGP Responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalySummary {
    pub total_score: u16,
    pub events: Vec<AnomalyEvent>,
}

impl AnomalySummary {
    pub fn new() -> Self {
        Self {
            total_score: 0,
            events: vec![],
        }
    }

    pub fn add(&mut self, kind: AnomalyKind, weight: u8, msg: impl Into<String>) {
        let ev = AnomalyEvent {
            kind,
            weight,
            message: msg.into(),
        };
        self.total_score += weight as u16;
        self.events.push(ev);
    }

    pub fn is_clean(&self) -> bool {
        self.total_score == 0
    }
}

// ============================================================================
// Domain Trust (NEW)
// ============================================================================

/// Controllers may score domains to modulate routing logic.
///
/// Used in:
///   • QUERY validation
///   • OFFER generation
///   • anomaly scoring (e.g., buyer domain mismatch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainTrust {
    Unknown,
    Low,
    Medium,
    High,
}

impl DomainTrust {
    pub fn weight(&self) -> u8 {
        match self {
            DomainTrust::Unknown => 10,
            DomainTrust::Low => 25,
            DomainTrust::Medium => 60,
            DomainTrust::High => 100,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_scoring() {
        let mut a = AnomalySummary::new();
        a.add(AnomalyKind::MissingTxHash, 5, "tx missing");
        a.add(AnomalyKind::SuspiciousTxSource, 10, "low-trust source");
        assert_eq!(a.total_score, 15);
        assert_eq!(a.events.len(), 2);
    }

    #[test]
    fn test_domain_trust_weights() {
        assert_eq!(DomainTrust::Unknown.weight(), 10);
        assert_eq!(DomainTrust::Low.weight(), 25);
        assert_eq!(DomainTrust::Medium.weight(), 60);
        assert_eq!(DomainTrust::High.weight(), 100);
    }
}