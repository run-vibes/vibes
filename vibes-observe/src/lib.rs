//! Observability and tracing infrastructure for vibes.
//!
//! This crate provides OpenTelemetry-based tracing with support for:
//! - Distributed trace context propagation
//! - Multiple export targets (OTLP, Jaeger)
//! - Integration with the tracing ecosystem
//!
//! # Quick Start
//!
//! ```ignore
//! use vibes_observe::{init_tracing, TracerConfig};
//!
//! // Initialize with default config (console output)
//! let _guard = init_tracing(TracerConfig::default())?;
//!
//! // Use tracing macros as normal
//! tracing::info_span!("my_operation").in_scope(|| {
//!     tracing::info!("doing work");
//! });
//! ```

pub mod context;
pub mod export;
pub mod tracer;
pub mod types;

pub use tracer::{
    ExportTarget, TracerConfig, TracerError, TracingGuard, current_span_id, current_trace_id,
    init_tracing,
};
pub use types::{ParseIdError, SpanId, TraceId};
