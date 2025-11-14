//! CoreProver Pizza Delivery Simulation (Dual-Commit Escrow)
//!
//! Scenario:
//! 1. Buyer commits and funds escrow  → BuyerCommitted
//! 2. Seller accepts order            → SellerAccepted (fulfillment window starts)
//! 3. If seller fails to fulfill in time → FulfillmentExpired (buyer can withdraw)
//! 4. If seller fulfills late (after FulfillmentExpired but before withdraw):
//!      - Buyer withdrawal is re-locked
//!      - Receipt is marked DiscountEligible (10% coupon on future order)
//! 5. Seller claims funds  → SellerClaimed
//!
//! Buyer withdrawal rules:
//! - Allowed if seller NEVER accepts and acceptance window expires
//! - Allowed if seller accepted but NEVER fulfills and fulfillment window expires
//! - NOT allowed after fulfillment (early or late)

use std::time::{Duration, Instant};

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscrowState {
    BuyerCommitted,
    SellerAccepted,
    FulfillmentExpired,
    SellerFulfilled,
    SellerClaimed,
    BuyerRefunded,
    SellerRefunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReceiptType {
    Full,
    DiscountEligible, // late fulfillment; coupon for future purchase
    Refunded,
}

#[derive(Debug, Clone)]
struct PaymentProfile {
    acceptance_window: Duration,
    fulfillment_window: Duration,
}

#[derive(Debug, Clone)]
struct Escrow {
    order_id: [u8; 32],
    buyer: String,
    seller: String,
    amount: u64,
    state: EscrowState,
    profile: PaymentProfile,

    buyer_commit_time: Instant,
    seller_accept_time: Option<Instant>,
    seller_fulfill_time: Option<Instant>,

    buyer_withdraw_allowed: bool,
    buyer_withdrew: bool,

    late_fulfillment: bool,
}

#[derive(Debug, Clone)]
struct Receipt {
    receipt_id: u64,
    order_id: [u8; 32],
    buyer: String,
    seller: String,
    amount: u64,
    receipt_type: ReceiptType,
    late_fulfillment: bool,
    discount_pct: u8, // 10 for late fulfillment coupon
}

// ============================================================================
// CoreProver Escrow Engine
// ============================================================================

struct CoreProverEngine {
    escrows: Vec<Escrow>,
    receipts: Vec<Receipt>,
    next_receipt_id: u64,
    current_time: Instant,
    order_counter: u64,
}

impl CoreProverEngine {
    fn new() -> Self {
        Self {
            escrows: Vec::new(),
            receipts: Vec::new(),
            next_receipt_id: 1,
            current_time: Instant::now(),
            order_counter: 0,
        }
    }

    // ------------------------------------------------------------------------
    // Time control (simulation only)
    // ------------------------------------------------------------------------

    fn advance_time(&mut self, duration: Duration) {
        self.current_time += duration;
    }

    // ------------------------------------------------------------------------
    // BUYER ACTIONS
    // ------------------------------------------------------------------------

    /// Buyer commits payment - creates escrow and locks funds.
    /// This is the ONLY way to create an escrow.
    fn buyer_commit(
        &mut self,
        buyer: String,
        seller: String,
        amount: u64,
        profile: PaymentProfile,
    ) -> [u8; 32] {
        let order_id = self.generate_order_id(&buyer, &seller);

        let escrow = Escrow {
            order_id,
            buyer,
            seller,
            amount,
            state: EscrowState::BuyerCommitted,
            profile,
            buyer_commit_time: self.current_time,
            seller_accept_time: None,
            seller_fulfill_time: None,
            buyer_withdraw_allowed: false,
            buyer_withdrew: false,
            late_fulfillment: false,
        };

        self.escrows.push(escrow);

        println!("✅ BUYER_COMMITTED: Order 0x{}", hex_encode(&order_id[..4]));

        order_id
    }

    /// Buyer can withdraw ONLY in two cases:
    /// 1) Seller never accepts and acceptance window expires.
    /// 2) Seller accepted but never fulfills and fulfillment window expires.
    fn buyer_withdraw(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        self.update_timeouts_for(order_id)?;

        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.buyer_withdrew {
            return Err("Buyer already withdrew".to_string());
        }

        if matches!(
            escrow.state,
            EscrowState::SellerClaimed | EscrowState::SellerRefunded
        ) {
            return Err("Escrow already settled; cannot withdraw".to_string());
        }

        if !escrow.buyer_withdraw_allowed {
            return Err("Buyer withdraw not currently allowed".to_string());
        }

        if escrow.seller_fulfill_time.is_some() {
            return Err("Seller has already fulfilled; cannot withdraw".to_string());
        }

        escrow.buyer_withdrew = true;
        escrow.state = EscrowState::BuyerRefunded;

        let amount = escrow.amount;

        println!(
            "💰 BUYER_WITHDRAW: Refunded {} wei to buyer (Order 0x{})",
            amount,
            hex_encode(&escrow.order_id[..4])
        );

        // Mint a refunded receipt (optional; can be used for audit)
        self.mint_receipt(escrow, ReceiptType::Refunded, 0);

        Ok(amount)
    }

    // ------------------------------------------------------------------------
    // SELLER ACTIONS
    // ------------------------------------------------------------------------

    /// Seller accepts the order.
    /// - If acceptance window has already expired, this is "late acceptance"
    /// - In all cases (if allowed), buyer withdrawal is re-locked.
    fn seller_accept(&mut self, order_id: &[u8; 32]) -> Result<(), String> {
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.buyer_withdrew {
            return Err("Buyer already withdrew; cannot accept".to_string());
        }

        if escrow.state != EscrowState::BuyerCommitted {
            return Err(format!(
                "Invalid state for seller_accept: {:?}",
                escrow.state
            ));
        }

        let now = self.current_time;
        escrow.seller_accept_time = Some(now);
        escrow.state = EscrowState::SellerAccepted;
        escrow.buyer_withdraw_allowed = false; // re-lock

        let accept_deadline = escrow.buyer_commit_time + escrow.profile.acceptance_window;
        let fulfill_deadline = now + escrow.profile.fulfillment_window;
        let late_accept = now > accept_deadline;

        println!(
            "✍️  SELLER_ACCEPTED: Order 0x{}{}",
            hex_encode(&escrow.order_id[..4]),
            if late_accept { " (late acceptance)" } else { "" }
        );
        println!(
            "   Fulfillment window: {:?} (until ~now+{:?})",
            escrow.profile.fulfillment_window, escrow.profile.fulfillment_window
        );
        println!("   Buyer withdraw re-locked until/unless fulfillment expires\n");

        println!(
            "   → Acceptance deadline was {:?} after commit; fulfill deadline is {:?} from now",
            escrow.profile.acceptance_window, escrow.profile.fulfillment_window
        );
        println!("   (Fulfillment deadline instant: {:?})\n", fulfill_deadline);

        Ok(())
    }

    /// Seller fulfills the order (pizza delivered).
    /// - Can be on-time or late.
    /// - If late (after fulfillment window), receipt metadata will mark it
    ///   DiscountEligible and store a 10% coupon value.
    /// - Late fulfillment re-locks buyer withdrawal if it was previously unlocked.
    fn seller_fulfill(&mut self, order_id: &[u8; 32]) -> Result<(), String> {
        // Update timeouts first so we know whether we passed fulfillment window.
        self.update_timeouts_for(order_id)?;

        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.buyer_withdrew {
            return Err("Buyer already withdrew; cannot fulfill".to_string());
        }

        if !matches!(
            escrow.state,
            EscrowState::SellerAccepted | EscrowState::FulfillmentExpired
        ) {
            return Err(format!(
                "Invalid state for seller_fulfill: {:?}",
                escrow.state
            ));
        }

        if escrow.seller_fulfill_time.is_some() {
            return Err("Seller already fulfilled".to_string());
        }

        let now = self.current_time;
        let accept_time = escrow
            .seller_accept_time
            .ok_or("No seller_accept_time recorded")?;
        let fulfill_deadline = accept_time + escrow.profile.fulfillment_window;

        let is_late = now > fulfill_deadline;

        escrow.seller_fulfill_time = Some(now);
        escrow.state = EscrowState::SellerFulfilled;
        escrow.late_fulfillment = is_late;
        escrow.buyer_withdraw_allowed = false; // re-lock after fulfillment

        println!(
            "🍕 SELLER_FULFILLED: Order 0x{} ({}fulfillment)",
            hex_encode(&escrow.order_id[..4]),
            if is_late { "LATE " } else { "" }
        );
        println!("   👨‍🍳 Preparing pizza...");
        println!("   📦 Pizza ready");
        println!("   🚗 Driver dispatched");
        println!("   🏠 Pizza delivered!\n");

        if is_late {
            println!("   ⚠️ Fulfillment window expired before delivery.");
            println!("   → Receipt will be marked DiscountEligible.");
            println!("   → Coupon value: 10% of this order amount (off-chain).\n");
        }

        Ok(())
    }

    /// Seller claims payment and receipt is minted into the "vault".
    /// - If fulfillment was on-time: ReceiptType::Full
    /// - If fulfillment was late:   ReceiptType::DiscountEligible (10% coupon)
    fn seller_claim(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.buyer_withdrew {
            return Err("Buyer already withdrew; cannot claim".to_string());
        }

        if escrow.state != EscrowState::SellerFulfilled {
            return Err(format!(
                "Invalid state for seller_claim: {:?}",
                escrow.state
            ));
        }

        let amount = escrow.amount;

        let (receipt_type, discount_pct) = if escrow.late_fulfillment {
            (ReceiptType::DiscountEligible, 10)
        } else {
            (ReceiptType::Full, 0)
        };

        let receipt_id = self.mint_receipt(escrow, receipt_type, discount_pct);

        escrow.state = EscrowState::SellerClaimed;

        println!("💸 SELLER_CLAIMED: Payment released to seller");
        println!("   Amount: {} wei", amount);
        println!("   Receipt ID: {}", receipt_id);
        println!(
            "   Receipt type: {:?} (discount_pct = {}%)\n",
            receipt_type, discount_pct
        );

        Ok(amount)
    }

    /// Seller can voluntarily refund the buyer instead of claiming funds.
    fn seller_refund(&mut self, order_id: &[u8; 32]) -> Result<u64, String> {
        let escrow = self.get_escrow_mut(order_id)?;

        if escrow.buyer_withdrew {
            return Err("Buyer already withdrew; cannot seller_refund".to_string());
        }

        if matches!(
            escrow.state,
            EscrowState::SellerClaimed | EscrowState::SellerRefunded | EscrowState::BuyerRefunded
        ) {
            return Err("Escrow already settled; cannot seller_refund".to_string());
        }

        let amount = escrow.amount;
        escrow.state = EscrowState::SellerRefunded;
        escrow.buyer_withdrew = true; // from protocol perspective, funds go back to buyer

        println!(
            "🔁 SELLER_REFUNDED: Seller refunded {} wei to buyer (Order 0x{})",
            amount,
            hex_encode(&escrow.order_id[..4])
        );

        self.mint_receipt(escrow, ReceiptType::Refunded, 0);

        Ok(amount)
    }

    // ------------------------------------------------------------------------
    // STATE / TIMEOUT HELPERS
    // ------------------------------------------------------------------------

    fn get_state(&self, order_id: &[u8; 32]) -> Result<EscrowState, String> {
        Ok(self.get_escrow(order_id)?.state)
    }

    fn update_timeouts_for(&mut self, order_id: &[u8; 32]) -> Result<(), String> {
        let escrow = self.get_escrow_mut(order_id)?;
        let now = self.current_time;

        match escrow.state {
            EscrowState::BuyerCommitted => {
                if escrow.seller_accept_time.is_none() {
                    let accept_deadline =
                        escrow.buyer_commit_time + escrow.profile.acceptance_window;
                    if now >= accept_deadline {
                        escrow.buyer_withdraw_allowed = true;
                    }
                }
            }
            EscrowState::SellerAccepted => {
                if let Some(accept_time) = escrow.seller_accept_time {
                    let fulfill_deadline = accept_time + escrow.profile.fulfillment_window;
                    if now >= fulfill_deadline && escrow.seller_fulfill_time.is_none() {
                        escrow.state = EscrowState::FulfillmentExpired;
                        escrow.buyer_withdraw_allowed = true;
                        println!(
                            "⏰ FULFILLMENT_EXPIRED: Buyer may now withdraw (Order 0x{})",
                            hex_encode(&escrow.order_id[..4])
                        );
                    }
                }
            }
            EscrowState::FulfillmentExpired => {
                // No automatic transitions here; seller_fulfill or buyer_withdraw will act.
            }
            _ => {
                // Other states do not have time-based transitions in this sim.
            }
        }

        Ok(())
    }

    // ------------------------------------------------------------------------
    // INTERNAL HELPERS
    // ------------------------------------------------------------------------

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

    fn mint_receipt(
        &mut self,
        escrow: &Escrow,
        receipt_type: ReceiptType,
        discount_pct: u8,
    ) -> u64 {
        let receipt = Receipt {
            receipt_id: self.next_receipt_id,
            order_id: escrow.order_id,
            buyer: escrow.buyer.clone(),
            seller: escrow.seller.clone(),
            amount: escrow.amount,
            receipt_type,
            late_fulfillment: escrow.late_fulfillment,
            discount_pct,
        };

        let receipt_id = receipt.receipt_id;
        self.receipts.push(receipt);
        self.next_receipt_id += 1;
        receipt_id
    }

    fn generate_order_id(&mut self, buyer: &str, seller: &str) -> [u8; 32] {
        // Pseudo-keccak-style ID: counter + truncated buyer/seller bytes.
        self.order_counter += 1;
        let mut id = [0u8; 32];

        let counter_bytes = self.order_counter.to_be_bytes();
        id[0..8].copy_from_slice(&counter_bytes);

        let buyer_bytes = buyer.as_bytes();
        let seller_bytes = seller.as_bytes();

        // Mix buyer bytes
        for (i, b) in buyer_bytes.iter().take(12).enumerate() {
            id[8 + i] ^= *b;
        }
        // Mix seller bytes
        for (i, b) in seller_bytes.iter().take(12).enumerate() {
            id[20 + i] ^= *b;
        }

        id
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ============================================================================
// PIZZA DELIVERY SIMULATION TESTS
// ============================================================================

#[test]
fn test_pizza_delivery_happy_path() {
    println!("\n🍕 ═══════════════════════════════════════════════════");
    println!("   PIZZA DELIVERY - HAPPY PATH (Dual-Commit Model)");
    println!("   ═══════════════════════════════════════════════════\n");

    let mut engine = CoreProverEngine::new();
    let pizza_price = 25_000_000; // $25 in wei

    let profile = PaymentProfile {
        acceptance_window: Duration::from_secs(300),   // 5 minutes
        fulfillment_window: Duration::from_secs(1800), // 30 minutes
    };

    println!("📋 Setup:");
    println!("   Pizza price: {} wei ($25)", pizza_price);
    println!("   Seller Order Acceptance window: 5 minutes");
    println!("   Fulfillment window: 30 minutes");
    println!("   Timed release / coupon: late fulfillment → DiscountEligible receipt\n");

    // Step 1: Buyer commits
    println!("Step 1: After placing order, 💰 is required and Buyer now commits payment.\n");

    let order_id = engine
        .buyer_commit(
            "buyer_alice".to_string(),
            "pizza_hut".to_string(),
            pizza_price,
            profile.clone(),
        );

    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::BuyerCommitted
    );
    println!("   State: BuyerCommitted ✅\n");

    // Step 2: Seller accepts within acceptance window
    println!("Step 2: ✍️  Restaurant acknowledges and accepts the order via legal signature.\n");

    engine.advance_time(Duration::from_secs(120)); // 2 minutes
    engine.seller_accept(&order_id).unwrap();

    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::SellerAccepted
    );
    println!("   State: SellerAccepted ✅\n");

    // Step 3: Seller fulfills on time
    println!("Step 3: 🍕 Pizza preparation and delivery (on-time fulfillment)\n");

    engine.advance_time(Duration::from_secs(900)); // 15 minutes (within fulfillment window)
    engine.seller_fulfill(&order_id).unwrap();

    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::SellerFulfilled
    );

    // Step 4: Seller claims payment
    println!("Step 4: 💸 Restaurant claims payment");
    println!("   ⚠️  NO BUYER ACKNOWLEDGMENT REQUIRED\n");

    let claimed = engine.seller_claim(&order_id).unwrap();

    assert_eq!(claimed, pizza_price);
    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::SellerClaimed
    );

    println!("✨ Pizza Delivery Complete");
    println!("   ✓ Both parties committed");
    println!("   ✓ Payment released to seller");
    println!("   ✓ Receipt stored in vault");
    println!("   ✓ Buyer has NO veto power\n");
}

