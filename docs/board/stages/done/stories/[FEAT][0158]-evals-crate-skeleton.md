---
id: FEAT0158
title: vibes-evals crate skeleton
type: feat
status: done
priority: high
scope: evals/39-performance-evaluation
depends: []
estimate: 2h
---

# vibes-evals crate skeleton

## Summary

Create the foundational `vibes-evals` crate for evaluation and benchmarking. This establishes the crate structure, dependencies, and module layout.

## Features

### Crate Structure

```
vibes-evals/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public exports
│   ├── metrics.rs       # Metric definitions (placeholder)
│   ├── storage.rs       # Storage interface (placeholder)
│   ├── study.rs         # Study management (placeholder)
│   └── types.rs         # Core types
```

### Dependencies

```toml
[dependencies]
vibes-core = { path = "../vibes-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v7", "serde"] }
thiserror = "2"
tracing = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Core Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StudyId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BenchmarkId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(pub Uuid);
```

## Implementation

1. Create `vibes-evals/Cargo.toml`
2. Create `vibes-evals/src/lib.rs` with module structure
3. Define ID types in `types.rs`
4. Add crate to workspace `Cargo.toml`
5. Verify `just build` passes

## Acceptance Criteria

- [ ] `vibes-evals` crate exists in workspace
- [ ] Module structure matches design
- [ ] ID types defined and exported
- [ ] Crate compiles without errors
- [ ] Added to workspace members
