//! CoreProver Bridge - Rust ↔ Solidity Integration

pub mod client;
pub mod types;
pub mod events;

pub use client::escrow_client::EscrowClient;
pub use types::*;

/// Bridge version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}