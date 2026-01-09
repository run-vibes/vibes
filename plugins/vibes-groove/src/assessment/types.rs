//! Assessment context types for tracking attribution lineage.
//!
//! These types carry full context about which learnings were active during a session,
//! enabling the attribution engine to answer "which learnings helped in this session?"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{LearningId, Scope};

// Re-export CheckpointTrigger from checkpoint module so MediumEvent can use it
pub use super::checkpoint::CheckpointTrigger;

/// UUIDv7 wrapper for time-ordered event IDs.
///
/// Events use UUIDv7 which embeds a timestamp, allowing natural time-ordering
/// without requiring a separate timestamp field for sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    /// Create a new time-ordered event ID using UUIDv7.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Get the underlying UUID.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Extract the timestamp from the UUIDv7.
    #[must_use]
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.0.get_timestamp().map(|ts| {
            let (secs, nanos) = ts.to_unix();
            DateTime::from_timestamp(secs as i64, nanos).unwrap_or_else(Utc::now)
        })
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for EventId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// String wrapper for session identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID from a string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// String wrapper for user identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// Create a new user ID from a string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// String wrapper for project identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(String);

impl ProjectId {
    /// Create a new project ID from a string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// How learnings were injected into the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum InjectionMethod {
    /// Learnings were written to CLAUDE.md file.
    ClaudeMd,
    /// Learnings were injected via hooks.
    Hook,
    /// Learnings were injected via both CLAUDE.md and hooks.
    Both,
    /// No learnings were injected.
    #[default]
    None,
}

impl InjectionMethod {
    /// Convert to string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeMd => "claude_md",
            Self::Hook => "hook",
            Self::Both => "both",
            Self::None => "none",
        }
    }
}

impl std::fmt::Display for InjectionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The type of AI harness being used.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HarnessType {
    /// Claude Code CLI.
    #[default]
    ClaudeCode,
    /// Other harness type (name provided).
    Other(String),
}

impl HarnessType {
    /// Convert to string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::ClaudeCode => "claude_code",
            Self::Other(name) => name,
        }
    }
}

impl std::fmt::Display for HarnessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Full attribution context for an assessment event.
///
/// This struct carries all the information needed to attribute session outcomes
/// back to specific learnings. Every assessment event includes this context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentContext {
    /// Unique identifier for the session being assessed.
    pub session_id: SessionId,

    /// Unique identifier for this specific assessment event.
    pub event_id: EventId,

    /// When this assessment occurred.
    pub timestamp: DateTime<Utc>,

    /// The learnings that were active/injected for this session.
    pub active_learnings: Vec<LearningId>,

    /// How the learnings were injected.
    pub injection_method: InjectionMethod,

    /// The scope at which learnings were injected.
    pub injection_scope: Option<Scope>,

    /// The type of AI harness being used.
    pub harness_type: HarnessType,

    /// Version of the harness (e.g., "1.0.54").
    pub harness_version: Option<String>,

    /// The project this session is associated with.
    pub project_id: Option<ProjectId>,

    /// The user running this session.
    pub user_id: Option<UserId>,
}

impl AssessmentContext {
    /// Create a new assessment context with required fields.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            event_id: EventId::new(),
            timestamp: Utc::now(),
            active_learnings: Vec::new(),
            injection_method: InjectionMethod::None,
            injection_scope: None,
            harness_type: HarnessType::ClaudeCode,
            harness_version: None,
            project_id: None,
            user_id: None,
        }
    }

    /// Add active learnings to the context.
    #[must_use]
    pub fn with_learnings(mut self, learnings: Vec<LearningId>) -> Self {
        self.active_learnings = learnings;
        self
    }

    /// Set the injection method.
    #[must_use]
    pub fn with_injection_method(mut self, method: InjectionMethod) -> Self {
        self.injection_method = method;
        self
    }

    /// Set the injection scope.
    #[must_use]
    pub fn with_injection_scope(mut self, scope: Scope) -> Self {
        self.injection_scope = Some(scope);
        self
    }

    /// Set the harness type.
    #[must_use]
    pub fn with_harness_type(mut self, harness_type: HarnessType) -> Self {
        self.harness_type = harness_type;
        self
    }

    /// Set the harness version.
    #[must_use]
    pub fn with_harness_version(mut self, version: impl Into<String>) -> Self {
        self.harness_version = Some(version.into());
        self
    }

    /// Set the project ID.
    #[must_use]
    pub fn with_project_id(mut self, project_id: impl Into<ProjectId>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Set the user ID.
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<UserId>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

impl Default for AssessmentContext {
    fn default() -> Self {
        Self::new("default-session")
    }
}

// =============================================================================
// Three-Tier Assessment Event Types
// =============================================================================

/// A lightweight signal detected in a message.
///
/// These signals are detected per-message with <10ms latency and form the
/// foundation of the assessment system's signal detection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LightweightSignal {
    /// Negative sentiment detected (e.g., frustration, confusion).
    Negative {
        /// The pattern that matched.
        pattern: String,
        /// Confidence score (0.0 to 1.0).
        confidence: f64,
    },
    /// Positive sentiment detected (e.g., satisfaction, success).
    Positive {
        /// The pattern that matched.
        pattern: String,
        /// Confidence score (0.0 to 1.0).
        confidence: f64,
    },
    /// A tool call failed.
    ToolFailure {
        /// Name of the tool that failed.
        tool_name: String,
    },
    /// User corrected Claude's output.
    Correction,
    /// User requested a retry.
    Retry,
    /// Build/test status changed.
    BuildStatus {
        /// Whether the build/tests passed.
        passed: bool,
    },
}

