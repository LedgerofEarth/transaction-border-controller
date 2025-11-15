// ============================================================================
// CoreProver Engine (v0.3)
// Deterministic Triple-Clock Escrow Engine
//
// This engine implements:
// - Dual-commitment escrow
// - Explicit state machine
// - Required TXIDs for blockchain-anchored actions
// - Optional TXIDs for off-chain withdrawal
// - Legal-grade receipt creation at seller_fulfill
// - Cross-chain provenance (buyer_chain_id + seller_chain_id)
// - Settlement finalization via claim or refund
// - Monotonic block height simulation per-engine
// - Timed release for seller if claim window expires
//
// Dependencies:
//   use crate::types::*;
// ============================================================================

use crate::types::*;
use std::fmt;

// ============================================================================
// TimeTruth: deterministic triple-clock model
// ============================================================================

#[derive(Debug, Clone)]
pub struct TimeTruth {
    /// Monotonic seconds (simulated internal clock)
    pub mono: u64,

    /// Unix timestamp (seconds)
    pub unix: u64,

    /// ISO8601 timestamp (string)
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
    // Escrow storage
    escrows: Vec<Escrow>,

    // Receipt storage
    receipts: Vec<ReceiptMetadata>,
    next_session_counter: u64,

    // Triple-clock (engine-wide)
    current_mono: u64,
    current_unix: u64,

    // Chain params
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

        // simulate block progress deterministically
        let blocks = secs / self.block_interval_secs;
        self.current_block_height += blocks;
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

    // ------------------------------------------------------------------------
    // Order ID Generation
    // ------------------------------------------------------------------------

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
    // simple RFC3339 conversion
    let dt = chrono::NaiveDateTime::from_timestamp_opt(unix as i64, 0)
        .unwrap_or(chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    chrono::DateTime::<chrono::Utc>::from_utc(dt, chrono::Utc).to_rfc3339()
}
// ============================================================================
// BUYER → Commit
// Required: buyer_commit_txid
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
    // SELLER → Accept (Legal acceptance, MUST be on-chain)
    // Required: seller_accept_txid
    // ============================================================================

    pub fn seller_accept(
        &mut self,
        order_id: &[u8; 32],
        seller_accept_txid: String,
    ) -> Result<(), String> {
        let now = self.now();
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.state != EscrowState::BuyerCommitted {
            return Err("seller_accept only valid from BuyerCommitted".into());
        }

        if seller_accept_txid.trim().is_empty() {
            return Err("seller_accept_txid is required".into());
        }

        // Check acceptance window
        if now.mono > escrow.acceptance_deadline_mono {
            return Err("acceptance window expired".into());
        }

        // update seller chain id (engine chain)
        escrow.seller_chain_id = self.chain_id;
        escrow.seller_accept_mono = Some(now.mono);
        escrow.seller_accept_txid = Some(seller_accept_txid);

        // set fulfillment deadline
        escrow.fulfillment_deadline_mono =
            Some(now.mono + escrow.profile.timing.fulfillment_window_secs);

        escrow.state = EscrowState::SellerAccepted;

        Ok(())
    }

    // ============================================================================
    // SELLER → Fulfill (Legal receipt creation event)
    // Required: seller_fulfill_txid
    // ============================================================================

    pub fn seller_fulfill(
        &mut self,
        order_id: &[u8; 32],
        seller_fulfill_txid: String,
    ) -> Result<(), String> {
        let now = self.now();
        let escrow = self.get_escrow_mut(order_id)?;

        if !escrow.state.can_fulfill() {
            return Err(format!(
                "seller_fulfill invalid in state {:?}",
                escrow.state
            ));
        }

        if seller_fulfill_txid.trim().is_empty() {
            return Err("seller_fulfill_txid is required".into());
        }

        // Determine if late
        let is_late = match escrow.fulfillment_deadline_mono {
            Some(deadline) => now.mono > deadline,
            None => false,
        };

        escrow.fulfillment_mono = Some(now.mono);
        escrow.seller_fulfill_txid = Some(seller_fulfill_txid);

        escrow.state = if is_late {
            EscrowState::FulfillmentExpired // transition first
        } else {
            EscrowState::SellerFulfilled
        };

        // If late, record late fulfillment state change
        if is_late {
            escrow.state = EscrowState::FulfillmentExpired;
        }

        // If on-time, advance to SellerFulfilled explicitly
        if !is_late {
            escrow.state = EscrowState::SellerFulfilled;
        }

        // Receipt creation happens HERE, not during claim.
        self.create_receipt_stub(order_id, is_late)?;

        Ok(())
    }

