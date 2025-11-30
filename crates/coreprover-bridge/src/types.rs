//! Shared types for CoreProver Bridge

use serde::{Deserialize, Serialize};

/// Type of commitment required from the seller
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SellerCommitmentType {
    #[default]
    LegalSignature,
    CounterEscrow,
}

/// Type of fulfillment for the order
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum FulfillmentType {
    #[default]
    Service,
    Digital,
    Shipping,
}

/// Payment profile configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaymentProfile {
    pub required_commitment_type: SellerCommitmentType,
    pub counter_escrow_amount: u128,
    pub commitment_window: u64,
    pub claim_window: u64,
    pub fulfillment_type: FulfillmentType,
    pub requires_tracking: bool,
    pub allows_timed_release: bool,
    pub timed_release_delay: u64,
    pub payment_token: String,
    pub price_in_usd: u64,
    pub accepts_multiple_assets: bool,
}
