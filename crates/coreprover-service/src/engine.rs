/*! CoreProver Engine - Final Architecture

Implements the finalized state machine with:
- Explicit state transitions
- Re-lock/unlock withdrawal logic
- Late fulfillment discount mechanism
- Receipt minting with metadata

This replaces all prior versions.
*/

use crate::types::*;
use std::time::Instant;

// ============================================================================
// CoreProver Escrow Engine
// ============================================================================

pub struct CoreProverEngine {
    escrows: Vec<Escrow>,
    receipts: Vec<Receipt>,
    next_receipt_id: u64,
    current_time: Instant,
}

impl CoreProverEngine {
    pub fn new() -> Self {
        Self {
            escrows: Vec::new(),
            receipts: Vec::new(),
            next_receipt_id: 1,
            current_time: Instant::now(),
        }
    }

    // ========================================================================
    // BUYER ACTIONS
    // ========================================================================

    /// Buyer commits payment - creates escrow in BuyerCommitted state.
    ///
    /// Acceptance window starts immediately.
    /// Buyer can withdraw if seller doesn't accept within acceptance_window.
    pub fn buyer_commit(
        &mut self,
        buyer: String,
        seller: String,
        amount: u64,
        profile: PaymentProfile,
    ) -> Result<[u8; 32], String> {
        let order_id = self.generate_order_id();
        let escrow = Escrow::new(order_id, buyer, seller, amount, profile, self.current_time);

        self.escrows.push(escrow);

        println!("BuyerCommitted: Order 0x{}", hex_encode(&order_id[..4]));
        println!(
            "Acceptance window: {:?}",
            self.escrows.last().unwrap().profile.timing.acceptance_window
        );
        println!("Withdrawal: UNLOCKED if deadline expires");

        Ok(order_id)
    }

    /// Buyer withdraws funds.
    ///
    /// Allowed only when:
    /// 1. State is BuyerCommitted AND acceptance_deadline expired, OR
    /// 2. State is FulfillmentExpired
    ///
    /// NOT allowed when:
    /// - State is SellerAccepted (withdrawal LOCKED)
    /// - State is LateFulfilled (withdrawal RE-LOCKED)
    pub fn buyer_withdraw(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        let current_time = self.current_time;
        let escrow = self.get_escrow_mut(order_id)?;

        if !escrow.can_buyer_withdraw(current_time) {
            return Err(format!(
                "Buyer withdrawal not allowed in state {:?}. Withdrawal is locked.",
                escrow.state
            ));
        }

        let amount = escrow.amount;
        escrow.state = EscrowState::BuyerWithdrawn;
        escrow.settlement_time = Some(current_time);

        println!("BuyerWithdrawn: Returned {} wei to buyer", amount);
        println!("Escrow closed");

        Ok(amount)
    }

    // ========================================================================
    // SELLER ACTIONS
    // ========================================================================

    /// Seller accepts order via legal signature.
    ///
    /// Transitions: BuyerCommitted -> SellerAccepted.
    ///
    /// CRITICAL: Locks buyer withdrawal immediately.
    /// Fulfillment window begins here.
    pub fn seller_accept(
        &mut self,
        order_id: &[u8; 32],
        signature: LegalSignature,
    ) -> Result<(), String> {
        let current_time = self.current_time;
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.state != EscrowState::BuyerCommitted {
            return Err(format!(
                "Invalid state: {:?}. Expected BuyerCommitted",
                escrow.state
            ));
        }

        if let Some(deadline) = escrow.acceptance_deadline {
            if current_time > deadline {
                return Err("Acceptance window expired - buyer can withdraw".to_string());
            }
        }

        let fulfillment_deadline = current_time + escrow.profile.timing.fulfillment_window;

        escrow.seller_signature = Some(signature);
        escrow.seller_accept_time = Some(current_time);
        escrow.fulfillment_deadline = Some(fulfillment_deadline);
        escrow.state = EscrowState::SellerAccepted;

        println!("SellerAccepted: Order accepted by seller");
        println!("Buyer withdrawal LOCKED");
        println!(
            "Fulfillment window: {:?}",
            escrow.profile.timing.fulfillment_window
        );

        Ok(())
    }

    /// Seller fulfills order.
    ///
    /// Two transitions:
    /// - Within window: SellerAccepted -> SellerFulfilled
    /// - Late: FulfillmentExpired -> LateFulfilled
    pub fn seller_fulfill(&mut self, order_id: &[u8; 32]) -> Result<(), String> {
        let current_time = self.current_time;
        let escrow = self.get_escrow_mut(order_id)?;

        match escrow.state {
            EscrowState::SellerAccepted | EscrowState::FulfillmentExpired => {}
            _ => {
                return Err(format!(
                    "Invalid state: {:?}. Expected SellerAccepted or FulfillmentExpired",
                    escrow.state
                ));
            }
        }

        let is_late = escrow.is_late_fulfillment(current_time);
        escrow.fulfillment_time = Some(current_time);

        if is_late {
            escrow.state = EscrowState::LateFulfilled;

            println!("SellerLateFulfilled: Fulfilled AFTER deadline");
            println!("Buyer withdrawal RE-LOCKED");

            if escrow.profile.enables_late_discount {
                println!(
                    "DISCOUNT: {}% off next purchase",
                    escrow.profile.late_discount_pct
                );
                println!(
                    "Valid for {} days",
                    escrow.profile.discount_expiration_days
                );
            }
        } else {
            escrow.state = EscrowState::SellerFulfilled;

            println!("SellerFulfilled: On-time fulfillment");
            println!("Seller can now claim");
        }

        Ok(())
    }

