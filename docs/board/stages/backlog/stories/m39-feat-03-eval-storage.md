---
id: m39-feat-03
title: EvalStorage schema and implementation
type: feat
status: backlog
priority: high
epics: [evals]
depends: [m39-feat-02]
estimate: 4h
milestone: 39-eval-core
---

# EvalStorage schema and implementation

## Summary

Implement the storage layer for evaluation data. This includes tables for benchmark results, longitudinal studies, and checkpoints using SQLite.

## Features

### EvalStorage Trait

```rust
#[async_trait]
pub trait EvalStorage: Send + Sync {
    // Studies
    async fn create_study(&self, study: &LongitudinalStudy) -> Result<()>;
    async fn get_study(&self, id: StudyId) -> Result<Option<LongitudinalStudy>>;
    async fn list_studies(&self) -> Result<Vec<LongitudinalStudy>>;
    async fn update_study(&self, study: &LongitudinalStudy) -> Result<()>;

    // Checkpoints
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
    async fn get_checkpoints(&self, study_id: StudyId) -> Result<Vec<Checkpoint>>;
    async fn get_latest_checkpoint(&self, study_id: StudyId) -> Result<Option<Checkpoint>>;

    // Benchmarks (placeholder for future)
    async fn save_benchmark_result(&self, result: &BenchmarkResult) -> Result<()>;
    async fn get_benchmark_results(&self, benchmark_id: BenchmarkId) -> Result<Vec<BenchmarkResult>>;
}
```

### SQLite Implementation

```rust
pub struct SqliteEvalStorage {
    pool: SqlitePool,
}

impl SqliteEvalStorage {
    pub async fn new(path: &Path) -> Result<Self>;
    pub async fn migrate(&self) -> Result<()>;
}
```

### Database Schema

```sql
CREATE TABLE studies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    started_at TEXT NOT NULL,
    stopped_at TEXT,
    period_type TEXT NOT NULL,
    period_value INTEGER,
    status TEXT NOT NULL,
    config TEXT NOT NULL
);

CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY,
    study_id TEXT NOT NULL REFERENCES studies(id),
    timestamp TEXT NOT NULL,
    metrics TEXT NOT NULL,
    events_analyzed INTEGER NOT NULL,
    sessions_included TEXT NOT NULL
);

CREATE TABLE benchmark_results (
    id TEXT PRIMARY KEY,
    benchmark_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    scores TEXT NOT NULL,
    passed INTEGER NOT NULL,
    failed INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    config TEXT NOT NULL
);

CREATE INDEX idx_checkpoints_study ON checkpoints(study_id);
CREATE INDEX idx_checkpoints_timestamp ON checkpoints(timestamp);
```

## Implementation

1. Create `vibes-evals/src/storage.rs`
2. Define `EvalStorage` trait
3. Add `sqlx` dependency with SQLite feature
4. Implement `SqliteEvalStorage`
5. Write migration files
6. Write integration tests

## Acceptance Criteria

- [ ] `EvalStorage` trait defined
- [ ] SQLite implementation works
- [ ] Migrations run on startup
- [ ] CRUD operations for studies work
- [ ] Checkpoint storage and retrieval work
- [ ] Integration tests pass
