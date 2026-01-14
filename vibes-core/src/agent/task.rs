//! Task system types
//!
//! Stub types for the Agent trait - will be expanded in m38-feat-04.

use serde::{Deserialize, Serialize};

/// A task for an agent to execute
///
/// Stub type - will be expanded in m38-feat-04.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Human-readable task description
    pub description: String,
}

/// Result of task execution
///
/// Stub type - will be expanded in m38-feat-04.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Whether the task completed successfully
    pub success: bool,
    /// Optional output message
    pub output: Option<String>,
}
