//! TBC-RESPONSE: Signed approval or rejection

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TbcDecision {
    Approve,
    Reject(TbcRejectionReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TbcResponse {
    pub decision: TbcDecision,
    pub policy_hash: String,
    pub tbc_signature: String,
    pub anomaly_score: u32,
}