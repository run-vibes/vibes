//! Observability and tracing infrastructure for vibes.
//!
//! This crate provides OpenTelemetry-based tracing with support for:
//! - Distributed trace context propagation
//! - Multiple export targets (OTLP, Jaeger)
//! - Integration with the tracing ecosystem

pub mod context;
pub mod export;
pub mod tracer;
pub mod types;

pub use types::{ParseIdError, SpanId, TraceId};
