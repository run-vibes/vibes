//! Evaluation and benchmarking framework for vibes.
//!
//! This crate provides tools for evaluating AI assistant performance,
//! running benchmarks, and managing evaluation studies.
//!
//! # Architecture
//!
//! This crate uses event sourcing for study management:
//!
//! - **Events** ([`EvalEvent`]) are the source of truth, stored in Iggy
//! - **Storage** ([`EvalStorage`]) provides read-only queries from projections
//! - **Projection** ([`EvalProjection`]) updates read models from events
//!
//! See the milestone design doc for full architecture details.

mod commands;
mod consumer;
mod events;
mod manager;
mod metrics;
pub mod storage;
mod study;
mod types;

// Command types
pub use commands::{CreateStudy, RecordCheckpoint};

// Consumer
pub use consumer::EvalProjectionConsumer;

// Manager
pub use manager::StudyManager;

// Event types
pub use events::{EvalEvent, StoredEvalEvent};

// Metric types
pub use metrics::{AggregationType, LongitudinalMetrics, MetricDefinition, MetricUnit, TimePeriod};

// Study types
pub use study::{Checkpoint, PeriodType, Study, StudyConfig, StudyStatus};

// ID types
pub use types::{BenchmarkId, CheckpointId, StudyId};

// Storage traits (re-export from storage module)
pub use storage::{EvalProjection, EvalStorage};
