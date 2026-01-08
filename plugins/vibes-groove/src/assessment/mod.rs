//! Assessment framework for measuring session outcomes.
//!
//! This module provides the types and infrastructure for tracking assessment events
//! with full attribution context. Every assessment event carries information about
//! which learnings were active, enabling the attribution engine to answer
//! "which learnings helped in this session?"
//!
//! # Module Organization
//!
//! Types are organized by their purpose and audience:
//!
//! - **[`types`]** - Internal domain types for assessment processing
//!   (e.g., `AssessmentContext`, `LightweightEvent`, `HeavyEvent`)
//!
//! - **[`config`]** - Configuration types for assessment settings
//!   (e.g., `AssessmentConfig`, `SamplingConfig`, `CircuitBreakerConfig`)
//!
//! - **[`api_types`]** - HTTP/CLI response types for external consumers
//!   (e.g., `AssessmentStatusResponse`, `AssessmentHistoryResponse`)
//!
//! This separation ensures:
//! 1. Clear boundaries between internal and external interfaces
//! 2. API stability - external types can evolve independently
//! 3. Easy discovery - find the right type by its purpose

pub mod api_types;
pub mod checkpoint;
pub mod circuit_breaker;
pub mod config;
pub mod consumer;
pub mod harness_llm;
pub mod iggy;
pub mod intervention;
pub mod lightweight;
pub mod log;
pub mod processor;
pub mod sampling;
pub mod session_buffer;
pub mod session_end;
pub mod sync_processor;
pub mod types;

pub use checkpoint::{CheckpointConfig, CheckpointManager};
// Note: CheckpointTrigger is re-exported via types.rs
pub use api_types::{
    ActivityStatus, AssessmentHistoryResponse, AssessmentStatsResponse, AssessmentStatusResponse,
    CircuitBreakerStatus, SamplingStatus, SessionHistoryItem, SessionStats, TierDistribution,
};
pub use circuit_breaker::{CircuitBreaker, CircuitState, CircuitTransition};
pub use config::{
    AssessmentConfig, CircuitBreakerConfig, IggyServerConfig, LlmConfig, PatternConfig,
    RetentionConfig, SamplingConfig, SessionEndConfig,
};
pub use consumer::{
    AssessmentConsumerConfig, ConsumerResult, StartConsumerError, assessment_consumer_loop,
    start_assessment_consumer,
};
pub use harness_llm::{
    AnalysisContext, AnalysisResult, Finding, FindingType, HarnessError, HarnessLLM,
};
pub use iggy::{IggyAssessmentLog, IggyConfig, IggyManager, IggyState};
pub use intervention::{
    HookIntervention, InterventionConfig, InterventionError, InterventionResult, Learning,
};
pub use lightweight::{LightweightDetector, LightweightDetectorConfig, SessionState};
pub use log::{AssessmentLog, InMemoryAssessmentLog};
pub use processor::AssessmentProcessor;
pub use sampling::{SamplingContext, SamplingDecision, SamplingStrategy};
pub use session_buffer::{SessionBuffer, SessionBufferConfig};
pub use session_end::{SessionEnd, SessionEndDetector, SessionEndReason};
pub use sync_processor::{CircuitBreakerSummary, SamplingSummary, SyncAssessmentProcessor};
pub use types::*;
