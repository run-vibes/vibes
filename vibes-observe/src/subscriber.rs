//! Trace event broadcasting for CLI consumption.
//!
//! Provides a tracing subscriber layer that broadcasts span events
//! to connected CLI clients for real-time trace viewing.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::span::{Attributes, Id};
use tracing::{Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

use crate::context::attributes as vibes_attrs;

/// Status of a completed span.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    /// Span completed successfully
    #[default]
    Ok,
    /// Span completed with an error
    Error,
}

/// A trace event representing a completed span.
///
/// This is sent to CLI clients for display in `vibes observe traces`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    /// The trace ID (128-bit hex string)
    pub trace_id: String,
    /// The span ID (64-bit hex string)
    pub span_id: String,
    /// Parent span ID if this is a child span
    pub parent_span_id: Option<String>,
    /// Span name (e.g., "agent::run", "tool::execute")
    pub name: String,
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// When the span started
    pub timestamp: DateTime<Utc>,
    /// Duration in milliseconds (None if span not yet closed)
    pub duration_ms: Option<f64>,
    /// Session ID if present in span attributes
    pub session_id: Option<String>,
    /// Agent ID if present in span attributes
    pub agent_id: Option<String>,
    /// Additional span attributes
    pub attributes: HashMap<String, String>,
    /// Span completion status
    pub status: SpanStatus,
}

/// Storage for span data while it's open.
#[derive(Debug)]
struct SpanData {
    name: String,
    level: Level,
    start_time: DateTime<Utc>,
    attributes: HashMap<String, String>,
    status: SpanStatus,
}

/// A tracing layer that broadcasts span events to subscribers.
///
/// This layer captures spans when they close and broadcasts them
/// to connected clients via a tokio broadcast channel.
pub struct TraceBroadcaster {
    tx: broadcast::Sender<TraceEvent>,
}

impl TraceBroadcaster {
    /// Default channel capacity for trace events.
    pub const DEFAULT_CAPACITY: usize = 1024;

    /// Create a new TraceBroadcaster with the specified channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Subscribe to receive trace events.
    pub fn subscribe(&self) -> broadcast::Receiver<TraceEvent> {
        self.tx.subscribe()
    }

    /// Get the sender for cloning (used in AppState).
    pub fn sender(&self) -> broadcast::Sender<TraceEvent> {
        self.tx.clone()
    }
}

impl Default for TraceBroadcaster {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

impl<S> Layer<S> for TraceBroadcaster
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("span should exist");

        // Collect attributes from the span
        let mut attributes = HashMap::new();
        let mut visitor = AttributeVisitor(&mut attributes);
        attrs.record(&mut visitor);

        let data = SpanData {
            name: span.name().to_string(),
            level: *attrs.metadata().level(),
            start_time: Utc::now(),
            attributes,
            status: SpanStatus::Ok,
        };

        span.extensions_mut().insert(data);
    }

    fn on_record(&self, id: &Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id)
            && let Some(data) = span.extensions_mut().get_mut::<SpanData>()
        {
            let mut visitor = AttributeVisitor(&mut data.attributes);
            values.record(&mut visitor);
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        // Check if this is an error event - mark the current span as error
        if *event.metadata().level() == Level::ERROR
            && let Some(span_id) = ctx.current_span().id()
            && let Some(span) = ctx.span(span_id)
            && let Some(data) = span.extensions_mut().get_mut::<SpanData>()
        {
            data.status = SpanStatus::Error;
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("span should exist");
        let extensions = span.extensions();

        let Some(data) = extensions.get::<SpanData>() else {
            return;
        };

        let end_time = Utc::now();
        let duration_ms = (end_time - data.start_time)
            .num_microseconds()
            .map(|us| us as f64 / 1000.0);

        // Generate span ID from the tracing span ID
        let span_id = format!("{:016x}", id.into_u64());

        // Try to get trace ID from OpenTelemetry context or generate one
        let trace_id = crate::current_trace_id()
            .map(|t| t.to_string())
            .unwrap_or_else(|| format!("{:032x}", rand_u128()));

        // Get parent span ID
        let parent_span_id = span.parent().map(|p| format!("{:016x}", p.id().into_u64()));

        // Extract vibes-specific attributes
        let session_id = data.attributes.get(vibes_attrs::SESSION_ID).cloned();
        let agent_id = data.attributes.get(vibes_attrs::AGENT_ID).cloned();

        let event = TraceEvent {
            trace_id,
            span_id,
            parent_span_id,
            name: data.name.clone(),
            level: level_to_string(data.level),
            timestamp: data.start_time,
            duration_ms,
            session_id,
            agent_id,
            attributes: data.attributes.clone(),
            status: data.status,
        };

        // Best-effort broadcast - ignore errors (no receivers is fine)
        let _ = self.tx.send(event);
    }
}

/// Visitor for collecting span attributes into a HashMap.
struct AttributeVisitor<'a>(&'a mut HashMap<String, String>);

impl tracing::field::Visit for AttributeVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }
}

fn level_to_string(level: Level) -> String {
    match level {
        Level::TRACE => "trace".to_string(),
        Level::DEBUG => "debug".to_string(),
        Level::INFO => "info".to_string(),
        Level::WARN => "warn".to_string(),
        Level::ERROR => "error".to_string(),
    }
}

/// Generate a random u128 for trace IDs when OpenTelemetry context isn't available.
fn rand_u128() -> u128 {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("getrandom failed");
    u128::from_be_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_status_serializes_correctly() {
        assert_eq!(serde_json::to_string(&SpanStatus::Ok).unwrap(), r#""ok""#);
        assert_eq!(
            serde_json::to_string(&SpanStatus::Error).unwrap(),
            r#""error""#
        );
    }

    #[test]
    fn trace_event_serializes_correctly() {
        let event = TraceEvent {
            trace_id: "0123456789abcdef0123456789abcdef".to_string(),
            span_id: "0123456789abcdef".to_string(),
            parent_span_id: None,
            name: "test::span".to_string(),
            level: "info".to_string(),
            timestamp: Utc::now(),
            duration_ms: Some(1.5),
            session_id: Some("sess-123".to_string()),
            agent_id: None,
            attributes: HashMap::new(),
            status: SpanStatus::Ok,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""name":"test::span""#));
        assert!(json.contains(r#""status":"ok""#));
    }

    #[test]
    fn trace_broadcaster_can_subscribe() {
        let broadcaster = TraceBroadcaster::default();
        let _rx = broadcaster.subscribe();
        // Should not panic
    }

    #[tokio::test]
    async fn trace_broadcaster_broadcasts_events() {
        let broadcaster = TraceBroadcaster::default();
        let mut rx = broadcaster.subscribe();

        let event = TraceEvent {
            trace_id: "test".to_string(),
            span_id: "test".to_string(),
            parent_span_id: None,
            name: "test".to_string(),
            level: "info".to_string(),
            timestamp: Utc::now(),
            duration_ms: Some(1.0),
            session_id: None,
            agent_id: None,
            attributes: HashMap::new(),
            status: SpanStatus::Ok,
        };

        broadcaster.tx.send(event.clone()).unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.name, "test");
    }
}
