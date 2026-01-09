//! Attribution engine for tracking learning value
//!
//! The attribution engine determines which learnings help or hurt sessions
//! by analyzing signals across multiple dimensions and time.

mod activation;
mod store;
mod types;

pub use activation::{
    ActivationConfig, ActivationDetector, ActivationResult, HybridActivationDetector,
};
pub use store::{ATTRIBUTION_SCHEMA, AttributionStore, CozoAttributionStore};
pub use types::{
    AblationExperiment, AblationResult, ActivationSignal, AttributionRecord, LearningStatus,
    LearningValue, SessionOutcome,
};
