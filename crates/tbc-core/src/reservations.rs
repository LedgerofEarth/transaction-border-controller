//! Reservation lifecycle: reserve spend limit on PAYMENT_REQUIRED,
//! release on failure/timeout, convert to deduction on settlement.

use chrono::{DateTime, Utc};

pub struct ReservationEntry {
    pub amount: u128,
    pub created_at: DateTime<Utc>,
    pub confirmed: bool,
}

pub struct ReservationState {
    pub entries: Vec<ReservationEntry>,
}

impl ReservationState {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn reserve(&mut self, amount: u128) {
        self.entries.push(ReservationEntry {
            amount,
            created_at: Utc::now(),
            confirmed: false,
        });
    }
}