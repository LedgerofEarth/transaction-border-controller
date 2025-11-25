use serde::{Deserialize, Serialize};
use ethers::types::U256;

pub type Hex = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Nullifier {
    pub value: Hex,     // 0x-prefixed 32-byte hex
    pub epoch: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdentityBinding {
    pub pk_hash: Hex,
    pub session_id: Hex,
    pub chain_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZKB01 {
    pub r#type: String, // "ZKB01"
    pub session_id: Hex,

    pub proof: GrothProof,
    pub public_inputs: BuyerPublicInputs,
    pub identity: IdentityBinding,
    pub nonce: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZKS01 {
    pub r#type: String, // "ZKS01"
    pub session_id: Hex,

    pub proof: GrothProof,
    pub public_inputs: SellerPublicInputs,
    pub identity: IdentityBinding,
    pub nonce: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZKM01 {
    pub r#type: String, // "ZKM01"
    pub session_id: Hex,

    pub commit: MerchantCommit,
    pub public_inputs: MerchantPublicInputs,
    pub identity: IdentityBinding,
    pub nonce: u64,
}

// Unified enum
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ZKEnvelope {
    ZKB01(ZKB01),
    ZKS01(ZKS01),
    ZKM01(ZKM01),
}

// -------------------
// Proof + Public Inputs
// -------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrothProof {
    pub a: Hex,
    pub b: (Hex, Hex),
    pub c: Hex,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuyerPublicInputs {
    pub nullifier: Nullifier,
    pub amount: String,      // decimal U256 string
    pub asset_id: Hex,
    pub expiry: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellerPublicInputs {
    pub nullifier: Nullifier,
    pub fulfil_hash: Hex,
    pub expiry: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MerchantCommit {
    pub signature: Option<Hex>,
    pub escrow_lock: Option<Hex>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MerchantPublicInputs {
    pub policy_hash: Hex,
    pub expiry: u64,
}