//! Adaptive strategies for learning injection
//!
//! This module implements strategy selection via Thompson sampling,
//! learning which injection approaches work best for different contexts.

mod consumer;
mod learner;
mod router;
mod store;
mod types;
mod updater;

pub use consumer::{
    ConsumerLoopResult as StrategyConsumerLoopResult, LearningLoader as StrategyLearningLoader,
    NoveltyHook, SessionContextProvider, StartConsumerError as StartStrategyConsumerError,
    StrategyConsumer, StrategyConsumerConfig, StrategyConsumerResult, StrategyInput,
    UsedStrategyProvider, start_strategy_consumer, strategy_consumer_loop,
};
pub use learner::{SessionContext, StrategyLearner, StrategyLearnerConfig};
pub use router::{OutcomeRouter, OutcomeRouterConfig};
pub use store::{CozoStrategyStore, STRATEGY_SCHEMA, StrategyStore};
pub use types::*;
pub use updater::{DistributionUpdater, UpdaterConfig};
