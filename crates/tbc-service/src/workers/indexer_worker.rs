//! Indexer worker for blockchain events

use anyhow::Result;
use tokio::time::{interval, Duration};
use tracing::info;

/// Worker that indexes blockchain events to database
pub struct IndexerWorker {
    interval_secs: u64,
}

impl IndexerWorker {
    pub fn new(interval_secs: u64) -> Self {
        Self { interval_secs }
    }
    
    /// Start the worker
    pub async fn run(&self) -> Result<()> {
        let mut ticker = interval(Duration::from_secs(self.interval_secs));
        
        loop {
            ticker.tick().await;
            self.index_events().await?;
        }
    }
    
    async fn index_events(&self) -> Result<()> {
        info!("Indexing blockchain events");
        // Indexing logic placeholder
        Ok(())
    }
}