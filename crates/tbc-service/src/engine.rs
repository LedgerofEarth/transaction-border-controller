// ============================================================================
// CoreProver Engine (v0.3) — Borrow-Checker-Clean Version
// ============================================================================

use crate::types::*;
use chrono;
use std::fmt;

// ============================================================================
// TimeTruth: deterministic triple-clock model
// ============================================================================

#[derive(Debug, Clone)]
pub struct TimeTruth {
    pub mono: u64,
    pub unix: u64,
    pub iso: String,
}

impl TimeTruth {
    pub fn new(mono: u64, unix: u64) -> Self {
        let iso = iso8601(unix);
        Self { mono, unix, iso }
    }
}

// ============================================================================
// CoreProverEngine
// ============================================================================

pub struct CoreProverEngine {
    escrows: Vec<Escrow>,
    receipts: Vec<ReceiptMetadata>,
    next_session_counter: u64,

    // deterministic clocks
    current_mono: u64,
    current_unix: u64,

    // blockchain params
    pub chain_id: u64,
    pub block_interval_secs: u64,
    pub current_block_height: u64,
}

impl CoreProverEngine {
    pub fn new(chain_id: u64, block_interval_secs: u64, genesis_unix: u64) -> Self {
        Self {
            escrows: Vec::new(),
            receipts: Vec::new(),
            next_session_counter: 1,
            current_mono: 0,
            current_unix: genesis_unix,
            chain_id,
            block_interval_secs,
            current_block_height: 1,
        }
    }

    // ------------------------------------------------------------------------
    // Time Advancement
    // ------------------------------------------------------------------------

    pub fn advance_time(&mut self, secs: u64) {
        self.current_mono += secs;
        self.current_unix += secs;
        self.current_block_height += secs / self.block_interval_secs;
    }

    fn now(&self) -> TimeTruth {
        TimeTruth::new(self.current_mono, self.current_unix)
    }

    // ------------------------------------------------------------------------
    // Escrow Lookup Helpers
    // ------------------------------------------------------------------------

    fn get_escrow(&self, order_id: &[u8; 32]) -> Result<&Escrow, String> {
        self.escrows
            .iter()
            .find(|e| &e.order_id == order_id)
            .ok_or_else(|| "Escrow not found".to_string())
    }

    fn get_escrow_mut(&mut self, order_id: &[u8; 32]) -> Result<&mut Escrow, String> {
        self.escrows
            .iter_mut()
            .find(|e| &e.order_id == order_id)
            .ok_or_else(|| "Escrow not found".to_string())
    }

    fn generate_order_id(&mut self) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = (self.next_session_counter & 0xFF) as u8;
        id[1] = ((self.next_session_counter >> 8) & 0xFF) as u8;
        self.next_session_counter += 1;
        id
    }
}

