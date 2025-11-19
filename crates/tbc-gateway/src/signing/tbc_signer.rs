use sha2::{Sha256, Digest};

pub struct TbcSigner {
    key: Vec<u8>,
}

impl TbcSigner {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    pub fn sign(&self, msg: &str) -> String {
        let mut h = Sha256::new();
        h.update(&self.key);
        h.update(msg.as_bytes());
        format!("0x{}", hex::encode(h.finalize()))
    }
}