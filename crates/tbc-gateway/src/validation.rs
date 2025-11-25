use crate::zk::types::*;
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static::lazy_static! {
    static ref HEX_32: Regex = Regex::new(r"^0x[0-9a-fA-F]{64}$").unwrap();
}

pub fn validate_nullifier(n: &Nullifier) -> bool {
    HEX_32.is_match(&n.value) && n.epoch >= 0
}

pub fn validate_ttl(expiry: u64) -> bool {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    expiry > now
}

pub fn validate_pk_hash(pk_hash: &str) -> bool {
    HEX_32.is_match(pk_hash)
}

pub fn validate_session_id(id: &str) -> bool {
    HEX_32.is_match(id)
}

pub fn validate_chain_id(chain_id: u64, expected: u64) -> bool {
    chain_id == expected
}