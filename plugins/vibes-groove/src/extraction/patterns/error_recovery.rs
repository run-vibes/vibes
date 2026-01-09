//! Error recovery pattern detector
//!
//! Detects tool failure → fix → success sequences to extract error recovery learnings.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::assessment::{EventId, SessionId};
use crate::capture::{ParsedTranscript, TranscriptToolUse};
use crate::extraction::{ExtractionMethod, ExtractionSource, LearningCandidate, PatternType};

/// Type of error that was encountered
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Compilation error (rustc, tsc, etc.)
    CompilationError,
    /// Test failure (FAILED, FAIL)
    TestFailure,
    /// Command error (non-zero exit code)
    CommandError,
    /// Runtime error
    RuntimeError,
    /// Other error type
    Other(String),
}

/// A detected error recovery sequence
#[derive(Debug, Clone)]
pub struct ErrorRecoveryCandidate {
    /// Type of error encountered
    pub error_type: ErrorType,
    /// Error message extracted from tool output
    pub error_message: String,
    /// Index of the failed tool in the transcript
    pub failed_tool_index: usize,
    /// Indices of recovery tool calls
    pub recovery_tool_indices: Vec<usize>,
    /// Extracted recovery strategy
    pub recovery_strategy: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// Configuration for the error recovery detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    /// Whether error recovery detection is enabled
    pub enabled: bool,
    /// Minimum confidence threshold
    pub min_confidence: f64,
    /// Maximum distance (in tool calls) to look for recovery
    pub max_recovery_distance: usize,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_confidence: 0.5,
            max_recovery_distance: 5,
        }
    }
}

/// A detected failure from a tool call
#[derive(Debug, Clone)]
pub struct DetectedFailure {
    /// Type of error
    pub error_type: ErrorType,
    /// Extracted error message
    pub error_message: String,
}

/// Detects error recovery patterns in transcripts
pub struct ErrorRecoveryDetector {
    /// Pattern for Rust compilation errors
    rust_error_pattern: Regex,
    /// Pattern for TypeScript errors
    ts_error_pattern: Regex,
    /// Pattern for test failures
    test_failure_pattern: Regex,
    /// Configuration
    config: ErrorRecoveryConfig,
}

