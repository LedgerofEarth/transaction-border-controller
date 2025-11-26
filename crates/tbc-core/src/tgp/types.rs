// ============================================================================
// TGP Supporting Types -- v3.2 Aligned
// crates/tbc-core/src/tgp/types.rs
//
// This file intentionally contains *no gateway state*, *no OFFER semantics*,
// and no mutable fields. Everything here is deterministic, WASM-safe, portable,
// and suitable for offline validation.
//
// Aligned with TGP-00 v3.2:
//   • OFFER removed
//   • EconomicEnvelope attached to ACK(status=allow)
//   • ZkProfile retained as pure client preference
//   • DomainTrust optional for multi-gateway routing
//   • Anomaly scoring remains pure
//   • SettleSource remains pure
// ============================================================================

use serde::{Deserialize, Serialize};

// ============================================================================
// ZkProfile (§4.1 intent.mode)
// ============================================================================
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
    pub fn description(&self) -> &'static str {
        match self {
            ZkProfile::None => "Client prefers direct settlement",
            ZkProfile::Optional => "Client defers to gateway routing",
            ZkProfile::Required => "Client demands shielded settlement",
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
// EconomicEnvelope (§7) -- For ACK(status=allow)
// ============================================================================
//
// In TGP v3.2, the envelope appears *only* on ACK(status=allow),
// never on QUERY, never on OFFER (removed).
//
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
}

// ============================================================================
// SettleSource (§5.4)
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SettleSource {
    BuyerNotify,
    ControllerWatcher,
    CoreproverIndexer,
}

impl SettleSource {
    pub fn requires_verification(&self) -> bool {
        !matches!(self, SettleSource::ControllerWatcher)
    }

    pub fn trust_level(&self) -> u8 {
        match self {
            SettleSource::ControllerWatcher => 100,
            SettleSource::CoreproverIndexer => 60,
            SettleSource::BuyerNotify => 30,
        }
    }
}

// ============================================================================
// Anomaly Engine (pure, stateless)
// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalyKind {
    MissingTxHash,
    SuspiciousTxSource,
    ExpiredEnvelope,
    DomainMismatch,
    PolicyMismatch,
    UnexpectedAck,         // ← OFFER no longer exists
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalyEvent {
    pub kind: AnomalyKind,
    pub weight: u8,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalySummary {
    pub total_score: u16,
    pub events: Vec<AnomalyEvent>,
}

impl AnomalySummary {
    pub fn new() -> Self {
        Self { total_score: 0, events: vec![] }
    }

    pub fn add(&mut self, kind: AnomalyKind, weight: u8, msg: impl Into<String>) {
        self.total_score += weight as u16;
        self.events.push(AnomalyEvent {
            kind,
            weight,
            message: msg.into(),
        });
    }
}

// ============================================================================
// Optional Domain Trust (routing metadata support)
// ============================================================================
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
}