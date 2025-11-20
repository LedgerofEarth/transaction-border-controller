//! TBC Core - Gateway Protocol Implementation
//!
//! This crate provides core types and traits for the Transaction Border Controller.

// Gateway module is in tbc-gateway crate, not here

pub mod protocol;
pub mod tgp;
pub mod codec_tx;

// pub use gateway::Gateway;
// pub use types::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
