---
id: m39-feat-03
title: EvalStorage schema and implementation
type: feat
status: done
priority: high
epics: [evals]
depends: [m39-feat-02]
estimate: 4h
milestone: 39-eval-core
---

# EvalStorage schema and implementation

## Summary

Implement the event-sourced storage layer for evaluation data. Events are stored in a separate Iggy stream, with a Turso projection for fast queries.

See [milestone design](../../milestones/39-eval-core/design.md) for architecture.

## Features

### Event Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvalEvent {
    // Study lifecycle
    StudyCreated {
        id: StudyId,
        name: String,
        period_type: PeriodType,
        period_value: Option<u32>,
        config: StudyConfig,
    },
    StudyStarted { id: StudyId },
    StudyPaused { id: StudyId },
    StudyResumed { id: StudyId },
    StudyStopped { id: StudyId },

    // Data capture
    CheckpointRecorded {
        id: CheckpointId,
        study_id: StudyId,
        timestamp: DateTime<Utc>,
        metrics: LongitudinalMetrics,
        events_analyzed: u64,
        sessions_included: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvalEvent {
    pub event_id: Uuid,  // UUIDv7 for ordering
    pub event: EvalEvent,
}
```

### EvalStorage Trait (Read-Only Projection)

```rust
#[async_trait]
pub trait EvalStorage: Send + Sync {
    // Studies (read from projection)
    async fn get_study(&self, id: StudyId) -> Result<Option<Study>>;
    async fn list_studies(&self) -> Result<Vec<Study>>;
    async fn list_studies_by_status(&self, status: StudyStatus) -> Result<Vec<Study>>;

    // Checkpoints (read from projection)
    async fn get_checkpoints(&self, study_id: StudyId) -> Result<Vec<Checkpoint>>;
    async fn get_latest_checkpoint(&self, study_id: StudyId) -> Result<Option<Checkpoint>>;
    async fn get_checkpoints_in_range(
        &self,
        study_id: StudyId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Checkpoint>>;
}
```

### EvalProjection Trait (Event Consumer)

```rust
#[async_trait]
pub trait EvalProjection: Send + Sync {
    async fn apply(&self, event: &StoredEvalEvent) -> Result<()>;
    async fn rebuild(&self) -> Result<()>;
}
```

### Turso Implementation

```rust
pub struct TursoEvalStorage {
    db: Database,
}

impl TursoEvalStorage {
    pub async fn new(url: &str, token: &str) -> Result<Self>;
    pub async fn new_local(path: &Path) -> Result<Self>;  // Embedded replica
}
```

### Database Schema

```sql
CREATE TABLE studies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    period_type TEXT NOT NULL,
    period_value INTEGER,
    config TEXT NOT NULL,
    created_at TEXT NOT NULL,
    started_at TEXT,
    stopped_at TEXT
);

CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY,
    study_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    metrics TEXT NOT NULL,
    events_analyzed INTEGER NOT NULL
);

CREATE INDEX idx_checkpoints_study_time ON checkpoints(study_id, timestamp);
```

## Implementation

1. Define `EvalEvent` enum in `vibes-evals/src/events.rs`
2. Define `StoredEvalEvent` wrapper with UUIDv7
3. Add `libsql` dependency for Turso
4. Implement `TursoEvalStorage` (read queries)
5. Implement `TursoEvalProjection` (apply events)
6. Write integration tests with in-memory Turso

## Acceptance Criteria

- [ ] `EvalEvent` enum covers all study lifecycle events
- [ ] `StoredEvalEvent` uses UUIDv7 for ordering
- [ ] `EvalStorage` trait provides read-only queries
- [ ] `EvalProjection` trait applies events to projection
- [ ] Turso implementation works (local embedded mode)
- [ ] Projection can be rebuilt from events
- [ ] Integration tests pass
