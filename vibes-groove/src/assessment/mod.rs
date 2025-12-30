//! Assessment framework for measuring session outcomes.
//!
//! This module provides the types and infrastructure for tracking assessment events
//! with full attribution context. Every assessment event carries information about
//! which learnings were active, enabling the attribution engine to answer
//! "which learnings helped in this session?"

pub mod config;
pub mod iggy;
pub mod log;
pub mod processor;
pub mod types;

pub use config::{
    AssessmentConfig, CircuitBreakerConfig, IggyServerConfig, LlmConfig, PatternConfig,
    RetentionConfig, SamplingConfig, SessionEndConfig,
};
pub use iggy::{IggyAssessmentLog, IggyConfig, IggyManager, IggyState};
pub use log::{AssessmentLog, InMemoryAssessmentLog};
pub use processor::AssessmentProcessor;
pub use types::*;
