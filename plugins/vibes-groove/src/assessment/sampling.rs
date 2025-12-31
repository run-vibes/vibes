//! Sampling strategy for assessment tiers.
//!
//! The `SamplingStrategy` decides which assessment events reach the Medium and Heavy
//! tiers, based on configurable sampling rates. This prevents overwhelming the system
//! with expensive LLM-based assessments while still gathering meaningful data.
//!
//! ## Assessment Tiers
//!
//! | Tier | Frequency | Cost | Purpose |
//! |------|-----------|------|---------|
//! | Lightweight | Every message | ~0 | Signal detection, EMA tracking |
//! | Medium | Sampled at checkpoints | ~$ | Aggregated summaries, basic analysis |
//! | Heavy | Sampled at session end | ~$$ | Full session analysis, learning extraction |
//!
//! ## Sampling Logic
//!
//! 1. During **burnin** (first N sessions): Always sample to establish baseline
//! 2. After burnin: Sample based on configured `base_rate`
//! 3. Certain triggers (high frustration) may override to always sample

use rand::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use super::checkpoint::CheckpointTrigger;
use super::config::SamplingConfig;

/// Decision on whether to sample for higher assessment tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingDecision {
    /// Skip this checkpoint for higher-tier assessment.
    Skip,
    /// Perform Medium-tier assessment (checkpoint summary).
    Medium,
    /// Perform Heavy-tier assessment (full session analysis).
    Heavy,
}

impl SamplingDecision {
    /// Check if this decision means we should skip.
    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip)
    }

    /// Check if this decision means we should do Medium-tier.
    pub fn is_medium(&self) -> bool {
        matches!(self, Self::Medium)
    }

    /// Check if this decision means we should do Heavy-tier.
    pub fn is_heavy(&self) -> bool {
        matches!(self, Self::Heavy)
    }
}

impl std::fmt::Display for SamplingDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Skip => write!(f, "skip"),
            Self::Medium => write!(f, "medium"),
            Self::Heavy => write!(f, "heavy"),
        }
    }
}

/// Context for sampling decisions.
#[derive(Debug, Clone)]
pub struct SamplingContext {
    /// The checkpoint trigger that prompted this decision.
    pub trigger: CheckpointTrigger,
    /// Whether this is a session end (triggers Heavy tier consideration).
    pub is_session_end: bool,
    /// Current frustration EMA (higher = more likely to sample).
    pub frustration_ema: f64,
    /// Number of completed sessions for this user/project.
    pub completed_sessions: u32,
}

impl SamplingContext {
    /// Create a new sampling context for a checkpoint.
    pub fn checkpoint(trigger: CheckpointTrigger) -> Self {
        Self {
            trigger,
            is_session_end: false,
            frustration_ema: 0.0,
            completed_sessions: 0,
        }
    }

    /// Create a new sampling context for session end.
    pub fn session_end() -> Self {
        Self {
            trigger: CheckpointTrigger::TimeInterval,
            is_session_end: true,
            frustration_ema: 0.0,
            completed_sessions: 0,
        }
    }

    /// Set the frustration EMA.
    #[must_use]
    pub fn with_frustration_ema(mut self, ema: f64) -> Self {
        self.frustration_ema = ema;
        self
    }

    /// Set the completed sessions count.
    #[must_use]
    pub fn with_completed_sessions(mut self, count: u32) -> Self {
        self.completed_sessions = count;
        self
    }
}

/// Sampling strategy for assessment tiers.
///
/// Determines which checkpoints get promoted to higher assessment tiers
/// based on configured rates and contextual factors.
pub struct SamplingStrategy {
    config: SamplingConfig,
    rng: StdRng,
}

impl SamplingStrategy {
    /// Create a new sampling strategy with the given configuration.
    pub fn new(config: SamplingConfig) -> Self {
        Self {
            config,
            rng: StdRng::from_entropy(),
        }
    }

