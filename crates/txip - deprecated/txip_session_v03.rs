// crates/tbc-gateway/src/txip/txip_session_v03.rs
// FINAL - TxIP Session Management with CoreProver v0.3 Compatibility
//
// Session management using triple-clock timestamps from engine.
// NO Instant, Duration, or SystemTime usage allowed.
//
// All timing decisions must be made by comparing monotonic/unix timestamps
// provided by the engine's TimestampProvider.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::blockchain_types_v03::ChainId;
use super::timestamp_types_v03::{TimestampProvider, TripleTimestamp};
use super::txip_types_v03::*;

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub agent_id: String,
    pub role: Role,
    
    /// When session was created (triple timestamp)
    pub created_mono: u64,
    pub created_unix: u64,
    pub created_iso: String,
    
    /// Last activity (triple timestamp)
    pub last_activity_mono: u64,
    pub last_activity_unix: u64,
    pub last_activity_iso: String,
    
    pub negotiated_tgp_version: String,
    pub negotiated_chains: Vec<ChainId>,
    pub features: Features,
}

impl SessionInfo {
    /// Check if session has timed out
    pub fn is_timed_out(&self, current_mono: u64, timeout_seconds: u64) -> bool {
        current_mono > self.last_activity_mono + timeout_seconds
    }

    /// Get last activity timestamp
    pub fn last_activity_timestamp(&self) -> TripleTimestamp {
        TripleTimestamp::new(
            self.last_activity_mono,
            self.last_activity_unix,
            self.last_activity_iso.clone(),
        )
    }

    /// Get created timestamp
    pub fn created_timestamp(&self) -> TripleTimestamp {
        TripleTimestamp::new(
            self.created_mono,
            self.created_unix,
            self.created_iso.clone(),
        )
    }
}

/// Session manager
pub struct SessionManager<T: TimestampProvider> {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    
    /// Message ID tracking per session (for idempotency)
    message_cache: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    
    /// Configuration
    config: SessionConfig,
    
    /// Timestamp provider (engine)
    timestamp_provider: Arc<T>,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    
    /// How long to keep message IDs in cache (seconds)
    pub message_cache_ttl_seconds: u64,
    
    /// Heartbeat interval (seconds)
    pub heartbeat_interval_seconds: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_timeout_seconds: 300,     // 5 minutes
            message_cache_ttl_seconds: 600,   // 10 minutes
            heartbeat_interval_seconds: 30,   // 30 seconds
        }
    }
}

