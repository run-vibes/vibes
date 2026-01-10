//! Open-world adaptation module
//!
//! Detects unknown patterns and surfaces capability gaps. Implements novelty
//! detection using embedding similarity and incremental clustering, and
//! graduated response for progressive escalation.
//!
//! # Overview
//!
//! The open-world adaptation system has three main components:
//!
//! - **NoveltyDetector** - Detects patterns not matching known fingerprints
//! - **CapabilityGapDetector** - Identifies recurring failures
//! - **GraduatedResponse** - Progressive handling from monitor to surface
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

mod novelty;
mod traits;
mod types;

pub use novelty::{NoveltyConfig, NoveltyContext, NoveltyDetector};

pub use traits::{NoOpOpenWorldStore, OpenWorldStore};
pub use types::{
    AnomalyCluster, CapabilityGap, ClusterId, FailureId, FailureRecord, FailureType, GapCategory,
    GapId, GapSeverity, GapStatus, NoveltyResult, OpenWorldEvent, PatternFingerprint,
    ResponseAction, ResponseStage, SolutionAction, SolutionSource, StrategyChange,
    SuggestedSolution,
};
