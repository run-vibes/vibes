---
id: m39-feat-07
title: Wire StudyManager into server startup
type: feat
status: done
priority: high
epics: [evals]
depends: [m39-feat-04]
estimate: 2h
milestone: 39-eval-core
---

# Wire StudyManager into server startup

## Summary

The `StudyManager` from vibes-evals is never instantiated in the server. All `AppState` constructors set `study_manager: None`, causing "Eval studies not enabled" errors when using eval CLI commands.

## Problem

```rust
// vibes-server/src/state.rs - all constructors do this:
study_manager: None,  // Never wired up!
```

The vibes-evals crate has all the components:
- `StudyManager` - CQRS manager for study lifecycle
- `TursoEvalStorage` - SQLite projection for queries
- `EvalProjectionConsumer` - Background task to process events

But they're not connected to the server startup.

## Solution

Wire up the eval system in `AppState::new_with_iggy()`:

1. Create a separate Iggy stream for `StoredEvalEvent`
2. Create `TursoEvalStorage` with local SQLite database
3. Create `EvalProjectionConsumer` and spawn as background task
4. Create `StudyManager` with event log and storage
5. Set `study_manager: Some(Arc::new(manager))`

## Implementation

### 1. Add dependency

```toml
# vibes-server/Cargo.toml
vibes-evals = { path = "../vibes-evals" }
```

### 2. Create eval event log

Use a separate Iggy stream for eval events to keep them isolated from main vibes events.

### 3. Wire up in new_with_iggy()

```rust
// Create eval storage (SQLite for projection)
let eval_db_path = data_dir.join("eval.db");
let eval_storage = Arc::new(TursoEvalStorage::new_local(&eval_db_path).await?);

// Create eval event log (separate stream)
let eval_event_log = Arc::new(/* Iggy stream for StoredEvalEvent */);

// Create and start projection consumer
let consumer = EvalProjectionConsumer::new(eval_event_log.clone(), eval_storage.clone());
tokio::spawn(async move { consumer.run().await });

// Create study manager
let study_manager = Some(Arc::new(StudyManager::new(eval_event_log, eval_storage)));
```

## Acceptance Criteria

- [x] `vibes eval study status` works without "not enabled" error
- [x] `vibes eval study start` creates a study (StudyManager wired with event log)
- [x] Studies persist across server restarts (SQLite at ~/.local/share/vibes/eval.db)
- [x] Projection consumer processes events correctly (spawned as background task)
