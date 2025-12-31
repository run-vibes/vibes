//! Subprocess-based LLM harness for assessment analysis.
//!
//! The `HarnessLLM` runs LLM analysis in a subprocess to avoid blocking the main
//! async runtime. This is important because LLM calls can take 10-30+ seconds,
//! and running them inline would block other tasks.
//!
//! ## Why Subprocess?
//!
//! 1. **Isolation**: LLM client libraries may use blocking I/O internally
//! 2. **Timeout handling**: Clean process termination on timeout
//! 3. **Resource limits**: Can constrain memory/CPU per analysis
//! 4. **Fault isolation**: Crashes don't affect the main process
//!
//! ## Usage
//!
//! ```text
//! HarnessLLM::analyze(context)
//!     │
//!     ├─ Check circuit breaker → If open, return Err(CircuitOpen)
//!     │
//!     ├─ Spawn subprocess with JSON input
//!     │
//!     ├─ Wait with timeout
//!     │
//!     └─ Parse JSON output → AnalysisResult
//! ```

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::config::LlmConfig;
use super::types::SessionId;

/// Context provided to the LLM for analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// Session being analyzed.
    pub session_id: SessionId,
    /// Message range being analyzed.
    pub message_range: (u32, u32),
    /// Recent events in the session (serialized).
    pub events_json: String,
    /// The prompt template to use.
    pub prompt_template: String,
    /// Additional context from the project.
    pub project_context: Option<String>,
}

impl AnalysisContext {
    /// Create a new analysis context.
    pub fn new(session_id: impl Into<SessionId>, message_range: (u32, u32)) -> Self {
        Self {
            session_id: session_id.into(),
            message_range,
            events_json: "[]".to_string(),
            prompt_template: String::new(),
            project_context: None,
        }
    }

    /// Set the events JSON.
    #[must_use]
    pub fn with_events_json(mut self, events_json: impl Into<String>) -> Self {
        self.events_json = events_json.into();
        self
    }

    /// Set the prompt template.
    #[must_use]
    pub fn with_prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = template.into();
        self
    }

    /// Set the project context.
    #[must_use]
    pub fn with_project_context(mut self, context: impl Into<String>) -> Self {
        self.project_context = Some(context.into());
        self
    }
}

/// Result of an LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Summary of the analysis.
    pub summary: String,
    /// Overall score (-1.0 to 1.0).
    pub score: f64,
    /// Detected issues or patterns.
    pub findings: Vec<Finding>,
    /// Suggestions for improvement.
    pub suggestions: Vec<String>,
    /// Raw response from the LLM (for debugging).
    pub raw_response: Option<String>,
}

impl AnalysisResult {
    /// Create an empty analysis result.
    pub fn empty() -> Self {
        Self {
            summary: String::new(),
            score: 0.0,
            findings: Vec::new(),
            suggestions: Vec::new(),
            raw_response: None,
        }
    }
}

/// A finding from the LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Type of finding.
    pub finding_type: FindingType,
    /// Description of the finding.
    pub description: String,
    /// Message range where the finding applies.
    pub message_range: Option<(u32, u32)>,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

/// Type of finding from analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    /// User showed frustration.
    Frustration,
    /// A pattern was successfully applied.
    PatternApplied,
    /// A tool failed multiple times.
    ToolFailures,
    /// User had to correct Claude.
    Correction,
    /// Positive progress was made.
    Progress,
    /// The task was completed.
    TaskComplete,
}

/// Error type for HarnessLLM operations.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// LLM is disabled in configuration.
    #[error("LLM is disabled")]
    Disabled,
    /// Subprocess failed to start.
    #[error("failed to spawn subprocess: {0}")]
    SpawnFailed(std::io::Error),
    /// Subprocess timed out.
    #[error("analysis timed out after {0} seconds")]
    Timeout(u32),
    /// Subprocess exited with error.
    #[error("subprocess exited with code {0}: {1}")]
    SubprocessFailed(i32, String),
    /// Failed to parse LLM response.
    #[error("failed to parse response: {0}")]
    ParseFailed(String),
    /// Serialization error.
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// Subprocess-based LLM harness.
///
/// Runs LLM analysis in a subprocess to avoid blocking the main runtime.
pub struct HarnessLLM {
    config: LlmConfig,
}

