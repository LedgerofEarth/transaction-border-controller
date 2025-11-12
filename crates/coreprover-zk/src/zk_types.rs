use ethers::types::{Address};

impl ZkBuyerInput {
    pub fn to_circuit_format(&self) -> Vec<u8> {
        // Dummy logic — replace with real circuit encoding later
        let mut output = vec![];
        output.extend_from_slice(&self.buyer_address);
        output.extend_from_slice(&self.salt);
        output.extend_from_slice(&self.order_id);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ZkBuyerInput {
    pub buyer_address: [u8; 20],
    pub salt: [u8; 32],
    pub order_id: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ZkSellerInput {
    pub seller_address: [u8; 20],
    pub salt: [u8; 32],
    pub order_id: [u8; 32],
}

impl ZkSellerInput {
    pub fn compute_commitment(&self) -> [u8; 32] {
        // Dummy hash implementation (replace with real logic later)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.seller_address);
        hasher.update(&self.salt);
        hasher.update(&self.order_id);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result[..]);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ZkExchangeInput {
    pub buyer_commitment: [u8; 32],
    pub seller_commitment: [u8; 32],
}