/// Lightweight assessment event (per-message).
///
/// These events are emitted for every message in the session and carry
/// lightweight signals that can be aggregated for trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightEvent {
    /// Full attribution context.
    pub context: AssessmentContext,

    /// Index of the message in the session (0-based).
    pub message_idx: u32,

    /// Signals detected in this message.
    pub signals: Vec<LightweightSignal>,

    /// Exponential moving average of frustration (0.0 to 1.0).
    pub frustration_ema: f64,

    /// Exponential moving average of success indicators (0.0 to 1.0).
    pub success_ema: f64,

    /// Event ID of the VibesEvent that triggered this assessment.
    /// Used to link back to the original event in the main firehose.
    pub triggering_event_id: uuid::Uuid,
}

impl LightweightEvent {
    /// Create a new lightweight event.
    #[must_use]
    pub fn new(
        context: AssessmentContext,
        message_idx: u32,
        triggering_event_id: uuid::Uuid,
    ) -> Self {
        Self {
            context,
            message_idx,
            signals: Vec::new(),
            frustration_ema: 0.0,
            success_ema: 0.0,
            triggering_event_id,
        }
    }

    /// Add a signal to the event.
    #[must_use]
    pub fn with_signal(mut self, signal: LightweightSignal) -> Self {
        self.signals.push(signal);
        self
    }

    /// Set the frustration EMA.
    #[must_use]
    pub fn with_frustration_ema(mut self, ema: f64) -> Self {
        self.frustration_ema = ema;
        self
    }

    /// Set the success EMA.
    #[must_use]
    pub fn with_success_ema(mut self, ema: f64) -> Self {
        self.success_ema = ema;
        self
    }
}

/// UUIDv7 wrapper for checkpoint IDs.
///
/// Similar to EventId but specifically for checkpoint identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(Uuid);

impl CheckpointId {
    /// Create a new time-ordered checkpoint ID using UUIDv7.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Get the underlying UUID.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Extract the timestamp from the UUIDv7.
    #[must_use]
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.0.get_timestamp().map(|ts| {
            let (secs, nanos) = ts.to_unix();
            DateTime::from_timestamp(secs as i64, nanos).unwrap_or_else(Utc::now)
        })
    }
}

impl Default for CheckpointId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for CheckpointId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for CheckpointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Token usage metrics for a message segment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenMetrics {
    /// Total tokens used in the segment.
    pub total_tokens: u64,

    /// Ratio of correction-related tokens to total (0.0 to 1.0).
    pub correction_ratio: f64,

    /// Number of retry attempts in the segment.
    pub retry_count: u32,
}

impl TokenMetrics {
    /// Create new token metrics.
    #[must_use]
    pub fn new(total_tokens: u64, correction_ratio: f64, retry_count: u32) -> Self {
        Self {
            total_tokens,
            correction_ratio,
            retry_count,
        }
    }
}