#[test]
fn test_buyer_timeout_no_acceptance() {
    println!("\n🍕 ═══════════════════════════════════════════════════");
    println!("   BUYER TIMEOUT REFUND - NO SELLER ACCEPTANCE");
    println!("   ═══════════════════════════════════════════════════\n");

    let mut engine = CoreProverEngine::new();
    let pizza_price = 20_000_000;

    let profile = PaymentProfile {
        acceptance_window: Duration::from_secs(300),   // 5 minutes
        fulfillment_window: Duration::from_secs(1800), // 30 minutes
    };

    println!("📋 Scenario: Restaurant never accepts the order\n");

    let order_id = engine
        .buyer_commit(
            "buyer_charlie".to_string(),
            "slow_pizza".to_string(),
            pizza_price,
            profile,
        );

    println!("✅ Buyer committed: {} wei", pizza_price);
    println!("⏰ Waiting for seller to accept...\n");

    // Advance past acceptance window without seller_accept
    engine.advance_time(Duration::from_secs(301));

    println!("❌ Seller never accepted");
    println!("⏰ Acceptance window expired\n");

    let refunded = engine.buyer_withdraw(&order_id).unwrap();

    assert_eq!(refunded, pizza_price);
    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::BuyerRefunded
    );

    println!("✨ Timeout Refund Complete");
    println!("   ✓ Buyer received: {} wei", refunded);
    println!("   ✓ Escrow closed without seller involvement\n");
}

