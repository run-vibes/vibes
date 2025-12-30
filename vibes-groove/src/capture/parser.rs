//! Claude Code transcript JSONL parser

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::GrooveError;

/// A message from a transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMessage {
    /// Role of the message sender (user, assistant, system)
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Optional timestamp
    pub timestamp: Option<DateTime<Utc>>,
}

/// A tool use from a transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptToolUse {
    /// Name of the tool
    pub tool_name: String,
    /// Input provided to the tool
    pub input: Value,
    /// Output from the tool
    pub output: Option<String>,
    /// Whether the tool succeeded
    pub success: bool,
}

/// Metadata about a transcript
#[derive(Debug, Clone, Default)]
pub struct TranscriptMetadata {
    /// Total message count
    pub total_messages: usize,
    /// Count of user messages
    pub user_messages: usize,
    /// Count of assistant messages
    pub assistant_messages: usize,
    /// Count of tool uses
    pub tool_uses: usize,
}

/// Parsed transcript data
#[derive(Debug, Clone)]
pub struct ParsedTranscript {
    /// Session identifier
    pub session_id: String,
    /// Messages in order
    pub messages: Vec<TranscriptMessage>,
    /// Tool uses extracted
    pub tool_uses: Vec<TranscriptToolUse>,
    /// Computed metadata
    pub metadata: TranscriptMetadata,
}

/// Parser for Claude Code JSONL transcripts
pub struct TranscriptParser {
    /// Supported transcript versions
    supported_versions: Vec<String>,
}

impl Default for TranscriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptParser {
    /// Create a new transcript parser
    pub fn new() -> Self {
        Self {
            supported_versions: vec!["1".to_string()],
        }
    }

    /// Get supported versions
    pub fn supported_versions(&self) -> &[String] {
        &self.supported_versions
    }

    /// Parse a transcript from a file path
    pub fn parse_file(&self, path: &Path) -> Result<ParsedTranscript, GrooveError> {
        let content = std::fs::read_to_string(path)?;
        let session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        self.parse(&content, session_id)
    }

    /// Parse transcript content
    pub fn parse(&self, content: &str, session_id: &str) -> Result<ParsedTranscript, GrooveError> {
        let mut messages = Vec::new();
        let mut tool_uses = Vec::new();
        let mut user_count = 0;
        let mut assistant_count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(line) {
                Ok(value) => {
                    // Extract message if it has a role
                    if let Some(role) = value.get("role").and_then(|r| r.as_str()) {
                        // Extract content, handling both string and complex JSON types.
                        // For strings, we use the raw value. For arrays/objects (e.g., tool
                        // results with structured data), Value::to_string() produces JSON
                        // format with quotes and brackets. This preserves structure for
                        // complex content types that appear in Claude transcripts.
                        let content_text = value
                            .get("content")
                            .map(|c| {
                                if let Some(s) = c.as_str() {
                                    s.to_string()
                                } else {
                                    // Non-string content: serialize as JSON to preserve structure
                                    c.to_string()
                                }
                            })
                            .unwrap_or_default();

                        match role {
                            "user" => user_count += 1,
                            "assistant" => assistant_count += 1,
                            _ => {}
                        }

                        messages.push(TranscriptMessage {
                            role: role.to_string(),
                            content: content_text,
                            timestamp: None,
                        });
                    }

                    // Extract tool use if present
                    if let Some(tool_name) = value.get("tool_name").and_then(|t| t.as_str()) {
                        tool_uses.push(TranscriptToolUse {
                            tool_name: tool_name.to_string(),
                            input: value.get("input").cloned().unwrap_or(Value::Null),
                            output: value
                                .get("output")
                                .and_then(|o| o.as_str())
                                .map(String::from),
                            success: value
                                .get("success")
                                .and_then(|s| s.as_bool())
                                .unwrap_or(true),
                        });
                    }
                }
                Err(_) => {
                    // Skip malformed lines silently
                    continue;
                }
            }
        }

        let metadata = TranscriptMetadata {
            total_messages: messages.len(),
            user_messages: user_count,
            assistant_messages: assistant_count,
            tool_uses: tool_uses.len(),
        };

        Ok(ParsedTranscript {
            session_id: session_id.to_string(),
            messages,
            tool_uses,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_jsonl() {
        let content = r#"{"role": "user", "content": "Hello"}
{"role": "assistant", "content": "Hi there!"}
{"role": "user", "content": "Help me code"}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test-session").unwrap();

        assert_eq!(result.session_id, "test-session");
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.metadata.user_messages, 2);
        assert_eq!(result.metadata.assistant_messages, 1);
    }

    #[test]
    fn test_parse_extracts_tool_uses() {
        let content = r#"{"role": "user", "content": "Run ls"}
{"tool_name": "Bash", "input": {"command": "ls"}, "output": "file.txt", "success": true}
{"role": "assistant", "content": "Done"}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.tool_uses.len(), 1);
        assert_eq!(result.tool_uses[0].tool_name, "Bash");
        assert!(result.tool_uses[0].success);
    }

    #[test]
    fn test_parse_handles_empty_content() {
        let parser = TranscriptParser::new();
        let result = parser.parse("", "empty").unwrap();

        assert_eq!(result.messages.len(), 0);
        assert_eq!(result.metadata.total_messages, 0);
    }

    #[test]
    fn test_parse_skips_malformed_lines() {
        let content = r#"{"role": "user", "content": "Hello"}
this is not json
{"role": "assistant", "content": "Hi"}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.messages.len(), 2);
    }

    #[test]
    fn test_parse_handles_blank_lines() {
        let content = r#"{"role": "user", "content": "Hello"}

{"role": "assistant", "content": "Hi"}

"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.messages.len(), 2);
    }

    #[test]
    fn test_tool_use_with_null_output() {
        let content = r#"{"tool_name": "Read", "input": {"path": "file.rs"}, "success": true}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.tool_uses.len(), 1);
        assert!(result.tool_uses[0].output.is_none());
    }

    #[test]
    fn test_metadata_is_computed_correctly() {
        let content = r#"{"role": "user", "content": "1"}
{"role": "assistant", "content": "2"}
{"role": "user", "content": "3"}
{"role": "assistant", "content": "4"}
{"role": "assistant", "content": "5"}
{"role": "system", "content": "sys"}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.metadata.total_messages, 6);
        assert_eq!(result.metadata.user_messages, 2);
        assert_eq!(result.metadata.assistant_messages, 3);
    }

    #[test]
    fn test_supported_versions() {
        let parser = TranscriptParser::new();
        assert!(parser.supported_versions().contains(&"1".to_string()));
    }

    #[test]
    fn test_non_string_content_serialized_as_json() {
        // When content is an array or object, it's serialized as JSON to preserve structure
        let content = r#"{"role": "assistant", "content": ["item1", "item2"]}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.messages.len(), 1);
        // Array content becomes JSON string with brackets and quotes
        assert_eq!(result.messages[0].content, r#"["item1","item2"]"#);
    }
}
