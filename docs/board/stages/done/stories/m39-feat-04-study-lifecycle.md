---
id: m39-feat-04
title: Study lifecycle management
type: feat
status: done
priority: high
epics: [evals]
depends: [m39-feat-03]
estimate: 4h
milestone: 39-eval-core
---

# Study lifecycle management

## Summary

Implement the study lifecycle using CQRS: commands emit events to Iggy, queries read from the Turso projection.

See [milestone design](../../milestones/39-eval-core/design.md) for architecture.

## Features

### Study Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Study {
    pub id: StudyId,
    pub name: String,
    pub status: StudyStatus,
    pub period_type: PeriodType,
    pub period_value: Option<u32>,
    pub config: StudyConfig,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StudyStatus {
    Pending,
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PeriodType {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyConfig {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
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
    pub sessions_included: Vec<String>,
}
```

### StudyManager (CQRS)

```rust
pub struct StudyManager {
    event_log: Arc<dyn EventLog<StoredEvalEvent>>,  // Commands → Events
    storage: Arc<dyn EvalStorage>,                   // Queries → Projection
}

impl StudyManager {
    pub fn new(
        event_log: Arc<dyn EventLog<StoredEvalEvent>>,
        storage: Arc<dyn EvalStorage>,
    ) -> Self;

    // Commands (emit events)
    pub async fn create_study(&self, cmd: CreateStudy) -> Result<StudyId>;
    pub async fn start_study(&self, id: StudyId) -> Result<()>;
    pub async fn pause_study(&self, id: StudyId) -> Result<()>;
    pub async fn resume_study(&self, id: StudyId) -> Result<()>;
    pub async fn stop_study(&self, id: StudyId) -> Result<()>;
    pub async fn record_checkpoint(&self, cmd: RecordCheckpoint) -> Result<CheckpointId>;

    // Queries (read from projection)
    pub async fn get_study(&self, id: StudyId) -> Result<Option<Study>>;
    pub async fn list_studies(&self) -> Result<Vec<Study>>;
    pub async fn get_checkpoints(&self, study_id: StudyId) -> Result<Vec<Checkpoint>>;
    pub async fn get_latest_checkpoint(&self, study_id: StudyId) -> Result<Option<Checkpoint>>;
}
```

### Command Types

```rust
pub struct CreateStudy {
    pub name: String,
    pub period_type: PeriodType,
    pub period_value: Option<u32>,
    pub config: StudyConfig,
}

pub struct RecordCheckpoint {
    pub study_id: StudyId,
    pub metrics: LongitudinalMetrics,
    pub events_analyzed: u64,
    pub sessions_included: Vec<String>,
}
```

### Projection Consumer

```rust
pub struct EvalProjectionConsumer {
    event_log: Arc<dyn EventLog<StoredEvalEvent>>,
    projection: Arc<dyn EvalProjection>,
}

impl EvalProjectionConsumer {
    pub async fn run(&self) -> Result<()>;      // Main loop
    pub async fn rebuild(&self) -> Result<()>;  // Replay all events
}
```

## Implementation

1. Define study types in `vibes-evals/src/study.rs`
2. Define command types in `vibes-evals/src/commands.rs`
3. Implement `StudyManager` with CQRS pattern
4. Implement `EvalProjectionConsumer`
5. Add consumer to server startup
6. Write unit tests for lifecycle transitions
7. Write integration tests

## Acceptance Criteria

- [x] `StudyManager` emits events for all commands
- [x] `StudyManager` queries projection for reads
- [x] `EvalProjectionConsumer` processes events
- [x] Lifecycle transitions emit correct events
- [x] Queries return correct projected state
- [x] Consumer can rebuild projection from scratch
- [x] Tests cover lifecycle transitions
