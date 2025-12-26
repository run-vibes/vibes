//! Session struct and state machine
//!
//! Session wraps a ClaudeBackend and forwards events to the EventBus.
//! It manages state transitions and provides recovery (retry) on failure.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::backend::traits::ClaudeBackend;
use crate::error::SessionError;
use crate::events::{EventBus, VibesEvent};

/// State of a session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    /// Ready for input
    Idle,
    /// Processing a request
    Processing,
    /// Waiting for permission approval
    WaitingPermission { request_id: String, tool: String },
    /// Failed with error
    Failed { message: String, recoverable: bool },
    /// Completed
    Finished,
}

/// A vibes session wrapping a Claude backend
///
/// Session manages:
/// - A Claude backend for actual communication
/// - An event bus for broadcasting events
/// - State machine for tracking session state
/// - Event forwarding from backend to bus
pub struct Session {
    /// Unique session identifier
    id: String,
    /// Optional human-readable name
    name: Option<String>,
    /// The underlying Claude backend
    backend: Box<dyn ClaudeBackend>,
    /// Event bus for broadcasting
    event_bus: Arc<dyn EventBus>,
    /// Current state
    state: SessionState,
}

impl Session {
    /// Create a new session
    pub fn new(
        id: impl Into<String>,
        name: Option<String>,
        backend: Box<dyn ClaudeBackend>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            id: id.into(),
            name,
            backend,
            event_bus,
            state: SessionState::Idle,
        }
    }

    /// Get the session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the session name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get Claude's session ID (for session continuity)
    pub fn claude_session_id(&self) -> &str {
        self.backend.claude_session_id()
    }

    /// Get current session state
    pub fn state(&self) -> SessionState {
        self.state.clone()
    }

    /// Send user input to Claude
    ///
    /// Returns when the turn is complete or an error occurs.
    /// Events are forwarded to the event bus as they're received.
    pub async fn send(&mut self, input: &str) -> Result<(), SessionError> {
        // Can only send from Idle state
        if !matches!(self.state, SessionState::Idle) {
            return Err(SessionError::InvalidState {
                expected: "Idle".to_string(),
                actual: format!("{:?}", self.state),
            });
        }

        // Subscribe to backend events before sending
        let mut rx = self.backend.subscribe();

        // Transition to Processing
        self.state = SessionState::Processing;
        self.publish_state_change().await;

        // Send to backend
        if let Err(e) = self.backend.send(input).await {
            self.state = SessionState::Failed {
                message: e.to_string(),
                recoverable: true,
            };
            self.publish_state_change().await;
            return Err(SessionError::Backend(e));
        }

        // Forward events from backend to event bus
        self.forward_events(&mut rx).await;

        // Update state based on backend state
        self.sync_state_from_backend();

        Ok(())
    }

    /// Respond to a permission request
    pub async fn respond_permission(
        &mut self,
        request_id: &str,
        approved: bool,
    ) -> Result<(), SessionError> {
        // Must be in WaitingPermission state
        if !matches!(self.state, SessionState::WaitingPermission { .. }) {
            return Err(SessionError::InvalidState {
                expected: "WaitingPermission".to_string(),
                actual: format!("{:?}", self.state),
            });
        }

        // Subscribe before responding
        let mut rx = self.backend.subscribe();

        // Respond to backend
        self.backend
            .respond_permission(request_id, approved)
            .await?;

        // Forward events
        self.forward_events(&mut rx).await;

        // Update state
        self.sync_state_from_backend();

        Ok(())
    }

    /// Retry after a recoverable failure
    ///
    /// Resets state to Idle if the failure was recoverable.
    pub fn retry(&mut self) -> Result<(), SessionError> {
        match &self.state {
            SessionState::Failed { recoverable, .. } if *recoverable => {
                self.state = SessionState::Idle;
                Ok(())
            }
            SessionState::Failed { recoverable, .. } if !recoverable => {
                Err(SessionError::NotRecoverable)
            }
            _ => Err(SessionError::InvalidState {
                expected: "Failed".to_string(),
                actual: format!("{:?}", self.state),
            }),
        }
    }

    /// Shutdown the session
    pub async fn shutdown(&mut self) -> Result<(), SessionError> {
        self.backend.shutdown().await?;
        self.state = SessionState::Finished;
        self.publish_state_change().await;
        Ok(())
    }

    /// Forward events from backend to event bus
    async fn forward_events(
        &mut self,
        rx: &mut tokio::sync::broadcast::Receiver<crate::events::ClaudeEvent>,
    ) {
        use crate::events::ClaudeEvent;

        // Drain all available events
        while let Ok(event) = rx.try_recv() {
            // Wrap as VibesEvent and publish
            let vibes_event = VibesEvent::Claude {
                session_id: self.id.clone(),
                event: event.clone(),
            };
            self.event_bus.publish(vibes_event).await;

            // Update local state based on certain events
            match &event {
                ClaudeEvent::Error {
                    message,
                    recoverable,
                } => {
                    self.state = SessionState::Failed {
                        message: message.clone(),
                        recoverable: *recoverable,
                    };
                }
                ClaudeEvent::TurnComplete { .. } => {
                    self.state = SessionState::Idle;
                }
                ClaudeEvent::PermissionRequest { id, tool, .. } => {
                    self.state = SessionState::WaitingPermission {
                        request_id: id.clone(),
                        tool: tool.clone(),
                    };
                }
                _ => {}
            }
        }
    }

    /// Sync our state from backend state
    fn sync_state_from_backend(&mut self) {
        use crate::backend::traits::BackendState;

        match self.backend.state() {
            BackendState::Idle => {
                self.state = SessionState::Idle;
            }
            BackendState::Processing => {
                self.state = SessionState::Processing;
            }
            BackendState::WaitingPermission { request_id, tool } => {
                self.state = SessionState::WaitingPermission { request_id, tool };
            }
            BackendState::Failed {
                message,
                recoverable,
            } => {
                self.state = SessionState::Failed {
                    message,
                    recoverable,
                };
            }
            BackendState::Finished => {
                self.state = SessionState::Finished;
            }
        }
    }

    /// Publish a state change event
    async fn publish_state_change(&self) {
        let event = VibesEvent::SessionStateChanged {
            session_id: self.id.clone(),
            state: format!("{:?}", self.state),
        };
        self.event_bus.publish(event).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;
    use crate::events::{ClaudeEvent, EventBus, MemoryEventBus, Usage};

    // ==================== SessionState Tests ====================

    #[test]
    fn session_state_idle_debug() {
        let state = SessionState::Idle;
        assert!(format!("{:?}", state).contains("Idle"));
    }

    #[test]
    fn session_state_processing_debug() {
        let state = SessionState::Processing;
        assert!(format!("{:?}", state).contains("Processing"));
    }

    #[test]
    fn session_state_waiting_permission_debug() {
        let state = SessionState::WaitingPermission {
            request_id: "req-1".to_string(),
            tool: "Bash".to_string(),
        };
        let debug = format!("{:?}", state);
        assert!(debug.contains("WaitingPermission"));
        assert!(debug.contains("req-1"));
    }

    #[test]
    fn session_state_failed_debug() {
        let state = SessionState::Failed {
            message: "Error".to_string(),
            recoverable: true,
        };
        let debug = format!("{:?}", state);
        assert!(debug.contains("Failed"));
        assert!(debug.contains("Error"));
    }

    #[test]
    fn session_state_finished_debug() {
        let state = SessionState::Finished;
        assert!(format!("{:?}", state).contains("Finished"));
    }

    #[test]
    fn session_state_clone_works() {
        let state = SessionState::Failed {
            message: "Error".to_string(),
            recoverable: false,
        };
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn session_state_serialization_roundtrip() {
        let states = vec![
            SessionState::Idle,
            SessionState::Processing,
            SessionState::WaitingPermission {
                request_id: "req-1".to_string(),
                tool: "Bash".to_string(),
            },
            SessionState::Failed {
                message: "Error".to_string(),
                recoverable: true,
            },
            SessionState::Finished,
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let parsed: SessionState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, parsed);
        }
    }

    // ==================== Creation Tests ====================

    #[tokio::test]
    async fn new_session_starts_in_idle_state() {
        let backend = MockBackend::new();
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let session = Session::new("test-session", None, Box::new(backend), event_bus);

        assert!(matches!(session.state(), SessionState::Idle));
    }

    #[tokio::test]
    async fn new_session_has_correct_id() {
        let backend = MockBackend::new();
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let session = Session::new("my-session-123", None, Box::new(backend), event_bus);

        assert_eq!(session.id(), "my-session-123");
    }

    #[tokio::test]
    async fn new_session_has_name() {
        let backend = MockBackend::new();
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let session = Session::new(
            "test",
            Some("My Session".to_string()),
            Box::new(backend),
            event_bus,
        );

        assert_eq!(session.name(), Some("My Session"));
    }

    #[tokio::test]
    async fn new_session_exposes_claude_session_id() {
        let backend = MockBackend::with_session_id("claude-123".to_string());
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let session = Session::new("test", None, Box::new(backend), event_bus);

        assert_eq!(session.claude_session_id(), "claude-123");
    }

    // ==================== Send Tests ====================

    #[tokio::test]
    async fn send_transitions_through_states() {
        let mut backend = MockBackend::new();
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);

        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let mut session = Session::new("test", None, Box::new(backend), event_bus);

        // Before send: Idle
        assert!(matches!(session.state(), SessionState::Idle));

        // Send completes, back to Idle
        session.send("Hello").await.unwrap();
        assert!(matches!(session.state(), SessionState::Idle));
    }

    #[tokio::test]
    async fn send_from_non_idle_fails() {
        let backend = MockBackend::new();
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let mut session = Session::new("test", None, Box::new(backend), event_bus);

        // Force to Processing state
        session.state = SessionState::Processing;

        let result = session.send("test").await;
        assert!(result.is_err());
    }

    // ==================== Event Forwarding Tests ====================

    #[tokio::test]
    async fn claude_events_forwarded_to_event_bus() {
        let mut backend = MockBackend::new();
        backend.queue_response(vec![
            ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
            ClaudeEvent::TurnComplete {
                usage: Usage::default(),
            },
        ]);

        let event_bus = Arc::new(MemoryEventBus::new(100));
        let event_bus_dyn: Arc<dyn EventBus> = event_bus.clone();
        let mut session = Session::new("test-session", None, Box::new(backend), event_bus_dyn);

        session.send("Hi").await.unwrap();

        // Check events on the bus
        let events = event_bus.get_session_events("test-session").await;

        // Should have: SessionStateChanged (to Processing), TextDelta, TurnComplete, SessionStateChanged (to Idle)
        assert!(
            events.len() >= 2,
            "Expected at least 2 events, got {}",
            events.len()
        );

        // Find the Claude events (TextDelta and TurnComplete)
        let claude_events: Vec<_> = events
            .iter()
            .filter(|(_, e)| matches!(e, VibesEvent::Claude { .. }))
            .collect();

        assert_eq!(claude_events.len(), 2);
    }

    #[tokio::test]
    async fn state_changes_published_to_bus() {
        let mut backend = MockBackend::new();
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);

        let event_bus = Arc::new(MemoryEventBus::new(100));
        let event_bus_dyn: Arc<dyn EventBus> = event_bus.clone();
        let mut session = Session::new("test-session", None, Box::new(backend), event_bus_dyn);

        session.send("Hi").await.unwrap();

        // Check for state change events
        let events = event_bus.get_session_events("test-session").await;
        let state_changes: Vec<_> = events
            .iter()
            .filter(|(_, e)| matches!(e, VibesEvent::SessionStateChanged { .. }))
            .collect();

        // Should have state changes for Processing and back to Idle
        assert!(!state_changes.is_empty());
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn error_transitions_to_failed() {
        let mut backend = MockBackend::new();
        backend.queue_error("Connection lost", true);

        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let mut session = Session::new("test", None, Box::new(backend), event_bus);

        session.send("test").await.unwrap();

        assert!(matches!(
            session.state(),
            SessionState::Failed {
                recoverable: true,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn retry_resets_recoverable_failure_to_idle() {
        let mut backend = MockBackend::new();
        backend.queue_error("Connection lost", true);
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);

        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let mut session = Session::new("test", None, Box::new(backend), event_bus);

        // First send triggers error
        session.send("test").await.unwrap();
        assert!(matches!(session.state(), SessionState::Failed { .. }));

        // Retry resets state
        session.retry().unwrap();
        assert!(matches!(session.state(), SessionState::Idle));

        // Can send again
        session.send("test2").await.unwrap();
        assert!(matches!(session.state(), SessionState::Idle));
    }

    #[tokio::test]
    async fn retry_non_recoverable_failure_fails() {
        let mut backend = MockBackend::new();
        backend.queue_error("Fatal error", false);

        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let mut session = Session::new("test", None, Box::new(backend), event_bus);

        session.send("test").await.unwrap();

        let result = session.retry();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn retry_from_non_failed_state_fails() {
        let backend = MockBackend::new();
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let mut session = Session::new("test", None, Box::new(backend), event_bus);

        // In Idle state
        let result = session.retry();
        assert!(result.is_err());
    }

    // ==================== Shutdown Tests ====================

    #[tokio::test]
    async fn shutdown_transitions_to_finished() {
        let backend = MockBackend::new();
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
        let mut session = Session::new("test", None, Box::new(backend), event_bus);

        session.shutdown().await.unwrap();

        assert!(matches!(session.state(), SessionState::Finished));
    }
}
