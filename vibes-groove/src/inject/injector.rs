//! Claude Code injector for learnings
//!
//! Handles injection of formatted learnings via hook responses
//! or CLAUDE.md file modification.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use vibes_core::hooks::HookResponse;

use crate::GrooveError;

/// Methods for injecting learnings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionMethod {
    /// Inject via hook response (preferred, uses additionalContext)
    HookResponse,
    /// Inject by appending to CLAUDE.md (fallback)
    ClaudeMdAppend,
}

/// Result of an injection operation
#[derive(Debug, Clone)]
pub struct InjectionResult {
    /// Method used for injection
    pub method: InjectionMethod,
    /// Length of injected content
    pub content_length: usize,
    /// When the injection occurred
    pub timestamp: DateTime<Utc>,
}

impl InjectionResult {
    /// Create a new injection result
    pub fn new(method: InjectionMethod, content_length: usize) -> Self {
        Self {
            method,
            content_length,
            timestamp: Utc::now(),
        }
    }
}

/// Injector for Claude Code learnings
pub struct ClaudeCodeInjector {
    /// Marker comment for groove sections in CLAUDE.md
    section_marker: String,
}

impl Default for ClaudeCodeInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeCodeInjector {
    /// Create a new injector
    pub fn new() -> Self {
        Self {
            section_marker: "<!-- vibes-groove-learnings -->".to_string(),
        }
    }

    /// Create an injector with a custom section marker
    pub fn with_marker(marker: String) -> Self {
        Self {
            section_marker: marker,
        }
    }

    /// Get the section marker
    pub fn section_marker(&self) -> &str {
        &self.section_marker
    }

    /// Inject learnings via hook response
    ///
    /// This is the preferred method as it injects context directly
    /// into the session without modifying files.
    pub fn inject_via_hook(&self, formatted: &str) -> HookResponse {
        if formatted.is_empty() {
            HookResponse::default()
        } else {
            HookResponse {
                additional_context: Some(formatted.to_string()),
            }
        }
    }

    /// Inject learnings by appending to CLAUDE.md
    ///
    /// This is the fallback method when hook injection isn't available.
    /// It appends a marked section to the file that can be updated later.
    pub fn inject_to_claude_md(
        &self,
        formatted: &str,
        path: &Path,
    ) -> Result<InjectionResult, GrooveError> {
        if formatted.is_empty() {
            return Ok(InjectionResult::new(InjectionMethod::ClaudeMdAppend, 0));
        }

        let content = if path.exists() {
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        let section = self.format_section(formatted);
        let new_content = if content.contains(&self.section_marker) {
            // Replace existing section
            self.replace_section(&content, &section)
        } else {
            // Append new section
            format!("{}\n\n{}", content.trim_end(), section)
        };

        fs::write(path, &new_content)?;

        Ok(InjectionResult::new(
            InjectionMethod::ClaudeMdAppend,
            formatted.len(),
        ))
    }

    /// Check if a hook event supports response injection
    pub fn supports_hook_response(hook_name: &str) -> bool {
        matches!(hook_name, "SessionStart" | "UserPromptSubmit")
    }

    /// Format the learnings section with markers
    fn format_section(&self, formatted: &str) -> String {
        format!(
            "{}\n# Groove Learnings\n\n{}\n{}",
            self.section_marker, formatted, self.section_marker
        )
    }

    /// Replace an existing marked section in content
    fn replace_section(&self, content: &str, section: &str) -> String {
        let marker = &self.section_marker;

        // Find both markers and replace the section between them
        if let Some((start, end)) = content.find(marker).and_then(|start| {
            content[start + marker.len()..]
                .find(marker)
                .map(|end_offset| (start, start + marker.len() + end_offset + marker.len()))
        }) {
            let mut result = content[..start].to_string();
            result.push_str(section);
            result.push_str(&content[end..]);
            return result;
        }

        // Fallback: append if markers not found properly
        format!("{}\n\n{}", content.trim_end(), section)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_inject_via_hook_creates_response() {
        let injector = ClaudeCodeInjector::new();
        let formatted = "## Project Patterns\n- Use TDD";

        let response = injector.inject_via_hook(formatted);

        assert!(response.additional_context.is_some());
        assert_eq!(response.additional_context.unwrap(), formatted);
    }

    #[test]
    fn test_inject_via_hook_empty_content() {
        let injector = ClaudeCodeInjector::new();

        let response = injector.inject_via_hook("");

        assert!(response.additional_context.is_none());
    }

    #[test]
    fn test_inject_to_claude_md_creates_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("CLAUDE.md");

        let injector = ClaudeCodeInjector::new();
        let formatted = "## Patterns\n- Pattern A";

        let result = injector.inject_to_claude_md(formatted, &path).unwrap();

        assert_eq!(result.method, InjectionMethod::ClaudeMdAppend);
        assert_eq!(result.content_length, formatted.len());
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Pattern A"));
        assert!(content.contains("<!-- vibes-groove-learnings -->"));
    }

    #[test]
    fn test_inject_to_claude_md_appends_to_existing() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("CLAUDE.md");

        fs::write(&path, "# Existing Content\n\nSome text here.").unwrap();

        let injector = ClaudeCodeInjector::new();
        let result = injector
            .inject_to_claude_md("## New Learnings", &path)
            .unwrap();

        assert_eq!(result.method, InjectionMethod::ClaudeMdAppend);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Existing Content"));
        assert!(content.contains("## New Learnings"));
    }

    #[test]
    fn test_inject_to_claude_md_replaces_existing_section() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("CLAUDE.md");

        let initial = "# Project\n\n<!-- vibes-groove-learnings -->\n# Groove Learnings\n\nOld content\n<!-- vibes-groove-learnings -->\n\n# Other Section";
        fs::write(&path, initial).unwrap();

        let injector = ClaudeCodeInjector::new();
        let result = injector.inject_to_claude_md("New content", &path).unwrap();

        assert_eq!(result.method, InjectionMethod::ClaudeMdAppend);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("New content"));
        assert!(!content.contains("Old content"));
        assert!(content.contains("# Other Section"));
    }

    #[test]
    fn test_inject_empty_returns_zero_length() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("CLAUDE.md");

        let injector = ClaudeCodeInjector::new();
        let result = injector.inject_to_claude_md("", &path).unwrap();

        assert_eq!(result.content_length, 0);
        assert!(!path.exists());
    }

    #[test]
    fn test_supports_hook_response() {
        assert!(ClaudeCodeInjector::supports_hook_response("SessionStart"));
        assert!(ClaudeCodeInjector::supports_hook_response(
            "UserPromptSubmit"
        ));
        assert!(!ClaudeCodeInjector::supports_hook_response("PostToolUse"));
        assert!(!ClaudeCodeInjector::supports_hook_response("Stop"));
    }

    #[test]
    fn test_custom_marker() {
        let injector = ClaudeCodeInjector::with_marker("<!-- custom -->".to_string());
        assert_eq!(injector.section_marker(), "<!-- custom -->");
    }

    #[test]
    fn test_injection_result_timestamp() {
        let before = Utc::now();
        let result = InjectionResult::new(InjectionMethod::HookResponse, 100);
        let after = Utc::now();

        assert!(result.timestamp >= before);
        assert!(result.timestamp <= after);
    }
}