/// Medium assessment event (checkpoint summary).
///
/// These events are emitted asynchronously, typically every ~10 messages,
/// and provide aggregated insights over a segment of the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumEvent {
    /// Full attribution context.
    pub context: AssessmentContext,

    /// Unique identifier for this checkpoint.
    pub checkpoint_id: CheckpointId,

    /// Message range covered by this checkpoint (start, end exclusive).
    pub message_range: (u32, u32),

    /// What triggered this checkpoint.
    pub trigger: CheckpointTrigger,

    /// Brief summary of the segment.
    pub summary: String,

    /// Token usage metrics for the segment.
    pub token_metrics: TokenMetrics,

    /// Overall score for the segment (-1.0 to 1.0).
    pub segment_score: f64,

    /// IDs of learnings that were referenced/used in this segment.
    pub learnings_referenced: Vec<LearningId>,

    /// Event IDs of the VibesEvents in this segment.
    /// Used to link back to the original events in the main firehose.
    pub event_ids_in_segment: Vec<Uuid>,
}

impl MediumEvent {
    /// Create a new medium event.
    #[must_use]
    pub fn new(
        context: AssessmentContext,
        message_range: (u32, u32),
        trigger: CheckpointTrigger,
    ) -> Self {
        Self {
            context,
            checkpoint_id: CheckpointId::new(),
            message_range,
            trigger,
            summary: String::new(),
            token_metrics: TokenMetrics::default(),
            segment_score: 0.0,
            learnings_referenced: Vec::new(),
            event_ids_in_segment: Vec::new(),
        }
    }

    /// Set the event IDs in this segment.
    #[must_use]
    pub fn with_event_ids(mut self, event_ids: Vec<Uuid>) -> Self {
        self.event_ids_in_segment = event_ids;
        self
    }

    /// Set the summary.
    #[must_use]
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Set the token metrics.
    #[must_use]
    pub fn with_token_metrics(mut self, metrics: TokenMetrics) -> Self {
        self.token_metrics = metrics;
        self
    }

    /// Set the segment score.
    #[must_use]
    pub fn with_segment_score(mut self, score: f64) -> Self {
        self.segment_score = score;
        self
    }

    /// Add referenced learnings.
    #[must_use]
    pub fn with_learnings_referenced(mut self, learnings: Vec<LearningId>) -> Self {
        self.learnings_referenced = learnings;
        self
    }
}

/// Session outcome classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The task was completed successfully.
    Success,
    /// The task was partially completed.
    Partial,
    /// The task failed.
    Failure,
    /// The task was abandoned before completion.
    Abandoned,
}

impl Outcome {
    /// Check if this is a successful outcome.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if this is any form of completion (success or partial).
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Success | Self::Partial)
    }
}

/// Attribution score for a learning (-1.0 to 1.0).
///
/// - Positive values indicate the learning helped
/// - Negative values indicate the learning hindered
/// - Zero indicates neutral/unknown impact
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttributionScore(f64);

impl AttributionScore {
    /// Create a new attribution score, clamping to [-1.0, 1.0].
    #[must_use]
    pub fn new(score: f64) -> Self {
        Self(score.clamp(-1.0, 1.0))
    }

    /// Get the underlying score value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if this is a positive attribution.
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }

    /// Check if this is a negative attribution.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.0 < 0.0
    }
}

impl Default for AttributionScore {
    fn default() -> Self {
        Self(0.0)
    }
}

impl From<f64> for AttributionScore {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// A candidate for learning extraction from a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionCandidate {
    /// Message range where the potential learning was observed.
    pub message_range: (u32, u32),

    /// Description of the potential learning.
    pub description: String,

    /// Confidence that this should be extracted (0.0 to 1.0).
    pub confidence: f64,
}

