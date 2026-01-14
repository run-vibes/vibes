//! Trace context propagation for vibes-specific attributes.
//!
//! This module provides:
//! - `TraceContext` for carrying vibes-specific context through spans
//! - `VibesSpanExt` trait for recording context on spans
//! - Standard attribute names for consistent telemetry

use std::fmt;

/// Standard attribute names for vibes telemetry.
///
/// These follow the `vibes.` namespace convention for custom attributes.
pub mod attributes {
    pub const SESSION_ID: &str = "vibes.session_id";
    pub const AGENT_ID: &str = "vibes.agent_id";
    pub const AGENT_TYPE: &str = "vibes.agent_type";
    pub const SWARM_ID: &str = "vibes.swarm_id";
    pub const MODEL_ID: &str = "vibes.model_id";
    pub const TASK_ID: &str = "vibes.task_id";
    pub const TOKENS_INPUT: &str = "vibes.tokens.input";
    pub const TOKENS_OUTPUT: &str = "vibes.tokens.output";
    pub const TOOL_NAME: &str = "vibes.tool.name";
    pub const COST_CENTER: &str = "vibes.cost_center";
    pub const USER_ID: &str = "vibes.user_id";
}

/// Session identifier for tracing context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Agent identifier for tracing context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Swarm identifier for tracing context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwarmId(String);

impl SwarmId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for SwarmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User identifier for tracing context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Model identifier for tracing context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelId(String);

impl ModelId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trace context carrying vibes-specific attributes.
///
/// This struct enriches traces with session, agent, and model information.
#[derive(Debug, Clone, Default)]
pub struct TraceContext {
    pub session_id: Option<SessionId>,
    pub agent_id: Option<AgentId>,
    pub swarm_id: Option<SwarmId>,
    pub user_id: Option<UserId>,
    pub model: Option<ModelId>,
    pub cost_center: Option<String>,
}

impl TraceContext {
    /// Create a new empty trace context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context for a session.
    pub fn for_session(session_id: SessionId) -> Self {
        Self {
            session_id: Some(session_id),
            ..Default::default()
        }
    }

    /// Create context for an agent within a session.
    pub fn for_agent(session_id: SessionId, agent_id: AgentId) -> Self {
        Self {
            session_id: Some(session_id),
            agent_id: Some(agent_id),
            ..Default::default()
        }
    }

    /// Set the session ID.
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the agent ID.
    pub fn with_agent(mut self, agent_id: AgentId) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Set the swarm ID.
    pub fn with_swarm(mut self, swarm_id: SwarmId) -> Self {
        self.swarm_id = Some(swarm_id);
        self
    }

