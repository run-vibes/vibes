//! Error recovery pattern detector
//!
//! Detects tool failure → fix → success sequences to extract error recovery learnings.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::capture::TranscriptToolUse;

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
    /// Configuration (used in recovery detection)
    #[allow(dead_code)]
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
}
