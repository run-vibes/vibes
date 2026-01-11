//! Open-world adaptation module
//!
//! Detects unknown patterns and surfaces capability gaps. Implements novelty
//! detection using embedding similarity and incremental clustering, and
//! graduated response for progressive escalation.
//!
//! # Overview
//!
//! The open-world adaptation system handles patterns the learning system hasn't
//! seen before. Rather than failing silently, it:
//!
//! 1. Detects novel contexts using embedding similarity
//! 2. Clusters similar novel patterns for analysis
//! 3. Tracks repeated failures to identify capability gaps
//! 4. Generates solutions based on gap analysis
//! 5. Adjusts strategy exploration to handle uncertainty
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Strategy Consumer                            │
//! │  (processes learning outcomes, invokes NoveltyHook)             │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     OpenWorldHook                               │
//! │  - Implements NoveltyHook trait                                 │
//! │  - Analyzes outcomes for novelty/failures                       │
//! │  - Tracks observations per context                              │
//! └───────────┬───────────────────────────────────┬─────────────────┘
//!             │                                   │
//!             ▼                                   ▼
//! ┌───────────────────────┐           ┌─────────────────────────────┐
//! │   NoveltyDetector     │           │  CapabilityGapDetector      │
//! │  - Hash pre-filter    │           │  - Failure tracking         │
//! │  - Embedding match    │           │  - Gap creation             │
//! │  - Cluster analysis   │           │  - Severity escalation      │
//! └───────────┬───────────┘           └──────────────┬──────────────┘
//!             │                                      │
//!             ▼                                      ▼
//! ┌───────────────────────┐           ┌─────────────────────────────┐
//! │  GraduatedResponse    │           │    SolutionGenerator        │
//! │  - Monitor            │           │  - Template matching        │
//! │  - Cluster            │           │  - Pattern analysis         │
//! │  - Auto-adjust        │           │  - Confidence scoring       │
//! │  - Surface            │           └─────────────────────────────┘
//! └───────────────────────┘
//! ```
//!
//! # Components
//!
//! - **[`NoveltyDetector`]** - Detects patterns not matching known fingerprints
//!   using fast hash pre-filtering and embedding similarity comparison.
//!
//! - **[`CapabilityGapDetector`]** - Identifies recurring failures by tracking
//!   negative outcomes and low-confidence results per context.
//!
//! - **[`GraduatedResponse`]** - Progressive handling based on cluster size:
//!   Monitor → Cluster → AutoAdjust → Surface.
//!
//! - **[`SolutionGenerator`]** - Generates solutions for capability gaps using
//!   templates and pattern analysis.
//!
//! - **[`OpenWorldHook`]** - Integration point implementing [`NoveltyHook`] trait
//!   for the strategy consumer. Create with [`OpenWorldHook::from_openworld_config`].
//!
//! - **[`OpenWorldProducer`]** - Emits events to the Iggy stream for consumers.
//!
//! # Configuration
//!
//! Configure via [`crate::config::OpenWorldConfig`]:
//!
//! ```toml
//! [openworld]
//! enabled = true
//!
//! [openworld.novelty]
//! initial_threshold = 0.85     # Similarity threshold for novelty
//! max_pending_outliers = 50    # Max pending before clustering
//!
//! [openworld.gaps]
//! min_failures_for_gap = 3     # Failures before creating gap
//! negative_attribution_threshold = -0.3
//!
//! [openworld.response]
//! monitor_threshold = 3        # < 3 observations = monitor
//! cluster_threshold = 10       # 3-10 = cluster
//! auto_adjust_threshold = 25   # 10-25 = auto-adjust
//! exploration_adjustment = 0.1
//!
//! [openworld.solutions]
//! max_solutions = 5
//! template_confidence = 0.7
//! ```
//!
//! # Types
//!
//! Core types for the system:
//!
//! - [`PatternFingerprint`] - Fingerprint of a known pattern
//! - [`AnomalyCluster`] - Cluster of similar novel patterns
//! - [`NoveltyResult`] - Result of novelty detection
//! - [`CapabilityGap`] - A detected capability gap
//! - [`FailureRecord`] - Record of a failure contributing to gap detection
//! - [`SuggestedSolution`] - A suggested solution for a gap
//! - [`ResponseAction`] - Action to take in response to novelty
//! - [`OpenWorldEvent`] - Events emitted by the system
//!
//! # Usage
//!
//! ```ignore
//! use vibes_groove::{OpenWorldConfig, OpenWorldHook};
//! use vibes_groove::strategy::StrategyConsumer;
//!
//! // Create hook from config
//! let config = OpenWorldConfig::default();
//! let hook = Arc::new(OpenWorldHook::from_openworld_config(&config));
//!
//! // Wire into strategy consumer
//! let consumer = StrategyConsumer::new(/* ... */)
//!     .with_novelty_hook(hook);
//! ```
//!
//! [`NoveltyHook`]: crate::strategy::NoveltyHook

mod clustering;
mod consumer;
mod gaps;
mod hook;
mod novelty;
mod response;
mod solutions;
mod traits;
mod types;

pub use clustering::{
    DbscanConfig, DbscanResult, DistanceMetric, compute_centroid, cosine_distance,
    euclidean_distance, incremental_dbscan, region_query,
};
pub use consumer::{
    OPENWORLD_STREAM, OpenWorldProducer, OpenWorldProducerConfig, ProducerStats, topics,
};
pub use gaps::{CapabilityGapDetector, GapsConfig};
pub use hook::{HookStats, OpenWorldHook, OpenWorldHookConfig};
pub use novelty::{NoveltyConfig, NoveltyContext, NoveltyDetector};
pub use response::{GraduatedResponse, ResponseConfig};
pub use solutions::{SolutionGenerator, SolutionsConfig};

pub use traits::{NoOpOpenWorldStore, OpenWorldStore};
pub use types::{
    AnomalyCluster, CapabilityGap, ClusterId, FailureId, FailureRecord, FailureType, GapCategory,
    GapId, GapSeverity, GapStatus, NoveltyResult, OpenWorldEvent, PatternFingerprint,
    ResponseAction, ResponseStage, SolutionAction, SolutionSource, StrategyChange,
    SuggestedSolution,
};
