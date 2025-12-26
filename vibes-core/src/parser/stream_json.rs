//! Stream-JSON parsing for Claude's output format
//!
//! Claude Code's stream-json format emits one JSON object per line,
//! each with a "type" field identifying the message kind.

use serde::Deserialize;

use crate::events::{ClaudeEvent, Usage};

/// Content block types in Claude's stream-json output
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String },
    Thinking { thinking: String },
}

/// Delta types for streaming content updates
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    InputJsonDelta { partial_json: String },
}

/// Content item within an assistant message
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssistantContentItem {
    Text {
        text: String,
    },
    #[serde(other)]
    Other,
}

/// Assistant message payload
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessagePayload {
    #[serde(default)]
    pub content: Vec<AssistantContentItem>,
}

/// Messages from Claude's stream-json output format
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// System message (startup, info)
    System { message: String },

    /// Assistant message with complete content (print mode)
    Assistant { message: AssistantMessagePayload },

    /// Assistant message start (streaming mode)
    AssistantMessage { id: String },

    /// Content block starting
    ContentBlockStart {
        index: u32,
        content_block: ContentBlock,
    },

    /// Streaming content delta
    ContentBlockDelta { index: u32, delta: Delta },

    /// Content block finished
    ContentBlockStop { index: u32 },

    /// Tool use request
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    /// Tool execution result
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },

    /// Final result with usage stats
    Result {
        is_error: bool,
        duration_ms: u64,
        usage: Usage,
    },

    /// Unknown message type (future-proofing)
    #[serde(other)]
    Unknown,
}

/// Parse a line of stream-json, returning None for unparseable lines
///
/// This is resilient: empty lines, invalid JSON, and missing fields
/// all return None rather than errors.
pub fn parse_line(line: &str) -> Option<StreamMessage> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    serde_json::from_str(trimmed).ok()
}

