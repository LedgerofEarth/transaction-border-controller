//! Prevents replay or duplicate transaction attempts.
//!
//! Tracks idempotency keys and expiration windows.

use std::collections::HashMap;
use chrono::{Utc, DateTime, Duration};

pub struct IdempotencyState {
    pub seen: HashMap<String, DateTime<Utc>>,
}

impl IdempotencyState {
    pub fn new() -> Self {
        Self { seen: HashMap::new() }
    }

    pub fn is_duplicate(&self, key: &str) -> bool {
        self.seen.contains_key(key)
    }

    pub fn record(&mut self, key: &str) {
        self.seen.insert(key.to_string(), Utc::now());
    }

    pub fn cleanup(&mut self) {
        let cutoff = Utc::now() - Duration::days(1);
        self.seen.retain(|_, ts| *ts > cutoff);
    }
}