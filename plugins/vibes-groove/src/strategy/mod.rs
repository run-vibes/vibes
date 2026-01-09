//! Adaptive strategies for learning injection
//!
//! This module implements strategy selection via Thompson sampling,
//! learning which injection approaches work best for different contexts.

mod learner;
mod store;
mod types;

pub use learner::{SessionContext, StrategyLearner, StrategyLearnerConfig};
pub use store::{CozoStrategyStore, STRATEGY_SCHEMA, StrategyStore};
pub use types::*;
