//! Message aggregation from streaming events

use super::types::HistoricalMessage;
use crate::events::{ClaudeEvent, InputSource};
use std::collections::HashMap;

/// Aggregates streaming events into complete messages
pub struct MessageBuilder {
    session_id: String,
    /// Accumulated text for current assistant turn
    current_text: String,
    /// Active tool calls being built
    active_tools: HashMap<String, ToolBuilder>,
    /// Completed messages ready to persist
    pending_messages: Vec<HistoricalMessage>,
    /// Current timestamp (updated on each event)
    current_time: i64,
}

struct ToolBuilder {
    name: String,
    input: String,
    started_at: i64,
}

impl MessageBuilder {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            current_text: String::new(),
            active_tools: HashMap::new(),
            pending_messages: Vec::new(),
            current_time: now(),
        }
    }

    /// Process a Claude event
    pub fn process_event(&mut self, event: &ClaudeEvent) {
        self.current_time = now();

        match event {
            ClaudeEvent::TextDelta { text } => {
                self.current_text.push_str(text);
            }

            ClaudeEvent::ToolUseStart { id, name } => {
                self.active_tools.insert(
                    id.clone(),
                    ToolBuilder {
                        name: name.clone(),
                        input: String::new(),
                        started_at: self.current_time,
                    },
                );
            }

            ClaudeEvent::ToolInputDelta { id, delta } => {
                if let Some(tool) = self.active_tools.get_mut(id) {
                    tool.input.push_str(delta);
                }
            }

            ClaudeEvent::ToolResult {
                id,
                output,
                is_error: _,
            } => {
                // Finalize tool_use message
                if let Some(tool) = self.active_tools.remove(id) {
                    self.pending_messages.push(HistoricalMessage::tool_use(
                        self.session_id.clone(),
                        id.clone(),
                        tool.name.clone(),
                        tool.input,
                        tool.started_at,
                    ));

                    // Add tool_result message
                    self.pending_messages.push(HistoricalMessage::tool_result(
                        self.session_id.clone(),
                        id.clone(),
                        tool.name,
                        output.clone(),
                        self.current_time,
                    ));
                }
            }

            ClaudeEvent::TurnComplete { usage: _ } => {
                // Finalize assistant message if there's accumulated text
                if !self.current_text.is_empty() {
                    self.pending_messages.push(HistoricalMessage::assistant(
                        self.session_id.clone(),
                        std::mem::take(&mut self.current_text),
                        self.current_time,
                    ));
                }
            }

            _ => {}
        }
    }

    /// Add a user input message
    pub fn add_user_input(&mut self, content: String) {
        self.pending_messages.push(HistoricalMessage::user(
            self.session_id.clone(),
            content,
            now(),
        ));
    }

    /// Add a user input message with source attribution
    pub fn add_user_input_with_source(&mut self, content: String, source: InputSource) {
        self.pending_messages.push(HistoricalMessage::user_with_source(
            self.session_id.clone(),
            content,
            source,
            now(),
        ));
    }

    /// Drain all pending messages
    pub fn take_pending(&mut self) -> Vec<HistoricalMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Check if there are pending messages
    pub fn has_pending(&self) -> bool {
        !self.pending_messages.is_empty()
    }
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::super::types::MessageRole;
    use super::*;
    use crate::events::Usage;

    #[test]
    fn test_user_input() {
        let mut builder = MessageBuilder::new("sess-1".into());
        builder.add_user_input("Hello".into());

        let messages = builder.take_pending();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::User);
        assert_eq!(messages[0].content, "Hello");
    }

    #[test]
    fn test_text_aggregation() {
        let mut builder = MessageBuilder::new("sess-1".into());

        builder.process_event(&ClaudeEvent::TextDelta {
            text: "Hello ".into(),
        });
        builder.process_event(&ClaudeEvent::TextDelta {
            text: "world!".into(),
        });
        builder.process_event(&ClaudeEvent::TurnComplete {
            usage: Usage {
                input_tokens: 10,
                output_tokens: 5,
            },
        });

        let messages = builder.take_pending();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::Assistant);
        assert_eq!(messages[0].content, "Hello world!");
    }

    #[test]
    fn test_tool_use_flow() {
        let mut builder = MessageBuilder::new("sess-1".into());

        builder.process_event(&ClaudeEvent::ToolUseStart {
            id: "tool-1".into(),
            name: "Read".into(),
        });
        builder.process_event(&ClaudeEvent::ToolInputDelta {
            id: "tool-1".into(),
            delta: "{\"path\":".into(),
        });
        builder.process_event(&ClaudeEvent::ToolInputDelta {
            id: "tool-1".into(),
            delta: "\"/tmp\"}".into(),
        });
        builder.process_event(&ClaudeEvent::ToolResult {
            id: "tool-1".into(),
            output: "file contents".into(),
            is_error: false,
        });

        let messages = builder.take_pending();
        assert_eq!(messages.len(), 2);

        assert_eq!(messages[0].role, MessageRole::ToolUse);
        assert_eq!(messages[0].tool_name, Some("Read".into()));
        assert_eq!(messages[0].content, "{\"path\":\"/tmp\"}");

        assert_eq!(messages[1].role, MessageRole::ToolResult);
        assert_eq!(messages[1].content, "file contents");
    }

    #[test]
    fn test_mixed_text_and_tools() {
        let mut builder = MessageBuilder::new("sess-1".into());

        // Text before tool
        builder.process_event(&ClaudeEvent::TextDelta {
            text: "Let me check ".into(),
        });

        // Tool use
        builder.process_event(&ClaudeEvent::ToolUseStart {
            id: "t1".into(),
            name: "Read".into(),
        });
        builder.process_event(&ClaudeEvent::ToolResult {
            id: "t1".into(),
            output: "data".into(),
            is_error: false,
        });

        // More text
        builder.process_event(&ClaudeEvent::TextDelta {
            text: "that file.".into(),
        });
        builder.process_event(&ClaudeEvent::TurnComplete {
            usage: Usage {
                input_tokens: 10,
                output_tokens: 5,
            },
        });

        let messages = builder.take_pending();
        // tool_use, tool_result, assistant text
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[2].content, "Let me check that file.");
    }

    #[test]
    fn test_empty_turn_no_message() {
        let mut builder = MessageBuilder::new("sess-1".into());

        builder.process_event(&ClaudeEvent::TurnComplete {
            usage: Usage {
                input_tokens: 0,
                output_tokens: 0,
            },
        });

        let messages = builder.take_pending();
        assert!(messages.is_empty());
    }
}
