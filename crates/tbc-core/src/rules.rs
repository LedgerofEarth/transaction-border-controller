//! Enforcement rules: spend limits, frequency, chain, expiration,
//! session-key constraints.

use crate::policy::SessionKeyPolicyRecord;
use crate::state::{ReservationState, IdempotencyState};
use crate::types::{TbcDecision, TbcRejectionReason, TbcResponse};
use crate::enforcement::{compute_anomaly_score};

use chrono::{DateTime, Utc};

pub struct EnforcementContext<'a> {
    pub policy: &'a SessionKeyPolicyRecord,
    pub reservation_state: &'a mut ReservationState,
    pub idempotency: &'a mut IdempotencyState,
    pub now: DateTime<Utc>,
}

impl<'a> EnforcementContext<'a> {
    pub fn evaluate(
        &mut self,
        amount: u128,
        idempotency_key: &str,
    ) -> TbcDecision {

        // 1. Idempotency
        if self.idempotency.is_duplicate(idempotency_key) {
            return TbcDecision::Reject(TbcRejectionReason::DuplicateIdempotencyKey);
        }

        // 2. Spend limit
        if amount > self.policy.spend_limit {
            return TbcDecision::Reject(TbcRejectionReason::SpendLimitExceeded);
        }

        // 3. Frequency
        if !self.policy.frequency_window.permits(self.now) {
            return TbcDecision::Reject(TbcRejectionReason::FrequencyExceeded);
        }

        // 4. Anomaly score
        let anomaly = compute_anomaly_score(self.policy, amount, self.now);
        if anomaly > self.policy.anomaly_threshold {
            return TbcDecision::Reject(TbcRejectionReason::AnomalyDetected);
        }

        // Passed all checks
        TbcDecision::Approve
    }
}