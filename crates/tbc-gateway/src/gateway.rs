//! In-Memory TBC Gateway
//!
//! This provides:
//! - handle_tgp_query()
//! - handle_settlement_success()
//! - handle_settlement_failure()

use tbc_core::{
    state::TbcStorage,
    types::TbcResponse,
};
use crate::handlers::{
    tgp_query::TgpQueryHandler,
    settlement::SettlementHandler,
};
use crate::signing::tbc_signer::TbcSigner;

use tgp::messages::TgpQuery;

pub struct TbcGateway {
    pub storage: TbcStorage,
    pub signer: TbcSigner,
}

impl TbcGateway {
    pub fn new(private_key: Vec<u8>) -> Self {
        Self {
            storage: TbcStorage::new(),
            signer: TbcSigner::new(private_key),
        }
    }

    /// Main entrypoint for the autopay enforcement flow.
    pub fn handle_tgp_query(&mut self, query: TgpQuery) -> TbcResponse {
        let mut handler = TgpQueryHandler {
            storage: &mut self.storage,
            signer: &self.signer,
        };

        handler.handle(query)
    }

    /// Settlement success → mark reservation "confirmed".
    pub fn handle_settlement_success(&mut self) {
        let mut handler = SettlementHandler { storage: &mut self.storage };
        handler.handle_success();
    }

    /// Settlement failure → clean expired entries.
    pub fn handle_settlement_failure(&mut self) {
        let mut handler = SettlementHandler { storage: &mut self.storage };
        handler.handle_failure();
    }
}