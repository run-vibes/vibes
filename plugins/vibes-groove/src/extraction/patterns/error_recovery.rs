//! Error recovery pattern detector
//!
//! Detects tool failure → fix → success sequences to extract error recovery learnings.

use serde::{Deserialize, Serialize};

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

/// Placeholder for the detector (to be implemented in Task 2)
pub struct ErrorRecoveryDetector;

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
}
