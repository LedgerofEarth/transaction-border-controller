//! Enforcement rules: spend limit, frequency, function selector,
//! expiration, chain match, policy hash match, anomaly scoring,
//! reservation lifecycle, and idempotency protection.

use crate::policy::SessionKeyPolicyRecord;
use crate::enforcement::{
    compute_anomaly_score,
};
use crate::state::{
    ReservationState,
    IdempotencyState,
};
use crate::types::{
    TbcDecision,
    TbcRejectionReason,
};
use chrono::{DateTime, Utc};

pub struct EnforcementContext<'a> {
    pub policy: &'a SessionKeyPolicyRecord,
    pub reservations: &'a mut ReservationState,
    pub idempotency: &'a mut IdempotencyState,
    pub now: DateTime<Utc>,
}

impl<'a> EnforcementContext<'a> {
    pub fn evaluate(
        &mut self,
        amount: u128,
        chain: u64,
        function_selector: Option<String>,
        policy_hash_from_coreprove: Option<String>,
        merchant_policy_hash: Option<String>,
        idempotency_key: &str,
    ) -> TbcDecision {

        // =============================
        // 0. EXPIRED KEY?
        // =============================
        if let Some(exp) = self.policy.expires_at {
            if self.now > exp {
                return TbcDecision::Reject(TbcRejectionReason::SessionKeyExpired);
            }
        }

        // =============================
        // 1. CHAIN MATCH?
        // =============================
        if chain != self.policy.chain {
            return TbcDecision::Reject(TbcRejectionReason::ChainMismatch);
        }

        // =============================
        // 2. FUNCTION SELECTOR MATCH?
        // =============================
        if let Some(ref required) = self.policy.function_selector {
            if let Some(ref actual) = function_selector {
                if actual != required {
                    return TbcDecision::Reject(TbcRejectionReason::FunctionSelectorMismatch);
                }
            } else {
                return TbcDecision::Reject(TbcRejectionReason::FunctionSelectorMismatch);
            }
        }

        // =============================
        // 3. POLICY HASH MATCH?
        // =============================
        if let (Some(cp), Some(mp)) = (policy_hash_from_coreprove, merchant_policy_hash) {
            if cp != mp {
                return TbcDecision::Reject(TbcRejectionReason::PolicyHashMismatch);
            }
        }

        // =============================
        // 4. IDEMPOTENCY CHECK
        // =============================
        if self.idempotency.is_duplicate(idempotency_key) {
            return TbcDecision::Reject(TbcRejectionReason::DuplicateIdempotencyKey);
        }

        // =============================
        // 5. SPEND LIMIT
        // =============================
        if amount > self.policy.spend_limit {
            return TbcDecision::Reject(TbcRejectionReason::SpendLimitExceeded);
        }

        // =============================
        // 6. FREQUENCY WINDOW
        // =============================
        if !self.policy.frequency_window.permits(self.now) {
            return TbcDecision::Reject(TbcRejectionReason::FrequencyExceeded);
        }

        // =============================
        // 7. ANOMALY SCORE
        // =============================
        let anomaly = compute_anomaly_score(self.policy, amount, self.now);
        if anomaly > self.policy.anomaly_threshold {
            return TbcDecision::Reject(TbcRejectionReason::AnomalyDetected);
        }

        // =============================
        // 8. RESERVATION ("Pending settlement")
        // =============================
        self.reservations.reserve(amount);

        // =============================
        // 9. IDEMPOTENCY RECORD
        // =============================
        self.idempotency.record(idempotency_key);

        TbcDecision::Approve
    }
}