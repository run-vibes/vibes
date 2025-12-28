//! Slow mock backend for concurrency testing
//!
//! SlowMockBackend wraps MockBackend and adds a configurable delay
//! before processing each send. This is useful for testing concurrent
//! access to the SessionManager.

use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::broadcast;

use super::mock::MockBackend;
use super::traits::{BackendState, ClaudeBackend};
use crate::error::BackendError;
use crate::events::ClaudeEvent;

/// MockBackend wrapper that adds configurable delay
pub struct SlowMockBackend {
    inner: MockBackend,
    delay: Duration,
}

impl SlowMockBackend {
    /// Create with specified delay
    pub fn new(delay: Duration) -> Self {
        Self {
            inner: MockBackend::new(),
            delay,
        }
    }

    /// Queue a response (delegates to inner)
    pub fn queue_response(&mut self, events: Vec<ClaudeEvent>) {
        self.inner.queue_response(events);
    }
}

#[async_trait]
impl ClaudeBackend for SlowMockBackend {
    async fn send(&mut self, input: &str) -> Result<(), BackendError> {
        // Delay before processing
        tokio::time::sleep(self.delay).await;
        self.inner.send(input).await
    }

    fn subscribe(&self) -> broadcast::Receiver<ClaudeEvent> {
        self.inner.subscribe()
    }

    async fn respond_permission(
        &mut self,
        request_id: &str,
        approved: bool,
    ) -> Result<(), BackendError> {
        self.inner.respond_permission(request_id, approved).await
    }

    fn claude_session_id(&self) -> &str {
        self.inner.claude_session_id()
    }

    fn state(&self) -> BackendState {
        self.inner.state()
    }

    async fn shutdown(&mut self) -> Result<(), BackendError> {
        self.inner.shutdown().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Usage;
    use std::time::Instant;

    #[tokio::test]
    async fn send_delays_by_configured_duration() {
        let mut backend = SlowMockBackend::new(Duration::from_millis(50));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);

        let start = Instant::now();
        backend.send("test").await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn delegates_to_inner_mock_backend() {
        let mut backend = SlowMockBackend::new(Duration::from_millis(1));
        let mut rx = backend.subscribe();

        backend.queue_response(vec![ClaudeEvent::TextDelta {
            text: "Hello".to_string(),
        }]);

        backend.send("test").await.unwrap();

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ClaudeEvent::TextDelta { text } if text == "Hello"));
    }
}
