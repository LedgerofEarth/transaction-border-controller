// ============================================================================
// CoreProver Types (v0.3)
// Triple-Clock Model + Full TXID Provenance + Chain-Aware Receipts
//
// This file defines:
// - EscrowState
// - TimingWindows
// - PaymentProfile
// - ReceiptMetadata
// - Escrow (session record)
// - Full provenance requirements
//
// Notes:
// - Uses ONLY u64 seconds for time (monotonic + unix)
// - ISO8601 strings for receipt readability
// - Required txids for all blockchain-anchored actions:
//      buyer_commit_txid
//      seller_accept_txid
//      seller_fulfill_txid
//      seller_claim_txid OR seller_refund_txid
// - Optional txid for buyer_withdraw
// - Supports multi-chain by including chain_id for each actor
// ============================================================================

use serde::{Deserialize, Serialize};

// ============================================================================
// Escrow State Machine (v0.3)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowState {
    BuyerCommitted,
    SellerAccepted,
    SellerFulfilled,
    FulfillmentExpired,
    SellerClaimed,
    SellerRefunded,
    BuyerWithdrawn,
}

impl EscrowState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            EscrowState::SellerClaimed
                | EscrowState::SellerRefunded
                | EscrowState::BuyerWithdrawn
        )
    }

    pub fn can_fulfill(self) -> bool {
        matches!(self, EscrowState::SellerAccepted | EscrowState::FulfillmentExpired)
    }
}

// ============================================================================
// Timing Windows (pure u64 seconds)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingWindows {
    pub acceptance_window_secs: u64,
    pub fulfillment_window_secs: u64,
    pub claim_window_secs: u64,
}

impl TimingWindows {
    pub fn pizza_delivery() -> Self {
        Self {
            acceptance_window_secs: 1800, // 30 minutes
            fulfillment_window_secs: 3600, // 1 hour
            claim_window_secs: 3600,       // 1 hour
        }
    }
}

// ============================================================================
// Payment Profile
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProfile {
    pub timing: TimingWindows,
    pub allows_timed_release: bool,
    pub enables_late_discount: bool,
    pub late_discount_pct: u8,
    pub discount_expiration_days: u64,
}

impl PaymentProfile {
    pub fn pizza_delivery() -> Self {
        Self {
            timing: TimingWindows::pizza_delivery(),
            allows_timed_release: true,
            enables_late_discount: true,
            late_discount_pct: 10,
            discount_expiration_days: 90,
        }
    }
}

// ============================================================================
// Receipt Metadata (FULL version, v0.3)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMetadata {
    pub session_id: [u8; 32],
    pub order_amount: u128,

    // Timing (Triple-Clock)
    pub fulfillment_mono: u64,
    pub fulfillment_unix: u64,
    pub fulfillment_iso: String,

    pub settlement_mono: u64,
    pub settlement_unix: u64,
    pub settlement_iso: String,

    // Discounts
    pub late_fulfilled: bool,
    pub discount_pct: u8,
    pub discount_expiration_unix: u64,

    // Blockchain provenance (required)
    pub buyer_chain_id: u64,
    pub buyer_commit_txid: String,

    pub seller_chain_id: u64,
    pub seller_accept_txid: String,
    pub seller_fulfill_txid: String,

    // Final settlement (one of these MUST be present)
    pub seller_claim_txid: Option<String>,
    pub seller_refund_txid: Option<String>,

    // Optional economic termination before fulfillment
    pub buyer_withdraw_txid: Option<String>,

    // Settlement ordering anchor
    pub seller_block_height: u64,
}

// ============================================================================
// Escrow Session Record
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    // Identity
    pub order_id: [u8; 32],
    pub buyer: String,
    pub seller: String,

    // Value
    pub amount: u64,
    pub profile: PaymentProfile,

    pub state: EscrowState,

    // Timing (monotonic seconds)
    pub buyer_commit_mono: u64,
    pub seller_accept_mono: Option<u64>,
    pub fulfillment_mono: Option<u64>,
    pub settlement_mono: Option<u64>,

    // Deadlines (monotonic seconds)
    pub acceptance_deadline_mono: u64,
    pub fulfillment_deadline_mono: Option<u64>,

    // Blockchain provenance (required + optional)
    pub buyer_chain_id: u64,
    pub buyer_commit_txid: String,

    pub seller_chain_id: u64,
    pub seller_accept_txid: Option<String>,
    pub seller_fulfill_txid: Option<String>,

    pub seller_claim_txid: Option<String>,
    pub seller_refund_txid: Option<String>,

    pub buyer_withdraw_txid: Option<String>,

    // Final settlement anchor
    pub seller_block_height: Option<u64>,
}

impl Escrow {
    pub fn new(
        order_id: [u8; 32],
        buyer: String,
        seller: String,
        amount: u64,
        profile: PaymentProfile,
        buyer_chain_id: u64,
        buyer_commit_txid: String,
        current_mono: u64,
    ) -> Self {
        let acceptance_deadline = current_mono + profile.timing.acceptance_window_secs;

        Self {
            order_id,
            buyer,
            seller,
            amount,
            profile,
            state: EscrowState::BuyerCommitted,

            buyer_commit_mono: current_mono,
            seller_accept_mono: None,
            fulfillment_mono: None,
            settlement_mono: None,

            acceptance_deadline_mono: acceptance_deadline,
            fulfillment_deadline_mono: None,

            // provenance
            buyer_chain_id,
            buyer_commit_txid,

            seller_chain_id: 0, // will be set at accept-time
            seller_accept_txid: None,
            seller_fulfill_txid: None,

            seller_claim_txid: None,
            seller_refund_txid: None,

            buyer_withdraw_txid: None,

            seller_block_height: None,
        }
    }
}