//! Task system types
//!
//! Defines tasks (work units) and results for agent execution.

use crate::agent::types::{TaskId, ToolId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// A task for an agent to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: TaskId,
    /// Human-readable task description
    pub description: String,
    /// Additional context for the task
    pub context: TaskContext,
    /// Execution constraints
    pub constraints: TaskConstraints,
    /// Parent task if this is a subtask
    pub parent: Option<TaskId>,
}

/// Additional context for task execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskContext {
    /// System prompt override
    pub system_prompt: Option<String>,
    /// Files relevant to the task
    pub files: Vec<PathBuf>,
    /// Key-value metadata
    pub metadata: HashMap<String, Value>,
}

/// Constraints on task execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskConstraints {
    /// Maximum iterations allowed
    pub max_iterations: Option<u32>,
    /// Maximum tokens to consume
    pub max_tokens: Option<u64>,
    /// Maximum execution time
    #[serde(with = "option_duration_serde")]
    pub timeout: Option<Duration>,
    /// Restrict to specific tools
    pub allowed_tools: Option<Vec<ToolId>>,
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// The task that was executed
    pub task_id: TaskId,
    /// Completion status
    pub status: TaskStatus,
    /// Output value (if any)
    pub output: Option<Value>,
    /// Artifacts produced
    pub artifacts: Vec<Artifact>,
    /// Execution metrics
    pub metrics: TaskMetrics,
}

/// Task completion status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task completed successfully
    Completed,
    /// Task failed with error
    Failed { error: String },
    /// Task was cancelled
    Cancelled,
    /// Task exceeded time limit
    TimedOut,
}

/// Execution metrics for a task
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Total execution duration
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    /// Tokens consumed
    pub tokens_used: u64,
    /// Number of tool invocations
    pub tool_calls: u32,
    /// Agent loop iterations
    pub iterations: u32,
}

/// An artifact produced by task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact name
    pub name: String,
    /// Type of artifact
    pub artifact_type: ArtifactType,
    /// File path (if persisted)
    pub path: Option<PathBuf>,
    /// Inline content (if small)
    pub content: Option<String>,
}

/// Types of artifacts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtifactType {
    /// A file
    File,
    /// A code diff
    Diff,
    /// Log output
    Log,
    /// Other artifact type
    Other(String),
}

/// Builder for creating Task instances
#[derive(Debug, Clone, Default)]
pub struct TaskBuilder {
    description: Option<String>,
    context: TaskContext,
    constraints: TaskConstraints,
    parent: Option<TaskId>,
}

impl TaskBuilder {
    /// Create a new task builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the task description (required)
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set a system prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.context.system_prompt = Some(prompt.into());
        self
    }

    /// Add a file to the context
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.context.files.push(path.into());
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.context.metadata.insert(key.into(), value);
        self
    }

    /// Set maximum iterations
    pub fn max_iterations(mut self, n: u32) -> Self {
        self.constraints.max_iterations = Some(n);
        self
    }

    /// Set maximum tokens
    pub fn max_tokens(mut self, n: u64) -> Self {
        self.constraints.max_tokens = Some(n);
        self
    }

    /// Set timeout
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.constraints.timeout = Some(duration);
        self
    }

    /// Restrict to specific tools
    pub fn allowed_tools(mut self, tools: Vec<ToolId>) -> Self {
        self.constraints.allowed_tools = Some(tools);
        self
    }

    /// Set parent task
    pub fn parent(mut self, parent: TaskId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Build the task
    ///
    /// # Panics
    /// Panics if description is not set.
    pub fn build(self) -> Task {
        Task {
            id: TaskId::new(),
            description: self.description.expect("description is required"),
            context: self.context,
            constraints: self.constraints,
            parent: self.parent,
        }
    }
}

impl Task {
    /// Create a new task builder
    pub fn builder() -> TaskBuilder {
        TaskBuilder::new()
    }

    /// Create a simple task with just a description
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: TaskId::new(),
            description: description.into(),
            context: TaskContext::default(),
            constraints: TaskConstraints::default(),
            parent: None,
        }
    }
}

/// Serde helper for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    #[derive(Serialize, Deserialize)]
    struct DurationRepr {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr = DurationRepr {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        };
        repr.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr: DurationRepr = DurationRepr::deserialize(deserializer)?;
        Ok(Duration::new(repr.secs, repr.nanos))
    }
}

