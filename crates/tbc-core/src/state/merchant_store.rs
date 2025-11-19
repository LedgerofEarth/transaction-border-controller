//! Track merchant public keys for policy-hash + 402 verification.

use std::collections::HashMap;

pub struct MerchantStore {
    pub whitelist: HashMap<String, String>, // merchant -> pubkey
}

impl MerchantStore {
    pub fn new() -> Self {
        Self { whitelist: HashMap::new() }
    }

    pub fn verify(&self, merchant: &str, pubkey: &str) -> bool {
        match self.whitelist.get(merchant) {
            Some(pk) => pk == pubkey,
            None => false,
        }
    }
}