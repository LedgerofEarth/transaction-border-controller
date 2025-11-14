//! Pizza Delivery Simulation - Final Architecture
//!
//! Canonical test sequence demonstrating:
//! 1. Buyer commits payment -> BuyerCommitted
//! 2. Seller accepts -> SellerAccepted (LOCKS withdrawal)
//! 3. Fulfillment window expires -> FulfillmentExpired (UNLOCKS withdrawal)
//! 4. Seller fulfills late -> LateFulfilled (RE-LOCKS withdrawal, triggers discount)
//! 5. Seller claims -> SellerClaimed + receipt with discount metadata
//!
//! This is the authoritative reference for the re-lock logic.

use std::time::Duration;

use coreprover_bridge::{
    EscrowState,
    PaymentProfile,
    SellerCommitmentType,
};

use coreprover_service::engine::CoreProverEngine;

// ============================================================================
// CANONICAL PIZZA SIMULATION TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------------

    fn unix_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    // ------------------------------------------------------------------------
    // Test 1: Happy Path - On-Time Fulfillment
    // ------------------------------------------------------------------------

    /// Flow: BuyerCommitted -> SellerAccepted -> SellerFulfilled -> SellerClaimed
    #[test]
    fn test_pizza_delivery_on_time() {
        println!();
        println!("==============================");
        println!("TEST 1: PIZZA ON-TIME");
        println!("==============================");
        println!();

        let mut engine = CoreProverEngine::new();
        let pizza_price = 25_000_000u64; // 25 units (for example, 25 USDC in 6 decimals)

        let profile = PaymentProfile::pizza_delivery();

        println!("Setup:");
        println!("  Pizza price: {} (25 nominal units)", pizza_price);
        println!("  Acceptance window: 30 minutes");
        println!("  Fulfillment window: 1 hour");
        println!("  Claim window: 1 hour");
        println!();

        // Step 1: Buyer commits payment
        println!("Step 1: Buyer commits payment");
        let order_id = engine
            .buyer_commit(
                "buyer_alice".to_string(),
                "pizza_hut".to_string(),
                pizza_price,
                profile,
            )
            .expect("buyer_commit failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::BuyerCommitted
        );
        println!("  State: BuyerCommitted");
        println!("  Withdrawal: UNLOCKED if acceptance deadline passes");
        println!();

        // Step 2: Seller accepts order (5 minutes later)
        println!("Step 2: Seller accepts order");
        engine.advance_time(Duration::from_secs(300)); // 5 minutes

        let signature = SellerCommitmentType::LegalSignature {
            signature: vec![0xAB; 65],
            business_name: "Pizza Hut Franchise #4521".to_string(),
            business_license: "CA-REST-182947".to_string(),
            document_hash: [0xCD; 32],
            timestamp: unix_timestamp(),
        };

        engine
            .seller_accept(&order_id, signature)
            .expect("seller_accept failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerAccepted
        );
        println!("  State: SellerAccepted");
        println!("  Buyer withdrawal LOCKED");
        println!();

        // Step 3: Pizza preparation and delivery (30 minutes later - within window)
        println!("Step 3: Pizza preparation and delivery");
        engine.advance_time(Duration::from_secs(1800)); // 30 minutes

        println!("  Preparing pizza...");
        println!("  Pizza ready");
        println!("  Driver dispatched");
        println!("  Pizza delivered");
        println!();

        // Step 4: Seller fulfills order (on time)
        println!("Step 4: Seller marks order as fulfilled");
        engine
            .seller_fulfill(&order_id)
            .expect("seller_fulfill failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerFulfilled
        );
        println!("  State: SellerFulfilled");
        println!("  Fulfillment: ON TIME");
        println!("  No discount coupon");
        println!();

        // Step 5: Seller claims payment
        println!("Step 5: Seller claims payment");
        let receipt_id = engine
            .seller_claim(&order_id)
            .expect("seller_claim failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerClaimed
        );
        println!("  State: SellerClaimed");
        println!("  Receipt minted: #{}", receipt_id);
        println!();

        // Verify receipt
        let receipt = engine.get_receipt(receipt_id).expect("missing receipt");
        assert!(!receipt.metadata.late_fulfilled);
        assert_eq!(receipt.metadata.discount_pct, 0);

        println!("On-time delivery complete:");
        println!("  - Seller received full payment");
        println!("  - No discount issued");
        println!("  - Receipt stored in vault");
        println!();
    }

    // ------------------------------------------------------------------------
    // Test 2: Canonical Re-Lock Flow (Late Fulfillment With Discount)
    // ------------------------------------------------------------------------

    /// Flow:
    ///   BuyerCommitted
    ///   -> SellerAccepted (LOCK)
    ///   -> FulfillmentExpired (UNLOCK)
    ///   -> LateFulfilled (RE-LOCK, discount applied)
    ///   -> SellerClaimed
    #[test]
    fn test_pizza_delivery_late_fulfillment_with_relock() {
        println!();
        println!("=======================================");
        println!("TEST 2: LATE FULFILLMENT WITH RE-LOCK");
        println!("=======================================");
        println!();

        let mut engine = CoreProverEngine::new();
        let pizza_price = 30_000_000u64;

        let profile = PaymentProfile::pizza_delivery();

        println!("Scenario: Seller fulfills after deadline (re-lock + discount)");
        println!();

        // Step 1: Buyer commits
        println!("Step 1: Buyer commits payment");
        let order_id = engine
            .buyer_commit(
                "buyer_bob".to_string(),
                "pizza_palace".to_string(),
                pizza_price,
                profile,
            )
            .expect("buyer_commit failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::BuyerCommitted
        );
        println!("  State: BuyerCommitted");
        println!("  Withdrawal: UNLOCKED if acceptance deadline passes");
        println!();

        // Step 2: Seller accepts (10 minutes later)
        println!("Step 2: Seller accepts order");
        engine.advance_time(Duration::from_secs(600)); // 10 minutes

        let signature = SellerCommitmentType::LegalSignature {
            signature: vec![0xEF; 65],
            business_name: "Pizza Palace".to_string(),
            business_license: "NY-REST-847261".to_string(),
            document_hash: [0x12; 32],
            timestamp: unix_timestamp(),
        };

        engine
            .seller_accept(&order_id, signature)
            .expect("seller_accept failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerAccepted
        );
        println!("  State: SellerAccepted");
        println!("  Buyer withdrawal LOCKED");
        println!("  Fulfillment window: 1 hour");
        println!();

        // Step 3: Wait past fulfillment window (1 hour + 1 second)
        println!("Step 3: Fulfillment window expires");
        engine.advance_time(Duration::from_secs(3601));

        engine
            .update_escrow_state(&order_id)
            .expect("update_escrow_state failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::FulfillmentExpired
        );
        println!("  State: FulfillmentExpired");
        println!("  Buyer withdrawal UNLOCKED");
        println!("  Buyer could withdraw now, but waits");
        println!();

        // Step 4: Seller fulfills late (5 minutes after deadline)
        println!("Step 4: Seller fulfills order (late)");
        engine.advance_time(Duration::from_secs(300)); // 5 minutes more

        engine
            .seller_fulfill(&order_id)
            .expect("seller_fulfill failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::LateFulfilled
        );
        println!("  State: LateFulfilled");
        println!("  Fulfilled AFTER deadline");
        println!("  Buyer withdrawal RE-LOCKED");
        println!("  Discount activated: 10% off next purchase (90 days)");
        println!();

        // Verify buyer CANNOT withdraw anymore
        let withdraw_result = engine.buyer_withdraw(&order_id);
        assert!(withdraw_result.is_err());
        println!("  Buyer withdrawal attempt after re-lock: BLOCKED");
        println!();

        // Step 5: Seller claims payment
        println!("Step 5: Seller claims payment");
        let receipt_id = engine
            .seller_claim(&order_id)
            .expect("seller_claim failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerClaimed
        );
        println!("  State: SellerClaimed");
        println!("  Receipt minted: #{}", receipt_id);
        println!();

        // Step 6: Verify receipt metadata
        println!("Step 6: Receipt metadata verification");
        let receipt = engine.get_receipt(receipt_id).expect("missing receipt");

        assert!(receipt.metadata.late_fulfilled);
        assert_eq!(receipt.metadata.discount_pct, 10);
        assert!(receipt.metadata.discount_expiration > 0);

        println!("  late_fulfilled: {}", receipt.metadata.late_fulfilled);
        println!("  discount_pct: {}", receipt.metadata.discount_pct);
        println!(
            "  discount_expiration: {}",
            receipt.metadata.discount_expiration
        );
        println!("  order_amount: {}", receipt.metadata.order_amount);
        println!(
            "  session_id prefix: 0x{}",
            hex_encode(&receipt.metadata.session_id[..4])
        );
        println!();

        println!("Late fulfillment flow complete:");
        println!("  - Seller received payment");
        println!("  - Buyer received 10% discount coupon");
        println!("  - Re-lock logic verified");
        println!("  - Receipt stored with discount metadata");
        println!();
    }

    // ------------------------------------------------------------------------
    // Test 3: Buyer Withdrawal After Acceptance Timeout
    // ------------------------------------------------------------------------

    /// Flow: BuyerCommitted -> (acceptance window expires) -> BuyerWithdrawn
    #[test]
    fn test_buyer_withdrawal_acceptance_timeout() {
        println!();
        println!("===================================");
        println!("TEST 3: BUYER WITHDRAW - NO ACCEPT");
        println!("===================================");
        println!();

        let mut engine = CoreProverEngine::new();
        let pizza_price = 20_000_000u64;

        let profile = PaymentProfile::pizza_delivery();

        println!("Scenario: Seller never accepts order");
        println!();

        // Buyer commits
        println!("Step 1: Buyer commits payment");
        let order_id = engine
            .buyer_commit(
                "buyer_charlie".to_string(),
                "slow_pizza".to_string(),
                pizza_price,
                profile,
            )
            .expect("buyer_commit failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::BuyerCommitted
        );
        println!("  State: BuyerCommitted");
        println!("  Acceptance deadline: 30 minutes");
        println!();

        // Wait past acceptance window
        println!("Step 2: Waiting for seller...");
        engine.advance_time(Duration::from_secs(1801)); // 30 min + 1 sec

        println!("  Seller never accepted; acceptance window expired");
        println!();

        // Buyer withdraws
        println!("Step 3: Buyer withdraws funds");
        let refunded = engine
            .buyer_withdraw(&order_id)
            .expect("buyer_withdraw failed");

        assert_eq!(refunded, pizza_price);
        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::BuyerWithdrawn
        );

        println!("  State: BuyerWithdrawn");
        println!("  Refunded: {}", refunded);
        println!();

        println!("Timeout refund complete:");
        println!("  - Buyer received full refund");
        println!("  - No loss to buyer");
        println!();
    }

    // ------------------------------------------------------------------------
    // Test 4: Buyer Withdrawal After Fulfillment Timeout
    // ------------------------------------------------------------------------

    /// Flow:
    ///   BuyerCommitted -> SellerAccepted (LOCK)
    ///   -> FulfillmentExpired (UNLOCK)
    ///   -> BuyerWithdrawn
    #[test]
    fn test_buyer_withdrawal_fulfillment_timeout() {
        println!();
        println!("=======================================");
        println!("TEST 4: BUYER WITHDRAW - NO FULFILL");
        println!("=======================================");
        println!();

        let mut engine = CoreProverEngine::new();
        let pizza_price = 25_000_000u64;

        let profile = PaymentProfile::pizza_delivery();

        println!("Scenario: Seller accepts but never fulfills");
        println!();

        // Buyer commits
        let order_id = engine
            .buyer_commit(
                "buyer_diana".to_string(),
                "unreliable_pizza".to_string(),
                pizza_price,
                profile,
            )
            .expect("buyer_commit failed");

        println!("Step 1: Buyer committed");

        // Seller accepts
        let signature = SellerCommitmentType::LegalSignature {
            signature: vec![0xCD; 65],
            business_name: "Unreliable Pizza".to_string(),
            business_license: "CA-REST-999999".to_string(),
            document_hash: [0xEF; 32],
            timestamp: unix_timestamp(),
        };

        engine
            .seller_accept(&order_id, signature)
            .expect("seller_accept failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerAccepted
        );
        println!("Step 2: Seller accepted");
        println!("  Withdrawal LOCKED");
        println!();

        // Wait past fulfillment window
        println!("Step 3: Fulfillment window expires");
        engine.advance_time(Duration::from_secs(3601)); // 1 hour + 1 sec
        engine
            .update_escrow_state(&order_id)
            .expect("update_escrow_state failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::FulfillmentExpired
        );
        println!("  State: FulfillmentExpired");
        println!("  Withdrawal UNLOCKED");
        println!();

        // Buyer withdraws
        println!("Step 4: Buyer withdraws funds");
        let refunded = engine
            .buyer_withdraw(&order_id)
            .expect("buyer_withdraw failed");

        assert_eq!(refunded, pizza_price);
        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::BuyerWithdrawn
        );

        println!("  State: BuyerWithdrawn");
        println!("  Refunded: {}", refunded);
        println!();

        println!("Fulfillment timeout refund complete:");
        println!("  - Buyer received full refund");
        println!("  - Seller failed to fulfill in time");
        println!();
    }

    // ------------------------------------------------------------------------
    // Test 5: Seller Cannot Claim Before Fulfillment
    // ------------------------------------------------------------------------

    #[test]
    fn test_seller_cannot_claim_before_fulfillment() {
        println!();
        println!("=============================================");
        println!("TEST 5: SELLER CANNOT CLAIM BEFORE FULFILL");
        println!("=============================================");
        println!();

        let mut engine = CoreProverEngine::new();
        let profile = PaymentProfile::pizza_delivery();

        let order_id = engine
            .buyer_commit(
                "buyer".to_string(),
                "seller".to_string(),
                1_000u64,
                profile,
            )
            .expect("buyer_commit failed");

        let signature = SellerCommitmentType::LegalSignature {
            signature: vec![0xAB; 65],
            business_name: "Test Seller".to_string(),
            business_license: "TEST".to_string(),
            document_hash: [0xCD; 32],
            timestamp: unix_timestamp(),
        };

        engine
            .seller_accept(&order_id, signature)
            .expect("seller_accept failed");

        // Try to claim before fulfillment
        let result = engine.seller_claim(&order_id);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Must fulfill first"),
            "unexpected error message: {err}"
        );

        println!("Seller claim before fulfillment correctly rejected:");
        println!("  Error: {}", err);
        println!();
    }

    // ------------------------------------------------------------------------
    // Test 6: Timed Release After Seller Forgets To Claim
    // ------------------------------------------------------------------------

    #[test]
    fn test_timed_release_after_late_fulfillment() {
        println!();
        println!("============================================");
        println!("TEST 6: TIMED RELEASE AFTER LATE FULFILL");
        println!("============================================");
        println!();

        let mut engine = CoreProverEngine::new();
        let pizza_price = 28_000_000u64;

        let profile = PaymentProfile::pizza_delivery();

        println!("Scenario: Late fulfillment + seller forgets to claim");
        println!();

        // Setup: buyer commits, seller accepts
        let order_id = engine
            .buyer_commit(
                "buyer_eve".to_string(),
                "forgetful_pizza".to_string(),
                pizza_price,
                profile,
            )
            .expect("buyer_commit failed");

        let signature = SellerCommitmentType::LegalSignature {
            signature: vec![0x11; 65],
            business_name: "Forgetful Pizza".to_string(),
            business_license: "CA-REST-111111".to_string(),
            document_hash: [0x22; 32],
            timestamp: unix_timestamp(),
        };

        engine
            .seller_accept(&order_id, signature)
            .expect("seller_accept failed");
        println!("  Seller accepted");
        println!();

        // Fulfillment expires
        engine.advance_time(Duration::from_secs(3601));
        engine
            .update_escrow_state(&order_id)
            .expect("update_escrow_state failed");
        println!("  Fulfillment window expired");
        println!();

        // Late fulfillment
        engine
            .seller_fulfill(&order_id)
            .expect("seller_fulfill failed");
        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::LateFulfilled
        );
        println!("  Late fulfillment completed");
        println!("  Withdrawal re-locked");
        println!();

        // Seller forgets to claim - wait past claim window
        println!("  Seller forgets to claim...");
        engine.advance_time(Duration::from_secs(3601)); // Past claim window

        println!("  Claim window expired; anyone can trigger timed release");
        println!();

        // Trigger timed release
        let receipt_id = engine
            .trigger_timed_release(&order_id)
            .expect("trigger_timed_release failed");

        assert_eq!(
            engine.get_state(&order_id).unwrap(),
            EscrowState::SellerClaimed
        );
        println!("Timed release complete:");
        println!("  Payment automatically released to seller");
        println!("  Receipt minted: #{}", receipt_id);
        println!("  Includes 10% discount coupon");
        println!();
    }
}