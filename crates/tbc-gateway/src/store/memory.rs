//! In-memory session storage for development/testing
//!
//! This implementation uses RwLock<HashMap> for thread-safe access.
//! NOT recommended for production (no persistence, no clustering).
//!
//! For production, use:
//!   • RedisStore (horizontal scaling)
//!   • PostgresStore (audit trail + durability)

use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::RwLock;
use tbc_core::tgp::state::{TGPSession, SessionStore};

pub struct InMemorySessionStore {
    sessions: RwLock<HashMap<String, TGPSession>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn session_count(&self) -> usize {
        self.sessions.read().unwrap().len()
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create_session(&self, session_id: String) -> Result<TGPSession> {
        let session = TGPSession::new(session_id.clone());
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id, session.clone());
        Ok(session)
    }
    
    async fn get_session(&self, session_id: &str) -> Result<Option<TGPSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.get(session_id).cloned())
    }
    
    async fn update_session(&self, session: &TGPSession) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.session_id.clone(), session.clone());
        Ok(())
    }
    
    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id);
        Ok(())
    }
    
    async fn list_sessions(&self, limit: usize) -> Result<Vec<TGPSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.values()
            .take(limit)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_and_retrieve() {
        let store = InMemorySessionStore::new();
        let session = store.create_session("test-1".to_string()).await.unwrap();
        assert_eq!(session.session_id, "test-1");
        
        let retrieved = store.get_session("test-1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, "test-1");
    }
    
    #[tokio::test]
    async fn test_update_session() {
        let store = InMemorySessionStore::new();
        let mut session = store.create_session("test-2".to_string()).await.unwrap();
        
        session.query_id = Some("q-123".to_string());
        store.update_session(&session).await.unwrap();
        
        let retrieved = store.get_session("test-2").await.unwrap().unwrap();
        assert_eq!(retrieved.query_id, Some("q-123".to_string()));
    }
    
    #[tokio::test]
    async fn test_delete_session() {
        let store = InMemorySessionStore::new();
        store.create_session("test-3".to_string()).await.unwrap();
        
        store.delete_session("test-3").await.unwrap();
        
        let retrieved = store.get_session("test-3").await.unwrap();
        assert!(retrieved.is_none());
    }
    
    #[tokio::test]
    async fn test_list_sessions() {
        let store = InMemorySessionStore::new();
        store.create_session("test-4".to_string()).await.unwrap();
        store.create_session("test-5".to_string()).await.unwrap();
        
        let sessions = store.list_sessions(10).await.unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
