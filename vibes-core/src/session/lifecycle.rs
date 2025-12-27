//! Session lifecycle management
//!
//! Handles ownership transfer and cleanup when clients disconnect.

use std::sync::Arc;

use super::manager::SessionManager;
use super::ownership::ClientId;

/// Manages session lifecycle events
pub struct SessionLifecycleManager {
    session_manager: Arc<SessionManager>,
}

enum LifecycleAction {
    Transfer(ClientId),
    Cleanup,
}

impl SessionLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Handle a client disconnecting
    ///
    /// Returns ownership transfers and cleanups that occurred.
    pub async fn handle_client_disconnect(&self, client_id: &ClientId) -> DisconnectResult {
        let mut result = DisconnectResult::default();

        // Get all sessions this client is subscribed to
        let session_ids = self
            .session_manager
            .get_sessions_subscribed_by(client_id)
            .await;

        for session_id in session_ids {
            let action = self
                .session_manager
                .with_session(&session_id, |session| {
                    let was_owner = session.ownership_mut().remove_subscriber(client_id);

                    if was_owner {
                        // Try to transfer ownership
                        if let Some(new_owner) = session.ownership().pick_next_owner().cloned() {
                            session.ownership_mut().transfer_to(&new_owner);
                            return Some(LifecycleAction::Transfer(new_owner));
                        }
                    }

                    // Check if cleanup needed
                    if session.ownership().should_cleanup() {
                        return Some(LifecycleAction::Cleanup);
                    }

                    None
                })
                .await;

            match action {
                Ok(Some(LifecycleAction::Transfer(new_owner))) => {
                    result.transfers.push((session_id, new_owner));
                }
                Ok(Some(LifecycleAction::Cleanup)) => {
                    self.session_manager.remove_session(&session_id).await.ok();
                    result.cleanups.push(session_id);
                }
                _ => {}
            }
        }

        result
    }
}

/// Result of handling a client disconnect
#[derive(Debug, Default)]
pub struct DisconnectResult {
    /// Sessions where ownership was transferred: (session_id, new_owner_id)
    pub transfers: Vec<(String, ClientId)>,
    /// Sessions that were cleaned up
    pub cleanups: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::traits::{BackendFactory, ClaudeBackend};
    use crate::backend::MockBackend;
    use crate::events::MemoryEventBus;

    struct MockBackendFactory;

    impl BackendFactory for MockBackendFactory {
        fn create(&self, claude_session_id: Option<String>) -> Box<dyn ClaudeBackend> {
            match claude_session_id {
                Some(id) => Box::new(MockBackend::with_session_id(id)),
                None => Box::new(MockBackend::new()),
            }
        }
    }

    fn create_test_lifecycle() -> (SessionLifecycleManager, Arc<SessionManager>) {
        let event_bus = Arc::new(MemoryEventBus::new(100));
        let factory: Arc<dyn BackendFactory> = Arc::new(MockBackendFactory);
        let manager = Arc::new(SessionManager::new(factory, event_bus));
        let lifecycle = SessionLifecycleManager::new(manager.clone());
        (lifecycle, manager)
    }

    #[tokio::test]
    async fn disconnect_owner_transfers_to_subscriber() {
        let (lifecycle, manager) = create_test_lifecycle();

        // Create session owned by client-a
        let session_id = manager
            .create_session_with_owner(None, Some("client-a".to_string()))
            .await;

        // Add subscriber
        manager
            .with_session(&session_id, |s| {
                s.ownership_mut().add_subscriber("client-b".to_string());
            })
            .await
            .unwrap();

        // Disconnect owner
        let result = lifecycle
            .handle_client_disconnect(&"client-a".to_string())
            .await;

        assert_eq!(result.transfers.len(), 1);
        assert_eq!(result.transfers[0].0, session_id);
        assert_eq!(result.transfers[0].1, "client-b");
        assert!(result.cleanups.is_empty());
    }
}
