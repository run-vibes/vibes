---
id: m38-feat-03
title: AgentStatus and AgentContext types
type: feat
status: done
priority: high
epics: [agents]
depends: [m38-feat-02]
estimate: 2h
milestone: 38-agent-core
---

# AgentStatus and AgentContext types

## Summary

Define the status tracking and execution context types for agents. `AgentStatus` tracks lifecycle state, `AgentContext` configures execution parameters.

## Features

### AgentStatus Enum

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Ready to accept tasks
    Idle,
    /// Actively executing a task
    Running {
        task: TaskId,
        started: DateTime<Utc>,
    },
    /// Execution paused
    Paused {
        task: TaskId,
        reason: String,
    },
    /// Waiting for user input
    WaitingForInput {
        prompt: String,
    },
    /// Task completed successfully
    Completed {
        result: TaskResult,
    },
    /// Task failed
    Failed {
        error: String,
    },
}
```

### AgentContext

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Where the agent executes
    pub location: ExecutionLocation,
    /// Model to use for inference
    pub model: ModelId,
    /// Available tools
    pub tools: Vec<ToolId>,
    /// Permission boundaries
    pub permissions: Permissions,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionLocation {
    /// Same machine as vibes server
    Local,
    /// Remote vibes instance
    Remote { endpoint: Url },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_tokens: Option<u64>,
    pub max_duration: Option<Duration>,
    pub max_tool_calls: Option<u32>,
}
```

## Implementation

1. Define `AgentStatus` in `types.rs`
2. Define `AgentContext` and supporting types
3. Implement `Default` for `AgentContext`
4. Write serialization round-trip tests
5. Export from module

## Acceptance Criteria

- [x] `AgentStatus` enum with all variants
- [x] `AgentContext` struct with location, model, tools, limits
- [x] `ExecutionLocation` enum (Local, Remote)
- [x] `ResourceLimits` with sensible defaults
- [x] Serialization tests pass
