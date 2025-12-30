//! Assessment context types for tracking attribution lineage.
//!
//! These types carry full context about which learnings were active during a session,
//! enabling the attribution engine to answer "which learnings helped in this session?"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{LearningId, Scope};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarnessType {
    /// Claude Code CLI.
    ClaudeCode,
    /// Other harness type (name provided).
    Other(String),
}

impl Default for HarnessType {
    fn default() -> Self {
        Self::ClaudeCode
    }
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
}
