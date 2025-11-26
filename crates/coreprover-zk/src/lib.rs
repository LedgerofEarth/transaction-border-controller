//! CoreProver ZK - Zero-knowledge proof system

pub mod prover;
pub mod verifier;
pub mod validation;

pub use prover::Prover;
pub use verifier::Verifier;
pub use validation::Validation;

/// ZK module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}