impl<T: TimestampProvider> SessionManager<T> {
    /// Create a new session manager with timestamp provider
    pub fn new(config: SessionConfig, timestamp_provider: Arc<T>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            timestamp_provider,
        }
    }

    /// Create or update a session from a HELLO message
    pub fn handle_hello(
        &self,
        hello: &HelloPayload,
        session_id: String,
        role: Role,
    ) -> Result<SessionInfo, String> {
        let now = self.timestamp_provider.now();
        
        // Negotiate TGP version
        let negotiated_tgp_version = Self::negotiate_tgp_version(&hello.supported_tgp_versions)?;
        
        // Negotiate chains
        let negotiated_chains = Self::negotiate_chains(&hello.supported_chains)?;
        
        let session_info = SessionInfo {
            session_id: session_id.clone(),
            agent_id: hello.agent_id.clone(),
            role,
            created_mono: now.mono,
            created_unix: now.unix,
            created_iso: now.iso.clone(),
            last_activity_mono: now.mono,
            last_activity_unix: now.unix,
            last_activity_iso: now.iso,
            negotiated_tgp_version,
            negotiated_chains,
            features: hello.features.clone(),
        };

        // Store session
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.clone(), session_info.clone());

        // Initialize message cache for this session
        let mut cache = self.message_cache.write().unwrap();
        cache.insert(session_id, HashSet::new());

        Ok(session_info)
    }

    /// Check if a message ID has been seen before (idempotency check)
    pub fn is_duplicate_message(&self, session_id: &str, msg_id: &str) -> bool {
        let cache = self.message_cache.read().unwrap();
        
        if let Some(msg_ids) = cache.get(session_id) {
            msg_ids.contains(msg_id)
        } else {
            false
        }
    }

    /// Record a message ID (for idempotency tracking)
    pub fn record_message(&self, session_id: &str, msg_id: &str) -> Result<(), String> {
        let mut cache = self.message_cache.write().unwrap();
        
        let msg_ids = cache.entry(session_id.to_string())
            .or_insert_with(HashSet::new);
        
        msg_ids.insert(msg_id.to_string());
        
        Ok(())
    }

    /// Update session activity timestamp
    pub fn touch_session(&self, session_id: &str) -> Result<(), String> {
        let now = self.timestamp_provider.now();
        let mut sessions = self.sessions.write().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity_mono = now.mono;
            session.last_activity_unix = now.unix;
            session.last_activity_iso = now.iso;
            Ok(())
        } else {
            Err(format!("Session not found: {}", session_id))
        }
    }

    /// Get session info
    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }

    /// Close a session
    pub fn close_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().unwrap();
        let mut cache = self.message_cache.write().unwrap();
        
        sessions.remove(session_id);
        cache.remove(session_id);
        
        Ok(())
    }

    /// Clean up expired sessions based on current time from provider
    pub fn cleanup_expired(&self) {
        let now = self.timestamp_provider.now();
        
        // Remove expired sessions
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, session| {
            !session.is_timed_out(now.mono, self.config.session_timeout_seconds)
        });

        // Remove message caches for inactive sessions
        let active_sessions: HashSet<String> = sessions.keys().cloned().collect();
        
        let mut cache = self.message_cache.write().unwrap();
        cache.retain(|session_id, _| active_sessions.contains(session_id));
    }

    /// Get heartbeat interval for negotiation
    pub fn heartbeat_interval_sec(&self) -> u64 {
        self.config.heartbeat_interval_seconds
    }

    /// Get current timestamp from provider
    pub fn now(&self) -> TripleTimestamp {
        self.timestamp_provider.now()
    }

    /// Negotiate TGP version
    fn negotiate_tgp_version(supported: &[String]) -> Result<String, String> {
        // For now, only support TGP 2.0
        if supported.contains(&"2.0".to_string()) {
            Ok("2.0".to_string())
        } else {
            Err("No compatible TGP version found".to_string())
        }
    }

    /// Negotiate chains
    fn negotiate_chains(supported: &[ChainId]) -> Result<Vec<ChainId>, String> {
        // For now, accept any chains the client supports
        // In production, intersect with TBC's supported chains
        if supported.is_empty() {
            Err("Client must support at least one chain".to_string())
        } else {
            Ok(supported.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test timestamp provider for unit tests
    struct TestTimestampProvider {
        current_mono: std::sync::atomic::AtomicU64,
        current_unix: std::sync::atomic::AtomicU64,
    }

    impl TestTimestampProvider {
        fn new(mono: u64, unix: u64) -> Self {
            Self {
                current_mono: std::sync::atomic::AtomicU64::new(mono),
                current_unix: std::sync::atomic::AtomicU64::new(unix),
            }
        }

        fn advance(&self, seconds: u64) {
            self.current_mono.fetch_add(seconds, std::sync::atomic::Ordering::SeqCst);
            self.current_unix.fetch_add(seconds, std::sync::atomic::Ordering::SeqCst);
        }
    }

    impl TimestampProvider for TestTimestampProvider {
        fn now(&self) -> TripleTimestamp {
            let mono = self.current_mono.load(std::sync::atomic::Ordering::SeqCst);
            let unix = self.current_unix.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(
                mono,
                unix,
                format!("2024-11-14T12:{}:00Z", mono % 60),
            )
        }

        fn at_unix(&self, unix: u64) -> TripleTimestamp {
            let mono = self.current_mono.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(
                mono,
                unix,
                format!("2024-11-14T12:00:00Z"),
            )
        }

        fn at_mono(&self, mono: u64) -> TripleTimestamp {
            let unix = self.current_unix.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(
                mono,
                unix,
                format!("2024-11-14T12:00:00Z"),
            )
        }
    }

    fn create_test_hello() -> HelloPayload {
        HelloPayload {
            agent_id: "buyer://alice".to_string(),
            supported_tgp_versions: vec!["2.0".to_string()],
            supported_transports: vec!["HTTP".to_string()],
            supported_chains: vec![1, 369],
            supported_assets: vec!["USDC".to_string()],
            features: Features {
                zk_discount_proofs: true,
                receipt_ownership_proofs: true,
                late_discount_support: true,
                cross_chain_support: true,
            },
            auth: AuthInfo {
                scheme: AuthScheme::None,
                token: None,
            },
        }
    }

    #[test]
    fn test_session_creation() {
        let provider = Arc::new(TestTimestampProvider::new(1000, 1731600000));
        let manager = SessionManager::new(SessionConfig::default(), provider);
        let hello = create_test_hello();
        
        let result = manager.handle_hello(&hello, "sess-123".to_string(), Role::BuyerAgent);
        assert!(result.is_ok());
        
        let session = manager.get_session("sess-123");
        assert!(session.is_some());
        
        let session = session.unwrap();
        assert_eq!(session.agent_id, "buyer://alice");
        assert_eq!(session.created_mono, 1000);
    }

    #[test]
    fn test_idempotency() {
        let provider = Arc::new(TestTimestampProvider::new(1000, 1731600000));
        let manager = SessionManager::new(SessionConfig::default(), provider);
        let hello = create_test_hello();
        
        manager.handle_hello(&hello, "sess-123".to_string(), Role::BuyerAgent).unwrap();
        
        // First message should not be duplicate
        assert!(!manager.is_duplicate_message("sess-123", "msg-1"));
        
        // Record the message
        manager.record_message("sess-123", "msg-1").unwrap();
        
        // Now it should be a duplicate
        assert!(manager.is_duplicate_message("sess-123", "msg-1"));
    }

    #[test]
    fn test_session_timeout() {
        let provider = Arc::new(TestTimestampProvider::new(1000, 1731600000));
        let mut config = SessionConfig::default();
        config.session_timeout_seconds = 60; // 1 minute timeout
        
        let manager = SessionManager::new(config, provider.clone());
        let hello = create_test_hello();
        
        manager.handle_hello(&hello, "sess-123".to_string(), Role::BuyerAgent).unwrap();
        
        let session = manager.get_session("sess-123").unwrap();
        assert!(!session.is_timed_out(1000, 60));
        assert!(!session.is_timed_out(1050, 60));
        assert!(session.is_timed_out(1061, 60));
    }

    #[test]
    fn test_cleanup_expired() {
        let provider = Arc::new(TestTimestampProvider::new(1000, 1731600000));
        let mut config = SessionConfig::default();
        config.session_timeout_seconds = 60;
        
        let manager = SessionManager::new(config, provider.clone());
        let hello = create_test_hello();
        
        manager.handle_hello(&hello, "sess-123".to_string(), Role::BuyerAgent).unwrap();
        assert!(manager.get_session("sess-123").is_some());
        
        // Advance time past timeout
        provider.advance(61);
        
        // Cleanup should remove expired session
        manager.cleanup_expired();
        assert!(manager.get_session("sess-123").is_none());
    }
}
