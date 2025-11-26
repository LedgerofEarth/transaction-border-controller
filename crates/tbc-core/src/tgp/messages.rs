// ============================================================================
// TGP Message Type Definitions -- TGP-00 v3.2
// crates/tbc-core/src/tgp/messages.rs
//
// OFFER → REMOVED
// ACK   → NEW (replaces OFFER)
// ============================================================================

use serde::{Deserialize, Serialize};

use super::types::{SettleSource, ZkProfile};
use super::validation::{
    validate_address,
    validate_non_empty,
    validate_positive_amount,
    validate_transaction_hash,
};


// ============================================================================
// Message Union (§3.8) -- Updated for TGP-00 v3.2
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "phase")]
pub enum TGPMessage {

    /// QUERY message -- initiates the transaction request
    #[serde(rename = "QUERY")]
    Query(QueryMessage),

    /// ACK message -- controller "yes/no + profile/constraints" response
    #[serde(rename = "ACK")]
    Ack(AckMessage),

    /// SETTLE message -- reports settlement completion
    #[serde(rename = "SETTLE")]
    Settle(SettleMessage),

    /// ERROR message -- protocol-level failure
    #[serde(rename = "ERROR")]
    Error(ErrorMessage),
}

impl TGPMessage {
    pub fn id(&self) -> &str {
        match self {
            TGPMessage::Query(m) => &m.id,
            TGPMessage::Ack(m)   => &m.id,
            TGPMessage::Settle(m) => &m.id,
            TGPMessage::Error(m) => &m.id,
        }
    }

    pub fn phase(&self) -> &str {
        match self {
            TGPMessage::Query(_)  => "QUERY",
            TGPMessage::Ack(_)    => "ACK",
            TGPMessage::Settle(_) => "SETTLE",
            TGPMessage::Error(_)  => "ERROR",
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self {
            TGPMessage::Query(m)  => m.validate(),
            TGPMessage::Ack(m)    => m.validate(),
            TGPMessage::Settle(m) => m.validate(),
            TGPMessage::Error(m)  => m.validate(),
        }
    }
}



// ============================================================================
// QUERY Message (§3.1)
// ============================================================================

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

        if let Some(ref c) = self.escrow_contract_from_402 {
            validate_address(c, "escrow_contract_from_402")?;
        }

        Ok(())
    }
}



// ============================================================================
// ACK Message (§3.2, replaces OFFER)
// ============================================================================
//
// ACK is a "routing permission + capability" message.
// It is NOT economic negotiation.
// It has no economic envelope.
// No fee curves.
// No OFFER semantics.
//
// Controller answers:
//   • allow = true/false
//   • escrow_required = true/false
//   • optional session_id
//   • optional coreprover_contract
//   • optional recommended zk_profile
//

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AckMessage {
    /// Unique ACK identifier ("ack-xxxx")
    pub id: String,

    /// Must reference a QUERY id ("q-xxxx")
    pub query_id: String,

    /// Whether controller permits moving forward
    pub allow: bool,

    /// Controller signal whether escrow/ZK path is required
    pub escrow_required: bool,

    /// Optional CoreProver destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreprover_contract: Option<String>,

    /// Optional controller-selected session_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Optional profile override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zk_profile: Option<ZkProfile>,
}

impl AckMessage {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty(&self.id, "id")?;
        validate_non_empty(&self.query_id, "query_id")?;

        if let Some(ref c) = self.coreprover_contract {
            validate_address(c, "coreprover_contract")?;
        }

        Ok(())
    }
}



// ============================================================================
// SETTLE Message (§3.3)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettleMessage {
    pub id: String,

    /// **v3.2 NOTE:** references QUERY only
    pub query_id: String,

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
        validate_non_empty(&self.query_id, "query_id")?;

        if let Some(ref tx) = self.layer8_tx {
            validate_transaction_hash(tx, "layer8_tx")?;
        }

        Ok(())
    }
}



// ============================================================================
// ERROR Message (§3.4)
// ============================================================================

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
}



// ============================================================================
// Tests (Updated)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tgp::types::ZkProfile;

    #[test]
    fn test_query_validation() {
        let q = QueryMessage {
            id: "q-1".into(),
            from: "buyer://alice".into(),
            to: "seller://bob".into(),
            asset: "USDC".into(),
            amount: 1000,
            escrow_from_402: false,
            escrow_contract_from_402: None,
            zk_profile: ZkProfile::Optional,
        };
        assert!(q.validate().is_ok());
    }

    #[test]
    fn test_ack_validation() {
        let a = AckMessage {
            id: "ack-1".into(),
            query_id: "q-1".into(),
            allow: true,
            escrow_required: false,
            coreprover_contract: None,
            session_id: None,
            zk_profile: None,
        };
        assert!(a.validate().is_ok());
    }

    #[test]
    fn test_settle_validation() {
        let s = SettleMessage {
            id: "settle-1".into(),
            query_id: "q-1".into(),
            success: true,
            source: SettleSource::BuyerNotify,
            layer8_tx: None,
            session_id: None,
        };
        assert!(s.validate().is_ok());
    }

    #[test]
    fn test_error_validation() {
        let e = ErrorMessage {
            id: "err-1".into(),
            code: "TIMEOUT".into(),
            message: "timed out".into(),
            correlation_id: None,
        };
        assert!(e.validate().is_ok());
    }
}