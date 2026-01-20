---
id: observability
title: Observability Stack
status: planned
description: Full observability - tracing, logging, metrics, analytics, cost tracking, alerts
---

# Observability Stack

Full observability stack: debugging (traces, logs), monitoring (metrics, health), and analytics (insights, cost). OpenTelemetry-based for standardization.

## Overview

Three pillars plus analytics:

1. **Tracing**: Distributed traces for request flows
2. **Logging**: Structured logs with context
3. **Metrics**: Quantitative measurements
4. **Analytics**: Insights and cost tracking

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    vibes-observe                     │
├─────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │  Tracer  │  │  Logger  │  │ Metrics  │          │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘          │
│       │             │             │                 │
│       └─────────────┼─────────────┘                 │
│                     │                               │
│            ┌────────▼────────┐                      │
│            │   Collector     │                      │
│            └────────┬────────┘                      │
│                     │                               │
│  ┌──────────────────┼──────────────────┐           │
│  │     ┌────────────┼────────────┐     │           │
│  ▼     ▼            ▼            ▼     ▼           │
│ Jaeger  OTLP     Console      File   Custom        │
└─────────────────────────────────────────────────────┘
```

## Tracing

OpenTelemetry-based distributed tracing:

```rust
pub struct Trace {
    pub trace_id: TraceId,
    pub root_span: Span,
    pub spans: Vec<Span>,
}

pub struct Span {
    pub span_id: SpanId,
    pub parent_id: Option<SpanId>,
    pub name: String,
    pub start: DateTime,
    pub end: Option<DateTime>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, Value>,
    pub events: Vec<SpanEvent>,
}

// Automatic instrumentation
#[instrument]
async fn process_request(&self, req: Request) -> Result<Response> {
    let span = tracing::span!(Level::INFO, "process_request",
        session_id = %req.session_id,
        agent_id = %req.agent_id,
    );
    // ...
}
```

### Trace Context

```rust
pub struct TraceContext {
    pub session_id: SessionId,
    pub agent_id: Option<AgentId>,
    pub swarm_id: Option<SwarmId>,
    pub user_id: Option<UserId>,
    pub model: Option<ModelId>,
    pub cost_center: Option<String>,
}
```

## Logging

Structured logging with automatic context:

```rust
pub struct LogEntry {
    pub timestamp: DateTime,
    pub level: Level,
    pub target: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
}

// Usage
tracing::info!(
    session_id = %session.id,
    agent = %agent.name,
    tokens = token_count,
    "Completed inference request"
);
```

### Log Levels

| Level | Use Case |
|-------|----------|
| ERROR | Failures requiring attention |
| WARN | Degraded but functional |
| INFO | Significant events |
| DEBUG | Development details |
| TRACE | Fine-grained debugging |

## Built-in Metrics

```rust
pub struct ModelMetrics {
    pub requests_total: Counter,
    pub tokens_input: Counter,
    pub tokens_output: Counter,
    pub request_duration_ms: Histogram,
    pub cache_hit_rate: Gauge,
    pub errors_total: Counter,
}

pub struct AgentMetrics {
    pub agents_active: Gauge,
    pub agents_spawned_total: Counter,
    pub task_duration_ms: Histogram,
    pub tool_calls_total: Counter,
    pub tool_errors_total: Counter,
    pub iterations_per_task: Histogram,
}

pub struct SessionMetrics {
    pub sessions_active: Gauge,
    pub session_duration_ms: Histogram,
    pub events_total: Counter,
    pub messages_total: Counter,
}

pub struct SwarmMetrics {
    pub swarms_active: Gauge,
    pub coordination_overhead_ms: Histogram,
    pub agent_utilization: Gauge,
}