    /// Create a sampling strategy with a specific seed (for testing).
    pub fn with_seed(config: SamplingConfig, seed: u64) -> Self {
        Self {
            config,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Get the configured base rate.
    pub fn base_rate(&self) -> f64 {
        self.config.base_rate
    }

    /// Get the burnin session count.
    pub fn burnin_sessions(&self) -> u32 {
        self.config.burnin_sessions
    }

    /// Decide whether to sample based on context.
    ///
    /// During burnin period, always samples. After burnin, uses base_rate
    /// with adjustments for high-frustration scenarios.
    pub fn should_sample(&mut self, context: &SamplingContext) -> SamplingDecision {
        // During burnin, always sample
        if context.completed_sessions < self.config.burnin_sessions {
            return if context.is_session_end {
                SamplingDecision::Heavy
            } else {
                SamplingDecision::Medium
            };
        }

        // High frustration always samples
        if context.frustration_ema >= 0.7 {
            return if context.is_session_end {
                SamplingDecision::Heavy
            } else {
                SamplingDecision::Medium
            };
        }

        // Pattern match trigger may boost sampling rate
        let effective_rate = match &context.trigger {
            CheckpointTrigger::PatternMatch { .. } => {
                // Pattern matches are interesting, boost rate
                (self.config.base_rate * 2.0).min(1.0)
            }
            CheckpointTrigger::ThresholdExceeded { .. } => {
                // Threshold exceeded is notable, boost rate
                (self.config.base_rate * 1.5).min(1.0)
            }
            CheckpointTrigger::TimeInterval => {
                // Regular interval, use base rate
                self.config.base_rate
            }
        };

        // Roll the dice
        if self.rng.r#gen::<f64>() < effective_rate {
            if context.is_session_end {
                SamplingDecision::Heavy
            } else {
                SamplingDecision::Medium
            }
        } else {
            SamplingDecision::Skip
        }
    }

    /// Simple sample check without full context.
    ///
    /// Uses base_rate only, useful for quick decisions.
    pub fn should_sample_simple(&mut self) -> bool {
        self.rng.r#gen::<f64>() < self.config.base_rate
    }
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        Self::new(SamplingConfig::default())
    }
}

