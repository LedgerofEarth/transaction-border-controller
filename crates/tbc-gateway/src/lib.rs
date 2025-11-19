//! TBC Gateway – In-Memory Gateway Layer
//!
//! This crate exposes an in-memory API for handling TGP-QUERY
//! flows and settlement notifications.

pub mod gateway;
pub mod handlers;
pub mod validation;
pub mod signing;

pub use gateway::TbcGateway;