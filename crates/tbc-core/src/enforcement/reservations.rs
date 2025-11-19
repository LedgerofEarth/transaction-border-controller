//! Reservation lifecycle: reserve -> settle -> expire.

use chrono::{DateTime, Utc, Duration};

pub struct ReservationEntry {
    pub amount: u128,
    pub created_at: DateTime<Utc>,
    pub confirmed: bool,
}

pub struct ReservationState {
    pub entries: Vec<ReservationEntry>,
    pub timeout_secs: u64,
}

impl ReservationState {
    pub fn new() -> Self {
        Self {
            entries: vec![],
            timeout_secs: 600, // default 10 min
        }
    }

    pub fn reserve(&mut self, amount: u128) {
        self.entries.push(ReservationEntry {
            amount,
            created_at: Utc::now(),
            confirmed: false,
        });
    }

    /// Called when settlement event received
    pub fn settle_oldest(&mut self) {
        for ent in self.entries.iter_mut() {
            if !ent.confirmed {
                ent.confirmed = true;
                break;
            }
        }
    }

    /// Clears expired pending reservations
    pub fn cleanup(&mut self) {
        let cutoff = Utc::now() - Duration::seconds(self.timeout_secs as i64);
        self.entries.retain(|e| e.confirmed || e.created_at > cutoff);
    }

    /// Total reserved amount (optional helper)
    pub fn total_reserved(&self) -> u128 {
        self.entries.iter().map(|e| e.amount).sum()
    }
}