#[test]
fn test_buyer_withdraw_after_fulfillment_expired() {
    println!("\n🍕 ═══════════════════════════════════════════════════");
    println!("   BUYER WITHDRAW AFTER FULFILLMENT EXPIRED");
    println!("   ═══════════════════════════════════════════════════\n");

    let mut engine = CoreProverEngine::new();
    let pizza_price = 30_000_000;

    let profile = PaymentProfile {
        acceptance_window: Duration::from_secs(300),
        fulfillment_window: Duration::from_secs(1800),
    };

    let order_id = engine
        .buyer_commit(
            "buyer_delta".to_string(),
            "sometimes_late_pizza".to_string(),
            pizza_price,
            profile,
        );

    engine.advance_time(Duration::from_secs(100));
    engine.seller_accept(&order_id).unwrap();

    println!("✅ Both committed; now testing fulfillment expiry...\n");

    // Advance past fulfillment window without fulfill
    engine.advance_time(Duration::from_secs(2000)); // > 1800

    // Buyer withdraws after fulfillment expired
    let refunded = engine.buyer_withdraw(&order_id).unwrap();

    assert_eq!(refunded, pizza_price);
    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::BuyerRefunded
    );

    println!("✨ Buyer Withdraw Complete");
    println!("   ✓ Fulfillment window expired with no delivery");
    println!("   ✓ Buyer refunded: {} wei", refunded);
    println!("   ✓ Seller received nothing\n");
}

