//! Evaluation and benchmarking framework for vibes.
//!
//! This crate provides tools for evaluating AI assistant performance,
//! running benchmarks, and managing evaluation studies.

mod metrics;
mod storage;
mod study;
mod types;

pub use metrics::{AggregationType, LongitudinalMetrics, MetricDefinition, MetricUnit, TimePeriod};
pub use types::{BenchmarkId, CheckpointId, StudyId};