    /// Seller claims funds and mints receipt.
    ///
    /// Only valid after SellerFulfilled or LateFulfilled.
    pub fn seller_claim(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        let current_time = self.current_time;

        let (metadata, amount) = {
            let escrow = self.get_escrow(order_id)?;

            if !escrow.can_seller_claim(current_time) {
                return Err(format!("Seller cannot claim in state {:?}", escrow.state));
            }

            let session_id = *order_id;
            let time_unix = unix_timestamp();
            let metadata = escrow.generate_receipt_metadata(session_id, time_unix);

            (metadata, escrow.amount)
        };

        let receipt_id = {
            let escrow = self.get_escrow(order_id)?;
            let receipt = Receipt {
                receipt_id: self.next_receipt_id,
                order_id: escrow.order_id,
                buyer: escrow.buyer.clone(),
                seller: escrow.seller.clone(),
                metadata: metadata.clone(),
            };
            let id = receipt.receipt_id;
            self.receipts.push(receipt);
            self.next_receipt_id += 1;
            id
        };

        let escrow = self.get_escrow_mut(order_id)?;
        escrow.state = EscrowState::SellerClaimed;
        escrow.settlement_time = Some(current_time);

        println!("SellerClaimed: released {} wei", amount);
        println!("Receipt ID: {}", receipt_id);

        if metadata.late_fulfilled {
            println!("Receipt includes discount:");
            println!("  - {}% off", metadata.discount_pct);
            println!("  - Expires: {}", metadata.discount_expiration);
        }

        Ok(receipt_id)
    }

    // ========================================================================
    // AUTOMATIC STATE TRANSITION
    // ========================================================================

    pub fn update_escrow_state(&mut self, order_id: &[u8; 32]) -> Result<(), String> {
        let now = self.current_time;
        let escrow = self.get_escrow_mut(order_id)?;

        escrow.update_withdrawal_lock(now);
        Ok(())
    }

    // ========================================================================
    // TIMED RELEASE
    // ========================================================================

    pub fn trigger_timed_release(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        let now = self.current_time;

        {
            let escrow = self.get_escrow(order_id)?;

            if !escrow.can_seller_claim(now) {
                return Err(format!("Cannot triggers timed release in {:?}", escrow.state));
            }

            if !escrow.profile.allows_timed_release {
                return Err("Timed release disabled".to_string());
            }

            if let Some(ft) = escrow.fulfillment_time {
                let elapsed = now.duration_since(ft);
                if elapsed < escrow.profile.timing.claim_window {
                    return Err("Claim window not expired".to_string());
                }
            } else {
                return Err("No fulfillment time".to_string());
            }
        }

        println!("TIMED RELEASE: seller forgot to claim, auto release");
        self.seller_claim(order_id)
    }

    // ========================================================================
    // INTERNAL HELPERS
    // ========================================================================

    fn get_escrow_mut(&mut self, order_id: &[u8; 32]) -> Result<&mut Escrow, String> {
        self.escrows
            .iter_mut()
            .find(|e| &e.order_id == order_id)
            .ok_or_else(|| "Escrow not found".to_string())
    }

    fn get_escrow(&self, order_id: &[u8; 32]) -> Result<&Escrow, String> {
        self.escrows
            .iter()
            .find(|e| &e.order_id == order_id)
            .ok_or_else(|| "Escrow not found".to_string())
    }

    fn generate_order_id(&self) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = self.escrows.len() as u8;
        id[1] = unix_timestamp() as u8;
        id
    }

    pub fn advance_time(&mut self, duration: std::time::Duration) {
        self.current_time += duration;
    }

    pub fn get_state(&self, order_id: &[u8; 32]) -> Result<EscrowState, String> {
        Ok(self.get_escrow(order_id)?.state)
    }

    pub fn get_receipt(&self, receipt_id: u64) -> Result<&Receipt, String> {
        self.receipts
            .iter()
            .find(|r| r.receipt_id == receipt_id)
            .ok_or_else(|| "Receipt not found".to_string())
    }
}

impl Default for CoreProverEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buyer_commit_creates_escrow() {
        let mut engine = CoreProverEngine::new();
        let profile = PaymentProfile::pizza_delivery();

        let order_id = engine
            .buyer_commit("buyer".to_string(), "seller".to_string(), 1000, profile)
            .unwrap();

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::BuyerCommitted
        );
    }

    #[test]
    fn test_seller_accept_locks_withdrawal() {
        let mut engine = CoreProverEngine::new();
        let profile = PaymentProfile::pizza_delivery();

        let order_id = engine
            .buyer_commit("buyer".to_string(), "seller".to_string(), 1000, profile)
            .unwrap();

        let sig = LegalSignature {
            signature: vec![0xAB; 65],
            business_name: "Biz".to_string(),
            business_license: "LIC-123".to_string(),
            document_hash: [0xCD; 32],
            timestamp: unix_timestamp(),
        };

        engine.seller_accept(&order_id, sig).unwrap();

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerAccepted
        );

        assert!(engine.buyer_withdraw(&order_id).is_err());
    }

    #[test]
    fn test_late_fulfillment_triggers_discount() {
        let mut engine = CoreProverEngine::new();
        let profile = PaymentProfile::pizza_delivery();

        let order_id = engine
            .buyer_commit("buyer".to_string(), "seller".to_string(), 1000, profile)
            .unwrap();

        let sig = LegalSignature {
            signature: vec![0xAB; 65],
            business_name: "Biz".to_string(),
            business_license: "LIC-123".to_string(),
            document_hash: [0xCD; 32],
            timestamp: unix_timestamp(),
        };

        engine.seller_accept(&order_id, sig).unwrap();

        engine.advance_time(std::time::Duration::from_secs(3601));
        engine.update_escrow_state(&order_id).unwrap();

        engine.seller_fulfill(&order_id).unwrap();

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::LateFulfilled
        );
    }
}