pub struct SystemMetrics {
    pub memory_bytes: Gauge,
    pub cpu_percent: Gauge,
    pub disk_io_bytes: Counter,
    pub network_io_bytes: Counter,
}
```

## Cost Tracking

```rust
pub struct CostTracker {
    pub totals: CostTotals,
    pub by_model: HashMap<ModelId, CostBreakdown>,
    pub by_session: HashMap<SessionId, CostBreakdown>,
    pub by_agent: HashMap<AgentId, CostBreakdown>,
}

pub struct CostBreakdown {
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub cost_input: f64,
    pub cost_output: f64,
    pub total_cost: f64,
    pub requests: u64,
}

pub struct CostTotals {
    pub today: f64,
    pub this_week: f64,
    pub this_month: f64,
    pub all_time: f64,
}
```

## Analytics Dashboard

```rust
pub struct AnalyticsDashboard {
    pub time_range: TimeRange,

    // Overview
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub active_sessions: u32,
    pub active_agents: u32,

    // Performance
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub success_rate: f64,
    pub error_rate: f64,

    // Usage patterns
    pub requests_by_hour: Vec<(DateTime, u64)>,
    pub top_models: Vec<(ModelId, u64)>,
    pub top_tools: Vec<(String, u64)>,
}
```

## Alerts

```rust
pub struct AlertRule {
    pub id: AlertId,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: Severity,
    pub channels: Vec<NotifyChannel>,
}

pub enum AlertCondition {
    MetricAbove { metric: String, threshold: f64 },
    MetricBelow { metric: String, threshold: f64 },
    RateOfChange { metric: String, percent: f64, window: Duration },
    ErrorRate { above: f64, window: Duration },
    CostExceeds { amount: f64, period: Period },
}

pub enum NotifyChannel {
    Console,
    File { path: PathBuf },
    Webhook { url: Url },
    Slack { channel: String },
    Email { addresses: Vec<String> },
}
```

## Export Targets

```rust
pub enum ExportTarget {
    // Console
    Console { format: ConsoleFormat },

    // Files
    File { path: PathBuf, format: FileFormat },

    // OpenTelemetry
    Otlp { endpoint: Url },

    // Tracing backends
    Jaeger { endpoint: Url },
    Zipkin { endpoint: Url },

    // Metrics backends
    Prometheus { port: u16 },

    // Cloud
    Datadog { api_key: String },
    NewRelic { api_key: String },
}
```

## Configuration

```rust
pub struct ObserveConfig {
    // Tracing
    pub tracing_enabled: bool,
    pub trace_sample_rate: f64,
    pub trace_exporters: Vec<ExportTarget>,

    // Logging
    pub log_level: Level,
    pub log_format: LogFormat,
    pub log_exporters: Vec<ExportTarget>,

    // Metrics
    pub metrics_enabled: bool,
    pub metrics_interval: Duration,
    pub metrics_exporters: Vec<ExportTarget>,

    // Cost
    pub cost_tracking_enabled: bool,
    pub cost_alerts: Vec<AlertRule>,

    // Retention
    pub trace_retention: Duration,
    pub log_retention: Duration,
    pub metrics_retention: Duration,
}
```

## CLI Commands

```
# Real-time
vibes observe logs                    # Tail logs
vibes observe logs --level error      # Filter by level
vibes observe traces <session>        # View session traces
vibes observe metrics                 # Current metrics

# Analytics
vibes observe dashboard               # Open TUI dashboard
vibes observe cost                    # Cost summary
vibes observe cost --by model         # Cost by model
vibes observe cost --by session       # Cost by session

# Alerts
vibes observe alerts list             # List alert rules
vibes observe alerts add <rule>       # Add alert rule
vibes observe alerts test <rule>      # Test alert

# Export
vibes observe export logs <file>      # Export logs
vibes observe export metrics <file>   # Export metrics
```

<!-- BEGIN GENERATED -->
## Milestones

**Progress:** 1/1 milestones complete, 7/7 stories done

| ID | Milestone | Stories | Status |
|----|-----------|---------|--------|
| 01 | [Distributed Tracing](milestones/01-distributed-tracing/) | 7/7 | done |
<!-- END GENERATED -->
