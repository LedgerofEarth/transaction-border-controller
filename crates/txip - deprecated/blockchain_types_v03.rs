// crates/tbc-gateway/src/txip/blockchain_types_v03.rs
// FINAL - CoreProver v0.3 Blockchain Provenance Types
//
// This module defines TXID tracking and chain provenance for v0.3
//
// Required TXIDs:
// - buyer_commit_txid
// - seller_accept_txid
// - seller_fulfill_txid
// - seller_claim_txid OR seller_refund_txid
//
// Optional:
// - buyer_withdraw_txid

use serde::{Deserialize, Serialize};

/// Transaction ID (blockchain transaction hash)
/// Always serialized as hex string with 0x prefix
pub type TxId = String;

/// Chain ID (EVM chain identifier)
pub type ChainId = u64;

/// Block height on blockchain
pub type BlockHeight = u64;

/// Blockchain transaction reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxRef {
    /// Chain where transaction occurred
    pub chain_id: ChainId,
    
    /// Transaction hash
    pub txid: TxId,
    
    /// Block height where transaction was mined
    pub block_height: BlockHeight,
}

impl TxRef {
    /// Create a new transaction reference
    pub fn new(chain_id: ChainId, txid: TxId, block_height: BlockHeight) -> Self {
        Self {
            chain_id,
            txid,
            block_height,
        }
    }

    /// Validate transaction ID format (should start with 0x and be 66 chars)
    pub fn validate_txid(&self) -> Result<(), String> {
        if !self.txid.starts_with("0x") {
            return Err("TXID must start with 0x".to_string());
        }
        
        if self.txid.len() != 66 {
            return Err(format!(
                "TXID must be 66 characters (0x + 64 hex), got {}",
                self.txid.len()
            ));
        }
        
        // Check all characters after 0x are hex
        for c in self.txid[2..].chars() {
            if !c.is_ascii_hexdigit() {
                return Err(format!("Invalid hex character in TXID: {}", c));
            }
        }
        
        Ok(())
    }
}

/// Buyer blockchain actions with TXIDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuyerTxIds {
    /// REQUIRED: Buyer commits payment to escrow
    pub commit_txid: TxId,
    
    /// Chain where buyer committed
    pub chain_id: ChainId,
    
    /// OPTIONAL: Buyer withdraws (only valid in specific states)
    pub withdraw_txid: Option<TxId>,
}

impl BuyerTxIds {
    /// Create buyer TXIDs with commit only
    pub fn new(chain_id: ChainId, commit_txid: TxId) -> Self {
        Self {
            commit_txid,
            chain_id,
            withdraw_txid: None,
        }
    }

    /// Add withdraw transaction
    pub fn with_withdraw(mut self, withdraw_txid: TxId) -> Self {
        self.withdraw_txid = Some(withdraw_txid);
        self
    }
}

/// Seller blockchain actions with TXIDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SellerTxIds {
    /// REQUIRED: Seller accepts order (legal signature or counter-escrow)
    pub accept_txid: TxId,
    
    /// Chain where seller operates
    pub chain_id: ChainId,
    
    /// REQUIRED: Seller fulfills order
    pub fulfill_txid: TxId,
    
    /// Block height of fulfill transaction
    pub fulfill_block_height: BlockHeight,
    
    /// REQUIRED: Seller claims payment OR gets refunded
    /// Exactly one must be present
    pub claim_txid: Option<TxId>,
    pub refund_txid: Option<TxId>,
}

impl SellerTxIds {
    /// Create seller TXIDs with accept and fulfill
    pub fn new(
        chain_id: ChainId,
        accept_txid: TxId,
        fulfill_txid: TxId,
        fulfill_block_height: BlockHeight,
    ) -> Self {
        Self {
            accept_txid,
            chain_id,
            fulfill_txid,
            fulfill_block_height,
            claim_txid: None,
            refund_txid: None,
        }
    }

    /// Add claim transaction
    pub fn with_claim(mut self, claim_txid: TxId) -> Self {
        self.claim_txid = Some(claim_txid);
        self
    }

    /// Add refund transaction
    pub fn with_refund(mut self, refund_txid: TxId) -> Self {
        self.refund_txid = Some(refund_txid);
        self
    }

