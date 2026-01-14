//! Agent system for vibes
//!
//! This module provides the foundation for agent orchestration:
//! - Agent trait and lifecycle management
//! - Agent types (Ad-hoc, Background, Subagent, Interactive)
//! - Task system with metrics

pub mod task;
pub mod traits;
pub mod types;

pub use types::AgentId;
