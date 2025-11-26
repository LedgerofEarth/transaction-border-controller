//! Escrow builder for fluent API

use anyhow::Result;
use coreprover_bridge::types::PaymentProfile;

/// Builder for creating escrows
pub struct EscrowBuilder {
    buyer: Option<String>,
    seller: Option<String>,
    amount: Option<u128>,
    profile: Option<PaymentProfile>,
}

impl EscrowBuilder {
    pub fn new() -> Self {
        Self {
            buyer: None,
            seller: None,
            amount: None,
            profile: None,
        }
    }
    
    pub fn with_buyer(mut self, buyer: &str) -> Self {
        self.buyer = Some(buyer.to_string());
        self
    }
    
    pub fn with_seller(mut self, seller: &str) -> Self {
        self.seller = Some(seller.to_string());
        self
    }
    
    pub fn with_amount(mut self, amount: u128) -> Self {
        self.amount = Some(amount);
        self
    }
    
    pub fn with_profile(mut self, profile: PaymentProfile) -> Self {
        self.profile = Some(profile);
        self
    }
    
    pub async fn build(self) -> Result<Escrow> {
        let buyer = self.buyer.ok_or_else(|| anyhow::anyhow!("Buyer address required"))?;
        let seller = self.seller.ok_or_else(|| anyhow::anyhow!("Seller address required"))?;
        let amount = self.amount.ok_or_else(|| anyhow::anyhow!("Amount required"))?;
        let profile = self.profile.unwrap_or_default();
        
        Ok(Escrow {
            buyer,
            seller,
            amount,
            profile,
        })
    }
}

impl Default for EscrowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Created escrow
pub struct Escrow {
    pub buyer: String,
    pub seller: String,
    pub amount: u128,
    pub profile: PaymentProfile,
}