//! rewriter.rs
//! Converts raw ZK envelopes (extension → TBC) into contract-ready proof structs.

use ethers::types::U256;
use anyhow::{Result, bail};

use crate::zk::types::*;
use crate::zk::validation::*;

// ---------------------------------------------------------
// Contract-ready output structs
// ---------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BuyerZKProof {
    pub proof: GrothProof,
    pub nullifier: Nullifier,
    pub amount: U256,
    pub asset_id: Hex,
    pub expiry: u64,
    pub pk_hash: Hex,
}

#[derive(Debug, Clone)]
pub struct SellerZKProof {
    pub proof: GrothProof,
    pub nullifier: Nullifier,
    pub fulfil_hash: Hex,
    pub expiry: u64,
    pub pk_hash: Hex,
}

#[derive(Debug, Clone)]
pub struct MerchantCommitRewritten {
    pub signature: Option<Hex>,
    pub escrow_lock: Option<Hex>,
    pub policy_hash: Hex,
    pub expiry: u64,
    pub pk_hash: Hex,
}

// ---------------------------------------------------------
// Unified rewritten envelope enum
// ---------------------------------------------------------

#[derive(Debug, Clone)]
pub enum RewrittenEnvelope {
    Buyer(BuyerZKProof),
    Seller(SellerZKProof),
    Merchant(MerchantCommitRewritten),
}

// ---------------------------------------------------------
// Dispatcher entry point
// ---------------------------------------------------------

pub fn rewrite_envelope(env: ZKEnvelope, expected_chain_id: u64) -> Result<RewrittenEnvelope> {
    match env {
        ZKEnvelope::ZKB01(b) => {
            Ok(RewrittenEnvelope::Buyer(rewrite_buyer(b, expected_chain_id)?))
        }
        ZKEnvelope::ZKS01(s) => {
            Ok(RewrittenEnvelope::Seller(rewrite_seller(s, expected_chain_id)?))
        }
        ZKEnvelope::ZKM01(m) => {
            Ok(RewrittenEnvelope::Merchant(rewrite_merchant(m, expected_chain_id)?))
        }
    }
}

// ---------------------------------------------------------
// Buyer Rewrite
// ---------------------------------------------------------

fn rewrite_buyer(b: ZKB01, expected_chain_id: u64) -> Result<BuyerZKProof> {
    // ---- Identity validation ----
    if !validate_session_id(&b.session_id) {
        bail!("invalid session id");
    }
    if !validate_pk_hash(&b.identity.pk_hash) {
        bail!("invalid pkHash");
    }
    if !validate_chain_id(b.identity.chain_id, expected_chain_id) {
        bail!("incorrect chainId");
    }

    // ---- TTL ----
    if !validate_ttl(b.public_inputs.expiry) {
        bail!("TTL expired");
    }

    // ---- Nullifier ----
    if !validate_nullifier(&b.public_inputs.nullifier) {
        bail!("invalid nullifier");
    }

    // ---- Amount conversion ----
    let amount = U256::from_dec_str(&b.public_inputs.amount)
        .map_err(|_| anyhow::anyhow!("amount is not valid U256 decimal"))?;

    Ok(BuyerZKProof {
        proof: b.proof,
        nullifier: b.public_inputs.nullifier,
        amount,
        asset_id: b.public_inputs.asset_id,
        expiry: b.public_inputs.expiry,
        pk_hash: b.identity.pk_hash,
    })
}

// ---------------------------------------------------------
// Seller Rewrite
// ---------------------------------------------------------

fn rewrite_seller(s: ZKS01, expected_chain_id: u64) -> Result<SellerZKProof> {
    if !validate_session_id(&s.session_id) {
        bail!("invalid session id");
    }
    if !validate_pk_hash(&s.identity.pk_hash) {
        bail!("invalid pkHash");
    }
    if !validate_chain_id(s.identity.chain_id, expected_chain_id) {
        bail!("incorrect chainId");
    }

    if !validate_ttl(s.public_inputs.expiry) {
        bail!("TTL expired");
    }

    if !validate_nullifier(&s.public_inputs.nullifier) {
        bail!("invalid nullifier");
    }

    Ok(SellerZKProof {
        proof: s.proof,
        nullifier: s.public_inputs.nullifier,
        fulfil_hash: s.public_inputs.fulfil_hash,
        expiry: s.public_inputs.expiry,
        pk_hash: s.identity.pk_hash,
    })
}

// ---------------------------------------------------------
// Merchant Rewrite
// ---------------------------------------------------------

fn rewrite_merchant(m: ZKM01, expected_chain_id: u64) -> Result<MerchantCommitRewritten> {
    if !validate_session_id(&m.session_id) {
        bail!("invalid session id");
    }
    if !validate_pk_hash(&m.identity.pk_hash) {
        bail!("invalid pkHash");
    }
    if !validate_chain_id(m.identity.chain_id, expected_chain_id) {
        bail!("incorrect chainId");
    }

    if !validate_ttl(m.public_inputs.expiry) {
        bail!("TTL expired");
    }

    Ok(MerchantCommitRewritten {
        signature: m.commit.signature,
        escrow_lock: m.commit.escrow_lock,
        policy_hash: m.public_inputs.policy_hash,
        expiry: m.public_inputs.expiry,
        pk_hash: m.identity.pk_hash,
    })
}