//! Session cleanup worker for terminal/expired session removal
//!
//! Runs on configurable interval (default: 5 minutes)
//! Removes sessions that are:
//!   • Terminal (Settled or Errored)
//!   • Expired (age > max_session_age)
//!
//! For production, consider:
//!   • Redis TTL (automatic expiration)
//!   • Database partitioning (monthly archives)

use tokio::time::{interval, Duration};
use tbc_core::tgp::state::SessionStore;
use anyhow::Result;
use crate::logging::*;

pub struct CleanupConfig {
    pub interval: Duration,
    pub max_session_age: u64,      // seconds
    pub max_terminal_age: u64,     // seconds (terminal sessions expire faster)
    pub batch_size: usize,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(300),     // 5 minutes
            max_session_age: 86400,                 // 24 hours
            max_terminal_age: 3600,                 // 1 hour
            batch_size: 1000,
        }
    }
}

pub async fn run_cleanup_worker<S: SessionStore>(
    store: S,
    config: CleanupConfig,
) -> Result<()> {
    let mut ticker = interval(config.interval);
    
    info(
        "Session cleanup worker started",
        serde_json::json!({
            "interval_secs": config.interval.as_secs(),
            "max_age": config.max_session_age,
            "max_terminal_age": config.max_terminal_age,
        })
    );
    
    loop {
        ticker.tick().await;
        
        let sessions = store.list_sessions(config.batch_size).await?;
        let mut removed = 0;
        
        for session in sessions {
            let age = session.age();
            let should_remove = if session.is_terminal() {
                age > config.max_terminal_age
            } else {
                age > config.max_session_age
            };
            
            if should_remove {
                store.delete_session(&session.session_id).await?;
                removed += 1;
                
                debug(
                    "Session cleaned up",
                    serde_json::json!({
                        "session_id": session.session_id,
                        "state": format!("{:?}", session.state),
                        "age_seconds": age,
                    })
                );
            }
        }
        
        if removed > 0 {
            info(
                "Cleanup cycle complete",
                serde_json::json!({
                    "sessions_removed": removed,
                    "sessions_scanned": sessions.len(),
                })
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemorySessionStore;
    
    #[tokio::test]
    async fn test_cleanup_terminal_sessions() {
        let store = InMemorySessionStore::new();
        let mut session = store.create_session("old-1".to_string()).await.unwrap();
        
        // Make session terminal and old
        session.force_error();
        session.created_at = 0; // Very old
        store.update_session(&session).await.unwrap();
        
        let config = CleanupConfig {
            interval: Duration::from_secs(1),
            max_terminal_age: 100,
            ..Default::default()
        };
        
        // Manually trigger cleanup logic
        let sessions = store.list_sessions(100).await.unwrap();
        for s in sessions {
            if s.is_terminal() && s.age() > config.max_terminal_age {
                store.delete_session(&s.session_id).await.unwrap();
            }
        }
        
        // Verify removed
        let result = store.get_session("old-1").await.unwrap();
        assert!(result.is_none());
    }
}
