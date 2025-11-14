// CoreProver Types - Final Architecture
//
// Implements the finalized dual-commitment escrow with:
// - Explicit state enumeration (EVM-friendly)
// - Re-lock/unlock withdrawal logic
// - Late fulfillment discount mechanism
// - Receipt metadata structure
//
// This replaces all prior versions.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// ============================================================================
// Core State Machine (Final - Explicit States)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowState {
    BuyerInitiated,
    BuyerCommitted,
    SellerAccepted,
    SellerFulfilled,
    FulfillmentExpired,
    LateFulfilled,
    BuyerWithdrawn,
    SellerClaimed,
    SellerRefunded,
    EscrowClosed,
}

impl EscrowState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            EscrowState::BuyerWithdrawn
                | EscrowState::SellerClaimed
                | EscrowState::SellerRefunded
                | EscrowState::EscrowClosed
        )
    }

    pub fn is_withdrawal_locked(&self) -> bool {
        matches!(
            self,
            EscrowState::SellerAccepted
                | EscrowState::SellerFulfilled
                | EscrowState::LateFulfilled
        )
    }

    pub fn can_seller_claim(&self) -> bool {
        matches!(self, EscrowState::SellerFulfilled | EscrowState::LateFulfilled)
    }
}

// ============================================================================
// Timing Windows
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingWindows {
    pub acceptance_window: Duration,
    pub fulfillment_window: Duration,
    pub claim_window: Duration,
}

impl TimingWindows {
    pub fn pizza_delivery() -> Self {
        Self {
            acceptance_window: Duration::from_secs(1800),
            fulfillment_window: Duration::from_secs(3600),
            claim_window: Duration::from_secs(3600),
        }
    }

    pub fn ecommerce() -> Self {
        Self {
            acceptance_window: Duration::from_secs(86400),
            fulfillment_window: Duration::from_secs(604800),
            claim_window: Duration::from_secs(2592000),
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
// Legal Signature
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalSignature {
    pub signature: Vec<u8>,
    pub business_name: String,
    pub business_license: String,
    pub document_hash: [u8; 32],
    pub timestamp: u64,
}

// ============================================================================
// Receipt Metadata
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMetadata {
    pub session_id: [u8; 32],
    pub order_amount: u128,
    pub late_fulfilled: bool,
    pub discount_pct: u8,
    pub discount_expiration: u64,
    pub fulfillment_timestamp: u64,
    pub settlement_timestamp: u64,
}

impl ReceiptMetadata {
    pub fn on_time(
        session_id: [u8; 32],
        order_amount: u128,
        fulfillment_timestamp: u64,
        settlement_timestamp: u64,
    ) -> Self {
        Self {
            session_id,
            order_amount,
            late_fulfilled: false,
            discount_pct: 0,
            discount_expiration: 0,
            fulfillment_timestamp,
            settlement_timestamp,
        }
    }

    pub fn late_fulfilled(
        session_id: [u8; 32],
        order_amount: u128,
        discount_pct: u8,
        discount_expiration_days: u64,
        fulfillment_timestamp: u64,
        settlement_timestamp: u64,
    ) -> Self {
        let discount_expiration = fulfillment_timestamp + (discount_expiration_days * 86400);

        Self {
            session_id,
            order_amount,
            late_fulfilled: true,
            discount_pct,
            discount_expiration,
            fulfillment_timestamp,
            settlement_timestamp,
        }
    }

    pub fn is_discount_valid(&self, current_time: u64) -> bool {
        self.late_fulfilled
            && self.discount_pct > 0
            && current_time < self.discount_expiration
    }
}

// ============================================================================
// Receipt
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: u64,
    pub order_id: [u8; 32],
    pub buyer: String,
    pub seller: String,
    pub metadata: ReceiptMetadata,
}

// ============================================================================
// Escrow Record
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub order_id: [u8; 32],
    pub buyer: String,
    pub seller: String,
    pub amount: u64,
    pub state: EscrowState,
    pub profile: PaymentProfile,

    pub buyer_commit_time: Option<Instant>,
    pub seller_accept_time: Option<Instant>,
    pub fulfillment_time: Option<Instant>,
    pub settlement_time: Option<Instant>,

    pub seller_signature: Option<LegalSignature>,

    pub acceptance_deadline: Option<Instant>,
    pub fulfillment_deadline: Option<Instant>,
}

impl Escrow {
    pub fn new(
        order_id: [u8; 32],
        buyer: String,
        seller: String,
        amount: u64,
        profile: PaymentProfile,
        current_time: Instant,
    ) -> Self {
        let acceptance_deadline = Some(current_time + profile.timing.acceptance_window);

        Self {
            order_id,
            buyer,
            seller,
            amount,
            state: EscrowState::BuyerCommitted,
            profile,
            buyer_commit_time: Some(current_time),
            seller_accept_time: None,
            fulfillment_time: None,
            settlement_time: None,
            seller_signature: None,
            acceptance_deadline,
            fulfillment_deadline: None,
        }
    }

