---
id: m40-feat-05
title: Export targets configuration
type: feat
status: in-progress
priority: high
epics: [observability]
depends: [m40-feat-02]
estimate: 3h
milestone: 40-observability-tracing
---

# Export targets configuration

## Summary

Implement configurable export targets for traces. Support console, file, Jaeger, and OTLP endpoints.

## Features

### ExportTarget Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportTarget {
    /// Print to console (stdout)
    Console {
        format: ConsoleFormat,
    },

    /// Write to file
    File {
        path: PathBuf,
        format: FileFormat,
    },

    /// OpenTelemetry Protocol
    Otlp {
        endpoint: Url,
    },

    /// Jaeger backend
    Jaeger {
        endpoint: Url,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ConsoleFormat {
    #[default]
    Pretty,
    Json,
    Compact,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum FileFormat {
    #[default]
    Json,
    JsonLines,
}
```

### Exporter Builder

```rust
pub struct ExporterBuilder {
    targets: Vec<ExportTarget>,
}

impl ExporterBuilder {
    pub fn new() -> Self;

    /// Add console output
    pub fn with_console(self, format: ConsoleFormat) -> Self;

    /// Add file output
    pub fn with_file(self, path: PathBuf, format: FileFormat) -> Self;

    /// Add OTLP endpoint
    pub fn with_otlp(self, endpoint: Url) -> Self;

    /// Add Jaeger endpoint
    pub fn with_jaeger(self, endpoint: Url) -> Self;

    /// Build the exporter pipeline
    pub fn build(self) -> Result<TracerProvider>;
}
```

### Configuration File Support

```toml
# vibes.toml
[observe.tracing]
enabled = true
sample_rate = 1.0

[[observe.tracing.exporters]]
type = "console"
format = "pretty"

[[observe.tracing.exporters]]
type = "otlp"
endpoint = "http://localhost:4317"
```

## Implementation

1. Create `vibes-observe/src/export.rs`
2. Define `ExportTarget` enum
3. Implement `ExporterBuilder`
4. Add support for each target type
5. Add configuration file parsing
6. Write tests for each exporter

## Acceptance Criteria

- [x] Console exporter works (pretty/json/compact)
- [x] File exporter writes JSON lines
- [x] OTLP exporter connects to collector
- [x] ~~Jaeger exporter sends spans~~ Removed - Jaeger now supports OTLP natively
- [ ] Configuration file parsing works (handled by vibes-core, separate story)
- [x] Multiple exporters can be active

## Design Notes

- Removed `Jaeger` variant from `ExportTarget` since `opentelemetry-jaeger` is deprecated and Jaeger natively supports OTLP. Users targeting Jaeger should use `Otlp` with Jaeger's OTLP endpoint.
- `ExporterBuilder` was not implemented as a separate struct; `TracerConfig.exporters` provides equivalent functionality via `init_tracing()`.
