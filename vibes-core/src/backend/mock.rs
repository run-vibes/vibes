//! Mock backend for testing and PTY mode
//!
//! MockBackend allows scripting Claude responses for unit tests,
//! enabling fast, deterministic testing of Session logic.
//!
//! In PTY mode, MockBackendFactory is used since actual I/O happens
//! through PtyManager, not through the backend.

use std::collections::VecDeque;

use async_trait::async_trait;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::traits::{BackendFactory, BackendState, ClaudeBackend};
use crate::error::BackendError;
use crate::events::ClaudeEvent;

/// Mock implementation of ClaudeBackend for testing
///
/// Queue responses with `queue_response()` before calling `send()`.
/// Each `send()` consumes one queued response and emits its events.
pub struct MockBackend {
    /// Claude session ID
    claude_session_id: String,
    /// Current state
    state: BackendState,
    /// Event broadcast channel
    tx: broadcast::Sender<ClaudeEvent>,
    /// Queued responses (each send() consumes one)
    responses: VecDeque<Vec<ClaudeEvent>>,
}

impl MockBackend {
    /// Create a new MockBackend with a generated session ID
    pub fn new() -> Self {
        Self::with_session_id(Uuid::new_v4().to_string())
    }

    /// Create a new MockBackend with a specific session ID
    pub fn with_session_id(claude_session_id: String) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            claude_session_id,
            state: BackendState::Idle,
            tx,
            responses: VecDeque::new(),
        }
    }

    /// Queue a response to be emitted on the next send()
    pub fn queue_response(&mut self, events: Vec<ClaudeEvent>) {
        self.responses.push_back(events);
    }

    /// Queue an error response (convenience method)
    pub fn queue_error(&mut self, message: &str, recoverable: bool) {
        self.queue_response(vec![ClaudeEvent::Error {
            message: message.to_string(),
            recoverable,
        }]);
    }

    /// Check if there are queued responses
    pub fn has_queued_responses(&self) -> bool {
        !self.responses.is_empty()
    }

    /// Get the number of queued responses
    pub fn queued_response_count(&self) -> usize {
        self.responses.len()
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClaudeBackend for MockBackend {
    async fn send(&mut self, _input: &str) -> Result<(), BackendError> {
        // Get next queued response
        let events = self.responses.pop_front().ok_or_else(|| {
            BackendError::ClaudeError("No queued response in MockBackend".to_string())
        })?;

        // Transition to Processing
        self.state = BackendState::Processing;

        // Emit all events
        for event in events {
            // Check for state-changing events
            match &event {
                ClaudeEvent::Error {
                    message,
                    recoverable,
                } => {
                    self.state = BackendState::Failed {
                        message: message.clone(),
                        recoverable: *recoverable,
                    };
                }
                ClaudeEvent::TurnComplete { .. } => {
                    self.state = BackendState::Idle;
                }
                _ => {}
            }

            // Broadcast to subscribers
            let _ = self.tx.send(event);
        }

        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<ClaudeEvent> {
        self.tx.subscribe()
    }

    async fn respond_permission(
        &mut self,
        _request_id: &str,
        _approved: bool,
    ) -> Result<(), BackendError> {
        // MockBackend doesn't use permissions - just acknowledge
        Ok(())
    }

    fn claude_session_id(&self) -> &str {
        &self.claude_session_id
    }

    fn state(&self) -> BackendState {
        self.state.clone()
    }

    async fn shutdown(&mut self) -> Result<(), BackendError> {
        self.state = BackendState::Finished;
        Ok(())
    }
}

/// Factory for creating MockBackend instances
///
/// Used in PTY mode where actual I/O happens through PtyManager.
/// The backend is kept for API compatibility but does not perform
/// any real Claude interaction.
#[derive(Clone, Default)]
pub struct MockBackendFactory;

impl MockBackendFactory {
    /// Create a new MockBackendFactory
    pub fn new() -> Self {
        Self
    }
}

impl BackendFactory for MockBackendFactory {
    fn create(&self, claude_session_id: Option<String>) -> Box<dyn ClaudeBackend> {
        match claude_session_id {
            Some(id) => Box::new(MockBackend::with_session_id(id)),
            None => Box::new(MockBackend::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::traits::ClaudeBackend;
    use crate::events::{ClaudeEvent, Usage};

    // ==================== Creation Tests ====================

    #[tokio::test]
    async fn new_creates_with_generated_session_id() {
        let backend = super::MockBackend::new();
        assert!(!backend.claude_session_id().is_empty());
    }

    #[tokio::test]
    async fn new_with_session_id_uses_provided_id() {
        let backend = super::MockBackend::with_session_id("my-session-123".to_string());
        assert_eq!(backend.claude_session_id(), "my-session-123");
    }

    #[tokio::test]
    async fn new_starts_in_idle_state() {
        let backend = super::MockBackend::new();
        assert_eq!(backend.state(), super::super::traits::BackendState::Idle);
    }

    // ==================== Queue Response Tests ====================

    #[tokio::test]
    async fn queue_response_stores_events() {
        let mut backend = super::MockBackend::new();
        backend.queue_response(vec![ClaudeEvent::TextDelta {
            text: "Hello".to_string(),
        }]);

        assert!(backend.has_queued_responses());
    }

    #[tokio::test]
    async fn queue_multiple_responses() {
        let mut backend = super::MockBackend::new();

        backend.queue_response(vec![ClaudeEvent::TextDelta {
            text: "First".to_string(),
        }]);
        backend.queue_response(vec![ClaudeEvent::TextDelta {
            text: "Second".to_string(),
        }]);

        assert_eq!(backend.queued_response_count(), 2);
    }

    #[tokio::test]
    async fn queue_error_convenience_method() {
        let mut backend = super::MockBackend::new();
        backend.queue_error("Something failed", true);

        assert!(backend.has_queued_responses());
    }

    // ==================== Send Tests ====================

    #[tokio::test]
    async fn send_emits_queued_events() {
        let mut backend = super::MockBackend::new();
        let mut rx = backend.subscribe();

        backend.queue_response(vec![
            ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
            ClaudeEvent::TurnComplete {
                usage: Usage::default(),
            },
        ]);

        backend.send("Hi there").await.unwrap();

        let event1 = rx.recv().await.unwrap();
        assert!(matches!(event1, ClaudeEvent::TextDelta { text } if text == "Hello"));

        let event2 = rx.recv().await.unwrap();
        assert!(matches!(event2, ClaudeEvent::TurnComplete { .. }));
    }

    #[tokio::test]
    async fn send_transitions_to_processing_then_idle() {
        let mut backend = super::MockBackend::new();
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);

        // Before send: Idle
        assert_eq!(backend.state(), super::super::traits::BackendState::Idle);

        backend.send("test").await.unwrap();

        // After send with TurnComplete: back to Idle
        assert_eq!(backend.state(), super::super::traits::BackendState::Idle);
    }

    #[tokio::test]
    async fn send_with_error_transitions_to_failed() {
        let mut backend = super::MockBackend::new();
        backend.queue_error("Something went wrong", false);

        backend.send("test").await.unwrap();

        assert!(matches!(
            backend.state(),
            super::super::traits::BackendState::Failed {
                recoverable: false,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn send_consumes_queued_response() {
        let mut backend = super::MockBackend::new();
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);
        backend.queue_response(vec![ClaudeEvent::TextDelta {
            text: "Second".to_string(),
        }]);

        assert_eq!(backend.queued_response_count(), 2);

        backend.send("first").await.unwrap();
        assert_eq!(backend.queued_response_count(), 1);

        backend.send("second").await.unwrap();
        assert_eq!(backend.queued_response_count(), 0);
    }

    #[tokio::test]
    async fn send_without_queued_response_returns_error() {
        let mut backend = super::MockBackend::new();
        let result = backend.send("test").await;

        assert!(result.is_err());
    }

    // ==================== Subscribe Tests ====================

    #[tokio::test]
    async fn subscribe_receives_events_from_send() {
        let mut backend = super::MockBackend::new();
        let mut rx = backend.subscribe();

        backend.queue_response(vec![ClaudeEvent::TextDelta {
            text: "Test".to_string(),
        }]);

        backend.send("input").await.unwrap();

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ClaudeEvent::TextDelta { text } if text == "Test"));
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_events() {
        let mut backend = super::MockBackend::new();
        let mut rx1 = backend.subscribe();
        let mut rx2 = backend.subscribe();

        backend.queue_response(vec![ClaudeEvent::TextDelta {
            text: "Shared".to_string(),
        }]);

        backend.send("input").await.unwrap();

        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();

        assert!(matches!(event1, ClaudeEvent::TextDelta { text } if text == "Shared"));
        assert!(matches!(event2, ClaudeEvent::TextDelta { text } if text == "Shared"));
    }

    // ==================== Permission Tests ====================

    #[tokio::test]
    async fn respond_permission_is_noop_for_mock() {
        let mut backend = super::MockBackend::new();
        let result = backend.respond_permission("req-1", true).await;
        assert!(result.is_ok());
    }

    // ==================== Shutdown Tests ====================

    #[tokio::test]
    async fn shutdown_succeeds() {
        let mut backend = super::MockBackend::new();
        let result = backend.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn shutdown_transitions_to_finished() {
        let mut backend = super::MockBackend::new();
        backend.shutdown().await.unwrap();

        assert_eq!(
            backend.state(),
            super::super::traits::BackendState::Finished
        );
    }
}
