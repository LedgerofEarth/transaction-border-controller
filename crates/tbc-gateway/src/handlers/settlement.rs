use tbc_core::state::TbcStorage;

pub struct SettlementHandler<'a> {
    pub storage: &'a mut TbcStorage,
}

impl<'a> SettlementHandler<'a> {
    pub fn handle_success(&mut self) {
        self.storage.reservations.settle_oldest();
    }

    pub fn handle_failure(&mut self) {
        self.storage.reservations.cleanup();
    }
}