    pub fn update_withdrawal_lock(&mut self, current_time: Instant) {
        match self.state {
            EscrowState::BuyerCommitted => {
                if let Some(deadline) = self.acceptance_deadline {
                    if current_time > deadline {
                        // Buyer can withdraw; no state change until action
                    }
                }
            }
            EscrowState::SellerAccepted => {
                if let Some(deadline) = self.fulfillment_deadline {
                    if current_time > deadline {
                        self.state = EscrowState::FulfillmentExpired;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn can_buyer_withdraw(&self, current_time: Instant) -> bool {
        match self.state {
            EscrowState::BuyerCommitted => {
                if let Some(deadline) = self.acceptance_deadline {
                    current_time > deadline
                } else {
                    false
                }
            }
            EscrowState::FulfillmentExpired => true,
            _ => false,
        }
    }

    pub fn can_seller_claim(&self, _current_time: Instant) -> bool {
        matches!(
            self.state,
            EscrowState::SellerFulfilled | EscrowState::LateFulfilled
        )
    }

    pub fn is_late_fulfillment(&self, fulfillment_time: Instant) -> bool {
        if let Some(deadline) = self.fulfillment_deadline {
            fulfillment_time > deadline
        } else {
            false
        }
    }

    pub fn generate_receipt_metadata(
        &self,
        session_id: [u8; 32],
        current_time: u64,
    ) -> ReceiptMetadata {
        let fulfillment_timestamp = self
            .fulfillment_time
            .map(|t| {
                self.buyer_commit_time
                    .map(|start| (t - start).as_secs())
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        match self.state {
            EscrowState::LateFulfilled => ReceiptMetadata::late_fulfilled(
                session_id,
                self.amount as u128,
                self.profile.late_discount_pct,
                self.profile.discount_expiration_days,
                fulfillment_timestamp,
                current_time,
            ),
            _ => ReceiptMetadata::on_time(
                session_id,
                self.amount as u128,
                fulfillment_timestamp,
                current_time,
            ),
        }
    }
}

// ============================================================================
// Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscrowEvent {
    BuyerCommitted {
        order_id: [u8; 32],
        amount: u64,
        acceptance_deadline: u64,
    },
    SellerAccepted {
        order_id: [u8; 32],
        fulfillment_deadline: u64,
    },
    SellerFulfilled {
        order_id: [u8; 32],
        fulfillment_timestamp: u64,
    },
    FulfillmentExpired {
        order_id: [u8; 32],
        buyer_withdrawal_unlocked: bool,
    },
    SellerLateFulfilled {
        order_id: [u8; 32],
        fulfillment_timestamp: u64,
        discount_pct: u8,
        discount_expiration: u64,
    },
    SellerClaimed {
        order_id: [u8; 32],
        amount: u64,
        receipt_id: u64,
    },
    BuyerWithdrawn {
        order_id: [u8; 32],
        amount: u64,
    },
    ReceiptMinted {
        order_id: [u8; 32],
        receipt_id: u64,
        metadata: ReceiptMetadata,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_terminal_checks() {
        assert!(EscrowState::BuyerWithdrawn.is_terminal());
        assert!(EscrowState::SellerClaimed.is_terminal());
        assert!(!EscrowState::BuyerCommitted.is_terminal());
        assert!(!EscrowState::SellerAccepted.is_terminal());
    }

    #[test]
    fn test_withdrawal_lock_states() {
        assert!(!EscrowState::BuyerCommitted.is_withdrawal_locked());
        assert!(EscrowState::SellerAccepted.is_withdrawal_locked());
        assert!(!EscrowState::FulfillmentExpired.is_withdrawal_locked());
        assert!(EscrowState::LateFulfilled.is_withdrawal_locked());
    }

    #[test]
    fn test_escrow_creation() {
        let order_id = [1u8; 32];
        let profile = PaymentProfile::pizza_delivery();
        let now = Instant::now();

        let escrow = Escrow::new(
            order_id,
            "buyer".to_string(),
            "seller".to_string(),
            1000,
            profile,
            now,
        );

        assert_eq!(escrow.state, EscrowState::BuyerCommitted);
        assert!(escrow.acceptance_deadline.is_some());
        assert!(escrow.fulfillment_deadline.is_none());
    }
}