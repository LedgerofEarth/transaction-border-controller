//! Computes anomaly scores based on:
//! - vendor drift
//! - bytecode changes
//! - amount variance
//! - frequency variance
//! - behavioral patterns

use crate::policy::SessionKeyPolicyRecord;
use chrono::{DateTime, Utc};

pub fn compute_anomaly_score(
    policy: &SessionKeyPolicyRecord,
    amount: u128,
    now: DateTime<Utc>,
) -> u32 {
    // Placeholder until Phase 2
    let mut score = 0;

    if amount > policy.spend_limit * 2 {
        score += 50;
    }

    // TODO: add behavioral & contract-drift analysis
    score
}