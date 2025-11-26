// crates/tbc-gateway/src/txip/coreprover_types_v03.rs
// FINAL -- CoreProver v0.3 Receipt + EscrowView (READ-ONLY)
//
// IMPORTANT:
// - This file no longer implements ANY business logic.
// - All escrow transitions occur inside CoreProverEngine.
// - EscrowView is a read-only, serialization-friendly mirror of engine state.
// - CoreProverReceipt is the ONLY canonical receipt format.

use serde::{Deserialize, Serialize};

use super::timestamp_types_v03::TripleTimestamp;
use super::blockchain_types_v03::{BuyerTxIds, ChainId, SellerTxIds};

// IMPORTANT: use the engine’s real escrow state.
// No shadow enums.
use coreprover_service::engine::EscrowState;

/// =======================================================================
/// COREPROVER RECEIPT -- CANONICAL & UNCHANGED
/// =======================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreProverReceipt {
    pub session_id: String,
    pub order_amount: u128,

    // fulfillment timestamps
    pub fulfillment_mono: u64,
    pub fulfillment_unix: u64,
    pub fulfillment_iso: String,

    // settlement timestamps
    pub settlement_mono: u64,
    pub settlement_unix: u64,
    pub settlement_iso: String,

    // discount
    pub discount_pct: u8,
    pub discount_expiration_unix: u64,

    // provenance
    pub buyer_chain_id: ChainId,
    pub buyer_commit_txid: String,

    pub seller_chain_id: ChainId,
    pub seller_accept_txid: String,
    pub seller_fulfill_txid: String,

    // settlement outcome
    pub seller_claim_txid: Option<String>,
    pub seller_refund_txid: Option<String>,

    // optional withdrawal
    pub buyer_withdraw_txid: Option<String>,

    // block height where seller fulfilled
    pub seller_block_height: u64,
}

impl CoreProverReceipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        order_amount: u128,
        fulfillment: TripleTimestamp,
        settlement: TripleTimestamp,
        discount_pct: u8,
        discount_expiration_unix: u64,
        buyer_chain_id: ChainId,
        buyer_commit_txid: String,
        seller_chain_id: ChainId,
        seller_accept_txid: String,
        seller_fulfill_txid: String,
        seller_block_height: u64,
    ) -> Self {
        Self {
            session_id,
            order_amount,

            fulfillment_mono: fulfillment.mono,
            fulfillment_unix: fulfillment.unix,
            fulfillment_iso: fulfillment.iso,

            settlement_mono: settlement.mono,
            settlement_unix: settlement.unix,
            settlement_iso: settlement.iso,

            discount_pct,
            discount_expiration_unix,

            buyer_chain_id,
            buyer_commit_txid,

            seller_chain_id,
            seller_accept_txid,
            seller_fulfill_txid,

            seller_claim_txid: None,
            seller_refund_txid: None,
            buyer_withdraw_txid: None,

            seller_block_height,
        }
    }

    pub fn with_seller_claim(mut self, txid: String) -> Self {
        self.seller_claim_txid = Some(txid);
        self
    }

    pub fn with_seller_refund(mut self, txid: String) -> Self {
        self.seller_refund_txid = Some(txid);
        self
    }

    pub fn with_buyer_withdraw(mut self, txid: String) -> Self {
        self.buyer_withdraw_txid = Some(txid);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.session_id.is_empty() {
            return Err("session_id is required".into());
        }
        if self.order_amount == 0 {
            return Err("order_amount must be > 0".into());
        }
        if self.buyer_commit_txid.is_empty() {
            return Err("buyer_commit_txid is required".into());
        }
        if self.seller_accept_txid.is_empty() {
            return Err("seller_accept_txid is required".into());
        }
        if self.seller_fulfill_txid.is_empty() {
            return Err("seller_fulfill_txid is required".into());
        }

        match (&self.seller_claim_txid, &self.seller_refund_txid) {
            (Some(_), None) | (None, Some(_)) => Ok(()),
            (Some(_), Some(_)) => Err("Cannot set BOTH seller_claim_txid AND seller_refund_txid".into()),
            (None, None) => Err("Must set EITHER seller_claim_txid OR seller_refund_txid".into()),
        }
    }

    pub fn has_discount(&self) -> bool {
        self.discount_pct > 0
    }

    pub fn seller_was_paid(&self) -> bool {
        self.seller_claim_txid.is_some()
    }

    pub fn is_cross_chain(&self) -> bool {
        self.buyer_chain_id != self.seller_chain_id
    }

    pub fn fulfillment_timestamp(&self) -> TripleTimestamp {
        TripleTimestamp::new(
            self.fulfillment_mono,
            self.fulfillment_unix,
            self.fulfillment_iso.clone(),
        )
    }

    pub fn settlement_timestamp(&self) -> TripleTimestamp {
        TripleTimestamp::new(
            self.settlement_mono,
            self.settlement_unix,
            self.settlement_iso.clone(),
        )
    }
}

/// =======================================================================
/// ESCROW VIEW -- READ-ONLY MIRROR OF ENGINE STATE
/// =======================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowView {
    pub order_id: String,
    pub session_id: String,
    pub state: EscrowState,
    pub amount: u128,
    pub asset: String,

    // engine-produced timestamps
    pub created_at: TripleTimestamp,
    pub acceptance_deadline: TripleTimestamp,
    pub fulfillment_deadline: Option<TripleTimestamp>,
    pub claim_deadline: Option<TripleTimestamp>,

    // provenance fields
    pub buyer_txids: Option<BuyerTxIds>,
    pub seller_txids: Option<SellerTxIds>,

    // discount applied by engine
    pub late_fulfilled: bool,
    pub discount_pct: u8,

    // optional times from engine
    pub fulfillment_time: Option<TripleTimestamp>,
    pub settlement_time: Option<TripleTimestamp>,

    // block height (engine-supplied)
    pub seller_block_height: Option<u64>,
}

impl EscrowView {
    /// Convert engine escrow → view snapshot.
    /// IMPORTANT: This does NOT compute ANY logic.
    pub fn from_engine(e: &coreprover_service::engine::Escrow) -> Self {
        Self {
            order_id: hex::encode(&e.order_id),
            session_id: e.session_id.clone(),
            state: e.state,
            amount: e.amount as u128,
            asset: e.asset.clone(),

            created_at: e.created_at.clone(),
            acceptance_deadline: e.acceptance_deadline.clone(),
            fulfillment_deadline: e.fulfillment_deadline.clone(),
            claim_deadline: e.claim_deadline.clone(),

            buyer_txids: e.buyer_txids.clone(),
            seller_txids: e.seller_txids.clone(),

            late_fulfilled: e.late_fulfilled,
            discount_pct: e.discount_pct,

            fulfillment_time: e.fulfillment_time.clone(),
            settlement_time: e.settlement_time.clone(),

            seller_block_height: e.seller_block_height,
        }
    }
}