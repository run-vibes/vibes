---
id: FEAT0164
title: TraceContext with vibes-specific attributes
type: feat
status: done
priority: high
scope: observability/01-distributed-tracing
depends: [m40-feat-02]
estimate: 3h
---

# TraceContext with vibes-specific attributes

## Summary

Define `TraceContext` to carry vibes-specific context through spans. This enriches traces with session, agent, and model information.

## Features

### TraceContext

```rust
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
    pub fn new() -> Self;

    /// Create context for a session
    pub fn for_session(session_id: SessionId) -> Self;

    /// Create context for an agent within a session
    pub fn for_agent(session_id: SessionId, agent_id: AgentId) -> Self;

    /// Add to current span as attributes
    pub fn record_on_span(&self);
}
```

### Context Propagation

```rust
/// Extension trait for Span to record vibes context
pub trait VibesSpanExt {
    fn record_vibes_context(&self, ctx: &TraceContext);
}

impl VibesSpanExt for tracing::Span {
    fn record_vibes_context(&self, ctx: &TraceContext) {
        if let Some(session_id) = &ctx.session_id {
            self.record("vibes.session_id", session_id.to_string());
        }
        if let Some(agent_id) = &ctx.agent_id {
            self.record("vibes.agent_id", agent_id.to_string());
        }
        // ... etc
    }
}
```

### Standard Attribute Names

```rust
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
}
```

## Implementation

1. Create `vibes-observe/src/context.rs`
2. Define `TraceContext` struct
3. Implement `VibesSpanExt` trait
4. Define standard attribute constants
5. Write tests for context recording

## Acceptance Criteria

- [x] `TraceContext` carries vibes-specific data
- [x] `VibesSpanExt` records context on spans
- [x] Standard attribute names defined
- [x] Context can be created for session/agent
- [x] Attributes appear in exported spans
