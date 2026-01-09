//! Attribution engine for tracking learning value
//!
//! The attribution engine determines which learnings help or hurt sessions
//! by analyzing signals across multiple dimensions and time.

mod ablation;
mod activation;
mod aggregation;
mod consumer;
mod store;
mod temporal;
mod types;

pub use ablation::{AblationConfig, AblationManager, AblationStrategy, ConservativeAblation};
pub use activation::{
    ActivationConfig, ActivationDetector, ActivationResult, HybridActivationDetector,
};
pub use aggregation::{AggregationConfig, ValueAggregator};
pub use consumer::{
    AttributionConfig, AttributionConsumer, AttributionResult as AttributionConsumerResult,
    ConsumerLoopResult as AttributionConsumerLoopResult, LearningLoader, LightweightEventFetcher,
    StartConsumerError as AttributionStartConsumerError,
    TranscriptFetcher as AttributionTranscriptFetcher, attribution_consumer_loop,
    start_attribution_consumer,
};
pub use store::{ATTRIBUTION_SCHEMA, AttributionStore, CozoAttributionStore};
pub use temporal::{
    ExponentialDecayCorrelator, TemporalConfig, TemporalCorrelator, TemporalResult,
};
pub use types::{
    AblationExperiment, AblationResult, ActivationSignal, AttributionRecord, LearningStatus,
    LearningValue, SessionOutcome,
};