/// Serde helper for Option<Duration>
mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    #[derive(Serialize, Deserialize)]
    struct DurationRepr {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => {
                let repr = DurationRepr {
                    secs: d.as_secs(),
                    nanos: d.subsec_nanos(),
                };
                Some(repr).serialize(serializer)
            }
            None => None::<DurationRepr>.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<DurationRepr> = Option::deserialize(deserializer)?;
        Ok(opt.map(|repr| Duration::new(repr.secs, repr.nanos)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Task and TaskContext Tests =====

    #[test]
    fn task_new_creates_with_defaults() {
        let task = Task::new("Test task");

        assert_eq!(task.description, "Test task");
        assert!(task.context.system_prompt.is_none());
        assert!(task.context.files.is_empty());
        assert!(task.context.metadata.is_empty());
        assert!(task.constraints.max_iterations.is_none());
        assert!(task.parent.is_none());
    }

    #[test]
    fn task_context_default() {
        let ctx = TaskContext::default();

        assert!(ctx.system_prompt.is_none());
        assert!(ctx.files.is_empty());
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn task_constraints_default() {
        let constraints = TaskConstraints::default();

        assert!(constraints.max_iterations.is_none());
        assert!(constraints.max_tokens.is_none());
        assert!(constraints.timeout.is_none());
        assert!(constraints.allowed_tools.is_none());
    }

    // ===== TaskResult Tests =====

    #[test]
    fn task_result_completed() {
        let task_id = TaskId::new();
        let result = TaskResult {
            task_id,
            status: TaskStatus::Completed,
            output: Some(Value::String("done".to_string())),
            artifacts: vec![],
            metrics: TaskMetrics::default(),
        };

        assert_eq!(result.status, TaskStatus::Completed);
    }

    #[test]
    fn task_result_failed_has_error() {
        let task_id = TaskId::new();
        let result = TaskResult {
            task_id,
            status: TaskStatus::Failed {
                error: "oops".to_string(),
            },
            output: None,
            artifacts: vec![],
            metrics: TaskMetrics::default(),
        };

        match result.status {
            TaskStatus::Failed { error } => assert_eq!(error, "oops"),
            _ => panic!("expected Failed"),
        }
    }

    #[test]
    fn task_status_variants() {
        let statuses = [
            TaskStatus::Completed,
            TaskStatus::Failed {
                error: "err".to_string(),
            },
            TaskStatus::Cancelled,
            TaskStatus::TimedOut,
        ];

        // Each variant should be distinct
        assert_ne!(statuses[0], statuses[1]);
        assert_ne!(statuses[2], statuses[3]);
    }

    // ===== TaskMetrics Tests =====

    #[test]
    fn task_metrics_default() {
        let metrics = TaskMetrics::default();

        assert_eq!(metrics.duration, Duration::ZERO);
        assert_eq!(metrics.tokens_used, 0);
        assert_eq!(metrics.tool_calls, 0);
        assert_eq!(metrics.iterations, 0);
    }

    #[test]
    fn task_metrics_tracks_values() {
        let metrics = TaskMetrics {
            duration: Duration::from_secs(10),
            tokens_used: 5000,
            tool_calls: 15,
            iterations: 3,
        };

        assert_eq!(metrics.duration.as_secs(), 10);
        assert_eq!(metrics.tokens_used, 5000);
        assert_eq!(metrics.tool_calls, 15);
        assert_eq!(metrics.iterations, 3);
    }

    // ===== Artifact Tests =====

    #[test]
    fn artifact_types() {
        let types = [
            ArtifactType::File,
            ArtifactType::Diff,
            ArtifactType::Log,
            ArtifactType::Other("custom".to_string()),
        ];

        assert_eq!(types[0], ArtifactType::File);
        assert_eq!(types[3], ArtifactType::Other("custom".to_string()));
    }

    #[test]
    fn artifact_with_path() {
        let artifact = Artifact {
            name: "output.txt".to_string(),
            artifact_type: ArtifactType::File,
            path: Some(PathBuf::from("/tmp/output.txt")),
            content: None,
        };

        assert_eq!(artifact.name, "output.txt");
        assert!(artifact.path.is_some());
    }

    #[test]
    fn artifact_with_content() {
        let artifact = Artifact {
            name: "patch".to_string(),
            artifact_type: ArtifactType::Diff,
            path: None,
            content: Some("--- a\n+++ b".to_string()),
        };

        assert!(artifact.content.is_some());
    }

    // ===== Builder Tests =====

    #[test]
    fn task_builder_minimal() {
        let task = Task::builder().description("Do something").build();

        assert_eq!(task.description, "Do something");
    }

    #[test]
    fn task_builder_with_context() {
        let task = Task::builder()
            .description("Analyze code")
            .system_prompt("You are a code reviewer")
            .file("/path/to/file.rs")
            .metadata("priority", Value::String("high".to_string()))
            .build();

        assert_eq!(
            task.context.system_prompt,
            Some("You are a code reviewer".to_string())
        );
        assert_eq!(task.context.files.len(), 1);
        assert!(task.context.metadata.contains_key("priority"));
    }

    #[test]
    fn task_builder_with_constraints() {
        let task = Task::builder()
            .description("Quick task")
            .max_iterations(5)
            .max_tokens(1000)
            .timeout(Duration::from_secs(60))
            .build();

        assert_eq!(task.constraints.max_iterations, Some(5));
        assert_eq!(task.constraints.max_tokens, Some(1000));
        assert_eq!(task.constraints.timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn task_builder_with_parent() {
        let parent_id = TaskId::new();
        let task = Task::builder()
            .description("Subtask")
            .parent(parent_id)
            .build();

        assert_eq!(task.parent, Some(parent_id));
    }

    #[test]
    fn task_builder_with_allowed_tools() {
        let tools = vec![ToolId("bash".to_string()), ToolId("read".to_string())];
        let task = Task::builder()
            .description("Limited task")
            .allowed_tools(tools.clone())
            .build();

        assert_eq!(task.constraints.allowed_tools, Some(tools));
    }

    #[test]
    #[should_panic(expected = "description is required")]
    fn task_builder_requires_description() {
        Task::builder().build();
    }

    // ===== Serialization Tests =====

    #[test]
    fn task_serialization_roundtrip() {
        let task = Task::builder()
            .description("Test")
            .system_prompt("prompt")
            .max_iterations(10)
            .build();

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.description, task.description);
        assert_eq!(
            deserialized.context.system_prompt,
            task.context.system_prompt
        );
        assert_eq!(
            deserialized.constraints.max_iterations,
            task.constraints.max_iterations
        );
    }

    #[test]
    fn task_context_serialization_roundtrip() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), Value::Bool(true));

        let ctx = TaskContext {
            system_prompt: Some("test".to_string()),
            files: vec![PathBuf::from("/test")],
            metadata,
        };

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: TaskContext = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.system_prompt, ctx.system_prompt);
        assert_eq!(deserialized.files, ctx.files);
    }

    #[test]
    fn task_constraints_serialization_roundtrip() {
        let constraints = TaskConstraints {
            max_iterations: Some(5),
            max_tokens: Some(1000),
            timeout: Some(Duration::from_secs(30)),
            allowed_tools: Some(vec![ToolId("bash".to_string())]),
        };

        let json = serde_json::to_string(&constraints).unwrap();
        let deserialized: TaskConstraints = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_iterations, constraints.max_iterations);
        assert_eq!(deserialized.max_tokens, constraints.max_tokens);
        assert_eq!(deserialized.timeout, constraints.timeout);
    }

    #[test]
    fn task_result_serialization_roundtrip() {
        let result = TaskResult {
            task_id: TaskId::new(),
            status: TaskStatus::Completed,
            output: Some(Value::String("result".to_string())),
            artifacts: vec![Artifact {
                name: "test".to_string(),
                artifact_type: ArtifactType::Log,
                path: None,
                content: Some("log content".to_string()),
            }],
            metrics: TaskMetrics {
                duration: Duration::from_millis(500),
                tokens_used: 100,
                tool_calls: 5,
                iterations: 2,
            },
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TaskResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.status, result.status);
        assert_eq!(deserialized.metrics, result.metrics);
    }

    #[test]
    fn task_status_serialization_roundtrip() {
        let statuses = [
            TaskStatus::Completed,
            TaskStatus::Failed {
                error: "test error".to_string(),
            },
            TaskStatus::Cancelled,
            TaskStatus::TimedOut,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn task_metrics_serialization_roundtrip() {
        let metrics = TaskMetrics {
            duration: Duration::from_secs(42),
            tokens_used: 12345,
            tool_calls: 7,
            iterations: 3,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: TaskMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, metrics);
    }

    #[test]
    fn artifact_serialization_roundtrip() {
        let artifact = Artifact {
            name: "output.diff".to_string(),
            artifact_type: ArtifactType::Diff,
            path: Some(PathBuf::from("/tmp/output.diff")),
            content: Some("diff content".to_string()),
        };

        let json = serde_json::to_string(&artifact).unwrap();
        let deserialized: Artifact = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, artifact.name);
        assert_eq!(deserialized.artifact_type, artifact.artifact_type);
    }

    #[test]
    fn artifact_type_serialization_roundtrip() {
        let types = [
            ArtifactType::File,
            ArtifactType::Diff,
            ArtifactType::Log,
            ArtifactType::Other("custom".to_string()),
        ];

        for art_type in types {
            let json = serde_json::to_string(&art_type).unwrap();
            let deserialized: ArtifactType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, art_type);
        }
    }
}