impl std::fmt::Debug for SamplingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SamplingStrategy")
            .field("base_rate", &self.config.base_rate)
            .field("burnin_sessions", &self.config.burnin_sessions)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_decision_variants() {
        assert!(SamplingDecision::Skip.is_skip());
        assert!(!SamplingDecision::Skip.is_medium());
        assert!(!SamplingDecision::Skip.is_heavy());

        assert!(!SamplingDecision::Medium.is_skip());
        assert!(SamplingDecision::Medium.is_medium());
        assert!(!SamplingDecision::Medium.is_heavy());

        assert!(!SamplingDecision::Heavy.is_skip());
        assert!(!SamplingDecision::Heavy.is_medium());
        assert!(SamplingDecision::Heavy.is_heavy());
    }

    #[test]
    fn test_sampling_decision_display() {
        assert_eq!(SamplingDecision::Skip.to_string(), "skip");
        assert_eq!(SamplingDecision::Medium.to_string(), "medium");
        assert_eq!(SamplingDecision::Heavy.to_string(), "heavy");
    }

    #[test]
    fn test_sampling_during_burnin_always_samples() {
        let config = SamplingConfig {
            base_rate: 0.0, // Would never sample without burnin
            burnin_sessions: 10,
        };
        let mut strategy = SamplingStrategy::new(config);

        // During burnin (completed_sessions < burnin_sessions)
        let ctx =
            SamplingContext::checkpoint(CheckpointTrigger::TimeInterval).with_completed_sessions(5);

        let decision = strategy.should_sample(&ctx);
        assert_eq!(decision, SamplingDecision::Medium);
    }

    #[test]
    fn test_sampling_session_end_during_burnin_is_heavy() {
        let config = SamplingConfig {
            base_rate: 0.0,
            burnin_sessions: 10,
        };
        let mut strategy = SamplingStrategy::new(config);

        let ctx = SamplingContext::session_end().with_completed_sessions(5);

        let decision = strategy.should_sample(&ctx);
        assert_eq!(decision, SamplingDecision::Heavy);
    }

    #[test]
    fn test_sampling_high_frustration_always_samples() {
        let config = SamplingConfig {
            base_rate: 0.0,     // Would never sample without frustration
            burnin_sessions: 0, // No burnin
        };
        let mut strategy = SamplingStrategy::new(config);

        // High frustration (>= 0.7)
        let ctx = SamplingContext::checkpoint(CheckpointTrigger::TimeInterval)
            .with_frustration_ema(0.8)
            .with_completed_sessions(100);

        let decision = strategy.should_sample(&ctx);
        assert_eq!(decision, SamplingDecision::Medium);
    }

    #[test]
    fn test_sampling_respects_base_rate() {
        let config = SamplingConfig {
            base_rate: 0.5, // 50% sample rate
            burnin_sessions: 0,
        };
        // Use fixed seed for reproducibility
        let mut strategy = SamplingStrategy::with_seed(config, 42);

        let ctx = SamplingContext::checkpoint(CheckpointTrigger::TimeInterval)
            .with_completed_sessions(100);

        // Run multiple samples
        let mut medium_count = 0;
        let mut skip_count = 0;
        for _ in 0..100 {
            match strategy.should_sample(&ctx) {
                SamplingDecision::Medium => medium_count += 1,
                SamplingDecision::Skip => skip_count += 1,
                SamplingDecision::Heavy => unreachable!("checkpoint should not return heavy"),
            }
        }

        // With 50% rate, should be roughly split (allow wide margin for randomness)
        assert!(medium_count > 20, "should have some medium samples");
        assert!(skip_count > 20, "should have some skips");
    }

    #[test]
    fn test_sampling_pattern_match_boosts_rate() {
        let config = SamplingConfig {
            base_rate: 0.2, // 20% base rate
            burnin_sessions: 0,
        };

        let pattern_ctx = SamplingContext::checkpoint(CheckpointTrigger::PatternMatch {
            pattern: "test".to_string(),
        })
        .with_completed_sessions(100);

        let interval_ctx = SamplingContext::checkpoint(CheckpointTrigger::TimeInterval)
            .with_completed_sessions(100);

        // Run samples for both
        let mut pattern_medium = 0;
        let mut interval_medium = 0;

        // Use same seed for fair comparison
        let mut strategy1 = SamplingStrategy::with_seed(config.clone(), 42);
        for _ in 0..100 {
            if strategy1.should_sample(&pattern_ctx).is_medium() {
                pattern_medium += 1;
            }
        }

        let mut strategy2 = SamplingStrategy::with_seed(config, 42);
        for _ in 0..100 {
            if strategy2.should_sample(&interval_ctx).is_medium() {
                interval_medium += 1;
            }
        }

        // Pattern match should sample more often (boosted rate)
        assert!(
            pattern_medium > interval_medium,
            "pattern match should have higher sample rate"
        );
    }

    #[test]
    fn test_sampling_zero_rate_after_burnin_skips() {
        let config = SamplingConfig {
            base_rate: 0.0,     // Never sample
            burnin_sessions: 0, // No burnin
        };
        let mut strategy = SamplingStrategy::new(config);

        let ctx = SamplingContext::checkpoint(CheckpointTrigger::TimeInterval)
            .with_frustration_ema(0.3) // Normal frustration
            .with_completed_sessions(100);

        // Should always skip with 0 rate
        for _ in 0..10 {
            let decision = strategy.should_sample(&ctx);
            assert_eq!(decision, SamplingDecision::Skip);
        }
    }

    #[test]
    fn test_sampling_full_rate_always_samples() {
        let config = SamplingConfig {
            base_rate: 1.0, // Always sample
            burnin_sessions: 0,
        };
        let mut strategy = SamplingStrategy::new(config);

        let ctx = SamplingContext::checkpoint(CheckpointTrigger::TimeInterval)
            .with_completed_sessions(100);

        // Should always sample with 100% rate
        for _ in 0..10 {
            let decision = strategy.should_sample(&ctx);
            assert_eq!(decision, SamplingDecision::Medium);
        }
    }

    #[test]
    fn test_sampling_simple_check() {
        let config = SamplingConfig {
            base_rate: 0.5,
            burnin_sessions: 0,
        };
        let mut strategy = SamplingStrategy::with_seed(config, 42);

        let mut true_count = 0;
        for _ in 0..100 {
            if strategy.should_sample_simple() {
                true_count += 1;
            }
        }

        // Should be roughly 50%
        assert!(true_count > 30 && true_count < 70);
    }

    #[test]
    fn test_sampling_context_builders() {
        let checkpoint_ctx = SamplingContext::checkpoint(CheckpointTrigger::TimeInterval)
            .with_frustration_ema(0.5)
            .with_completed_sessions(10);

        assert!(!checkpoint_ctx.is_session_end);
        assert!((checkpoint_ctx.frustration_ema - 0.5).abs() < f64::EPSILON);
        assert_eq!(checkpoint_ctx.completed_sessions, 10);

        let session_end_ctx = SamplingContext::session_end();
        assert!(session_end_ctx.is_session_end);
    }

    #[test]
    fn test_sampling_strategy_debug() {
        let strategy = SamplingStrategy::default();
        let debug = format!("{strategy:?}");
        assert!(debug.contains("SamplingStrategy"));
        assert!(debug.contains("base_rate"));
        assert!(debug.contains("burnin_sessions"));
    }

    #[test]
    fn test_sampling_strategy_accessors() {
        let config = SamplingConfig {
            base_rate: 0.3,
            burnin_sessions: 5,
        };
        let strategy = SamplingStrategy::new(config);

        assert!((strategy.base_rate() - 0.3).abs() < f64::EPSILON);
        assert_eq!(strategy.burnin_sessions(), 5);
    }

    #[test]
    fn test_sampling_decision_serialization() {
        let decisions = [
            SamplingDecision::Skip,
            SamplingDecision::Medium,
            SamplingDecision::Heavy,
        ];

        for decision in decisions {
            let json = serde_json::to_string(&decision).expect("should serialize");
            let parsed: SamplingDecision = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(parsed, decision);
        }
    }
}