// ============================================================================
// ISO8601 Utility
// ============================================================================
fn iso8601(unix: u64) -> String {
    let dt = chrono::NaiveDateTime::from_timestamp_opt(unix as i64, 0)
        .unwrap_or(chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    chrono::DateTime::<chrono::Utc>::from_utc(dt, chrono::Utc).to_rfc3339()
}

// ============================================================================
// BUYER → Commit
// ============================================================================

impl CoreProverEngine {
    pub fn buyer_commit(
        &mut self,
        buyer: String,
        seller: String,
        amount: u64,
        profile: PaymentProfile,
        buyer_chain_id: u64,
        buyer_commit_txid: String,
    ) -> Result<[u8; 32], String> {
        let now = self.now();

        if buyer_commit_txid.trim().is_empty() {
            return Err("buyer_commit_txid is required".into());
        }

        let order_id = self.generate_order_id();

        let escrow = Escrow::new(
            order_id,
            buyer,
            seller,
            amount,
            profile,
            buyer_chain_id,
            buyer_commit_txid,
            now.mono,
        );

        self.escrows.push(escrow);
        Ok(order_id)
    }

    // ============================================================================
    // SELLER → Accept
    // ============================================================================

    pub fn seller_accept(
        &mut self,
        order_id: &[u8; 32],
        seller_accept_txid: String,
    ) -> Result<(), String> {
        let now = self.now();
        let chain_id = self.chain_id; // <-- extract BEFORE borrow

        {
            let escrow = self.get_escrow_mut(order_id)?;

            if escrow.state != EscrowState::BuyerCommitted {
                return Err("seller_accept only valid from BuyerCommitted".into());
            }
            if seller_accept_txid.trim().is_empty() {
                return Err("seller_accept_txid is required".into());
            }
            if now.mono > escrow.acceptance_deadline_mono {
                return Err("acceptance window expired".into());
            }

            escrow.seller_chain_id = chain_id;
            escrow.seller_accept_mono = Some(now.mono);
            escrow.seller_accept_txid = Some(seller_accept_txid);

            escrow.fulfillment_deadline_mono =
                Some(now.mono + escrow.profile.timing.fulfillment_window_secs);

            escrow.state = EscrowState::SellerAccepted;
        }

        Ok(())
    }

    // ============================================================================
    // SELLER → Fulfill
    // ============================================================================

    pub fn seller_fulfill(
        &mut self,
        order_id: &[u8; 32],
        seller_fulfill_txid: String,
    ) -> Result<(), String> {
        let now = self.now();
        let mut is_late = false;

        {
            let escrow = self.get_escrow_mut(order_id)?;

            if !escrow.state.can_fulfill() {
                return Err(format!("seller_fulfill invalid in state {:?}", escrow.state));
            }

            if seller_fulfill_txid.trim().is_empty() {
                return Err("seller_fulfill_txid is required".into());
            }

            is_late = match escrow.fulfillment_deadline_mono {
                Some(d) => now.mono > d,
                None => false,
            };

            escrow.fulfillment_mono = Some(now.mono);
            escrow.seller_fulfill_txid = Some(seller_fulfill_txid);

            escrow.state = if is_late {
                EscrowState::FulfillmentExpired
            } else {
                EscrowState::SellerFulfilled
            };
        }

        self.create_receipt_stub(order_id, is_late)?;
        Ok(())
    }

    // ============================================================================
    // Receipt Stub Creation
    // ============================================================================

    fn create_receipt_stub(
        &mut self,
        order_id: &[u8; 32],
        is_late: bool,
    ) -> Result<(), String> {
        let now = self.now();
        let escrow = self.get_escrow(order_id)?;

        let late_discount = if is_late && escrow.profile.enables_late_discount {
            escrow.profile.late_discount_pct
        } else {
            0
        };

        let discount_expiration_unix = if late_discount > 0 {
            now.unix + escrow.profile.discount_expiration_days * 86400
        } else {
            0
        };

        let meta = ReceiptMetadata {
            session_id: escrow.order_id,
            order_amount: escrow.amount as u128,
            fulfillment_mono: escrow.fulfillment_mono.unwrap_or(now.mono),
            fulfillment_unix: now.unix,
            fulfillment_iso: now.iso.clone(),
            settlement_mono: 0,
            settlement_unix: 0,
            settlement_iso: "".into(),
            late_fulfilled: is_late,
            discount_pct: late_discount,
            discount_expiration_unix,
            buyer_chain_id: escrow.buyer_chain_id,
            buyer_commit_txid: escrow.buyer_commit_txid.clone(),
            seller_chain_id: escrow.seller_chain_id,
            seller_accept_txid: escrow.seller_accept_txid.clone().unwrap_or_default(),
            seller_fulfill_txid: escrow.seller_fulfill_txid.clone().unwrap_or_default(),
            seller_claim_txid: None,
            seller_refund_txid: None,
            buyer_withdraw_txid: None,
            seller_block_height: 0,
        };

        self.receipts.push(meta);
        Ok(())
    }

    // ============================================================================
    // SELLER → Claim
    // ============================================================================

    pub fn seller_claim(
        &mut self,
        order_id: &[u8; 32],
        seller_claim_txid: String,
    ) -> Result<u64, String> {
        let now = self.now();
        let block_height = self.current_block_height; // extract BEFORE borrow
        let amount;

        {
            let escrow = self.get_escrow_mut(order_id)?;

            if !matches!(escrow.state, EscrowState::SellerFulfilled | EscrowState::FulfillmentExpired)
            {
                return Err("seller_claim only valid after fulfillment".into());
            }

            if seller_claim_txid.trim().is_empty() {
                return Err("seller_claim_txid is required".into());
            }

            escrow.seller_claim_txid = Some(seller_claim_txid);
            escrow.settlement_mono = Some(now.mono);
            escrow.seller_block_height = Some(block_height);
            escrow.state = EscrowState::SellerClaimed;

            amount = escrow.amount;
        }

        self.finalize_receipt(order_id, false)?;
        Ok(amount)
    }

    // ============================================================================
    // SELLER → Refund
    // ============================================================================

    pub fn seller_refund(
        &mut self,
        order_id: &[u8; 32],
        seller_refund_txid: String,
    ) -> Result<u64, String> {
        let now = self.now();
        let block_height = self.current_block_height;
        let amount;

        {
            let escrow = self.get_escrow_mut(order_id)?;

            if !matches!(escrow.state, EscrowState::SellerFulfilled | EscrowState::FulfillmentExpired)
            {
                return Err("seller_refund only valid after fulfillment".into());
            }

            if seller_refund_txid.trim().is_empty() {
                return Err("seller_refund_txid is required".into());
            }

            escrow.seller_refund_txid = Some(seller_refund_txid);
            escrow.settlement_mono = Some(now.mono);
            escrow.seller_block_height = Some(block_height);
            escrow.state = EscrowState::SellerRefunded;

            amount = escrow.amount;
        }

        self.finalize_receipt(order_id, true)?;
        Ok(amount)
    }

    // ============================================================================
    // BUYER → Withdraw
    // ============================================================================

    pub fn buyer_withdraw(
        &mut self,
        order_id: &[u8; 32],
        buyer_withdraw_txid: Option<String>,
    ) -> Result<u64, String> {
        let now = self.now();
        let amount;

        {
            let escrow = self.get_escrow_mut(order_id)?;

            if !matches!(escrow.state, EscrowState::BuyerCommitted | EscrowState::FulfillmentExpired)
            {
                return Err("buyer_withdraw not allowed".into());
            }

            if escrow.state == EscrowState::BuyerCommitted
                && now.mono <= escrow.acceptance_deadline_mono
            {
                return Err("buyer_withdraw not yet allowed".into());
            }

            if let Some(tx) = buyer_withdraw_txid {
                escrow.buyer_withdraw_txid = Some(tx);
            }

            escrow.state = EscrowState::BuyerWithdrawn;
            escrow.settlement_mono = Some(now.mono);
            escrow.seller_block_height = None;

            amount = escrow.amount;
        }

        Ok(amount)
    }

    // ============================================================================
    // TIMED RELEASE
    // ============================================================================

    pub fn timed_release(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        let now = self.now();
        let block_height = self.current_block_height;
        let amount;

        {
            let escrow = self.get_escrow_mut(order_id)?;

            if !escrow.profile.allows_timed_release {
                return Err("timed_release disabled".into());
            }

            if !matches!(escrow.state, EscrowState::SellerFulfilled | EscrowState::FulfillmentExpired)
            {
                return Err("timed_release only after fulfillment".into());
            }

            let fulfill_mono = escrow.fulfillment_mono.unwrap_or(0);
            let elapsed = now.mono.saturating_sub(fulfill_mono);

            if elapsed < escrow.profile.timing.claim_window_secs {
                return Err("claim window not expired".into());
            }

            escrow.seller_claim_txid = Some(format!("auto_claim_{}", now.mono));
            escrow.settlement_mono = Some(now.mono);
            escrow.seller_block_height = Some(block_height);
            escrow.state = EscrowState::SellerClaimed;

            amount = escrow.amount;
        }

        self.finalize_receipt(order_id, false)?;
        Ok(amount)
    }

    // ============================================================================
    // Receipt Finalization
    // ============================================================================

    fn finalize_receipt(&mut self, order_id: &[u8; 32], refunded: bool) -> Result<(), String> {
    let now = self.now();

    // ---- FIRST: extract all fields we need from escrow (immutable borrow ends here!) ----
    let (
        settlement_mono,
        seller_block_height,
        seller_claim_txid,
        seller_refund_txid,
    ) = {
        let escrow = self.get_escrow(order_id)?;

        (
            escrow.settlement_mono.unwrap_or(now.mono),
            escrow.seller_block_height.unwrap_or(0),
            escrow.seller_claim_txid.clone(),
            escrow.seller_refund_txid.clone(),
        )
    }; 
    // <-- immutable borrow ends here

    // ---- SECOND: now safely borrow receipts mutably ----
    let idx = self
        .receipts
        .iter()
        .rposition(|m| &m.session_id == order_id)
        .ok_or("receipt stub not found")?;

    let meta = &mut self.receipts[idx];

    meta.settlement_mono = settlement_mono;
    meta.settlement_unix = now.unix;
    meta.settlement_iso = now.iso.clone();
    meta.seller_block_height = seller_block_height;

    if refunded {
        meta.seller_refund_txid = seller_refund_txid;
    } else {
        meta.seller_claim_txid = seller_claim_txid;
    }

    Ok(())
}

    // ============================================================================
    // STATE UPDATE
    // ============================================================================

    pub fn update_state(&mut self, order_id: &[u8; 32]) -> Result<(), String> {
        let now = self.now();

        let escrow = self.get_escrow_mut(order_id)?;
        if escrow.state == EscrowState::SellerAccepted {
            if let Some(deadline) = escrow.fulfillment_deadline_mono {
                if now.mono > deadline {
                    escrow.state = EscrowState::FulfillmentExpired;
                }
            }
        }
        Ok(())
    }

    // ============================================================================
    // GETTERS
    // ============================================================================

    pub fn get_state(&self, order_id: &[u8; 32]) -> Result<EscrowState, String> {
        Ok(self.get_escrow(order_id)?.state)
    }

    pub fn get_receipt(&self, order_id: &[u8; 32]) -> Option<&ReceiptMetadata> {
        self.receipts.iter().find(|r| &r.session_id == order_id)
    }

    pub fn get_receipts(&self) -> &Vec<ReceiptMetadata> {
        &self.receipts
    }
}