    /// Set the user ID.
    pub fn with_user(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the model ID.
    pub fn with_model(mut self, model: ModelId) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the cost center.
    pub fn with_cost_center(mut self, cost_center: impl Into<String>) -> Self {
        self.cost_center = Some(cost_center.into());
        self
    }

    /// Record this context on the current span.
    pub fn record_on_span(&self) {
        tracing::Span::current().record_vibes_context(self);
    }
}

/// Extension trait for `tracing::Span` to record vibes context.
pub trait VibesSpanExt {
    /// Record vibes-specific context attributes on this span.
    fn record_vibes_context(&self, ctx: &TraceContext);
}

impl VibesSpanExt for tracing::Span {
    fn record_vibes_context(&self, ctx: &TraceContext) {
        if let Some(session_id) = &ctx.session_id {
            self.record(attributes::SESSION_ID, session_id.to_string());
        }
        if let Some(agent_id) = &ctx.agent_id {
            self.record(attributes::AGENT_ID, agent_id.to_string());
        }
        if let Some(swarm_id) = &ctx.swarm_id {
            self.record(attributes::SWARM_ID, swarm_id.to_string());
        }
        if let Some(user_id) = &ctx.user_id {
            self.record(attributes::USER_ID, user_id.to_string());
        }
        if let Some(model) = &ctx.model {
            self.record(attributes::MODEL_ID, model.to_string());
        }
        if let Some(cost_center) = &ctx.cost_center {
            self.record(attributes::COST_CENTER, cost_center.as_str());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Attribute Constants Tests =====

    #[test]
    fn attribute_names_follow_vibes_namespace() {
        assert!(attributes::SESSION_ID.starts_with("vibes."));
        assert!(attributes::AGENT_ID.starts_with("vibes."));
        assert!(attributes::AGENT_TYPE.starts_with("vibes."));
        assert!(attributes::SWARM_ID.starts_with("vibes."));
        assert!(attributes::MODEL_ID.starts_with("vibes."));
        assert!(attributes::TASK_ID.starts_with("vibes."));
        assert!(attributes::TOKENS_INPUT.starts_with("vibes."));
        assert!(attributes::TOKENS_OUTPUT.starts_with("vibes."));
        assert!(attributes::TOOL_NAME.starts_with("vibes."));
        assert!(attributes::COST_CENTER.starts_with("vibes."));
        assert!(attributes::USER_ID.starts_with("vibes."));
    }

    // ===== TraceContext Tests =====

    #[test]
    fn trace_context_new_creates_empty_context() {
        let ctx = TraceContext::new();
        assert!(ctx.session_id.is_none());
        assert!(ctx.agent_id.is_none());
        assert!(ctx.swarm_id.is_none());
        assert!(ctx.user_id.is_none());
        assert!(ctx.model.is_none());
        assert!(ctx.cost_center.is_none());
    }

    #[test]
    fn trace_context_default_is_empty() {
        let ctx = TraceContext::default();
        assert!(ctx.session_id.is_none());
        assert!(ctx.agent_id.is_none());
    }

    #[test]
    fn trace_context_for_session_sets_session_id() {
        let session_id = SessionId::new("sess-123");
        let ctx = TraceContext::for_session(session_id.clone());
        assert_eq!(ctx.session_id, Some(session_id));
        assert!(ctx.agent_id.is_none());
    }

    #[test]
    fn trace_context_for_agent_sets_session_and_agent() {
        let session_id = SessionId::new("sess-123");
        let agent_id = AgentId::new("agent-456");
        let ctx = TraceContext::for_agent(session_id.clone(), agent_id.clone());
        assert_eq!(ctx.session_id, Some(session_id));
        assert_eq!(ctx.agent_id, Some(agent_id));
    }

    #[test]
    fn trace_context_builder_pattern() {
        let ctx = TraceContext::new()
            .with_session(SessionId::new("sess-1"))
            .with_agent(AgentId::new("agent-1"))
            .with_model(ModelId::new("claude-sonnet-4"))
            .with_cost_center("team-alpha");

        assert!(ctx.session_id.is_some());
        assert!(ctx.agent_id.is_some());
        assert!(ctx.model.is_some());
        assert_eq!(ctx.cost_center, Some("team-alpha".to_string()));
    }

    #[test]
    fn session_id_display() {
        let id = SessionId::new("my-session");
        assert_eq!(id.to_string(), "my-session");
    }

    #[test]
    fn agent_id_display() {
        let id = AgentId::new("my-agent");
        assert_eq!(id.to_string(), "my-agent");
    }

    // ===== VibesSpanExt Tests =====

    #[test]
    fn vibes_span_ext_records_context() {
        // Create a span with empty fields that will be recorded
        let span = tracing::info_span!(
            "test_span",
            "vibes.session_id" = tracing::field::Empty,
            "vibes.agent_id" = tracing::field::Empty,
            "vibes.model_id" = tracing::field::Empty,
            "vibes.cost_center" = tracing::field::Empty,
        );

        let ctx = TraceContext::new()
            .with_session(SessionId::new("sess-abc"))
            .with_agent(AgentId::new("agent-xyz"))
            .with_model(ModelId::new("claude-sonnet-4"))
            .with_cost_center("engineering");

        // This should compile and run without panic
        span.record_vibes_context(&ctx);
    }

    #[test]
    fn vibes_span_ext_handles_partial_context() {
        let span = tracing::info_span!("partial_span", "vibes.session_id" = tracing::field::Empty,);

        // Only session_id is set, others are None
        let ctx = TraceContext::for_session(SessionId::new("sess-only"));

        span.record_vibes_context(&ctx);
    }

    #[test]
    fn trace_context_record_on_span_records_current() {
        let span = tracing::info_span!("current_span", "vibes.session_id" = tracing::field::Empty,);
        let _guard = span.enter();

        let ctx = TraceContext::for_session(SessionId::new("sess-current"));
        ctx.record_on_span();
    }
}
