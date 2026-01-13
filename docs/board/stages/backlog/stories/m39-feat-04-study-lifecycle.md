---
id: m39-feat-04
title: Study lifecycle management
type: feat
status: backlog
priority: high
epics: [evals]
depends: [m39-feat-03]
estimate: 4h
milestone: 39-eval-core
---

# Study lifecycle management

## Summary

Implement the study lifecycle: create, checkpoint, and stop longitudinal studies. Studies track performance metrics over extended periods.

## Features

### LongitudinalStudy

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongitudinalStudy {
    pub id: StudyId,
    pub name: String,
    pub started: DateTime<Utc>,
    pub stopped: Option<DateTime<Utc>>,
    pub period: StudyPeriod,
    pub metrics: Vec<MetricDefinition>,
    pub status: StudyStatus,
    pub config: StudyConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StudyPeriod {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Indefinite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StudyStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyConfig {
    pub checkpoint_interval: Duration,
    pub include_sessions: Option<Vec<SessionId>>,
    pub exclude_sessions: Option<Vec<SessionId>>,
    pub groove_enabled: bool,
}
```

### Checkpoint

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub study_id: StudyId,
    pub timestamp: DateTime<Utc>,
    pub metrics: LongitudinalMetrics,
    pub events_analyzed: u64,
    pub sessions_included: Vec<SessionId>,
}
```

### StudyManager

```rust
pub struct StudyManager {
    storage: Box<dyn EvalStorage>,
}

impl StudyManager {
    pub fn new(storage: Box<dyn EvalStorage>) -> Self;

    /// Create a new longitudinal study
    pub async fn create_study(
        &self,
        name: String,
        period: StudyPeriod,
        config: StudyConfig,
    ) -> Result<LongitudinalStudy>;

    /// Record a checkpoint with current metrics
    pub async fn checkpoint(&self, study_id: StudyId) -> Result<Checkpoint>;

    /// Stop a running study
    pub async fn stop_study(&self, study_id: StudyId) -> Result<()>;

    /// Get study status with latest metrics
    pub async fn get_status(&self, study_id: StudyId) -> Result<StudyStatus>;

    /// List all studies
    pub async fn list_studies(&self) -> Result<Vec<LongitudinalStudy>>;
}
```

## Implementation

1. Create `vibes-evals/src/study.rs`
2. Define study-related types
3. Implement `StudyManager`
4. Add automatic checkpoint scheduling (background task)
5. Write unit tests for lifecycle transitions
6. Write integration tests

## Acceptance Criteria

- [ ] `LongitudinalStudy` type with all fields
- [ ] `StudyManager` creates and manages studies
- [ ] Checkpoints capture metrics at intervals
- [ ] Stop correctly finalizes studies
- [ ] Study status reflects current state
- [ ] Tests cover lifecycle transitions
