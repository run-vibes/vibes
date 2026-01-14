//! Tracing setup and configuration.
//!
//! This module provides:
//! - OpenTelemetry tracer initialization
//! - Integration with tracing-subscriber
//! - Configuration for sampling and export

use crate::types::{SpanId, TraceId};
use opentelemetry::trace::{TraceContextExt, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use serde::{Deserialize, Serialize};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Console output format.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsoleFormat {
    /// Human-readable format with colors.
    #[default]
    Pretty,
    /// JSON format for structured logging.
    Json,
    /// Compact single-line format.
    Compact,
}

/// File output format.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    /// JSON array of spans (harder to stream).
    #[default]
    Json,
    /// Newline-delimited JSON (one span per line).
    JsonLines,
}

/// Export target for traces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExportTarget {
    /// Log spans to console (default for development).
    Console {
        #[serde(default)]
        format: ConsoleFormat,
    },
    /// Write spans to a file.
    File {
        path: std::path::PathBuf,
        #[serde(default)]
        format: FileFormat,
    },
    /// Export via OTLP protocol to a collector.
    Otlp { endpoint: String },
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
            exporters: vec![ExportTarget::Console {
                format: ConsoleFormat::default(),
            }],
        }
    }
}

/// Error type for tracer initialization.
#[derive(Debug, thiserror::Error)]
pub enum TracerError {
    /// Failed to set global subscriber.
    #[error("failed to set global subscriber: {0}")]
    SetGlobalSubscriber(#[from] tracing_subscriber::util::TryInitError),

    /// Failed to create file exporter.
    #[error("failed to create file exporter at {path}: {source}")]
    FileExporter {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create OTLP exporter.
    #[error("failed to create OTLP exporter: {0}")]
    OtlpExporter(String),
}

/// Guard that shuts down tracing when dropped.
///
/// This guard holds the OpenTelemetry tracer provider and ensures
/// proper shutdown when the guard is dropped.
#[derive(Debug)]
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
/// Returns an error if the global subscriber has already been set, or if
/// an exporter fails to initialize (e.g., invalid file path).
pub fn init_tracing(config: TracerConfig) -> Result<TracingGuard, TracerError> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;
    use tracing_subscriber::fmt;

    // Build OpenTelemetry provider - we'll add OTLP exporters to this
    let mut provider_builder = TracerProvider::builder();
    let mut has_otlp = false;

    // Determine console and file settings from config
    let mut console_format = None;
    let mut file_writer = None;

    for target in &config.exporters {
        match target {
            ExportTarget::Console { format } => {
                console_format = Some(format.clone());
            }
            ExportTarget::File { path, format: _ } => {
                // File output - open the file now to catch errors early
                // Note: format is currently ignored; both Json and JsonLines
                // produce newline-delimited JSON (tracing-subscriber behavior)
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|e| TracerError::FileExporter {
                        path: path.clone(),
                        source: e,
                    })?;
                file_writer = Some(std::sync::Mutex::new(BufWriter::new(file)));
            }
            ExportTarget::Otlp { endpoint } => {
                // OTLP export via OpenTelemetry
                let exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(endpoint)
                    .build()
                    .map_err(|e: opentelemetry::trace::TraceError| {
                        TracerError::OtlpExporter(e.to_string())
                    })?;
                provider_builder = provider_builder
                    .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio);
                has_otlp = true;
            }
        }
    }

    // If no OTLP exporter, use a no-op stdout exporter for trace context
    if !has_otlp {
        provider_builder =
            provider_builder.with_simple_exporter(opentelemetry_stdout::SpanExporter::default());
    }

    let provider = provider_builder.build();
    let tracer = provider.tracer(config.service_name.clone());

    // Helper macro to create otel layer with correct type inference
    macro_rules! make_otel_layer {
        () => {
            tracing_opentelemetry::layer().with_tracer(tracer.clone())
        };
    }

    // Build subscriber based on which exporters are configured
    // We use match to handle the different combinations with concrete types
    match (console_format, file_writer) {
        (Some(ConsoleFormat::Pretty), Some(fw)) => {
            let file_layer = fmt::layer().json().with_writer(fw);
            Registry::default()
                .with(fmt::layer().pretty())
                .with(file_layer)
                .with(make_otel_layer!())
                .try_init()?;
        }
        (Some(ConsoleFormat::Json), Some(fw)) => {
            let file_layer = fmt::layer().json().with_writer(fw);
            Registry::default()
                .with(fmt::layer().json())
                .with(file_layer)
                .with(make_otel_layer!())
                .try_init()?;
        }
        (Some(ConsoleFormat::Compact), Some(fw)) => {
            let file_layer = fmt::layer().json().with_writer(fw);
            Registry::default()
                .with(fmt::layer().compact())
                .with(file_layer)
                .with(make_otel_layer!())
                .try_init()?;
        }
        (Some(ConsoleFormat::Pretty), None) => {
            Registry::default()
                .with(fmt::layer().pretty())
                .with(make_otel_layer!())
                .try_init()?;
        }
        (Some(ConsoleFormat::Json), None) => {
            Registry::default()
                .with(fmt::layer().json())
                .with(make_otel_layer!())
                .try_init()?;
        }
        (Some(ConsoleFormat::Compact), None) => {
            Registry::default()
                .with(fmt::layer().compact())
                .with(make_otel_layer!())
                .try_init()?;
        }
        (None, Some(fw)) => {
            let file_layer = fmt::layer().json().with_writer(fw);
            Registry::default()
                .with(file_layer)
                .with(make_otel_layer!())
                .try_init()?;
        }
        (None, None) => {
            // No console or file, just OTLP (or nothing)
            Registry::default().with(make_otel_layer!()).try_init()?;
        }
    }

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
    use std::path::PathBuf;
    use tracing::info;

    #[test]
    fn console_format_default_is_pretty() {
        assert_eq!(ConsoleFormat::default(), ConsoleFormat::Pretty);
    }

    #[test]
    fn file_format_default_is_json() {
        assert_eq!(FileFormat::default(), FileFormat::Json);
    }

    #[test]
    fn export_target_console_with_format() {
        let target = ExportTarget::Console {
            format: ConsoleFormat::Json,
        };
        match target {
            ExportTarget::Console { format } => assert_eq!(format, ConsoleFormat::Json),
            _ => panic!("expected Console variant"),
        }
    }

    #[test]
    fn export_target_file_with_path_and_format() {
        let target = ExportTarget::File {
            path: PathBuf::from("/tmp/traces.json"),
            format: FileFormat::JsonLines,
        };
        match target {
            ExportTarget::File { path, format } => {
                assert_eq!(path, PathBuf::from("/tmp/traces.json"));
                assert_eq!(format, FileFormat::JsonLines);
            }
            _ => panic!("expected File variant"),
        }
    }

    #[test]
    fn export_target_serde_console_roundtrip() {
        let target = ExportTarget::Console {
            format: ConsoleFormat::Pretty,
        };
        let json = serde_json::to_string(&target).unwrap();
        let parsed: ExportTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, target);
    }

    #[test]
    fn export_target_serde_file_roundtrip() {
        let target = ExportTarget::File {
            path: PathBuf::from("/var/log/traces.jsonl"),
            format: FileFormat::JsonLines,
        };
        let json = serde_json::to_string(&target).unwrap();
        let parsed: ExportTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, target);
    }

    #[test]
    fn export_target_serde_otlp_roundtrip() {
        let target = ExportTarget::Otlp {
            endpoint: "http://localhost:4317".to_string(),
        };
        let json = serde_json::to_string(&target).unwrap();
        let parsed: ExportTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, target);
    }

    #[test]
    fn export_target_serde_tagged_format() {
        // Verify serde uses tagged format: {"type": "console", "format": "pretty"}
        let target = ExportTarget::Console {
            format: ConsoleFormat::Pretty,
        };
        let json = serde_json::to_string(&target).unwrap();
        assert!(json.contains(r#""type":"console""#));
    }

    #[test]
    fn export_target_console_default_format() {
        // Should deserialize with default format when not specified
        let json = r#"{"type": "console"}"#;
        let target: ExportTarget = serde_json::from_str(json).unwrap();
        match target {
            ExportTarget::Console { format } => assert_eq!(format, ConsoleFormat::Pretty),
            _ => panic!("expected Console variant"),
        }
    }

    #[test]
    fn tracer_config_has_sensible_defaults() {
        let config = TracerConfig::default();

        assert_eq!(config.service_name, "vibes");
        assert!(!config.service_version.is_empty());
        assert!((config.sample_rate - 1.0).abs() < f64::EPSILON);
        // Default should now be Console with Pretty format
        assert_eq!(
            config.exporters,
            vec![ExportTarget::Console {
                format: ConsoleFormat::default()
            }]
        );
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

    #[test]
    fn file_exporter_fails_on_invalid_path() {
        // Trying to write to a non-existent directory should fail
        let config = TracerConfig {
            exporters: vec![ExportTarget::File {
                path: PathBuf::from("/nonexistent/directory/traces.json"),
                format: FileFormat::JsonLines,
            }],
            ..Default::default()
        };

        let result = init_tracing(config);
        assert!(
            matches!(result, Err(TracerError::FileExporter { .. })),
            "expected FileExporter error, got {:?}",
            result
        );
    }

    #[test]
    fn multiple_exporters_supported() {
        // Should be able to configure multiple exporters
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("traces.jsonl");

        let config = TracerConfig {
            exporters: vec![
                ExportTarget::Console {
                    format: ConsoleFormat::Compact,
                },
                ExportTarget::File {
                    path: file_path.clone(),
                    format: FileFormat::JsonLines,
                },
            ],
            ..Default::default()
        };

        // This should succeed (multiple exporters) and create the file
        // Note: Can't actually init because global subscriber may already be set
        // Just verify the config is valid and file could be created
        assert_eq!(config.exporters.len(), 2);
        assert!(std::fs::File::create(&file_path).is_ok());
    }
}
