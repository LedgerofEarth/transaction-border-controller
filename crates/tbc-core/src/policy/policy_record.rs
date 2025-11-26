//! Canonical record stored in TBC for each session key.
// NOTE (TGP-00 v3.2):
// Gateways MUST remain stateless. The TBC MUST NOT update or persist
// `last_use`. This field is merchant-provided policy metadata only.
//
// Enforcing frequency-window logic must occur:
//   • on the merchant side, OR
//   • via external attestation, OR
//   • via delegated-key constraints on the client.
// The TBC MAY read the field but MUST NOT mutate it.
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyWindow {
    pub last_use: Option<DateTime<Utc>>,
    pub min_interval_secs: u64,
}

impl FrequencyWindow {
    pub fn permits(&self, now: DateTime<Utc>) -> bool {
        match self.last_use {
            Some(ts) => (now - ts).num_seconds() >= self.min_interval_secs as i64,
            None => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKeyPolicyRecord {
    pub key_id: String,
    pub vendor: String,
    pub spend_limit: u128,
    pub chain: u64,
    pub anomaly_threshold: u32,
    pub frequency_window: FrequencyWindow,
    pub expires_at: Option<DateTime<Utc>>,
    pub function_selector: Option<String>,
    pub policy_hash: String,
}