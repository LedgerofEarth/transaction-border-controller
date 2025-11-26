//! Payment profile templates

use coreprover_bridge::types::{PaymentProfile, SellerCommitmentType, FulfillmentType};

/// Pizza delivery payment profile
pub fn pizza_delivery_profile() -> PaymentProfile {
    PaymentProfile {
        required_commitment_type: SellerCommitmentType::LegalSignature,
        counter_escrow_amount: 0,
        commitment_window: 1800,  // 30 minutes
        claim_window: 3600,       // 1 hour
        fulfillment_type: FulfillmentType::Service,
        requires_tracking: false,
        allows_timed_release: true,
        timed_release_delay: 3600,  // 1 hour auto-release
        payment_token: "USDC".to_string(),
        price_in_usd: 25,
        accepts_multiple_assets: false,
    }
}

/// Digital goods payment profile
pub fn digital_goods_profile() -> PaymentProfile {
    PaymentProfile {
        required_commitment_type: SellerCommitmentType::LegalSignature,
        counter_escrow_amount: 0,
        commitment_window: 3600,  // 1 hour
        claim_window: 86400,      // 24 hours
        fulfillment_type: FulfillmentType::Digital,
        requires_tracking: false,
        allows_timed_release: false,
        timed_release_delay: 0,
        payment_token: "USDC".to_string(),
        price_in_usd: 99,
        accepts_multiple_assets: true,
    }
}

/// Physical goods with counter-escrow
pub fn physical_goods_profile(price: u64) -> PaymentProfile {
    PaymentProfile {
        required_commitment_type: SellerCommitmentType::CounterEscrow,
        counter_escrow_amount: price as u128,  // Match buyer payment
        commitment_window: 86400,    // 24 hours
        claim_window: 604800,        // 7 days
        fulfillment_type: FulfillmentType::Shipping,
        requires_tracking: true,
        allows_timed_release: false,
        timed_release_delay: 0,
        payment_token: "USDC".to_string(),
        price_in_usd: price,
        accepts_multiple_assets: false,
    }
}