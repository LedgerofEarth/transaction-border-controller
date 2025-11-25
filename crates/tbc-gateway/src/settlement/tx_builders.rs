//! tx_builder.rs
//! Builds ABI-correct calldata for SettlementContractTemplate v0.2.2

use anyhow::{Result, bail};
use ethers::{
    types::{Bytes, U256, Address},
    abi::Abi,
    contract::Contract,
    providers::Middleware,
};

use crate::zk::rewriter::{
    BuyerZKProof, SellerZKProof, MerchantCommitRewritten
};

// Optional: for reading deployment metadata
use std::sync::Arc;

// ---------------------------------------------------------
// SettlementTxBuilder
// ---------------------------------------------------------

pub struct SettlementTxBuilder<M: Middleware> {
    pub client: Arc<M>,
    pub settlement_addr: Address,
    pub settlement_abi: Abi,
}

impl<M: Middleware> SettlementTxBuilder<M> {
    pub fn new(client: Arc<M>, settlement_addr: Address, settlement_abi: Abi) -> Self {
        Self {
            client,
            settlement_addr,
            settlement_abi,
        }
    }

    // -----------------------------------------------------
    // BUYER COMMIT
    // -----------------------------------------------------

    pub fn build_buyer_commit(
        &self,
        proof: BuyerZKProof
    ) -> Result<(Address, Bytes)> {
        let contract = Contract::new(self.settlement_addr, &self.settlement_abi, self.client.clone());

        let calldata = contract
            .method::<_, ()>(
                "buyerCommit",
                (
                    // Proof struct
                    (proof.proof.a, proof.proof.b, proof.proof.c),
                    // Nullifier struct
                    (proof.nullifier.value, proof.nullifier.epoch),
                    // Amount
                    proof.amount,
                    // Asset
                    proof.asset_id,
                    // Expiry
                    proof.expiry,
                    // pkHash
                    proof.pk_hash
                )
            )?
            .calldata()
            .ok_or_else(|| anyhow::anyhow!("failed to build buyerCommit calldata"))?;

        Ok((self.settlement_addr, calldata))
    }

    // -----------------------------------------------------
    // SELLER COMMIT
    // -----------------------------------------------------

    pub fn build_seller_commit(
        &self,
        proof: SellerZKProof
    ) -> Result<(Address, Bytes)> {
        let contract = Contract::new(self.settlement_addr, &self.settlement_abi, self.client.clone());

        let calldata = contract
            .method::<_, ()>(
                "sellerCommit",
                (
                    (proof.proof.a, proof.proof.b, proof.proof.c),
                    (proof.nullifier.value, proof.nullifier.epoch),
                    proof.fulfil_hash,
                    proof.expiry,
                    proof.pk_hash
                )
            )?
            .calldata()
            .ok_or_else(|| anyhow::anyhow!("failed to build sellerCommit calldata"))?;

        Ok((self.settlement_addr, calldata))
    }

    // -----------------------------------------------------
    // SETTLE
    // -----------------------------------------------------

    pub fn build_settle(&self) -> Result<(Address, Bytes)> {
        let contract = Contract::new(self.settlement_addr, &self.settlement_abi, self.client.clone());

        let calldata = contract
            .method::<_, ()>("settle", ())?
            .calldata()
            .ok_or_else(|| anyhow::anyhow!("failed to build settle calldata"))?;

        Ok((self.settlement_addr, calldata))
    }

    // -----------------------------------------------------
    // OPTIONAL: GAS ESTIMATION
    // -----------------------------------------------------

    pub async fn estimate_gas(
        &self,
        to: Address,
        data: Bytes
    ) -> Result<U256> {
        let gas = self.client
            .estimate_gas(
                &ethers::types::TransactionRequest {
                    to: Some(to),
                    data: Some(data),
                    ..Default::default()
                },
                None
            )
            .await?;

        Ok(gas)
    }
}