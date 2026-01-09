//! Adaptive strategies for learning injection
//!
//! This module implements strategy selection via Thompson sampling,
//! learning which injection approaches work best for different contexts.

mod learner;
mod router;
mod store;
mod types;
mod updater;

pub use learner::{SessionContext, StrategyLearner, StrategyLearnerConfig};
pub use router::{OutcomeRouter, OutcomeRouterConfig};
pub use store::{CozoStrategyStore, STRATEGY_SCHEMA, StrategyStore};
pub use types::*;
pub use updater::{DistributionUpdater, UpdaterConfig};
