//! Pizza Delivery Simulation - Final Architecture
//!
//! Canonical test sequence demonstrating:
//! 1. Buyer commits payment → BuyerCommitted
//! 2. Seller accepts → SellerAccepted (LOCKS withdrawal)
//! 3. Fulfillment window expires → FulfillmentExpired (UNLOCKS withdrawal)
//! 4. Seller fulfills late → LateFulfilled (RE-LOCKS withdrawal, discount)
//! 5. Seller claims → SellerClaimed + receipt with discount metadata
//!
//! This is the authoritative reference for the re-lock logic.

mod types;
mod engine;

use types::*;
use engine::CoreProverEngine;
use std::time::Duration;

// ============================================================================
// CANONICAL PIZZA SIMULATION TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --------------------------------------------------------------
    // Test 1: Happy Path - On-Time Fulfillment
    // --------------------------------------------------------------
    #[test]
    fn test_pizza_delivery_on_time() {
        let mut engine = CoreProverEngine::new();
        let pizza_price = 25_000_000;
        let profile = PaymentProfile::pizza_delivery();

        // Buyer commits
        let order_id = engine
            .buyer_commit(
                "buyer_alice".to_string(),
                "pizza_hut".to_string(),
                pizza_price,
                profile,
            )
            .unwrap();

        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::BuyerCommitted);

        // Seller accepts
        engine.advance_time(Duration::from_secs(300)); // 5 minutes
        let signature = LegalSignature {
            signature: vec![0xAB; 65],
            business_name: "Pizza Hut Franchise #4521".into(),
            business_license: "CA-REST-182947".into(),
            document_hash: [0xCD; 32],
            timestamp: unix_timestamp(),
        };
        engine.seller_accept(&order_id, signature).unwrap();

        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::SellerAccepted);

        // Seller fulfills on time
        engine.advance_time(Duration::from_secs(1800)); // 30 minutes
        engine.seller_fulfill(&order_id).unwrap();

        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::SellerFulfilled);

        // Seller claims payment
        let receipt_id = engine.seller_claim(&order_id).unwrap();
        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::SellerClaimed);

        // Receipt should have no discount
        let receipt = engine.get_receipt(receipt_id).unwrap();
        assert!(!receipt.metadata.late_fulfilled);
        assert_eq!(receipt.metadata.discount_pct, 0);
    }

    // --------------------------------------------------------------
    // Test 2: Late Fulfillment With Re-Lock Logic
    // --------------------------------------------------------------
    #[test]
    fn test_pizza_delivery_late_fulfillment_with_relock() {
        let mut engine = CoreProverEngine::new();
        let pizza_price = 30_000_000;
        let profile = PaymentProfile::pizza_delivery();

        // Commit
        let order_id = engine
            .buyer_commit(
                "buyer_bob".into(),
                "pizza_palace".into(),
                pizza_price,
                profile,
            )
            .unwrap();

        // Accept
        engine.advance_time(Duration::from_secs(600)); // 10 minutes
        let signature = LegalSignature {
            signature: vec![0xEF; 65],
            business_name: "Pizza Palace".into(),
            business_license: "NY-REST-847261".into(),
            document_hash: [0x12; 32],
            timestamp: unix_timestamp(),
        };
        engine.seller_accept(&order_id, signature).unwrap();

        // Fulfillment window expires
        engine.advance_time(Duration::from_secs(3601));
        engine.update_escrow_state(&order_id).unwrap();

        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::FulfillmentExpired);

        // Late fulfillment
        engine.advance_time(Duration::from_secs(300));
        engine.seller_fulfill(&order_id).unwrap();

        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::LateFulfilled);

        // Withdrawal should now be locked again
        let result = engine.buyer_withdraw(&order_id);
        assert!(result.is_err());

        // Claim
        let receipt_id = engine.seller_claim(&order_id).unwrap();
        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::SellerClaimed);

        // Validate discount
        let receipt = engine.get_receipt(receipt_id).unwrap();
        assert!(receipt.metadata.late_fulfilled);
        assert_eq!(receipt.metadata.discount_pct, 10);
    }

    // --------------------------------------------------------------
    // Test 3: Buyer Withdrawal After Acceptance Timeout
    // --------------------------------------------------------------
    #[test]
    fn test_buyer_withdrawal_acceptance_timeout() {
        let mut engine = CoreProverEngine::new();
        let taco_price = 20_000_000;
        let profile = PaymentProfile::pizza_delivery();

        // Commit
        let order_id = engine
            .buyer_commit(
                "buyer_charlie".into(),
                "slow_pizza".into(),
                taco_price,
                profile,
            )
            .unwrap();

        // Wait past acceptance
        engine.advance_time(Duration::from_secs(1801));

        // Withdraw
        let refunded = engine.buyer_withdraw(&order_id).unwrap();
        assert_eq!(refunded, taco_price);
        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::BuyerWithdrawn);
    }

    // --------------------------------------------------------------
    // Test 4: Buyer Withdrawal After Fulfillment Timeout
    // --------------------------------------------------------------
    #[test]
    fn test_buyer_withdrawal_fulfillment_timeout() {
        let mut engine = CoreProverEngine::new();
        let price = 25_000_000;
        let profile = PaymentProfile::pizza_delivery();

        // Commit
        let order_id = engine
            .buyer_commit(
                "buyer_diana".into(),
                "unreliable_pizza".into(),
                price,
                profile,
            )
            .unwrap();

        // Accept
        let signature = LegalSignature {
            signature: vec![0xCD; 65],
            business_name: "Unreliable Pizza".into(),
            business_license: "CA-REST-999999".into(),
            document_hash: [0xEF; 32],
            timestamp: unix_timestamp(),
        };
        engine.seller_accept(&order_id, signature).unwrap();

        // Fulfillment timeout
        engine.advance_time(Duration::from_secs(3601));
        engine.update_escrow_state(&order_id).unwrap();

        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::FulfillmentExpired);

        // Withdraw
        let refunded = engine.buyer_withdraw(&order_id).unwrap();
        assert_eq!(refunded, price);
        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::BuyerWithdrawn);
    }

    // --------------------------------------------------------------
    // Test 5: Seller Cannot Claim Before Fulfillment
    // --------------------------------------------------------------
    #[test]
    fn test_seller_cannot_claim_before_fulfillment() {
        let mut engine = CoreProverEngine::new();
        let profile = PaymentProfile::pizza_delivery();

        let order_id = engine
            .buyer_commit(
                "buyer".into(),
                "seller".into(),
                1000,
                profile,
            )
            .unwrap();

        let signature = LegalSignature {
            signature: vec![0xAB; 65],
            business_name: "Test".into(),
            business_license: "TEST".into(),
            document_hash: [0xCD; 32],
            timestamp: unix_timestamp(),
        };

        engine.seller_accept(&order_id, signature).unwrap();

        let result = engine.seller_claim(&order_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Must fulfill first"));
    }

    // --------------------------------------------------------------
    // Test 6: Timed Release After Seller Forgets to Claim
    // --------------------------------------------------------------
    #[test]
    fn test_timed_release_after_late_fulfillment() {
        let mut engine = CoreProverEngine::new();
        let price = 28_000_000;
        let profile = PaymentProfile::pizza_delivery();

        // Commit + accept
        let order_id = engine
            .buyer_commit(
                "buyer_eve".into(),
                "forgetful_pizza".into(),
                price,
                profile,
            )
            .unwrap();

        let signature = LegalSignature {
            signature: vec![0x11; 65],
            business_name: "Forgetful Pizza".into(),
            business_license: "CA-REST-111111".into(),
            document_hash: [0x22; 32],
            timestamp: unix_timestamp(),
        };
        engine.seller_accept(&order_id, signature).unwrap();

        // Fulfillment expires
        engine.advance_time(Duration::from_secs(3601));
        engine.update_escrow_state(&order_id).unwrap();

        // Late fulfillment
        engine.seller_fulfill(&order_id).unwrap();
        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::LateFulfilled);

        // Claim window expires
        engine.advance_time(Duration::from_secs(3601));

        // Trigger timed release
        let receipt_id = engine.trigger_timed_release(&order_id).unwrap();
        assert_eq!(engine.get_state(&order_id).unwrap(), EscrowState::SellerClaimed);

        let receipt = engine.get_receipt(receipt_id).unwrap();
        assert!(receipt.metadata.late_fulfilled);
        assert_eq!(receipt.metadata.discount_pct, 10);
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