#[test]
fn test_late_fulfillment_discount_eligible() {
    println!("\n🍕 ═══════════════════════════════════════════════════");
    println!("   LATE FULFILLMENT → DISCOUNT ELIGIBLE RECEIPT");
    println!("   ═══════════════════════════════════════════════════\n");

    let mut engine = CoreProverEngine::new();
    let pizza_price = 35_000_000;

    let profile = PaymentProfile {
        acceptance_window: Duration::from_secs(300),
        fulfillment_window: Duration::from_secs(1800),
    };

    let order_id = engine
        .buyer_commit(
            "buyer_echo".to_string(),
            "pizza_sometimes_late".to_string(),
            pizza_price,
            profile,
        );

    engine.advance_time(Duration::from_secs(60));
    engine.seller_accept(&order_id).unwrap();

    // Let fulfillment window expire first
    engine.advance_time(Duration::from_secs(2000)); // > 1800
    // Now buyer withdrawal is unlocked, but seller wins race with late fulfillment
    engine.seller_fulfill(&order_id).unwrap();

    // At this point, buyer withdrawal must be re-locked
    let withdraw_result = engine.buyer_withdraw(&order_id);
    assert!(withdraw_result.is_err());

    // Seller claims funds; receipt should be DiscountEligible
    let claimed = engine.seller_claim(&order_id).unwrap();
    assert_eq!(claimed, pizza_price);

    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::SellerClaimed
    );

    // Check receipt metadata from the engine
    let last_receipt = engine.receipts.last().expect("at least one receipt");
    assert_eq!(last_receipt.order_id, order_id);
    assert!(last_receipt.late_fulfillment);
    assert_eq!(last_receipt.receipt_type, ReceiptType::DiscountEligible);
    assert_eq!(last_receipt.discount_pct, 10);

    println!("✨ Late Fulfillment Complete");
    println!("   ✓ Fulfillment after window expiry");
    println!("   ✓ Buyer COULD have withdrawn but did not");
    println!("   ✓ Seller fulfilled late and then claimed");
    println!("   ✓ Receipt marked DiscountEligible with 10% coupon metadata\n");
}

