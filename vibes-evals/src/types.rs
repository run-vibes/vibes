//! Core type definitions for the evals framework.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an evaluation study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StudyId(pub Uuid);

/// Unique identifier for a benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BenchmarkId(pub Uuid);

/// Unique identifier for a checkpoint within a study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(pub Uuid);