    /// Validate that exactly one of claim or refund is present
    pub fn validate_settlement(&self) -> Result<(), String> {
        match (&self.claim_txid, &self.refund_txid) {
            (Some(_), None) => Ok(()),
            (None, Some(_)) => Ok(()),
            (Some(_), Some(_)) => {
                Err("Cannot have both claim_txid and refund_txid".to_string())
            }
            (None, None) => {
                Err("Must have either claim_txid or refund_txid".to_string())
            }
        }
    }

    /// Check if seller was paid (has claim_txid)
    pub fn was_paid(&self) -> bool {
        self.claim_txid.is_some()
    }

    /// Check if seller was refunded (has refund_txid)
    pub fn was_refunded(&self) -> bool {
        self.refund_txid.is_some()
    }
}

/// Complete TXID provenance for an order
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxIdProvenance {
    /// Buyer transactions
    pub buyer: BuyerTxIds,
    
    /// Seller transactions
    pub seller: SellerTxIds,
}

impl TxIdProvenance {
    /// Create new provenance record
    pub fn new(buyer: BuyerTxIds, seller: SellerTxIds) -> Self {
        Self { buyer, seller }
    }

    /// Validate all required TXIDs are present
    pub fn validate(&self) -> Result<(), String> {
        // Validate buyer commit
        if self.buyer.commit_txid.is_empty() {
            return Err("buyer_commit_txid is required".to_string());
        }

        // Validate seller accept
        if self.seller.accept_txid.is_empty() {
            return Err("seller_accept_txid is required".to_string());
        }

        // Validate seller fulfill
        if self.seller.fulfill_txid.is_empty() {
            return Err("seller_fulfill_txid is required".to_string());
        }

        // Validate settlement (claim OR refund)
        self.seller.validate_settlement()?;

        Ok(())
    }

    /// Get all chain IDs involved in this order
    pub fn chain_ids(&self) -> Vec<ChainId> {
        let mut chains = vec![self.buyer.chain_id, self.seller.chain_id];
        chains.sort_unstable();
        chains.dedup();
        chains
    }

    /// Check if order is cross-chain
    pub fn is_cross_chain(&self) -> bool {
        self.buyer.chain_id != self.seller.chain_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TXID: &str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    #[test]
    fn test_txref_validation() {
        let valid = TxRef::new(1, TEST_TXID.to_string(), 12345);
        assert!(valid.validate_txid().is_ok());

        let no_prefix = TxRef::new(1, "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(), 12345);
        assert!(no_prefix.validate_txid().is_err());

        let too_short = TxRef::new(1, "0x1234".to_string(), 12345);
        assert!(too_short.validate_txid().is_err());

        let invalid_hex = TxRef::new(1, "0xGGGG567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(), 12345);
        assert!(invalid_hex.validate_txid().is_err());
    }

    #[test]
    fn test_buyer_txids() {
        let buyer = BuyerTxIds::new(1, TEST_TXID.to_string())
            .with_withdraw("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string());

        assert_eq!(buyer.chain_id, 1);
        assert_eq!(buyer.commit_txid, TEST_TXID);
        assert!(buyer.withdraw_txid.is_some());
    }

    #[test]
    fn test_seller_txids() {
        let seller = SellerTxIds::new(
            1,
            TEST_TXID.to_string(),
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            12345,
        )
        .with_claim("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string());

        assert!(seller.was_paid());
        assert!(!seller.was_refunded());
        assert!(seller.validate_settlement().is_ok());
    }

    #[test]
    fn test_seller_settlement_validation() {
        let mut seller = SellerTxIds::new(
            1,
            TEST_TXID.to_string(),
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            12345,
        );

        // No settlement - should fail
        assert!(seller.validate_settlement().is_err());

        // Claim only - should pass
        seller.claim_txid = Some("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string());
        assert!(seller.validate_settlement().is_ok());

        // Both claim and refund - should fail
        seller.refund_txid = Some("0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string());
        assert!(seller.validate_settlement().is_err());

        // Refund only - should pass
        seller.claim_txid = None;
        assert!(seller.validate_settlement().is_ok());
    }

    #[test]
    fn test_provenance() {
        let buyer = BuyerTxIds::new(1, TEST_TXID.to_string());
        let seller = SellerTxIds::new(
            369,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            12345,
        )
        .with_claim("0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string());

        let provenance = TxIdProvenance::new(buyer, seller);

        assert!(provenance.validate().is_ok());
        assert!(provenance.is_cross_chain());
        assert_eq!(provenance.chain_ids(), vec![1, 369]);
    }
}
