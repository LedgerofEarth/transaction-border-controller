//! Store for session-key policies.

use std::collections::HashMap;
use crate::policy::SessionKeyPolicyRecord;

pub struct SessionStore {
    pub map: HashMap<String, SessionKeyPolicyRecord>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn insert(&mut self, policy: SessionKeyPolicyRecord) {
        self.map.insert(policy.key_id.clone(), policy);
    }

    pub fn get(&self, key_id: &str) -> Option<&SessionKeyPolicyRecord> {
        self.map.get(key_id)
    }
}