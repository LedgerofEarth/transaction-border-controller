// crates/tbc-core/src/tgp/policy.rs
//! TGP Policy Models (Vendor, Buyer, Session-Level)
//!
//! These structures define the policy layer used by the TBC gateway
//! to validate QUERY and OFFER messages according to TGP-00,
//! TGP-SEC-00, and MCP-AUTO-PAY-00.

use serde::{Deserialize, Serialize};
use super::types::{EconomicEnvelope, ZkProfile};

/// Vendor risk category (TGP-SEC-00 §5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    Low,
    Medium,
    High,
}

/// Policy governing Buyer preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuyerPolicy {
    /// ZK/escrow preference
    pub zk_profile: ZkProfile,

    /// Maximum amount Buyer will allow without requiring escrow
    pub max_direct_amount: u64,

    /// Maximum purchase allowed in any context
    pub max_total_amount: u64,
}

/// Policy governing vendor capabilities and constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VendorPolicy {
    /// Vendor trust/risk classification
    pub risk: RiskCategory,

    /// Whether this vendor requires buyers to use escrow
    pub require_escrow: bool,

    /// Vendor maximum allowed offer amount
    pub max_offer_amount: u64,

    /// Default economic envelope for offers
    pub economic: EconomicEnvelope,
}

/// MCP-compatible Session Key Policy (MCP-AUTO-PAY-00 §4)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionKeyPolicy {
    /// Max spendable amount under this key
    pub spend_limit: u64,

    /// Whether escrow is mandatory for this key
    pub escrow_required: bool,

    /// RFC3339 expiry for this key
    pub expiry: Option<String>,

    /// Whether ZK receipts must be generated
    pub zk_required: bool,
}

impl SessionKeyPolicy {
    pub fn is_expired(&self, now: &str) -> bool {
        match &self.expiry {
            Some(exp) => now > exp,
            None => false,
        }
    }
}