#[test]
fn test_seller_refund_flow() {
    println!("\n🍕 ═══════════════════════════════════════════════════");
    println!("   SELLER VOLUNTARY REFUND FLOW");
    println!("   ═══════════════════════════════════════════════════\n");

    let mut engine = CoreProverEngine::new();
    let pizza_price = 40_000_000;

    let profile = PaymentProfile {
        acceptance_window: Duration::from_secs(300),
        fulfillment_window: Duration::from_secs(1800),
    };

    let order_id = engine
        .buyer_commit(
            "buyer_foxtrot".to_string(),
            "pizza_refund_shop".to_string(),
            pizza_price,
            profile,
        );

    engine.advance_time(Duration::from_secs(120));
    engine.seller_accept(&order_id).unwrap();

    println!("📋 Scenario: Seller must cancel the order and refund buyer\n");

    let refunded = engine.seller_refund(&order_id).unwrap();
    assert_eq!(refunded, pizza_price);
    assert_eq!(
        engine.get_state(&order_id).unwrap(),
        EscrowState::SellerRefunded
    );

    println!("✨ Seller Refund Complete");
    println!("   ✓ Seller cancelled after acceptance");
    println!("   ✓ Full amount refunded to buyer");
    println!("   ✓ Refunded receipt stored in vault\n");
}