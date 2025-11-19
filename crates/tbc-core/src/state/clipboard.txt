//! Main storage abstraction for TBC Core.

use crate::state::{
    SessionStore,
    MerchantStore,
};
use crate::enforcement::{
    ReservationState,
    IdempotencyState,
};

pub struct TbcStorage {
    pub sessions: SessionStore,
    pub merchants: MerchantStore,
    pub reservations: ReservationState,
    pub idempotency: IdempotencyState,
}

impl TbcStorage {
    pub fn new() -> Self {
        Self {
            sessions: SessionStore::new(),
            merchants: MerchantStore::new(),
            reservations: ReservationState::new(),
            idempotency: IdempotencyState::new(),
        }
    }
}