    // ============================================================================
    // RECEIPT CREATION (stub, finalized at claim/refund)
    //
    // Why this design?
    // - legal receipt exists at fulfillment
    // - but settlement is completed later (claim or refund)
    // - so this produces a partial metadata object
    // - finalization is performed during claim or refund
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
            now.unix + (escrow.profile.discount_expiration_days * 86400)
        } else {
            0
        };

        // Construct a partial receipt; claim/refund will finalize settlement fields.
        let metadata = ReceiptMetadata {
            session_id: escrow.order_id,
            order_amount: escrow.amount as u128,

            fulfillment_mono: escrow.fulfillment_mono.unwrap_or(now.mono),
            fulfillment_unix: now.unix,
            fulfillment_iso: now.iso.clone(),

            // These fields finalized at claim/refund:
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

        self.receipts.push(metadata);
        Ok(())
    }
}
// ============================================================================
// SELLER → Claim (Settlement Finalization)
// Required: seller_claim_txid
// ============================================================================

impl CoreProverEngine {
    pub fn seller_claim(
        &mut self,
        order_id: &[u8; 32],
        seller_claim_txid: String,
    ) -> Result<u64, String> {
        let now = self.now();
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.state != EscrowState::SellerFulfilled
            && escrow.state != EscrowState::FulfillmentExpired
        {
            return Err("seller_claim only valid after fulfillment".into());
        }

        if seller_claim_txid.trim().is_empty() {
            return Err("seller_claim_txid is required".into());
        }

        // finalize settlement
        escrow.seller_claim_txid = Some(seller_claim_txid);
        escrow.settlement_mono = Some(now.mono);
        escrow.seller_block_height = Some(self.current_block_height);

        escrow.state = EscrowState::SellerClaimed;

        // attach to the last receipt (stub created at fulfillment)
        self.finalize_receipt(order_id, false)?;

        Ok(escrow.amount)
    }

    // ============================================================================
    // SELLER → Refund (Settlement Reversal)
    // Required: seller_refund_txid
    // ============================================================================

    pub fn seller_refund(
        &mut self,
        order_id: &[u8; 32],
        seller_refund_txid: String,
    ) -> Result<u64, String> {
        let now = self.now();
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.state != EscrowState::SellerFulfilled
            && escrow.state != EscrowState::FulfillmentExpired
        {
            return Err("seller_refund only valid after fulfillment".into());
        }

        if seller_refund_txid.trim().is_empty() {
            return Err("seller_refund_txid is required".into());
        }

        // finalize settlement reversal
        escrow.seller_refund_txid = Some(seller_refund_txid);
        escrow.settlement_mono = Some(now.mono);
        escrow.seller_block_height = Some(self.current_block_height);

        escrow.state = EscrowState::SellerRefunded;

        // attach to existing receipt
        self.finalize_receipt(order_id, true)?;

        Ok(escrow.amount)
    }

    // ============================================================================
    // BUYER → Withdraw (if allowed)
    // buyer_withdraw_txid is optional
    // ============================================================================

