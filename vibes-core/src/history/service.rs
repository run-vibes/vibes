//! History business logic

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::builder::MessageBuilder;
use super::error::HistoryError;
use super::query::{MessageListResult, MessageQuery, SessionListResult, SessionQuery};
use super::store::HistoryStore;
use super::types::HistoricalSession;
use crate::events::{ClaudeEvent, VibesEvent};
use crate::session::SessionState;

/// Service for managing chat history
pub struct HistoryService<S: HistoryStore> {
    store: Arc<S>,
    /// Active message builders per session
    builders: RwLock<HashMap<String, MessageBuilder>>,
}

impl<S: HistoryStore> HistoryService<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self {
            store,
            builders: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new session in history
    pub async fn create_session(
        &self,
        id: String,
        name: Option<String>,
    ) -> Result<(), HistoryError> {
        let session = HistoricalSession::new(id.clone(), name);
        self.store.save_session(&session)?;

        // Initialize message builder
        let mut builders = self.builders.write().await;
        builders.insert(id.clone(), MessageBuilder::new(id));

        Ok(())
    }

    /// Process an event for history persistence
    pub async fn process_event(&self, event: &VibesEvent) -> Result<(), HistoryError> {
        match event {
            VibesEvent::SessionCreated { session_id, name } => {
                self.create_session(session_id.clone(), name.clone())
                    .await?;
            }

            VibesEvent::UserInput {
                session_id,
                content,
                ..  // source handled in Task 2.4
            } => {
                let mut builders = self.builders.write().await;
                if let Some(builder) = builders.get_mut(session_id) {
                    builder.add_user_input(content.clone());
                    self.persist_pending(session_id, builder)?;
                }
            }

            VibesEvent::Claude { session_id, event } => {
                let mut builders = self.builders.write().await;
                if let Some(builder) = builders.get_mut(session_id) {
                    builder.process_event(event);

                    // Persist on turn complete
                    if matches!(event, ClaudeEvent::TurnComplete { .. }) {
                        self.persist_pending(session_id, builder)?;

                        // Update token stats
                        if let ClaudeEvent::TurnComplete { usage } = event {
                            self.store.update_session_stats(
                                session_id,
                                usage.input_tokens,
                                usage.output_tokens,
                            )?;
                        }
                    }
                }
            }

            VibesEvent::SessionStateChanged { session_id, state } => {
                if let Some(mut session) = self.store.get_session(session_id)? {
                    // Parse state string to SessionState
                    session.state = parse_state(state);
                    self.store.update_session(&session)?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn persist_pending(
        &self,
        _session_id: &str,
        builder: &mut MessageBuilder,
    ) -> Result<(), HistoryError> {
        for message in builder.take_pending() {
            self.store.save_message(&message)?;
        }
        Ok(())
    }

    /// List sessions with filtering
    pub fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult, HistoryError> {
        self.store.list_sessions(query)
    }

    /// Get a specific session
    pub fn get_session(&self, id: &str) -> Result<Option<HistoricalSession>, HistoryError> {
        self.store.get_session(id)
    }

    /// Get messages for a session
    pub fn get_messages(
        &self,
        session_id: &str,
        query: &MessageQuery,
    ) -> Result<MessageListResult, HistoryError> {
        self.store.get_messages(session_id, query)
    }

    /// Delete a session
    pub fn delete_session(&self, id: &str) -> Result<(), HistoryError> {
        self.store.delete_session(id)
    }

    /// Get Claude session ID for resume
    pub fn get_claude_session_id(&self, id: &str) -> Result<Option<String>, HistoryError> {
        Ok(self
            .store
            .get_session(id)?
            .and_then(|s| s.claude_session_id))
    }

    /// Update Claude session ID
    pub fn set_claude_session_id(&self, id: &str, claude_id: String) -> Result<(), HistoryError> {
        if let Some(mut session) = self.store.get_session(id)? {
            session.claude_session_id = Some(claude_id);
            self.store.update_session(&session)?;
        }
        Ok(())
    }

    /// Clean up builder for ended session
    pub async fn end_session(&self, session_id: &str) {
        let mut builders = self.builders.write().await;
        builders.remove(session_id);
    }
}

/// Parse state string to SessionState enum for history persistence.
///
/// Note: WaitingPermission and Failed use placeholder values for their inner fields.
/// The event bus only sends the state variant name (e.g., "failed"), not the full state
/// with inner details. This is acceptable because:
/// 1. Error details are stored separately in the session's error_message column
/// 2. For history viewing, the state category matters more than live session details
/// 3. Active sessions with full state info are available via the session manager
fn parse_state(state: &str) -> SessionState {
    match state {
        "idle" | "Idle" => SessionState::Idle,
        "processing" | "Processing" => SessionState::Processing,
        "waiting_permission" | "WaitingPermission" => SessionState::WaitingPermission {
            // Placeholder - event only contains variant name, not inner fields
            request_id: String::new(),
            tool: String::new(),
        },
        "finished" | "Finished" => SessionState::Finished,
        "failed" | "Failed" => SessionState::Failed {
            // Placeholder - error details stored in session.error_message
            message: String::new(),
            recoverable: false,
        },
        _ => SessionState::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Usage;

    fn create_test_service() -> HistoryService<super::super::store::SqliteHistoryStore> {
        let store = super::super::store::SqliteHistoryStore::open_in_memory().unwrap();
        HistoryService::new(Arc::new(store))
    }

    #[tokio::test]
    async fn test_create_session() {
        let service = create_test_service();

        service
            .create_session("sess-1".into(), Some("Test".into()))
            .await
            .unwrap();

        let session = service.get_session("sess-1").unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().name, Some("Test".into()));
    }

    #[tokio::test]
    async fn test_process_user_input() {
        let service = create_test_service();
        service.create_session("sess-1".into(), None).await.unwrap();

        service
            .process_event(&VibesEvent::UserInput {
                session_id: "sess-1".into(),
                content: "Hello".into(),
                source: crate::events::InputSource::Unknown,
            })
            .await
            .unwrap();

        let messages = service
            .get_messages("sess-1", &MessageQuery::new())
            .unwrap();
        assert_eq!(messages.total, 1);
        assert_eq!(messages.messages[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_process_claude_turn() {
        let service = create_test_service();
        service.create_session("sess-1".into(), None).await.unwrap();

        service
            .process_event(&VibesEvent::Claude {
                session_id: "sess-1".into(),
                event: ClaudeEvent::TextDelta {
                    text: "Hello!".into(),
                },
            })
            .await
            .unwrap();

        service
            .process_event(&VibesEvent::Claude {
                session_id: "sess-1".into(),
                event: ClaudeEvent::TurnComplete {
                    usage: Usage {
                        input_tokens: 10,
                        output_tokens: 5,
                    },
                },
            })
            .await
            .unwrap();

        let messages = service
            .get_messages("sess-1", &MessageQuery::new())
            .unwrap();
        assert_eq!(messages.total, 1);
        assert_eq!(messages.messages[0].content, "Hello!");

        let session = service.get_session("sess-1").unwrap().unwrap();
        assert_eq!(session.total_input_tokens, 10);
        assert_eq!(session.total_output_tokens, 5);
    }

    #[tokio::test]
    async fn test_session_state_update() {
        let service = create_test_service();
        service.create_session("sess-1".into(), None).await.unwrap();

        service
            .process_event(&VibesEvent::SessionStateChanged {
                session_id: "sess-1".into(),
                state: "finished".into(),
            })
            .await
            .unwrap();

        let session = service.get_session("sess-1").unwrap().unwrap();
        assert_eq!(session.state, SessionState::Finished);
    }
}