impl ExtractionCandidate {
    /// Create a new extraction candidate.
    #[must_use]
    pub fn new(message_range: (u32, u32), description: impl Into<String>, confidence: f64) -> Self {
        Self {
            message_range,
            description: description.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Heavy assessment event (full session analysis).
///
/// These events are emitted at session end (sampled) and provide
/// comprehensive analysis including outcome classification and
/// extraction candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeavyEvent {
    /// Full attribution context.
    pub context: AssessmentContext,

    /// The overall session outcome.
    pub outcome: Outcome,

    /// Summary of what the task was trying to accomplish.
    pub task_summary: String,

    /// Trajectory of frustration over the session (aggregated EMAs).
    pub frustration_trajectory: Vec<f64>,

    /// Attribution scores for each learning that was active.
    pub learning_attributions: std::collections::HashMap<LearningId, AttributionScore>,

    /// Candidates for new learning extraction.
    pub extraction_candidates: Vec<ExtractionCandidate>,
}

impl HeavyEvent {
    /// Create a new heavy event.
    #[must_use]
    pub fn new(context: AssessmentContext, outcome: Outcome) -> Self {
        Self {
            context,
            outcome,
            task_summary: String::new(),
            frustration_trajectory: Vec::new(),
            learning_attributions: std::collections::HashMap::new(),
            extraction_candidates: Vec::new(),
        }
    }

    /// Set the task summary.
    #[must_use]
    pub fn with_task_summary(mut self, summary: impl Into<String>) -> Self {
        self.task_summary = summary.into();
        self
    }

    /// Set the frustration trajectory.
    #[must_use]
    pub fn with_frustration_trajectory(mut self, trajectory: Vec<f64>) -> Self {
        self.frustration_trajectory = trajectory;
        self
    }

    /// Add a learning attribution.
    #[must_use]
    pub fn with_attribution(mut self, learning_id: LearningId, score: AttributionScore) -> Self {
        self.learning_attributions.insert(learning_id, score);
        self
    }

    /// Add extraction candidates.
    #[must_use]
    pub fn with_extraction_candidates(mut self, candidates: Vec<ExtractionCandidate>) -> Self {
        self.extraction_candidates = candidates;
        self
    }
}

/// Union type for all assessment event tiers.
///
/// This allows a single stream to carry all assessment events while
/// maintaining type safety and tier identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tier", rename_all = "snake_case")]
pub enum AssessmentEvent {
    /// Per-message lightweight assessment.
    Lightweight(LightweightEvent),
    /// Checkpoint medium assessment.
    Medium(MediumEvent),
    /// Full session heavy assessment.
    Heavy(HeavyEvent),
}

impl AssessmentEvent {
    /// Get the session ID for this event.
    #[must_use]
    pub fn session_id(&self) -> &SessionId {
        match self {
            Self::Lightweight(e) => &e.context.session_id,
            Self::Medium(e) => &e.context.session_id,
            Self::Heavy(e) => &e.context.session_id,
        }
    }

    /// Get the event ID for this event.
    #[must_use]
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::Lightweight(e) => &e.context.event_id,
            Self::Medium(e) => &e.context.event_id,
            Self::Heavy(e) => &e.context.event_id,
        }
    }

    /// Get the assessment context for this event.
    #[must_use]
    pub fn context(&self) -> &AssessmentContext {
        match self {
            Self::Lightweight(e) => &e.context,
            Self::Medium(e) => &e.context,
            Self::Heavy(e) => &e.context,
        }
    }

    /// Get the timestamp for this event.
    #[must_use]
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.context().timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_is_time_ordered() {
        // Create two EventIds with a small delay
        let id1 = EventId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = EventId::new();

        // UUIDv7 should be version 7
        assert_eq!(id1.as_uuid().get_version_num(), 7);
        assert_eq!(id2.as_uuid().get_version_num(), 7);

        // Later ID should sort after earlier ID (lexicographically for UUIDv7)
        assert!(id1.as_uuid() < id2.as_uuid());

        // Timestamps should be extractable and ordered
        let ts1 = id1.timestamp().expect("timestamp should be extractable");
        let ts2 = id2.timestamp().expect("timestamp should be extractable");
        assert!(ts1 <= ts2);
    }

    #[test]
    fn test_event_id_display() {
        let id = EventId::new();
        let display = format!("{id}");
        assert!(!display.is_empty());
        // Should be a valid UUID string
        assert!(Uuid::parse_str(&display).is_ok());
    }

    #[test]
    fn test_session_id_from_string() {
        let session_id = SessionId::from("test-session-123".to_string());
        assert_eq!(session_id.as_str(), "test-session-123");

        let session_id2 = SessionId::from("another-session");
        assert_eq!(session_id2.as_str(), "another-session");

        let session_id3 = SessionId::new("created-with-new");
        assert_eq!(session_id3.as_str(), "created-with-new");
    }

    #[test]
    fn test_session_id_display() {
        let session_id = SessionId::new("display-test");
        assert_eq!(format!("{session_id}"), "display-test");
    }

    #[test]
    fn test_user_id_and_project_id() {
        let user_id = UserId::new("user-123");
        assert_eq!(user_id.as_str(), "user-123");

        let project_id = ProjectId::new("/path/to/project");
        assert_eq!(project_id.as_str(), "/path/to/project");
    }

