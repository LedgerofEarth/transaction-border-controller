// ============================================================================
// TGP State Machine Implementation (TGP-00 §4)
// crates/tbc-core/src/tgp/state.rs
//
// This module defines the authoritative session state machine for the
// Transaction Gateway Protocol (TGP). It enforces:
//   • Valid state transitions
//   • Timeout behavior
//   • Settlement lifecycle rules
//   • Immutable session-level flags (zk requirements)
//   • Optional domain-level metadata
//
// NOTE: This crate does NOT perform any logging. Instead, it exposes the
// StateObserver trait. tbc-gateway implements the observer to emit logs,
// tracing spans, metrics, or telemetry. tbc-core remains pure and portable.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

// ============================================================================
// Observer Pattern
// ============================================================================

/// State transition observer.
///
/// tbc-gateway implements this trait to emit:
///   • JSON logs
///   • ANSI-colored console output
///   • tracing spans
///   • remote telemetry events
///
/// tbc-core fires callbacks but does not depend on logging crates.
pub trait StateObserver: Send + Sync {
    fn on_state_transition(
        &self,
        session_id: &str,
        old: TGPState,
        new: TGPState,
    );
}

// Default no-op observer for embedded/WASM environments.
pub struct NoopObserver;
impl StateObserver for NoopObserver {
    fn on_state_transition(&self, _sid: &str, _old: TGPState, _new: TGPState) {}
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum TGPStateError {
    #[error("Invalid transition: {0:?} → {1:?}")]
    InvalidTransition(TGPState, TGPState),

    #[error("Session timed out at {0}")]
    SessionTimeout(u64),

    #[error("Terminal state {0:?} cannot transition")]
    TerminalState(TGPState),

    #[error("Session already in state {0:?}")]
    AlreadyInState(TGPState),
}

// ============================================================================
// State enum (TGP-00 §4)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TGPState {
    Idle,
    QuerySent,
    OfferReceived,
    AcceptSent,
    Finalizing,
    Settled,
    Errored,
}

impl TGPState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, TGPState::Settled | TGPState::Errored)
    }

    pub fn can_transition_to(&self, target: TGPState) -> bool {
        use TGPState::*;

        if self.is_terminal() {
            return false;
        }

        match (self, target) {
            (Idle, QuerySent) => true,

            (QuerySent, OfferReceived) => true,
            (QuerySent, Errored) => true,

            (OfferReceived, AcceptSent) => true,
            (OfferReceived, Errored) => true,

            (AcceptSent, Finalizing) => true,
            (AcceptSent, Errored) => true,

            (Finalizing, Settled) => true,
            (Finalizing, Errored) => true,

            // Any non-terminal may go → Errored
            (_, Errored) => true,

            _ => false,
        }
    }

    pub fn timeout_seconds(&self) -> Option<u64> {
        match self {
            TGPState::QuerySent => Some(30),
            TGPState::OfferReceived => Some(300),
            TGPState::AcceptSent => Some(60),
            TGPState::Finalizing => Some(600),
            TGPState::Idle => None,
            TGPState::Settled => None,
            TGPState::Errored => None,
        }
    }
}

// ============================================================================
// Session struct (SSO) (TGP-00 §13)
// ============================================================================

#[derive(Clone, Serialize, Deserialize)]
pub struct TGPSession {
    pub session_id: String,
    pub state: TGPState,

    pub query_id: Option<String>,
    pub offer_id: Option<String>,

    /// Immutable: Derived from OFFER
    pub zk_must_verify: bool,

    /// Optional domain metadata (per TGP-00 §11)
    pub source_domain: Option<String>,
    pub counterparty_domain: Option<String>,

    pub created_at: u64,
    pub updated_at: u64,
    pub timeout_at: Option<u64>,

    #[serde(skip)]
    pub observer: Option<&'static dyn StateObserver>,
}

// Manual Debug implementation (skip observer field)
impl std::fmt::Debug for TGPSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TGPSession")
            .field("session_id", &self.session_id)
            .field("state", &self.state)
            .field("query_id", &self.query_id)
            .field("offer_id", &self.offer_id)
            .field("zk_must_verify", &self.zk_must_verify)
            .field("source_domain", &self.source_domain)
            .field("counterparty_domain", &self.counterparty_domain)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .field("timeout_at", &self.timeout_at)
            .finish()
    }
}

// Manual PartialEq implementation (skip observer field)
impl PartialEq for TGPSession {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.state == other.state
            && self.query_id == other.query_id
            && self.offer_id == other.offer_id
            && self.zk_must_verify == other.zk_must_verify
            && self.source_domain == other.source_domain
            && self.counterparty_domain == other.counterparty_domain
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
            && self.timeout_at == other.timeout_at
    }
}

impl TGPSession {
    pub fn new(session_id: impl Into<String>) -> Self {
        let now = now_ts();
        Self {
            session_id: session_id.into(),
            state: TGPState::Idle,

            query_id: None,
            offer_id: None,

            zk_must_verify: false,

            source_domain: None,
            counterparty_domain: None,

            created_at: now,
            updated_at: now,
            timeout_at: None,

            observer: None,
        }
    }

