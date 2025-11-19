//! TBC Enforcement Engine – Module Root
//!
//! Provides:
//! - spend-limit evaluation
//! - frequency evaluation
//! - anomaly scoring
//! - idempotency protection
//! - reservation lifecycle
//!
//! These functions are called by TGP handlers to determine whether a
//! PAYMENT_REQUIRED or REJECT should be returned.

pub mod rules;
pub mod anomaly;
pub mod idempotency;
pub mod reservations;

pub use rules::*;
pub use anomaly::*;
pub use idempotency::*;
pub use reservations::*;