//! Hash the policy to detect merchant-policy mismatch.

use crate::policy::SessionKeyPolicyRecord;
use sha2::{Sha256, Digest};

pub fn compute_policy_hash(policy: &SessionKeyPolicyRecord) -> String {
    let json = serde_json::to_string(policy).unwrap();
    let hash = Sha256::digest(json.as_bytes());
    format!("0x{}", hex::encode(hash))
}