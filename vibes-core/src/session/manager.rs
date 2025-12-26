//! SessionManager for managing multiple sessions
//!
//! SessionManager handles creation, retrieval, and lifecycle of sessions.
//! It uses a BackendFactory for dependency injection of backend implementations.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::backend::traits::BackendFactory;
use crate::error::SessionError;
use crate::events::EventBus;

use super::state::{Session, SessionState};

/// Manages multiple vibes sessions
///
/// SessionManager provides:
/// - Session creation with unique IDs
/// - Session retrieval by ID
/// - Session listing
/// - Backend factory injection for testability
pub struct SessionManager {
    /// Active sessions indexed by ID
    sessions: RwLock<HashMap<String, Session>>,
    /// Factory for creating backends
    backend_factory: Arc<dyn BackendFactory>,
    /// Event bus shared by all sessions
    event_bus: Arc<dyn EventBus>,
}

impl SessionManager {
    /// Create a new SessionManager
    pub fn new(backend_factory: Arc<dyn BackendFactory>, event_bus: Arc<dyn EventBus>) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            backend_factory,
            event_bus,
        }
    }

    /// Create a new session with an optional name
    ///
    /// Returns the session ID.
    pub async fn create_session(&self, name: Option<String>) -> String {
        let id = Uuid::new_v4().to_string();
        let backend = self.backend_factory.create(None);
        let session = Session::new(id.clone(), name, backend, self.event_bus.clone());

        self.sessions.write().await.insert(id.clone(), session);
        id
    }

    /// Create a session with a specific ID (for resumption)
    ///
    /// If a claude_session_id is provided, the backend will use it
    /// for session continuity.
    pub async fn create_session_with_id(
        &self,
        id: String,
        name: Option<String>,
        claude_session_id: Option<String>,
    ) -> Result<String, SessionError> {
        // Check if ID already exists
        if self.sessions.read().await.contains_key(&id) {
            return Err(SessionError::InvalidState {
                expected: "unique ID".to_string(),
                actual: format!("ID '{}' already exists", id),
            });
        }

        let backend = self.backend_factory.create(claude_session_id);
        let session = Session::new(id.clone(), name, backend, self.event_bus.clone());

        self.sessions.write().await.insert(id.clone(), session);
        Ok(id)
    }

    /// Get a session by ID
    ///
    /// Returns a mutable reference for sending messages.
    /// Uses the callback pattern to avoid lifetime issues with RwLock.
    pub async fn with_session<F, R>(&self, id: &str, f: F) -> Result<R, SessionError>
    where
        F: FnOnce(&mut Session) -> R,
    {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| SessionError::NotFound(id.to_string()))?;
        Ok(f(session))
    }

    /// Get session state by ID
    pub async fn get_session_state(&self, id: &str) -> Result<SessionState, SessionError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(id)
            .ok_or_else(|| SessionError::NotFound(id.to_string()))?;
        Ok(session.state())
    }

    /// Get session name by ID
    pub async fn get_session_name(&self, id: &str) -> Result<Option<String>, SessionError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(id)
            .ok_or_else(|| SessionError::NotFound(id.to_string()))?;
        Ok(session.name().map(|s| s.to_string()))
    }

    /// List all session IDs
    pub async fn list_sessions(&self) -> Vec<String> {
        self.sessions.read().await.keys().cloned().collect()
    }

    /// List sessions with their states
    pub async fn list_sessions_with_state(&self) -> Vec<(String, SessionState)> {
        self.sessions
            .read()
            .await
            .iter()
            .map(|(id, session)| (id.clone(), session.state()))
            .collect()
    }

    /// Remove a session
    pub async fn remove_session(&self, id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        if sessions.remove(id).is_none() {
            return Err(SessionError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Get the number of active sessions
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;
    use crate::backend::traits::{BackendFactory, ClaudeBackend};
    use crate::events::MemoryEventBus;

    /// Mock factory that creates MockBackends
    struct MockBackendFactory;

    impl BackendFactory for MockBackendFactory {
        fn create(&self, claude_session_id: Option<String>) -> Box<dyn ClaudeBackend> {
            match claude_session_id {
                Some(id) => Box::new(MockBackend::with_session_id(id)),
                None => Box::new(MockBackend::new()),
            }
        }
    }

    fn create_test_manager() -> SessionManager {
        let factory: Arc<dyn BackendFactory> = Arc::new(MockBackendFactory);
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        SessionManager::new(factory, event_bus)
    }

    // ==================== Creation Tests ====================

    #[tokio::test]
    async fn create_session_returns_unique_id() {
        let manager = create_test_manager();

        let id1 = manager.create_session(None).await;
        let id2 = manager.create_session(None).await;

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn create_session_with_name() {
        let manager = create_test_manager();

        let id = manager.create_session(Some("My Session".to_string())).await;
        let name = manager.get_session_name(&id).await.unwrap();

        assert_eq!(name, Some("My Session".to_string()));
    }

    #[tokio::test]
    async fn create_session_with_specific_id() {
        let manager = create_test_manager();

        let id = manager
            .create_session_with_id("custom-id".to_string(), None, None)
            .await
            .unwrap();

        assert_eq!(id, "custom-id");
    }

    #[tokio::test]
    async fn create_session_with_duplicate_id_fails() {
        let manager = create_test_manager();

        manager
            .create_session_with_id("my-id".to_string(), None, None)
            .await
            .unwrap();

        let result = manager
            .create_session_with_id("my-id".to_string(), None, None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_session_uses_backend_factory() {
        let manager = create_test_manager();

        // Create session with specific claude_session_id
        let id = manager
            .create_session_with_id("vibes-1".to_string(), None, Some("claude-abc".to_string()))
            .await
            .unwrap();

        // Access session and verify claude_session_id was passed through
        let result = manager
            .with_session(&id, |session| session.claude_session_id().to_string())
            .await
            .unwrap();

        assert_eq!(result, "claude-abc");
    }

    // ==================== Get Session Tests ====================

    #[tokio::test]
    async fn get_session_retrieves_by_id() {
        let manager = create_test_manager();

        let id = manager.create_session(Some("Test".to_string())).await;

        let state = manager.get_session_state(&id).await.unwrap();
        assert!(matches!(state, SessionState::Idle));
    }

    #[tokio::test]
    async fn get_session_not_found_returns_error() {
        let manager = create_test_manager();

        let result = manager.get_session_state("nonexistent").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn with_session_allows_mutation() {
        let manager = create_test_manager();
        let id = manager.create_session(None).await;

        // Use with_session to access and verify state
        let state = manager
            .with_session(&id, |session| session.state())
            .await
            .unwrap();

        assert!(matches!(state, SessionState::Idle));
    }

    // ==================== List Sessions Tests ====================

    #[tokio::test]
    async fn list_sessions_returns_all_ids() {
        let manager = create_test_manager();

        let id1 = manager.create_session(None).await;
        let id2 = manager.create_session(None).await;
        let id3 = manager.create_session(None).await;

        let sessions = manager.list_sessions().await;

        assert_eq!(sessions.len(), 3);
        assert!(sessions.contains(&id1));
        assert!(sessions.contains(&id2));
        assert!(sessions.contains(&id3));
    }

    #[tokio::test]
    async fn list_sessions_empty_when_no_sessions() {
        let manager = create_test_manager();

        let sessions = manager.list_sessions().await;

        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn list_sessions_with_state_includes_states() {
        let manager = create_test_manager();

        let id = manager.create_session(None).await;

        let sessions = manager.list_sessions_with_state().await;

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].0, id);
        assert!(matches!(sessions[0].1, SessionState::Idle));
    }

    // ==================== Remove Session Tests ====================

    #[tokio::test]
    async fn remove_session_removes_by_id() {
        let manager = create_test_manager();

        let id = manager.create_session(None).await;
        assert_eq!(manager.session_count().await, 1);

        manager.remove_session(&id).await.unwrap();
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn remove_session_not_found_returns_error() {
        let manager = create_test_manager();

        let result = manager.remove_session("nonexistent").await;

        assert!(result.is_err());
    }

    // ==================== Count Tests ====================

    #[tokio::test]
    async fn session_count_tracks_active_sessions() {
        let manager = create_test_manager();

        assert_eq!(manager.session_count().await, 0);

        let id1 = manager.create_session(None).await;
        assert_eq!(manager.session_count().await, 1);

        let _id2 = manager.create_session(None).await;
        assert_eq!(manager.session_count().await, 2);

        manager.remove_session(&id1).await.unwrap();
        assert_eq!(manager.session_count().await, 1);
    }

    // ==================== Concurrency Tests ====================

    #[tokio::test]
    async fn concurrent_session_creation_is_safe() {
        let manager = Arc::new(create_test_manager());
        let mut handles = vec![];

        for _ in 0..10 {
            let manager = Arc::clone(&manager);
            handles.push(tokio::spawn(
                async move { manager.create_session(None).await },
            ));
        }

        let mut ids = vec![];
        for handle in handles {
            ids.push(handle.await.unwrap());
        }

        // All IDs should be unique
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 10);

        // All sessions should exist
        assert_eq!(manager.session_count().await, 10);
    }
}