    #[test]
    fn test_injection_method_variants() {
        assert_eq!(InjectionMethod::ClaudeMd.as_str(), "claude_md");
        assert_eq!(InjectionMethod::Hook.as_str(), "hook");
        assert_eq!(InjectionMethod::Both.as_str(), "both");
        assert_eq!(InjectionMethod::None.as_str(), "none");

        // Default should be None
        assert_eq!(InjectionMethod::default(), InjectionMethod::None);
    }

    #[test]
    fn test_harness_type_variants() {
        assert_eq!(HarnessType::ClaudeCode.as_str(), "claude_code");
        assert_eq!(HarnessType::Other("cursor".into()).as_str(), "cursor");

        // Default should be ClaudeCode
        assert_eq!(HarnessType::default(), HarnessType::ClaudeCode);
    }

    #[test]
    fn test_assessment_context_has_sensible_defaults() {
        let ctx = AssessmentContext::default();

        // Should have a default session ID
        assert_eq!(ctx.session_id.as_str(), "default-session");

        // Should have a valid event ID
        assert_eq!(ctx.event_id.as_uuid().get_version_num(), 7);

        // Should have reasonable timestamp (within last minute)
        let now = Utc::now();
        assert!(ctx.timestamp <= now);
        assert!(ctx.timestamp > now - chrono::Duration::minutes(1));

        // Should have empty learnings
        assert!(ctx.active_learnings.is_empty());

        // Should have default injection method
        assert_eq!(ctx.injection_method, InjectionMethod::None);

        // Optional fields should be None
        assert!(ctx.injection_scope.is_none());
        assert!(ctx.harness_version.is_none());
        assert!(ctx.project_id.is_none());
        assert!(ctx.user_id.is_none());

        // Harness type should default to ClaudeCode
        assert_eq!(ctx.harness_type, HarnessType::ClaudeCode);
    }

