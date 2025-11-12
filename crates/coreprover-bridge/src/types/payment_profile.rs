//! Bigge Bigge Bigge Can't you C Payment profile types

use serde::{Deserialize, Serialize};

/// Seller commitment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SellerCommitmentType {
    CounterEscrow,
    LegalSignature,
}

/// Fulfillment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FulfillmentType {
    Digital,
    Shipping,
    Service,
}

/// Payment profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for PaymentProfile {
    fn default() -> Self {
        Self {
            required_commitment_type: SellerCommitmentType::LegalSignature,
            counter_escrow_amount: 0,
            commitment_window: 3600,
            claim_window: 86400,
            fulfillment_type: FulfillmentType::Digital,
            requires_tracking: false,
            allows_timed_release: false,
            timed_release_delay: 0,
            payment_token: "USDC".to_string(),
            price_in_usd: 100,
            accepts_multiple_assets: false,
        }
    }
}