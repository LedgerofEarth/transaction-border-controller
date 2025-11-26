//! CoreProver SDK - High-level API for escrow management

pub mod builder;
pub mod client;

pub use builder::escrow_builder::EscrowBuilder;
pub use client::CoreProverClient;

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}