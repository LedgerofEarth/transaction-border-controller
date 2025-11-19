//! Computes anomaly scores based on:
//! - amount drift
//! - frequency timing
//! - contract mismatch flags
//! - behavioral variance

use crate::policy::SessionKeyPolicyRecord;
use chrono::{DateTime, Utc};

/// Simple scoring model:
/// - +40 for >2x spend limit
/// - +20 for >1x spend limit
/// - +10 if frequency window is extremely tight
/// - +20 if any drift flag is set (future extension)

pub fn compute_anomaly_score(
    policy: &SessionKeyPolicyRecord,
    amount: u128,
    now: DateTime<Utc>,
) -> u32 {
    let mut score = 0;

    // Spend drift
    if amount > policy.spend_limit * 2 {
        score += 40;
    } else if amount > policy.spend_limit {
        score += 20;
    }

    // Frequency drift (if hitting immediately after last window)
    if let Some(last) = policy.frequency_window.last_use {
        let delta = (now - last).num_seconds().abs();
        if delta < 10 {
            score += 10;
        }
    }

    // Placeholder for contract-drift, bytecode changes, etc.
    // Real logic inserted later.
    // if policy.contract_changed_flag { score += 20; }

    score
}