    #[test]
    fn test_assessment_context_builder_pattern() {
        let learning_id = Uuid::now_v7();

        let ctx = AssessmentContext::new("my-session")
            .with_learnings(vec![learning_id])
            .with_injection_method(InjectionMethod::ClaudeMd)
            .with_injection_scope(Scope::Project("/home/alex/project".into()))
            .with_harness_type(HarnessType::ClaudeCode)
            .with_harness_version("1.0.54")
            .with_project_id("/home/alex/project")
            .with_user_id("alex");

        assert_eq!(ctx.session_id.as_str(), "my-session");
        assert_eq!(ctx.active_learnings.len(), 1);
        assert_eq!(ctx.active_learnings[0], learning_id);
        assert_eq!(ctx.injection_method, InjectionMethod::ClaudeMd);
        assert_eq!(
            ctx.injection_scope,
            Some(Scope::Project("/home/alex/project".into()))
        );
        assert_eq!(ctx.harness_type, HarnessType::ClaudeCode);
        assert_eq!(ctx.harness_version.as_deref(), Some("1.0.54"));
        assert_eq!(
            ctx.project_id.as_ref().map(|p| p.as_str()),
            Some("/home/alex/project")
        );
        assert_eq!(ctx.user_id.as_ref().map(|u| u.as_str()), Some("alex"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let learning_id = Uuid::now_v7();

        let ctx = AssessmentContext::new("serialize-test")
            .with_learnings(vec![learning_id])
            .with_injection_method(InjectionMethod::Both)
            .with_harness_type(HarnessType::Other("test-harness".into()))
            .with_user_id("test-user");

        // Serialize to JSON
        let json = serde_json::to_string(&ctx).expect("should serialize to JSON");

        // Deserialize back
        let parsed: AssessmentContext =
            serde_json::from_str(&json).expect("should deserialize from JSON");

        // Verify fields match
        assert_eq!(parsed.session_id.as_str(), ctx.session_id.as_str());
        assert_eq!(parsed.event_id, ctx.event_id);
        assert_eq!(parsed.active_learnings, ctx.active_learnings);
        assert_eq!(parsed.injection_method, ctx.injection_method);
        assert_eq!(parsed.harness_type, ctx.harness_type);
        assert_eq!(
            parsed.user_id.as_ref().map(|u| u.as_str()),
            Some("test-user")
        );
    }

    #[test]
    fn test_event_id_serialization() {
        let id = EventId::new();
        let json = serde_json::to_string(&id).expect("should serialize");
        let parsed: EventId = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed, id);
    }

    #[test]
    fn test_injection_method_serialization() {
        for method in [
            InjectionMethod::ClaudeMd,
            InjectionMethod::Hook,
            InjectionMethod::Both,
            InjectionMethod::None,
        ] {
            let json = serde_json::to_string(&method).expect("should serialize");
            let parsed: InjectionMethod = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(parsed, method);
        }
    }

    #[test]
    fn test_harness_type_serialization() {
        let types = [HarnessType::ClaudeCode, HarnessType::Other("custom".into())];

        for harness_type in types {
            let json = serde_json::to_string(&harness_type).expect("should serialize");
            let parsed: HarnessType = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(parsed, harness_type);
        }
    }

    // =========================================================================
    // Three-Tier Assessment Event Tests
    // =========================================================================

    #[test]
    fn test_lightweight_signal_serialization() {
        let signals = [
            LightweightSignal::Negative {
                pattern: "frustrated".into(),
                confidence: 0.85,
            },
            LightweightSignal::Positive {
                pattern: "thanks".into(),
                confidence: 0.95,
            },
            LightweightSignal::ToolFailure {
                tool_name: "bash".into(),
            },
            LightweightSignal::Correction,
            LightweightSignal::Retry,
            LightweightSignal::BuildStatus { passed: true },
        ];

        for signal in signals {
            let json = serde_json::to_string(&signal).expect("should serialize");
            let parsed: LightweightSignal =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(parsed, signal);
        }
    }

    #[test]
    fn test_lightweight_signal_json_structure() {
        // Verify the tagged structure
        let signal = LightweightSignal::Negative {
            pattern: "error".into(),
            confidence: 0.9,
        };
        let json = serde_json::to_string(&signal).expect("should serialize");
        assert!(json.contains("\"type\":\"negative\""));
        assert!(json.contains("\"pattern\":\"error\""));
        assert!(json.contains("\"confidence\":0.9"));

        // Check unit variants
        let correction = LightweightSignal::Correction;
        let json = serde_json::to_string(&correction).expect("should serialize");
        assert!(json.contains("\"type\":\"correction\""));
    }

    #[test]
    fn test_lightweight_event_serialization_roundtrip() {
        let ctx = AssessmentContext::new("lightweight-test");
        let event = LightweightEvent::new(ctx, 5, Uuid::now_v7())
            .with_signal(LightweightSignal::Negative {
                pattern: "ugh".into(),
                confidence: 0.7,
            })
            .with_signal(LightweightSignal::Retry)
            .with_frustration_ema(0.35)
            .with_success_ema(0.65);

        let json = serde_json::to_string(&event).expect("should serialize");
        let parsed: LightweightEvent = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(parsed.message_idx, 5);
        assert_eq!(parsed.signals.len(), 2);
        assert!((parsed.frustration_ema - 0.35).abs() < f64::EPSILON);
        assert!((parsed.success_ema - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn test_checkpoint_trigger_serialization() {
        let triggers = [
            CheckpointTrigger::TimeInterval,
            CheckpointTrigger::ThresholdExceeded {
                metric: "frustration_ema".to_string(),
                value: 0.75,
            },
            CheckpointTrigger::PatternMatch {
                pattern: "2 tool failures".to_string(),
            },
        ];

        for trigger in triggers {
            let json = serde_json::to_string(&trigger).expect("should serialize");
            let parsed: CheckpointTrigger =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(parsed, trigger);
        }
    }

    #[test]
    fn test_checkpoint_id_is_time_ordered() {
        let id1 = CheckpointId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = CheckpointId::new();

        assert_eq!(id1.as_uuid().get_version_num(), 7);
        assert_eq!(id2.as_uuid().get_version_num(), 7);
        assert!(id1.as_uuid() < id2.as_uuid());
    }

    #[test]
    fn test_medium_event_serialization_roundtrip() {
        let learning_id = Uuid::now_v7();
        let ctx = AssessmentContext::new("medium-test").with_learnings(vec![learning_id]);

        let event = MediumEvent::new(ctx, (0, 10), CheckpointTrigger::TimeInterval)
            .with_summary("Completed database setup")
            .with_token_metrics(TokenMetrics::new(5000, 0.1, 2))
            .with_segment_score(0.8)
            .with_learnings_referenced(vec![learning_id]);

        let json = serde_json::to_string(&event).expect("should serialize");
        let parsed: MediumEvent = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(parsed.message_range, (0, 10));
        assert_eq!(parsed.trigger, CheckpointTrigger::TimeInterval);
        assert_eq!(parsed.summary, "Completed database setup");
        assert_eq!(parsed.token_metrics.total_tokens, 5000);
        assert!((parsed.token_metrics.correction_ratio - 0.1).abs() < f64::EPSILON);
        assert_eq!(parsed.token_metrics.retry_count, 2);
        assert!((parsed.segment_score - 0.8).abs() < f64::EPSILON);
        assert_eq!(parsed.learnings_referenced.len(), 1);
    }

    #[test]
    fn test_outcome_variants() {
        assert!(Outcome::Success.is_success());
        assert!(Outcome::Success.is_completed());

        assert!(!Outcome::Partial.is_success());
        assert!(Outcome::Partial.is_completed());

        assert!(!Outcome::Failure.is_success());
        assert!(!Outcome::Failure.is_completed());

        assert!(!Outcome::Abandoned.is_success());
        assert!(!Outcome::Abandoned.is_completed());
    }

    #[test]
    fn test_outcome_serialization() {
        let outcomes = [
            Outcome::Success,
            Outcome::Partial,
            Outcome::Failure,
            Outcome::Abandoned,
        ];

        for outcome in outcomes {
            let json = serde_json::to_string(&outcome).expect("should serialize");
            let parsed: Outcome = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(parsed, outcome);
        }
    }

    #[test]
    fn test_attribution_score_clamps_to_range() {
        // Values within range should be preserved
        let score = AttributionScore::new(0.5);
        assert!((score.value() - 0.5).abs() < f64::EPSILON);

        let neg_score = AttributionScore::new(-0.5);
        assert!((neg_score.value() - (-0.5)).abs() < f64::EPSILON);

        // Values above 1.0 should clamp to 1.0
        let high = AttributionScore::new(1.5);
        assert!((high.value() - 1.0).abs() < f64::EPSILON);

        let very_high = AttributionScore::new(100.0);
        assert!((very_high.value() - 1.0).abs() < f64::EPSILON);

        // Values below -1.0 should clamp to -1.0
        let low = AttributionScore::new(-1.5);
        assert!((low.value() - (-1.0)).abs() < f64::EPSILON);

        let very_low = AttributionScore::new(-100.0);
        assert!((very_low.value() - (-1.0)).abs() < f64::EPSILON);

        // Edge cases
        let at_max = AttributionScore::new(1.0);
        assert!((at_max.value() - 1.0).abs() < f64::EPSILON);

        let at_min = AttributionScore::new(-1.0);
        assert!((at_min.value() - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_attribution_score_positive_negative() {
        let positive = AttributionScore::new(0.5);
        assert!(positive.is_positive());
        assert!(!positive.is_negative());

        let negative = AttributionScore::new(-0.5);
        assert!(!negative.is_positive());
        assert!(negative.is_negative());

        let zero = AttributionScore::new(0.0);
        assert!(!zero.is_positive());
        assert!(!zero.is_negative());
    }

    #[test]
    fn test_attribution_score_from_f64() {
        let score: AttributionScore = 0.75.into();
        assert!((score.value() - 0.75).abs() < f64::EPSILON);

        // Should still clamp
        let clamped: AttributionScore = 5.0.into();
        assert!((clamped.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extraction_candidate_clamps_confidence() {
        let candidate = ExtractionCandidate::new((0, 5), "A useful pattern", 0.85);
        assert!((candidate.confidence - 0.85).abs() < f64::EPSILON);

        // Should clamp high values
        let high = ExtractionCandidate::new((0, 5), "test", 1.5);
        assert!((high.confidence - 1.0).abs() < f64::EPSILON);

        // Should clamp negative values
        let low = ExtractionCandidate::new((0, 5), "test", -0.5);
        assert!((low.confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_heavy_event_serialization_roundtrip() {
        let learning_id = Uuid::now_v7();
        let ctx = AssessmentContext::new("heavy-test").with_learnings(vec![learning_id]);

        let event = HeavyEvent::new(ctx, Outcome::Success)
            .with_task_summary("Implemented authentication flow")
            .with_frustration_trajectory(vec![0.1, 0.2, 0.15, 0.05])
            .with_attribution(learning_id, AttributionScore::new(0.8))
            .with_extraction_candidates(vec![ExtractionCandidate::new(
                (5, 10),
                "Use builder pattern for complex types",
                0.9,
            )]);

        let json = serde_json::to_string(&event).expect("should serialize");
        let parsed: HeavyEvent = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(parsed.outcome, Outcome::Success);
        assert_eq!(parsed.task_summary, "Implemented authentication flow");
        assert_eq!(parsed.frustration_trajectory.len(), 4);
        assert!(parsed.learning_attributions.contains_key(&learning_id));
        assert!((parsed.learning_attributions[&learning_id].value() - 0.8).abs() < f64::EPSILON);
        assert_eq!(parsed.extraction_candidates.len(), 1);
        assert_eq!(
            parsed.extraction_candidates[0].description,
            "Use builder pattern for complex types"
        );
    }

    #[test]
    fn test_assessment_event_session_id_works_for_all_variants() {
        let session_id = "test-session-id";

        // Lightweight
        let lightweight_ctx = AssessmentContext::new(session_id);
        let lightweight =
            AssessmentEvent::Lightweight(LightweightEvent::new(lightweight_ctx, 0, Uuid::now_v7()));
        assert_eq!(lightweight.session_id().as_str(), session_id);

        // Medium
        let medium_ctx = AssessmentContext::new(session_id);
        let medium = AssessmentEvent::Medium(MediumEvent::new(
            medium_ctx,
            (0, 10),
            CheckpointTrigger::TimeInterval,
        ));
        assert_eq!(medium.session_id().as_str(), session_id);

        // Heavy
        let heavy_ctx = AssessmentContext::new(session_id);
        let heavy = AssessmentEvent::Heavy(HeavyEvent::new(heavy_ctx, Outcome::Success));
        assert_eq!(heavy.session_id().as_str(), session_id);
    }

    #[test]
    fn test_assessment_event_event_id_works_for_all_variants() {
        // Lightweight
        let lightweight_ctx = AssessmentContext::new("test");
        let lightweight_event_id = lightweight_ctx.event_id;
        let lightweight =
            AssessmentEvent::Lightweight(LightweightEvent::new(lightweight_ctx, 0, Uuid::now_v7()));
        assert_eq!(*lightweight.event_id(), lightweight_event_id);

        // Medium
        let medium_ctx = AssessmentContext::new("test");
        let medium_event_id = medium_ctx.event_id;
        let medium = AssessmentEvent::Medium(MediumEvent::new(
            medium_ctx,
            (0, 10),
            CheckpointTrigger::TimeInterval,
        ));
        assert_eq!(*medium.event_id(), medium_event_id);

        // Heavy
        let heavy_ctx = AssessmentContext::new("test");
        let heavy_event_id = heavy_ctx.event_id;
        let heavy = AssessmentEvent::Heavy(HeavyEvent::new(heavy_ctx, Outcome::Failure));
        assert_eq!(*heavy.event_id(), heavy_event_id);
    }

    #[test]
    fn test_assessment_event_tagged_serialization() {
        let ctx = AssessmentContext::new("tagged-test");
        let event = AssessmentEvent::Lightweight(LightweightEvent::new(ctx, 0, Uuid::now_v7()));

        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains("\"tier\":\"lightweight\""));

        let parsed: AssessmentEvent = serde_json::from_str(&json).expect("should deserialize");
        assert!(matches!(parsed, AssessmentEvent::Lightweight(_)));
    }

    #[test]
    fn test_assessment_event_context_accessor() {
        let learning_id = Uuid::now_v7();
        let ctx = AssessmentContext::new("context-test").with_learnings(vec![learning_id]);

        let event = AssessmentEvent::Heavy(HeavyEvent::new(ctx.clone(), Outcome::Success));

        // Access context through the accessor
        let accessed_ctx = event.context();
        assert_eq!(accessed_ctx.session_id.as_str(), "context-test");
        assert_eq!(accessed_ctx.active_learnings.len(), 1);
        assert_eq!(accessed_ctx.active_learnings[0], learning_id);
    }

    #[test]
    fn test_assessment_event_timestamp_accessor() {
        let ctx = AssessmentContext::new("timestamp-test");
        let expected_timestamp = ctx.timestamp;

        let event = AssessmentEvent::Lightweight(LightweightEvent {
            context: ctx,
            message_idx: 0,
            signals: vec![],
            frustration_ema: 0.0,
            success_ema: 1.0,
            triggering_event_id: Uuid::now_v7(),
        });

        // Timestamp should match context timestamp
        assert_eq!(event.timestamp(), expected_timestamp);
    }
}
