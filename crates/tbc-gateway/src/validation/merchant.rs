use sha2::{Sha256, Digest};
use tbc_core::state::MerchantStore;

pub fn verify_merchant(
    merchants: &MerchantStore,
    merchant_id: &str,
    pubkey: &str,
    signature: &str,
    payload: &str,
) -> bool {
    if !merchants.verify(merchant_id, pubkey) {
        return false;
    }

    let mut h = Sha256::new();
    h.update(pubkey.as_bytes());
    h.update(payload.as_bytes());

    let expected = format!("0x{}", hex::encode(h.finalize()));
    expected == signature
}