    pub fn buyer_withdraw(
        &mut self,
        order_id: &[u8; 32],
        buyer_withdraw_txid: Option<String>,
    ) -> Result<u64, String> {
        let now = self.now();
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.state != EscrowState::BuyerCommitted
            && escrow.state != EscrowState::FulfillmentExpired
        {
            return Err("buyer_withdraw not allowed in this state".into());
        }

        // Must be past acceptance deadline (if in BuyerCommitted)
        if escrow.state == EscrowState::BuyerCommitted
            && now.mono <= escrow.acceptance_deadline_mono
        {
            return Err("buyer_withdraw not yet allowed".into());
        }

        // Assign optional txid
        if let Some(tx) = buyer_withdraw_txid.clone() {
            escrow.buyer_withdraw_txid = Some(tx);
        }

        escrow.state = EscrowState::BuyerWithdrawn;
        escrow.settlement_mono = Some(now.mono);

        // No seller_block_height for buyer withdraw (off-chain optional)
        escrow.seller_block_height = None;

        Ok(escrow.amount)
    }

    // ============================================================================
    // TIMED RELEASE → Seller forgot to claim
    // ============================================================================

    pub fn timed_release(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        let now = self.now();
        let escrow = self.get_escrow_mut(order_id)?;

        if !escrow.profile.allows_timed_release {
            return Err("timed_release disabled for this profile".into());
        }

        if escrow.state != EscrowState::SellerFulfilled
            && escrow.state != EscrowState::FulfillmentExpired
        {
            return Err("timed_release only after fulfillment".into());
        }

        let fulfill_mono = escrow.fulfillment_mono.unwrap_or(0);
        let elapsed = now.mono.saturating_sub(fulfill_mono);

        if elapsed < escrow.profile.timing.claim_window_secs {
            return Err("claim window not expired".into());
        }

        // Simulate an auto-claim txid for demo purposes
        let auto_txid = format!("auto_claim_txid_{}", now.mono);
        escrow.seller_claim_txid = Some(auto_txid);
        escrow.state = EscrowState::SellerClaimed;
        escrow.seller_block_height = Some(self.current_block_height);

        self.finalize_receipt(order_id, false)?;

        Ok(escrow.amount)
    }

    // ============================================================================
    // INTERNAL → Finalize Receipt After Settlement
    // ============================================================================

    fn finalize_receipt(&mut self, order_id: &[u8; 32], refunded: bool) -> Result<(), String> {
        let now = self.now();
        let escrow = self.get_escrow(order_id)?;

        // Locate stub (last receipt)
        let meta = self
            .receipts
            .iter_mut()
            .rev()
            .find(|m| &m.session_id == order_id)
            .ok_or("receipt stub not found")?;

        meta.settlement_mono = escrow.settlement_mono.unwrap_or(now.mono);
        meta.settlement_unix = now.unix;
        meta.settlement_iso = now.iso.clone();

        meta.seller_block_height = escrow.seller_block_height.unwrap_or(0);

        if refunded {
            meta.seller_refund_txid = escrow.seller_refund_txid.clone();
        } else {
            meta.seller_claim_txid = escrow.seller_claim_txid.clone();
        }

        Ok(())
    }

    // ============================================================================
    // STATE UPDATE (acceptance → fulfillment expiration)
    // ============================================================================

    pub fn update_state(&mut self, order_id: &[u8; 32]) -> Result<(), String> {
        let now = self.now();
        let escrow = self.get_escrow_mut(order_id)?;

        match escrow.state {
            EscrowState::SellerAccepted => {
                if let Some(deadline) = escrow.fulfillment_deadline_mono {
                    if now.mono > deadline {
                        escrow.state = EscrowState::FulfillmentExpired;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    // ============================================================================
    // BASIC GETTERS
    // ============================================================================

    pub fn get_state(&self, order_id: &[u8; 32]) -> Result<EscrowState, String> {
        Ok(self.get_escrow(order_id)?.state)
    }

    pub fn get_receipt(&self, order_id: &[u8; 32]) -> Option<&ReceiptMetadata> {
        self.receipts
            .iter()
            .find(|r| &r.session_id == order_id)
    }

    pub fn get_receipts(&self) -> &Vec<ReceiptMetadata> {
        &self.receipts
    }
}