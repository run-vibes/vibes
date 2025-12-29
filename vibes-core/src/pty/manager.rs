//! PTY session manager

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use super::backend::{PtyBackend, create_backend};
use super::session::PtyState;
use super::{PtyConfig, PtyError, PtySession, PtySessionHandle};

/// Info about a session (without the handle)
#[derive(Debug, Clone)]
pub struct PtySessionInfo {
    pub id: String,
    pub name: Option<String>,
    pub state: PtyState,
    pub created_at: DateTime<Utc>,
}

/// Manages multiple PTY sessions
pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
    backend: Box<dyn PtyBackend>,
}

impl PtyManager {
    /// Create a new PTY manager with the specified config
    pub fn new(config: PtyConfig) -> Self {
        let backend = create_backend(config);
        Self {
            sessions: HashMap::new(),
            backend,
        }
    }

    /// Create a new PTY manager with a custom backend
    pub fn with_backend(backend: Box<dyn PtyBackend>) -> Self {
        Self {
            sessions: HashMap::new(),
            backend,
        }
    }

    /// Create a new PTY session with auto-generated ID
    pub fn create_session(
        &mut self,
        name: Option<String>,
        cwd: Option<String>,
    ) -> Result<String, PtyError> {
        let id = Uuid::new_v4().to_string();
        self.create_session_with_id(id, name, cwd)
    }

    /// Create a new PTY session with a specific ID
    pub fn create_session_with_id(
        &mut self,
        id: String,
        name: Option<String>,
        cwd: Option<String>,
    ) -> Result<String, PtyError> {
        let session = self.backend.create_session(id.clone(), name, cwd)?;
        self.sessions.insert(id.clone(), session);
        Ok(id)
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<&PtySession> {
        self.sessions.get(id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut PtySession> {
        self.sessions.get_mut(id)
    }

    /// Get session handle for I/O
    pub fn get_handle(&self, id: &str) -> Option<PtySessionHandle> {
        self.sessions.get(id).map(|s| s.handle.clone())
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<PtySessionInfo> {
        self.sessions
            .values()
            .map(|s| PtySessionInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                state: s.state.clone(),
                created_at: s.created_at,
            })
            .collect()
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Remove a session
    pub fn remove_session(&mut self, id: &str) -> Option<PtySession> {
        self.sessions.remove(id)
    }

    /// Kill a session (send SIGTERM and remove)
    pub async fn kill_session(&mut self, id: &str) -> Result<(), PtyError> {
        if let Some(session) = self.sessions.remove(id) {
            let mut inner = session.handle.inner.lock().await;
            let _ = inner.child.kill();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> PtyConfig {
        PtyConfig {
            claude_path: "cat".into(),
            ..Default::default()
        }
    }

    #[test]
    fn create_session_adds_to_manager() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(None, None).unwrap();
        assert!(!id.is_empty());
        assert!(manager.get_session(&id).is_some());
    }

    #[test]
    fn create_session_with_name() {
        let mut manager = PtyManager::new(test_config());

        let id = manager
            .create_session(Some("my-session".to_string()), None)
            .unwrap();
        let session = manager.get_session(&id).unwrap();
        assert_eq!(session.name, Some("my-session".to_string()));
    }

    #[test]
    fn list_sessions_returns_all() {
        let mut manager = PtyManager::new(test_config());

        manager
            .create_session(Some("session1".to_string()), None)
            .unwrap();
        manager
            .create_session(Some("session2".to_string()), None)
            .unwrap();

        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn remove_session_removes_from_manager() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(None, None).unwrap();
        assert!(manager.get_session(&id).is_some());

        manager.remove_session(&id);
        assert!(manager.get_session(&id).is_none());
    }

    #[test]
    fn get_handle_returns_cloneable_handle() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(None, None).unwrap();
        let handle1 = manager.get_handle(&id);
        let handle2 = manager.get_handle(&id);

        assert!(handle1.is_some());
        assert!(handle2.is_some());
    }

    #[tokio::test]
    async fn kill_session_removes_and_kills() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(None, None).unwrap();
        assert!(manager.get_session(&id).is_some());

        manager.kill_session(&id).await.unwrap();
        assert!(manager.get_session(&id).is_none());
    }
}
