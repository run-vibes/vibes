---
id: FEAT0154
title: Task and TaskResult types
type: feat
status: done
priority: high
scope: agents/38-autonomous-agents
depends: [m38-feat-01]
estimate: 3h
---

# Task and TaskResult types

## Summary

Define the task system types that represent work units for agents. Tasks are the primary way to request work from agents, and TaskResults capture outcomes with metrics.

## Features

### TaskId

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
```

### Task

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub context: TaskContext,
    pub constraints: TaskConstraints,
    /// Parent task if this is a subtask
    pub parent: Option<TaskId>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskContext {
    /// Additional context for the task
    pub system_prompt: Option<String>,
    /// Files relevant to the task
    pub files: Vec<PathBuf>,
    /// Key-value metadata
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskConstraints {
    pub max_iterations: Option<u32>,
    pub max_tokens: Option<u64>,
    pub timeout: Option<Duration>,
    pub allowed_tools: Option<Vec<ToolId>>,
}
```

### TaskResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub output: Option<Value>,
    pub artifacts: Vec<Artifact>,
    pub metrics: TaskMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Completed,
    Failed { error: String },
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub duration: Duration,
    pub tokens_used: u64,
    pub tool_calls: u32,
    pub iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub artifact_type: ArtifactType,
    pub path: Option<PathBuf>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtifactType {
    File,
    Diff,
    Log,
    Other(String),
}
```

## Implementation

1. Create `vibes-core/src/agent/task.rs`
2. Define all task-related types
3. Implement `Default` for context and constraints
4. Implement builder pattern for `Task`
5. Write comprehensive serialization tests

## Acceptance Criteria

- [ ] `Task` and `TaskId` types defined
- [ ] `TaskResult` with status, output, artifacts, metrics
- [ ] `TaskMetrics` capturing duration, tokens, tool calls
- [ ] Builder pattern for creating tasks
- [ ] All types serialize/deserialize correctly
