//! Event listener implementation

use anyhow::Result;
use ethers::prelude::*;

/// Event listener for contract events
pub struct EventListener {
    // Event listener state
}

impl EventListener {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Listen for BothCommitted events
    pub async fn on_both_committed<F>(&self, _callback: F) -> Result<()>
    where
        F: Fn(BothCommittedEvent) + Send + ‘static,
    {
        // Event listening placeholder
        Ok(())
    }
}

impl Default for EventListener {
    fn default() -> Self {
        Self::new()
    }
}

/// BothCommitted event
#[derive(Debug, Clone)]
pub struct BothCommittedEvent {
    pub order_id: [u8; 32],
    pub buyer: Address,
    pub seller: Address,
}