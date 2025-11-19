//! Rejection reasons for TBC enforcement.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TbcRejectionReason {
    SpendLimitExceeded,
    FrequencyExceeded,
    AnomalyDetected,
    DuplicateIdempotencyKey,
    SessionKeyExpired,
    VendorMismatch,
    PolicyHashMismatch,
    ContractDriftDetected,
    ChainMismatch,
}