impl ErrorRecoveryDetector {
    /// Create a new detector with default configuration
    pub fn new() -> Self {
        Self::with_config(&ErrorRecoveryConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: &ErrorRecoveryConfig) -> Self {
        Self {
            rust_error_pattern: Regex::new(r"error\[E\d+\]:").unwrap(),
            ts_error_pattern: Regex::new(r"error TS\d+:").unwrap(),
            test_failure_pattern: Regex::new(r"(?i)\bFAILED\b|\bFAIL\b").unwrap(),
            config: config.clone(),
        }
    }

    /// Detect if a tool call represents a failure
    pub fn detect_failure(&self, tool: &TranscriptToolUse) -> Option<DetectedFailure> {
        // Only failed tool calls are failures
        if tool.success {
            return None;
        }

        let output = tool.output.as_deref().unwrap_or("");

        // Classify the error type based on output patterns
        let error_type = self.classify_error(output);

        // Extract the most relevant error message
        let error_message = self.extract_error_message(output);

        Some(DetectedFailure {
            error_type,
            error_message,
        })
    }

    /// Classify error type from output
    fn classify_error(&self, output: &str) -> ErrorType {
        if self.rust_error_pattern.is_match(output) {
            return ErrorType::CompilationError;
        }
        if self.ts_error_pattern.is_match(output) {
            return ErrorType::CompilationError;
        }
        if self.test_failure_pattern.is_match(output) {
            return ErrorType::TestFailure;
        }
        ErrorType::CommandError
    }

    /// Extract the most relevant error message from output
    fn extract_error_message(&self, output: &str) -> String {
        // Find the line containing "error" for a more targeted message
        for line in output.lines() {
            let lower = line.to_lowercase();
            if lower.contains("error") {
                return line.trim().to_string();
            }
        }
        // Fall back to first non-empty line
        output
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or(output)
            .trim()
            .to_string()
    }

    /// Find recovery sequence after a failure at the given index
    pub fn find_recovery(
        &self,
        tools: &[TranscriptToolUse],
        failure_index: usize,
    ) -> Option<ErrorRecoveryCandidate> {
        // Must have a failure at the given index
        let failed_tool = tools.get(failure_index)?;
        let failure = self.detect_failure(failed_tool)?;

        // Look for successful tool calls within max_recovery_distance
        let max_index = (failure_index + self.config.max_recovery_distance + 1).min(tools.len());
        let mut recovery_indices = Vec::new();
        let mut success_index = None;

        for (i, tool) in tools
            .iter()
            .enumerate()
            .take(max_index)
            .skip(failure_index + 1)
        {
            if tool.success {
                // Track modification tools as part of recovery
                if self.is_modification_tool(&tool.tool_name) {
                    recovery_indices.push(i);
                }
                // Found a success with same tool type as failure - this is the recovery point
                if tool.tool_name == failed_tool.tool_name {
                    recovery_indices.push(i);
                    success_index = Some(i);
                    break;
                }
            }
        }

        // Recovery requires finding a successful call of the same tool type
        // (Edit/Write alone doesn't prove recovery without re-running the failed command)
        let final_success = success_index?;

        // Calculate confidence
        let distance = final_success - failure_index;
        let distance_factor = 1.0 - (distance as f64 * 0.1).min(0.4);
        let confidence = (0.6 + distance_factor * 0.3).min(1.0);

        // Extract recovery strategy from intermediate tools
        let strategy = self.extract_recovery_strategy(tools, failure_index, final_success);

        Some(ErrorRecoveryCandidate {
            error_type: failure.error_type,
            error_message: failure.error_message,
            failed_tool_index: failure_index,
            recovery_tool_indices: recovery_indices,
            recovery_strategy: strategy,
            confidence,
        })
    }

    /// Check if a tool modifies files (Edit, Write, etc.)
    fn is_modification_tool(&self, name: &str) -> bool {
        matches!(name, "Edit" | "Write" | "MultiEdit")
    }

    /// Extract a description of the recovery strategy from tool calls
    fn extract_recovery_strategy(
        &self,
        tools: &[TranscriptToolUse],
        failure_index: usize,
        success_index: usize,
    ) -> String {
        let mut actions = Vec::new();

        for i in (failure_index + 1)..success_index {
            if let Some(tool) = tools.get(i)
                && tool.success
            {
                let action = match tool.tool_name.as_str() {
                    "Edit" => {
                        if let Some(path) = tool.input.get("file_path").and_then(|v| v.as_str()) {
                            format!("Edit {}", path)
                        } else {
                            "Edit file".to_string()
                        }
                    }
                    "Write" => {
                        if let Some(path) = tool.input.get("file_path").and_then(|v| v.as_str()) {
                            format!("Write {}", path)
                        } else {
                            "Write file".to_string()
                        }
                    }
                    "Bash" => {
                        if let Some(cmd) = tool.input.get("command").and_then(|v| v.as_str()) {
                            format!("Run: {}", cmd.chars().take(50).collect::<String>())
                        } else {
                            "Run command".to_string()
                        }
                    }
                    name => format!("Use {}", name),
                };
                actions.push(action);
            }
        }

        if actions.is_empty() {
            "Retried command".to_string()
        } else {
            actions.join(", then ")
        }
    }

    /// Detect error recovery patterns in a transcript
    pub fn detect(&self, transcript: &ParsedTranscript) -> Result<Vec<LearningCandidate>> {
        let mut candidates = Vec::new();
        let tools = &transcript.tool_uses;

        // Track which failures we've already processed
        let mut processed_failures = std::collections::HashSet::new();

        for (i, tool) in tools.iter().enumerate() {
            // Skip if already processed or not a failure
            if processed_failures.contains(&i) || self.detect_failure(tool).is_none() {
                continue;
            }

            // Try to find a recovery for this failure
            if let Some(recovery) = self.find_recovery(tools, i) {
                // Skip if below confidence threshold
                if recovery.confidence < self.config.min_confidence {
                    continue;
                }

                // Mark all recovery indices as processed to avoid double-counting
                for &idx in &recovery.recovery_tool_indices {
                    processed_failures.insert(idx);
                }

                // Convert to LearningCandidate
                let candidate = self.to_learning_candidate(transcript, &recovery, i);
                candidates.push(candidate);
            }
        }

        Ok(candidates)
    }

    /// Convert an error recovery to a learning candidate
    fn to_learning_candidate(
        &self,
        transcript: &ParsedTranscript,
        recovery: &ErrorRecoveryCandidate,
        failure_index: usize,
    ) -> LearningCandidate {
        let error_type_str = match &recovery.error_type {
            ErrorType::CompilationError => "compilation error",
            ErrorType::TestFailure => "test failure",
            ErrorType::CommandError => "command error",
            ErrorType::RuntimeError => "runtime error",
            ErrorType::Other(s) => s.as_str(),
        };

        let description = format!(
            "Recovered from {}: {}",
            error_type_str,
            truncate(&recovery.error_message, 100)
        );

        let insight = format!(
            "When encountering '{}', resolve by: {}",
            truncate(&recovery.error_message, 50),
            &recovery.recovery_strategy
        );

        let last_recovery = recovery
            .recovery_tool_indices
            .last()
            .copied()
            .unwrap_or(failure_index);
        let source = ExtractionSource::new(
            SessionId::from(transcript.session_id.as_str()),
            EventId::new(),
            ExtractionMethod::Pattern(PatternType::ErrorRecovery),
        )
        .with_message_range(failure_index as u32, last_recovery as u32);

        LearningCandidate::new(description, insight, recovery.confidence, source)
    }
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

impl Default for ErrorRecoveryDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ErrorType tests ---

    #[test]
    fn test_error_type_compilation_error() {
        let error_type = ErrorType::CompilationError;
        assert!(matches!(error_type, ErrorType::CompilationError));
    }

    #[test]
    fn test_error_type_test_failure() {
        let error_type = ErrorType::TestFailure;
        assert!(matches!(error_type, ErrorType::TestFailure));
    }

    #[test]
    fn test_error_type_command_error() {
        let error_type = ErrorType::CommandError;
        assert!(matches!(error_type, ErrorType::CommandError));
    }

    #[test]
    fn test_error_type_runtime_error() {
        let error_type = ErrorType::RuntimeError;
        assert!(matches!(error_type, ErrorType::RuntimeError));
    }

    #[test]
    fn test_error_type_other() {
        let error_type = ErrorType::Other("custom error".to_string());
        if let ErrorType::Other(msg) = error_type {
            assert_eq!(msg, "custom error");
        } else {
            panic!("Expected ErrorType::Other");
        }
    }

    // --- ErrorRecoveryCandidate tests ---

    #[test]
    fn test_error_recovery_candidate_creation() {
        let candidate = ErrorRecoveryCandidate {
            error_type: ErrorType::CompilationError,
            error_message: "cannot find value `foo`".to_string(),
            failed_tool_index: 5,
            recovery_tool_indices: vec![6, 7],
            recovery_strategy: "Added missing import".to_string(),
            confidence: 0.8,
        };

        assert!(matches!(candidate.error_type, ErrorType::CompilationError));
        assert_eq!(candidate.error_message, "cannot find value `foo`");
        assert_eq!(candidate.failed_tool_index, 5);
        assert_eq!(candidate.recovery_tool_indices, vec![6, 7]);
        assert_eq!(candidate.recovery_strategy, "Added missing import");
        assert_eq!(candidate.confidence, 0.8);
    }

    // --- ErrorRecoveryConfig tests ---

    #[test]
    fn test_config_default() {
        let config = ErrorRecoveryConfig::default();

        assert!(config.enabled);
        assert!(config.min_confidence > 0.0);
        assert!(config.max_recovery_distance > 0);
    }

    // --- Failure detection tests ---

    use crate::capture::TranscriptToolUse;
    use serde_json::json;

    fn make_tool_use(name: &str, output: Option<&str>, success: bool) -> TranscriptToolUse {
        TranscriptToolUse {
            tool_name: name.to_string(),
            input: json!({}),
            output: output.map(String::from),
            success,
        }
    }

    #[test]
    fn test_detect_failure_from_success_false() {
        let detector = ErrorRecoveryDetector::new();
        let tool = make_tool_use("Bash", Some("command not found"), false);

        let failure = detector.detect_failure(&tool);

        assert!(failure.is_some());
        let f = failure.unwrap();
        assert!(matches!(f.error_type, ErrorType::CommandError));
    }

    #[test]
    fn test_detect_compilation_error_from_output() {
        let detector = ErrorRecoveryDetector::new();
        let tool = make_tool_use(
            "Bash",
            Some("error[E0433]: failed to resolve: use of undeclared type `Foo`"),
            false,
        );

        let failure = detector.detect_failure(&tool);

        assert!(failure.is_some());
        let f = failure.unwrap();
        assert!(matches!(f.error_type, ErrorType::CompilationError));
    }

    #[test]
    fn test_detect_test_failure() {
        let detector = ErrorRecoveryDetector::new();
        let tool = make_tool_use(
            "Bash",
            Some("test result: FAILED. 1 passed; 1 failed;"),
            false,
        );

        let failure = detector.detect_failure(&tool);

        assert!(failure.is_some());
        let f = failure.unwrap();
        assert!(matches!(f.error_type, ErrorType::TestFailure));
    }

    #[test]
    fn test_no_failure_on_success() {
        let detector = ErrorRecoveryDetector::new();
        let tool = make_tool_use("Bash", Some("Success!"), true);

        let failure = detector.detect_failure(&tool);

        assert!(failure.is_none());
    }

    #[test]
    fn test_detect_typescript_error() {
        let detector = ErrorRecoveryDetector::new();
        let tool = make_tool_use("Bash", Some("error TS2304: Cannot find name 'foo'."), false);

        let failure = detector.detect_failure(&tool);

        assert!(failure.is_some());
        let f = failure.unwrap();
        assert!(matches!(f.error_type, ErrorType::CompilationError));
    }

    #[test]
    fn test_extract_error_message() {
        let detector = ErrorRecoveryDetector::new();
        let tool = make_tool_use(
            "Bash",
            Some("error[E0433]: cannot find value `x` in this scope"),
            false,
        );

        let failure = detector.detect_failure(&tool);

        assert!(failure.is_some());
        let f = failure.unwrap();
        assert!(f.error_message.contains("cannot find value"));
    }

    // --- Recovery detection tests ---

    fn make_tool_use_with_input(
        name: &str,
        input: serde_json::Value,
        output: Option<&str>,
        success: bool,
    ) -> TranscriptToolUse {
        TranscriptToolUse {
            tool_name: name.to_string(),
            input,
            output: output.map(String::from),
            success,
        }
    }

    #[test]
    fn test_find_recovery_after_compilation_failure() {
        let detector = ErrorRecoveryDetector::new();
        let tools = vec![
            // Failed compilation
            make_tool_use(
                "Bash",
                Some("error[E0433]: use of undeclared type `Foo`"),
                false,
            ),
            // Edit to fix the issue
            make_tool_use_with_input(
                "Edit",
                json!({"file_path": "/src/main.rs"}),
                Some("File edited successfully"),
                true,
            ),
            // Successful compilation
            make_tool_use("Bash", Some("Compiling vibes v0.1.0\nFinished dev"), true),
        ];

        let recovery = detector.find_recovery(&tools, 0);

        assert!(recovery.is_some());
        let r = recovery.unwrap();
        assert!(!r.recovery_tool_indices.is_empty());
    }

    #[test]
    fn test_no_recovery_if_no_success_follows() {
        let detector = ErrorRecoveryDetector::new();
        let tools = vec![
            // Failed compilation
            make_tool_use(
                "Bash",
                Some("error[E0433]: use of undeclared type `Foo`"),
                false,
            ),
            // Another failure
            make_tool_use("Bash", Some("error[E0412]: cannot find type `Bar`"), false),
        ];

        let recovery = detector.find_recovery(&tools, 0);

        assert!(recovery.is_none());
    }

    #[test]
    fn test_recovery_distance_limit() {
        let config = ErrorRecoveryConfig {
            enabled: true,
            min_confidence: 0.5,
            max_recovery_distance: 2,
        };
        let detector = ErrorRecoveryDetector::with_config(&config);

        let tools = vec![
            make_tool_use("Bash", Some("error: compilation failed"), false),
            make_tool_use("Read", Some("contents"), true),
            make_tool_use("Read", Some("contents"), true),
            make_tool_use("Read", Some("contents"), true),
            // Success too far away (index 4, distance > 2)
            make_tool_use("Bash", Some("Success"), true),
        ];

        let recovery = detector.find_recovery(&tools, 0);

        assert!(recovery.is_none());
    }

    #[test]
    fn test_recovery_confidence_higher_for_same_command() {
        let detector = ErrorRecoveryDetector::new();

        // Same command (Bash) for recovery
        let tools_same = vec![
            make_tool_use("Bash", Some("error: failed"), false),
            make_tool_use("Edit", Some("edited"), true),
            make_tool_use("Bash", Some("Success"), true),
        ];

        // Different successful tool
        let tools_diff = vec![
            make_tool_use("Bash", Some("error: failed"), false),
            make_tool_use("Edit", Some("edited"), true),
            make_tool_use("Read", Some("contents"), true),
        ];

        let recovery_same = detector.find_recovery(&tools_same, 0);
        let recovery_diff = detector.find_recovery(&tools_diff, 0);

        assert!(recovery_same.is_some());
        // Recovery with same command type should have higher confidence
        // or at least be found (diff may not qualify as recovery if tool type differs)
        if let Some(r_diff) = recovery_diff {
            assert!(recovery_same.unwrap().confidence >= r_diff.confidence);
        }
    }

    #[test]
    fn test_extract_recovery_strategy() {
        let detector = ErrorRecoveryDetector::new();
        let tools = vec![
            make_tool_use(
                "Bash",
                Some("error[E0433]: use of undeclared type `Foo`"),
                false,
            ),
            make_tool_use_with_input(
                "Edit",
                json!({"file_path": "/src/main.rs", "old_string": "let x = Foo;", "new_string": "use crate::Foo;\nlet x = Foo;"}),
                Some("File edited successfully"),
                true,
            ),
            make_tool_use("Bash", Some("Compiling...\nFinished"), true),
        ];

        let recovery = detector.find_recovery(&tools, 0);

        assert!(recovery.is_some());
        let r = recovery.unwrap();
        assert!(!r.recovery_strategy.is_empty());
    }

    // --- Full detect() tests ---

    use crate::capture::{ParsedTranscript, TranscriptMessage, TranscriptMetadata};

    fn make_transcript(
        tool_uses: Vec<TranscriptToolUse>,
        messages: Vec<(&str, &str)>,
    ) -> ParsedTranscript {
        ParsedTranscript {
            session_id: "test-session".to_string(),
            messages: messages
                .into_iter()
                .map(|(role, content)| TranscriptMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                    timestamp: None,
                })
                .collect(),
            tool_uses,
            metadata: TranscriptMetadata::default(),
        }
    }

