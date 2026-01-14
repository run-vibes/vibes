# Eval Storage Design

Event-sourced storage for longitudinal evaluation studies.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Eval API      │────▶│  Iggy Stream    │────▶│ Turso Projection│
│  (commands)     │     │   "evals"       │     │  (read model)   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                       │                       │
   create_study()          EvalEvent              studies table
   start_study()           append-only            checkpoints table
   record_checkpoint()     source of truth        fast queries
```

### Flow

1. Commands (create study, record checkpoint) append events to Iggy "evals" stream
2. Consumer processes events and updates Turso projection
3. Queries read from Turso for fast access
4. Projection can be rebuilt by replaying all events

### Key Properties

- **Events are immutable, append-only** - source of truth in Iggy
- **Turso projection is derived state** - can be rebuilt from events
- **Separate stream from session events** - independent scaling
- **Partitioned by study_id** - ordered processing per study

## Event Types

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

## Storage Trait

```rust
/// Read-only queries against the projection
#[async_trait]
pub trait EvalStorage: Send + Sync {
    // Studies
    async fn get_study(&self, id: StudyId) -> Result<Option<Study>>;
    async fn list_studies(&self) -> Result<Vec<Study>>;
    async fn list_studies_by_status(&self, status: StudyStatus) -> Result<Vec<Study>>;

    // Checkpoints
    async fn get_checkpoints(&self, study_id: StudyId) -> Result<Vec<Checkpoint>>;
    async fn get_latest_checkpoint(&self, study_id: StudyId) -> Result<Option<Checkpoint>>;
    async fn get_checkpoints_in_range(
        &self,
        study_id: StudyId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Checkpoint>>;
}

/// Write side - processes events to update projection
#[async_trait]
pub trait EvalProjection: Send + Sync {
    async fn apply(&self, event: &StoredEvalEvent) -> Result<()>;
    async fn rebuild(&self) -> Result<()>;
}
```

## Turso Schema

```sql
CREATE TABLE studies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL,  -- pending/running/paused/stopped
    period_type TEXT NOT NULL,
    period_value INTEGER,
    config TEXT NOT NULL,  -- JSON
    created_at TEXT NOT NULL,
    started_at TEXT,
    stopped_at TEXT
);

CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY,
    study_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    metrics TEXT NOT NULL,  -- JSON
    events_analyzed INTEGER NOT NULL
);

CREATE INDEX idx_checkpoints_study_time ON checkpoints(study_id, timestamp);
```

## Consumer

```rust
pub struct EvalProjectionConsumer {
    event_log: Arc<dyn EventLog<StoredEvalEvent>>,
    projection: Arc<dyn EvalProjection>,
}

impl EvalProjectionConsumer {
    pub async fn run(&self) -> Result<()> {
        let consumer = self.event_log.consumer("eval-projection").await?;

        loop {
            let batch = consumer.poll(100, Duration::from_secs(1)).await?;
            for (offset, event) in batch.events {
                self.projection.apply(&event).await?;
                consumer.commit(offset).await?;
            }
        }
    }

    pub async fn rebuild(&self) -> Result<()> {
        self.projection.clear().await?;
        let consumer = self.event_log.consumer("eval-rebuild").await?;
        consumer.seek(SeekPosition::Beginning).await?;
        // replay all events
    }
}
```

## Command Side

```rust
pub async fn create_study(
    event_log: &dyn EventLog<StoredEvalEvent>,
    cmd: CreateStudy,
) -> Result<StudyId> {
    let id = StudyId::new();
    let event = StoredEvalEvent::new(EvalEvent::StudyCreated {
        id,
        name: cmd.name,
        period_type: cmd.period_type,
        period_value: cmd.period_value,
        config: cmd.config,
    });
    event_log.append(event).await?;
    Ok(id)
}
```

## Implementation Plan

1. Define `EvalEvent` enum and `StoredEvalEvent` wrapper
2. Add libsql/Turso dependency
3. Implement `TursoEvalStorage` (read side)
4. Implement `TursoEvalProjection` (write side)
5. Create `EvalProjectionConsumer`
6. Add command functions for study lifecycle
7. Write integration tests

## Dependencies

- `libsql` - Turso/libSQL client
- `vibes-iggy` - Event log traits
- Existing types from `vibes-evals` (LongitudinalMetrics, StudyId, etc.)