impl HarnessLLM {
    /// Create a new harness LLM with the given configuration.
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    /// Check if the harness is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the configured timeout duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.config.timeout_seconds.into())
    }

    /// Analyze session events using the LLM.
    ///
    /// This spawns a subprocess to run the LLM analysis, waiting up to
    /// the configured timeout for a response.
    pub async fn analyze(&self, context: AnalysisContext) -> Result<AnalysisResult, HarnessError> {
        if !self.config.enabled {
            return Err(HarnessError::Disabled);
        }

        // Serialize context to JSON for subprocess input
        let input_json = serde_json::to_string(&context)
            .map_err(|e| HarnessError::SerializationError(e.to_string()))?;

        // Run analysis with retries
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match self.run_subprocess(&input_json).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        max_retries = self.config.max_retries,
                        error = %e,
                        "LLM analysis attempt failed"
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(HarnessError::Disabled))
    }

    /// Run the actual subprocess for LLM analysis.
    async fn run_subprocess(&self, input_json: &str) -> Result<AnalysisResult, HarnessError> {
        use tokio::process::Command;

        // Build command based on backend
        let mut cmd = match self.config.backend.as_str() {
            "harness" => {
                // Use Claude Code as the LLM backend
                let mut cmd = Command::new("claude");
                cmd.args([
                    "--print",
                    "--model",
                    &self.config.model,
                    "--output-format",
                    "json",
                ]);
                cmd
            }
            "mock" => {
                // Mock backend for testing - just echoes a valid result
                let mut cmd = Command::new("echo");
                cmd.arg(
                    r#"{"summary":"Mock analysis","score":0.5,"findings":[],"suggestions":[]}"#,
                );
                cmd
            }
            other => {
                return Err(HarnessError::ParseFailed(format!(
                    "unknown backend: {other}"
                )));
            }
        };

        // Set up stdin for input
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn the process
        let mut child = cmd.spawn().map_err(HarnessError::SpawnFailed)?;

        // Write input to stdin (for harness backend)
        if self.config.backend == "harness"
            && let Some(mut stdin) = child.stdin.take()
        {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(input_json.as_bytes()).await;
            let _ = stdin.flush().await;
        }

        // Wait with timeout
        let timeout_duration = self.timeout();
        let output = tokio::time::timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| HarnessError::Timeout(self.config.timeout_seconds))?
            .map_err(HarnessError::SpawnFailed)?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HarnessError::SubprocessFailed(
                output.status.code().unwrap_or(-1),
                stderr.to_string(),
            ));
        }

        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| HarnessError::ParseFailed(format!("{e}: {stdout}")))
    }
}

impl Default for HarnessLLM {
    fn default() -> Self {
        Self::new(LlmConfig::default())
    }
}