    #[test]
    fn test_detect_returns_learning_candidates() {
        let detector = ErrorRecoveryDetector::new();
        let transcript = make_transcript(
            vec![
                make_tool_use("Bash", Some("error[E0433]: undeclared type"), false),
                make_tool_use_with_input(
                    "Edit",
                    json!({"file_path": "/src/main.rs"}),
                    Some("edited"),
                    true,
                ),
                make_tool_use("Bash", Some("Finished dev"), true),
            ],
            vec![
                ("user", "Build the project"),
                ("assistant", "I'll build it"),
            ],
        );

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].confidence >= 0.5);
    }

    #[test]
    fn test_detect_finds_multiple_recoveries() {
        let detector = ErrorRecoveryDetector::new();
        let transcript = make_transcript(
            vec![
                // First failure and recovery
                make_tool_use("Bash", Some("error: compilation failed"), false),
                make_tool_use("Edit", Some("edited"), true),
                make_tool_use("Bash", Some("Success"), true),
                // Second failure and recovery
                make_tool_use("Bash", Some("test result: FAILED"), false),
                make_tool_use("Edit", Some("fixed test"), true),
                make_tool_use("Bash", Some("test result: ok"), true),
            ],
            vec![],
        );

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn test_detect_filters_by_min_confidence() {
        let config = ErrorRecoveryConfig {
            enabled: true,
            min_confidence: 0.95, // Very high threshold
            max_recovery_distance: 5,
        };
        let detector = ErrorRecoveryDetector::with_config(&config);

        let transcript = make_transcript(
            vec![
                make_tool_use("Bash", Some("error: failed"), false),
                make_tool_use("Read", Some("contents"), true),
                make_tool_use("Read", Some("contents"), true),
                make_tool_use("Edit", Some("edited"), true),
                make_tool_use("Bash", Some("Success"), true),
            ],
            vec![],
        );

        let candidates = detector.detect(&transcript).unwrap();

        // Should filter out low-confidence recoveries
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_detect_creates_proper_learning_candidate() {
        let detector = ErrorRecoveryDetector::new();
        let transcript = make_transcript(
            vec![
                make_tool_use("Bash", Some("error[E0433]: cannot find type `Foo`"), false),
                make_tool_use_with_input(
                    "Edit",
                    json!({"file_path": "/src/lib.rs"}),
                    Some("edited"),
                    true,
                ),
                make_tool_use("Bash", Some("Compiling...\nFinished"), true),
            ],
            vec![],
        );

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        // Should have description about error recovery
        assert!(
            candidate.description.contains("error")
                || candidate.description.contains("Error")
                || candidate.description.contains("recovery")
        );
        // Should have insight about what to do
        assert!(!candidate.insight.is_empty());
        // Should have proper source
        assert_eq!(candidate.source.session_id.as_str(), "test-session");
    }
}
