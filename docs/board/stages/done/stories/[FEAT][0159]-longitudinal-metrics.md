---
id: FEAT0159
title: LongitudinalMetrics types
type: feat
status: done
priority: high
scope: evals/39-performance-evaluation
depends: [m39-feat-01]
estimate: 3h
---

# LongitudinalMetrics types

## Summary

Define the core metrics types for longitudinal evaluation. These types capture performance measurements across sessions, tasks, agents, and learning effectiveness.

## Features

### LongitudinalMetrics

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LongitudinalMetrics {
    // Session-level
    pub sessions_completed: u64,
    pub session_success_rate: f64,
    pub avg_session_duration: Duration,

    // Task-level
    pub tasks_completed: u64,
    pub first_attempt_success_rate: f64,
    pub avg_iterations_to_success: f64,

    // Agent-level
    pub agent_efficiency: f64,
    pub tool_success_rate: f64,
    pub self_correction_rate: f64,

    // Swarm-level (for future use)
    pub swarm_coordination_overhead: f64,
    pub parallelism_efficiency: f64,

    // Learning integration (groove)
    pub learnings_applied: u64,
    pub learning_effectiveness: f64,

    // Cost
    pub total_tokens: u64,
    pub total_cost: f64,
    pub cost_per_successful_task: f64,

    // Time window
    pub period: TimePeriod,
}
```

### TimePeriod

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimePeriod {
    pub fn duration(&self) -> Duration;
    pub fn contains(&self, instant: DateTime<Utc>) -> bool;
}
```

### MetricDefinition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub name: String,
    pub description: String,
    pub unit: MetricUnit,
    pub aggregation: AggregationType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricUnit {
    Count,
    Percentage,
    Duration,
    Tokens,
    Currency,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Min,
    Max,
    P50,
    P95,
    P99,
}
```

## Implementation

1. Create `vibes-evals/src/metrics.rs`
2. Define `LongitudinalMetrics` struct
3. Define `TimePeriod` with utility methods
4. Define `MetricDefinition` for custom metrics
5. Implement `Default` for all types
6. Write serialization tests

## Acceptance Criteria

- [ ] `LongitudinalMetrics` captures all metric categories
- [ ] `TimePeriod` utility methods work correctly
- [ ] `MetricDefinition` supports custom metrics
- [ ] All types serialize/deserialize correctly
- [ ] Default values are sensible
