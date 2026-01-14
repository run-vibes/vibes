//! Core type definitions for the evals framework.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an evaluation study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StudyId(pub Uuid);

impl StudyId {
    /// Create a new study ID with a UUIDv7 (time-ordered).
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for StudyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BenchmarkId(pub Uuid);

/// Unique identifier for a checkpoint within a study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(pub Uuid);

impl CheckpointId {
    /// Create a new checkpoint ID with a UUIDv7 (time-ordered).
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for CheckpointId {
    fn default() -> Self {
        Self::new()
    }
}