/// Convert StreamMessage to ClaudeEvent
///
/// Returns None for structural messages (block start/stop, system) that
/// don't represent actual content or state changes.
pub fn to_claude_event(msg: StreamMessage) -> Option<ClaudeEvent> {
    match msg {
        // Print mode: assistant message contains complete response
        StreamMessage::Assistant { message } => {
            // Collect all text content from the message
            let text: String = message
                .content
                .into_iter()
                .filter_map(|item| match item {
                    AssistantContentItem::Text { text } => Some(text),
                    AssistantContentItem::Other => None,
                })
                .collect::<Vec<_>>()
                .join("");

            if text.is_empty() {
                None
            } else {
                Some(ClaudeEvent::TextDelta { text })
            }
        }
        // Streaming mode: content block deltas
        StreamMessage::ContentBlockDelta { delta, .. } => match delta {
            Delta::TextDelta { text } => Some(ClaudeEvent::TextDelta { text }),
            Delta::ThinkingDelta { thinking } => {
                Some(ClaudeEvent::ThinkingDelta { text: thinking })
            }
            Delta::InputJsonDelta { partial_json } => Some(ClaudeEvent::ToolInputDelta {
                id: String::new(), // ID comes from ToolUse, tracked separately
                delta: partial_json,
            }),
        },
        StreamMessage::ToolUse { id, name, .. } => Some(ClaudeEvent::ToolUseStart { id, name }),
        StreamMessage::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => Some(ClaudeEvent::ToolResult {
            id: tool_use_id,
            output: content,
            is_error,
        }),
        StreamMessage::Result { usage, .. } => Some(ClaudeEvent::TurnComplete { usage }),
        // Structural/informational messages don't become events
        StreamMessage::System { .. }
        | StreamMessage::AssistantMessage { .. }
        | StreamMessage::ContentBlockStart { .. }
        | StreamMessage::ContentBlockStop { .. }
        | StreamMessage::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    // ==================== StreamMessage Parsing Tests ====================

    #[test]
    fn parse_system_message() {
        let json = r#"{"type":"system","message":"Claude Code started"}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(
            matches!(msg, super::StreamMessage::System { message } if message == "Claude Code started")
        );
    }

    #[test]
    fn parse_assistant_message() {
        let json = r#"{"type":"assistant_message","id":"msg_123"}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(
            matches!(msg, super::StreamMessage::AssistantMessage { id, .. } if id == "msg_123")
        );
    }

    #[test]
    fn parse_assistant_with_content() {
        let json =
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello world"}]}}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        match msg {
            super::StreamMessage::Assistant { message } => {
                assert_eq!(message.content.len(), 1);
                match &message.content[0] {
                    super::AssistantContentItem::Text { text } => {
                        assert_eq!(text, "Hello world");
                    }
                    _ => panic!("Expected Text content"),
                }
            }
            _ => panic!("Expected Assistant message"),
        }
    }

    #[test]
    fn parse_content_block_start() {
        let json =
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            super::StreamMessage::ContentBlockStart { index: 0, .. }
        ));
    }

    #[test]
    fn parse_content_block_delta_text() {
        let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            super::StreamMessage::ContentBlockDelta { index: 0, .. }
        ));
    }

    #[test]
    fn parse_content_block_delta_thinking() {
        let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"Analyzing..."}}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            super::StreamMessage::ContentBlockDelta { index: 0, .. }
        ));
    }

    #[test]
    fn parse_content_block_delta_input_json() {
        let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"cmd\":"}}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            super::StreamMessage::ContentBlockDelta { index: 0, .. }
        ));
    }

    #[test]
    fn parse_content_block_stop() {
        let json = r#"{"type":"content_block_stop","index":0}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            super::StreamMessage::ContentBlockStop { index: 0 }
        ));
    }

    #[test]
    fn parse_tool_use() {
        let json = r#"{"type":"tool_use","id":"tool_123","name":"Bash","input":{"command":"ls"}}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, super::StreamMessage::ToolUse { id, name, .. }
            if id == "tool_123" && name == "Bash"));
    }

    #[test]
    fn parse_tool_result() {
        let json = r#"{"type":"tool_result","tool_use_id":"tool_123","content":"file.txt","is_error":false}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(
            matches!(msg, super::StreamMessage::ToolResult { tool_use_id, is_error, .. }
            if tool_use_id == "tool_123" && !is_error)
        );
    }

    #[test]
    fn parse_result() {
        let json = r#"{"type":"result","is_error":false,"duration_ms":1500,"usage":{"input_tokens":100,"output_tokens":50}}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            super::StreamMessage::Result {
                is_error: false,
                duration_ms: 1500,
                ..
            }
        ));
    }

    #[test]
    fn parse_unknown_message_type() {
        let json = r#"{"type":"some_future_message_type","data":"whatever"}"#;
        let msg: super::StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, super::StreamMessage::Unknown));
    }

    // ==================== parse_line Tests ====================

    #[test]
    fn parse_line_empty_returns_none() {
        assert!(super::parse_line("").is_none());
        assert!(super::parse_line("   ").is_none());
        assert!(super::parse_line("\t\n").is_none());
    }

    #[test]
    fn parse_line_invalid_json_returns_none() {
        assert!(super::parse_line("not json at all").is_none());
        assert!(super::parse_line("{incomplete").is_none());
        assert!(super::parse_line("{}").is_none()); // Missing required fields
    }

    #[test]
    fn parse_line_valid_json_returns_some() {
        let json = r#"{"type":"system","message":"Hello"}"#;
        let result = super::parse_line(json);
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap(),
            super::StreamMessage::System { .. }
        ));
    }

    #[test]
    fn parse_line_unknown_type_returns_some_unknown() {
        let json = r#"{"type":"new_experimental_type","foo":"bar"}"#;
        let result = super::parse_line(json);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), super::StreamMessage::Unknown));
    }

    // ==================== to_claude_event Tests ====================

    #[test]
    fn to_claude_event_assistant_extracts_text() {
        let msg = super::StreamMessage::Assistant {
            message: super::AssistantMessagePayload {
                content: vec![super::AssistantContentItem::Text {
                    text: "Hello from assistant".to_string(),
                }],
            },
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::TextDelta { text }) if text == "Hello from assistant")
        );
    }

    #[test]
    fn to_claude_event_assistant_empty_returns_none() {
        let msg = super::StreamMessage::Assistant {
            message: super::AssistantMessagePayload { content: vec![] },
        };
        let event = super::to_claude_event(msg);
        assert!(event.is_none());
    }

    #[test]
    fn to_claude_event_text_delta() {
        let msg = super::StreamMessage::ContentBlockDelta {
            index: 0,
            delta: super::Delta::TextDelta {
                text: "Hello world".to_string(),
            },
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::TextDelta { text }) if text == "Hello world")
        );
    }

    #[test]
    fn to_claude_event_thinking_delta() {
        let msg = super::StreamMessage::ContentBlockDelta {
            index: 0,
            delta: super::Delta::ThinkingDelta {
                thinking: "Let me think...".to_string(),
            },
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::ThinkingDelta { text }) if text == "Let me think...")
        );
    }

    #[test]
    fn to_claude_event_input_json_delta() {
        let msg = super::StreamMessage::ContentBlockDelta {
            index: 0,
            delta: super::Delta::InputJsonDelta {
                partial_json: r#"{"cmd":"#.to_string(),
            },
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::ToolInputDelta { delta, .. }) if delta == r#"{"cmd":"#)
        );
    }

    #[test]
    fn to_claude_event_tool_use() {
        let msg = super::StreamMessage::ToolUse {
            id: "tool_123".to_string(),
            name: "Bash".to_string(),
            input: serde_json::json!({"command": "ls"}),
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::ToolUseStart { id, name })
            if id == "tool_123" && name == "Bash")
        );
    }

    #[test]
    fn to_claude_event_tool_result() {
        let msg = super::StreamMessage::ToolResult {
            tool_use_id: "tool_123".to_string(),
            content: "file.txt".to_string(),
            is_error: false,
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::ToolResult { id, output, is_error })
            if id == "tool_123" && output == "file.txt" && !is_error)
        );
    }

    #[test]
    fn to_claude_event_tool_result_error() {
        let msg = super::StreamMessage::ToolResult {
            tool_use_id: "tool_456".to_string(),
            content: "Permission denied".to_string(),
            is_error: true,
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::ToolResult { id, output, is_error })
            if id == "tool_456" && output == "Permission denied" && is_error)
        );
    }

    #[test]
    fn to_claude_event_result() {
        let msg = super::StreamMessage::Result {
            is_error: false,
            duration_ms: 1500,
            usage: crate::events::Usage {
                input_tokens: 100,
                output_tokens: 50,
            },
        };
        let event = super::to_claude_event(msg);
        assert!(
            matches!(event, Some(crate::events::ClaudeEvent::TurnComplete { usage })
            if usage.input_tokens == 100 && usage.output_tokens == 50)
        );
    }

    #[test]
    fn to_claude_event_unknown_returns_none() {
        let msg = super::StreamMessage::Unknown;
        let event = super::to_claude_event(msg);
        assert!(event.is_none());
    }

    #[test]
    fn to_claude_event_system_returns_none() {
        let msg = super::StreamMessage::System {
            message: "Starting...".to_string(),
        };
        let event = super::to_claude_event(msg);
        assert!(event.is_none()); // System messages are informational, not events
    }

    #[test]
    fn to_claude_event_content_block_start_returns_none() {
        let msg = super::StreamMessage::ContentBlockStart {
            index: 0,
            content_block: super::ContentBlock::Text {
                text: String::new(),
            },
        };
        let event = super::to_claude_event(msg);
        assert!(event.is_none()); // Block start is structural, deltas have content
    }

    #[test]
    fn to_claude_event_content_block_stop_returns_none() {
        let msg = super::StreamMessage::ContentBlockStop { index: 0 };
        let event = super::to_claude_event(msg);
        assert!(event.is_none()); // Block stop is structural
    }
}
