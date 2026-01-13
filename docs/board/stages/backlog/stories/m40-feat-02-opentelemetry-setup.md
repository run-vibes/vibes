---
id: m40-feat-02
title: OpenTelemetry tracer setup
type: feat
status: backlog
priority: high
epics: [observability]
depends: [m40-feat-01]
estimate: 4h
milestone: 40-observability-tracing
---

# OpenTelemetry tracer setup

## Summary

Configure OpenTelemetry tracing with the `tracing` crate for distributed tracing. This provides the foundation for all observability in vibes.

## Features

### Tracer Configuration

```rust
pub struct TracerConfig {
    pub service_name: String,
    pub service_version: String,
    pub sample_rate: f64,
    pub exporters: Vec<ExportTarget>,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self {
            service_name: "vibes".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            sample_rate: 1.0,
            exporters: vec![ExportTarget::Console],
        }
    }
}
```

### Tracer Setup

```rust
pub struct Tracer {
    config: TracerConfig,
    provider: TracerProvider,
}

impl Tracer {
    /// Initialize the global tracer
    pub fn init(config: TracerConfig) -> Result<Self>;

    /// Shutdown and flush pending spans
    pub fn shutdown(&self) -> Result<()>;

    /// Get the current trace ID if in a span
    pub fn current_trace_id() -> Option<TraceId>;

    /// Get the current span ID if in a span
    pub fn current_span_id() -> Option<SpanId>;
}
```

### Integration with tracing crate

```rust
/// Set up tracing subscriber with OpenTelemetry layer
pub fn init_tracing(config: TracerConfig) -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(/* ... */)
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let telemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer);

    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(telemetry_layer)
        .with(fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

## Implementation

1. Create `vibes-observe/src/tracer.rs`
2. Define `TracerConfig` struct
3. Implement `init_tracing` function
4. Set up OpenTelemetry pipeline
5. Configure tracing-subscriber layers
6. Write integration tests

## Acceptance Criteria

- [ ] `TracerConfig` configures tracer behavior
- [ ] `init_tracing` sets up global subscriber
- [ ] Spans automatically get trace/span IDs
- [ ] Console output shows spans (default)
- [ ] Shutdown flushes pending data
- [ ] Integration test verifies setup