    /// Create an ephemeral session for error handling.
    ///
    /// Ephemeral sessions are used when processing ERROR messages
    /// that have no correlation_id or reference unknown sessions.
    /// They are not persisted to the SessionStore.
    pub fn ephemeral(session_id: impl Into<String>) -> Self {
        Self::new(session_id)
    }

    // ---------------------------------------------------------------------
    // Observer registration
    // ---------------------------------------------------------------------

    pub fn with_observer(mut self, obs: &'static dyn StateObserver) -> Self {
        self.observer = Some(obs);
        self
    }

    // ---------------------------------------------------------------------
    // Transition
    // ---------------------------------------------------------------------

    pub fn transition(&mut self, new_state: TGPState) -> Result<(), TGPStateError> {
        if self.is_timed_out() {
            return Err(TGPStateError::SessionTimeout(
                self.timeout_at.unwrap_or(0),
            ));
        }

        if self.state == new_state {
            return Err(TGPStateError::AlreadyInState(new_state));
        }

        if self.state.is_terminal() {
            return Err(TGPStateError::TerminalState(self.state));
        }

        if !self.state.can_transition_to(new_state) {
            return Err(TGPStateError::InvalidTransition(self.state, new_state));
        }

        let old = self.state;

        self.state = new_state;
        self.updated_at = now_ts();

        self.timeout_at = new_state
            .timeout_seconds()
            .map(|sec| self.updated_at + sec);

        // Notify observer (gateway logger)
        if let Some(obs) = self.observer {
            obs.on_state_transition(&self.session_id, old, new_state);
        }

        Ok(())
    }

    // ---------------------------------------------------------------------
    // Timeout + helpers
    // ---------------------------------------------------------------------

    pub fn is_timed_out(&self) -> bool {
        match self.timeout_at {
            Some(t) => now_ts() > t,
            None => false,
        }
    }

    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout_at = Some(now_ts() + seconds);
    }

    pub fn clear_timeout(&mut self) {
        self.timeout_at = None;
    }

    pub fn remaining_timeout(&self) -> Option<u64> {
        self.timeout_at
            .and_then(|t| if now_ts() < t { Some(t - now_ts()) } else { None })
    }

    pub fn age(&self) -> u64 {
        now_ts() - self.created_at
    }

    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    pub fn force_error(&mut self) {
        let old = self.state;
        self.state = TGPState::Errored;
        self.updated_at = now_ts();
        self.timeout_at = None;

        if let Some(obs) = self.observer {
            obs.on_state_transition(&self.session_id, old, TGPState::Errored);
        }
    }
}

// ============================================================================
// Session Storage Abstraction
// ============================================================================

use async_trait::async_trait;
use anyhow::Result;

/// Storage abstraction for TGP sessions.
///
/// Implementations:
///   • InMemoryStore (testing/development)
///   • RedisStore (production cache)
///   • PostgresStore (audit trail + persistence)
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session with the given ID.
    async fn create_session(&self, session_id: String) -> Result<TGPSession>;
    
    /// Retrieve a session by ID.
    async fn get_session(&self, session_id: &str) -> Result<Option<TGPSession>>;
    
    /// Update an existing session.
    async fn update_session(&self, session: &TGPSession) -> Result<()>;
    
    /// Delete a session (for cleanup).
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    
    /// List sessions (for admin/debug, paginated).
    async fn list_sessions(&self, limit: usize) -> Result<Vec<TGPSession>>;
}

// ============================================================================
// Helper
// ============================================================================

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock moved backwards")
        .as_secs()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_OBSERVER: TestObserver = TestObserver;
    struct TestObserver;
    impl StateObserver for TestObserver {
        fn on_state_transition(&self, _: &str, _: TGPState, _: TGPState) {}
    }

    #[test]
    fn test_valid_path() {
        let mut s = TGPSession::new("sess").with_observer(&TEST_OBSERVER);
        assert!(s.transition(TGPState::QuerySent).is_ok());
        assert!(s.transition(TGPState::OfferReceived).is_ok());
        assert!(s.transition(TGPState::AcceptSent).is_ok());
        assert!(s.transition(TGPState::Finalizing).is_ok());
        assert!(s.transition(TGPState::Settled).is_ok());
    }

    #[test]
    fn test_timeout_logic() {
        let mut s = TGPSession::new("sess");
        s.transition(TGPState::QuerySent).unwrap();

        s.timeout_at = Some(1);
        assert!(s.is_timed_out());
        assert!(matches!(
            s.transition(TGPState::OfferReceived),
            Err(TGPStateError::SessionTimeout(_))
        ));
    }

    #[test]
    fn test_force_error() {
        let mut s = TGPSession::new("sess").with_observer(&TEST_OBSERVER);
        s.transition(TGPState::QuerySent).unwrap();
        s.force_error();
        assert_eq!(s.state, TGPState::Errored);
    }

    #[test]
    fn test_ephemeral_session() {
        let s = TGPSession::ephemeral("temp-123");
        assert_eq!(s.session_id, "temp-123");
        assert_eq!(s.state, TGPState::Idle);
    }
}
