//! Tracing setup and configuration.
//!
//! This module provides:
//! - OpenTelemetry tracer initialization
//! - Integration with tracing-subscriber
//! - Configuration for sampling and export

use crate::types::{SpanId, TraceId};
use opentelemetry::trace::{TraceContextExt, TracerProvider as _};
use opentelemetry_sdk::trace::TracerProvider;
use serde::{Deserialize, Serialize};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Export target for traces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportTarget {
    /// Log spans to console (default for development).
    Console,
    /// Export via OTLP protocol to a collector.
    Otlp { endpoint: String },
    /// Export to Jaeger.
    Jaeger { endpoint: String },
}

/// Configuration for the tracer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerConfig {
    /// Service name for spans.
    pub service_name: String,
    /// Service version for spans.
    pub service_version: String,
    /// Sample rate (0.0 to 1.0).
    pub sample_rate: f64,
    /// Export targets.
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

/// Error type for tracer initialization.
#[derive(Debug, thiserror::Error)]
pub enum TracerError {
    /// Failed to set global subscriber.
    #[error("failed to set global subscriber: {0}")]
    SetGlobalSubscriber(#[from] tracing_subscriber::util::TryInitError),
}

/// Guard that shuts down tracing when dropped.
///
/// This guard holds the OpenTelemetry tracer provider and ensures
/// proper shutdown when the guard is dropped.
pub struct TracingGuard {
    provider: Option<TracerProvider>,
}

impl TracingGuard {
    /// Shutdown the tracer and flush pending spans.
    pub fn shutdown(&mut self) {
        if let Some(provider) = self.provider.take()
            && let Err(e) = provider.shutdown()
        {
            tracing::warn!("failed to shutdown tracer provider: {e}");
        }
    }
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Initialize the global tracing subscriber with OpenTelemetry.
///
/// Returns a guard that must be kept alive for the duration of the program.
/// When the guard is dropped, tracing is shut down and pending spans are flushed.
///
/// # Errors
///
/// Returns an error if the global subscriber has already been set.
pub fn init_tracing(config: TracerConfig) -> Result<TracingGuard, TracerError> {
    // Build the OpenTelemetry tracer provider
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();

    // Get a tracer from the provider
    let tracer = provider.tracer(config.service_name.clone());

    // Create the OpenTelemetry layer for tracing-subscriber
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Build the subscriber with fmt layer (for console output) and OpenTelemetry layer
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer);

    // Set as the global default
    subscriber.try_init()?;

    Ok(TracingGuard {
        provider: Some(provider),
    })
}

/// Get the current trace ID if inside an active span.
///
/// Returns `None` if:
/// - Not inside any span
/// - The span doesn't have OpenTelemetry context
/// - The trace ID is invalid (all zeros)
#[must_use]
pub fn current_trace_id() -> Option<TraceId> {
    let span = tracing::Span::current();
    let context = span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    if span_context.is_valid() {
        Some(TraceId(span_context.trace_id().to_bytes()))
    } else {
        None
    }
}

/// Get the current span ID if inside an active span.
///
/// Returns `None` if:
/// - Not inside any span
/// - The span doesn't have OpenTelemetry context
/// - The span ID is invalid (all zeros)
#[must_use]
pub fn current_span_id() -> Option<SpanId> {
    let span = tracing::Span::current();
    let context = span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    if span_context.is_valid() {
        Some(SpanId(span_context.span_id().to_bytes()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[test]
    fn tracer_config_has_sensible_defaults() {
        let config = TracerConfig::default();

        assert_eq!(config.service_name, "vibes");
        assert!(!config.service_version.is_empty());
        assert!((config.sample_rate - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.exporters, vec![ExportTarget::Console]);
    }

    #[test]
    fn init_tracing_sets_up_global_subscriber() {
        // Initialize tracing with default config
        let _guard = init_tracing(TracerConfig::default()).expect("init_tracing should succeed");

        // Emit a span to verify the subscriber is active
        let span = tracing::info_span!("test_span", test_key = "test_value");
        let _enter = span.enter();
        info!("test message inside span");

        // If we got here without panicking, the subscriber is set up
    }

    #[test]
    fn current_trace_id_returns_none_outside_span() {
        // Outside any span, should return None
        assert!(current_trace_id().is_none());
    }

    #[test]
    fn current_span_id_returns_none_outside_span() {
        // Outside any span, should return None
        assert!(current_span_id().is_none());
    }
}