impl std::fmt::Debug for HarnessLLM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HarnessLLM")
            .field("enabled", &self.config.enabled)
            .field("backend", &self.config.backend)
            .field("model", &self.config.model)
            .field("timeout_seconds", &self.config.timeout_seconds)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_llm_creation() {
        let config = LlmConfig::default();
        let harness = HarnessLLM::new(config);

        assert!(harness.is_enabled());
        assert_eq!(harness.timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_harness_llm_disabled() {
        let config = LlmConfig {
            enabled: false,
            ..Default::default()
        };
        let harness = HarnessLLM::new(config);

        assert!(!harness.is_enabled());
    }

    #[test]
    fn test_analysis_context_builder() {
        let ctx = AnalysisContext::new("test-session", (0, 10))
            .with_events_json("[{\"type\":\"test\"}]")
            .with_prompt_template("Analyze this session")
            .with_project_context("A Rust project");

        assert_eq!(ctx.session_id.as_str(), "test-session");
        assert_eq!(ctx.message_range, (0, 10));
        assert_eq!(ctx.events_json, "[{\"type\":\"test\"}]");
        assert_eq!(ctx.prompt_template, "Analyze this session");
        assert_eq!(ctx.project_context, Some("A Rust project".to_string()));
    }

    #[test]
    fn test_analysis_result_empty() {
        let result = AnalysisResult::empty();

        assert!(result.summary.is_empty());
        assert!((result.score - 0.0).abs() < f64::EPSILON);
        assert!(result.findings.is_empty());
        assert!(result.suggestions.is_empty());
    }

    #[test]
    fn test_analysis_result_serialization() {
        let result = AnalysisResult {
            summary: "Session went well".to_string(),
            score: 0.8,
            findings: vec![Finding {
                finding_type: FindingType::Progress,
                description: "Made good progress".to_string(),
                message_range: Some((0, 5)),
                confidence: 0.9,
            }],
            suggestions: vec!["Keep up the good work".to_string()],
            raw_response: None,
        };

        let json = serde_json::to_string(&result).expect("should serialize");
        let parsed: AnalysisResult = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(parsed.summary, "Session went well");
        assert!((parsed.score - 0.8).abs() < f64::EPSILON);
        assert_eq!(parsed.findings.len(), 1);
        assert_eq!(parsed.findings[0].finding_type, FindingType::Progress);
    }

    #[test]
    fn test_finding_type_serialization() {
        let types = [
            FindingType::Frustration,
            FindingType::PatternApplied,
            FindingType::ToolFailures,
            FindingType::Correction,
            FindingType::Progress,
            FindingType::TaskComplete,
        ];

        for finding_type in types {
            let json = serde_json::to_string(&finding_type).expect("should serialize");
            let parsed: FindingType = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(parsed, finding_type);
        }
    }

    #[tokio::test]
    async fn test_harness_disabled_returns_error() {
        let config = LlmConfig {
            enabled: false,
            ..Default::default()
        };
        let harness = HarnessLLM::new(config);
        let ctx = AnalysisContext::new("test", (0, 10));

        let result = harness.analyze(ctx).await;
        assert!(matches!(result, Err(HarnessError::Disabled)));
    }

    #[tokio::test]
    async fn test_harness_mock_backend() {
        let config = LlmConfig {
            enabled: true,
            backend: "mock".to_string(),
            ..Default::default()
        };
        let harness = HarnessLLM::new(config);
        let ctx = AnalysisContext::new("test", (0, 10));

        let result = harness.analyze(ctx).await.expect("mock should succeed");
        assert_eq!(result.summary, "Mock analysis");
        assert!((result.score - 0.5).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_harness_unknown_backend() {
        let config = LlmConfig {
            enabled: true,
            backend: "unknown".to_string(),
            ..Default::default()
        };
        let harness = HarnessLLM::new(config);
        let ctx = AnalysisContext::new("test", (0, 10));

        let result = harness.analyze(ctx).await;
        assert!(matches!(result, Err(HarnessError::ParseFailed(_))));
    }

    #[test]
    fn test_harness_error_display() {
        let errors = [
            HarnessError::Disabled,
            HarnessError::Timeout(30),
            HarnessError::SubprocessFailed(1, "error".to_string()),
            HarnessError::ParseFailed("invalid json".to_string()),
        ];

        for error in errors {
            let display = format!("{error}");
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_harness_llm_debug() {
        let harness = HarnessLLM::default();
        let debug = format!("{harness:?}");
        assert!(debug.contains("HarnessLLM"));
        assert!(debug.contains("enabled"));
        assert!(debug.contains("backend"));
    }
}
