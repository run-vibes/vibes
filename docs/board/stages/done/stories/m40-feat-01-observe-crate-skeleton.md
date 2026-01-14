---
id: m40-feat-01
title: vibes-observe crate skeleton
type: feat
status: done
priority: high
epics: [observability]
depends: []
estimate: 2h
milestone: 40-observability-tracing
---

# vibes-observe crate skeleton

## Summary

Create the foundational `vibes-observe` crate for observability. This establishes the crate structure with OpenTelemetry dependencies and module layout.

## Features

### Crate Structure

```
vibes-observe/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public exports
│   ├── tracer.rs        # Tracing setup (placeholder)
│   ├── context.rs       # Trace context (placeholder)
│   ├── export.rs        # Export targets (placeholder)
│   └── types.rs         # Core types
```

### Dependencies

```toml
[dependencies]
vibes-core = { path = "../vibes-core" }
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
opentelemetry-jaeger = "0.22"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Core Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceId(pub [u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanId(pub [u8; 8]);

impl TraceId {
    pub fn new() -> Self;
    pub fn to_hex(&self) -> String;
}
```

## Implementation

1. Create `vibes-observe/Cargo.toml`
2. Create `vibes-observe/src/lib.rs` with module structure
3. Define core types in `types.rs`
4. Add crate to workspace `Cargo.toml`
5. Verify `just build` passes

## Acceptance Criteria

- [ ] `vibes-observe` crate exists in workspace
- [ ] OpenTelemetry dependencies configured
- [ ] Module structure matches design
- [ ] Core types defined and exported
- [ ] Crate compiles without errors
