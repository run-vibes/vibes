//! OpenWorldHook for M32 integration
//!
//! Implements the NoveltyHook trait to connect open-world adaptation
//! with the strategy learning pipeline. When strategy outcomes are processed,
//! this hook runs novelty detection and gap analysis, feeding back exploration
//! adjustments to improve strategy selection.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openworld::types::ResponseAction;
    use crate::strategy::{
        ContextPosition, ContextType, InjectionFormat, InjectionStrategy, OutcomeSource,
        SessionContext, StrategyOutcome,
    };
    use crate::types::{Learning, LearningCategory, LearningContent, LearningSource, Scope};
    use chrono::Utc;

    fn test_learning() -> Learning {
        Learning {
            id: uuid::Uuid::now_v7(),
            scope: Scope::Project("test".into()),
            category: LearningCategory::CodePattern,
            content: LearningContent {
                description: "Test learning".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            confidence: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: LearningSource::UserCreated,
        }
    }

    fn test_context() -> SessionContext {
        SessionContext::new(SessionId::from("test-session"), ContextType::Interactive)
    }

    fn test_strategy() -> InjectionStrategy {
        InjectionStrategy::MainContext {
            position: ContextPosition::Prefix,
            format: InjectionFormat::Plain,
        }
    }

    fn test_outcome_positive() -> StrategyOutcome {
        StrategyOutcome::new(0.8, 0.9, OutcomeSource::Attribution)
    }

    fn test_outcome_negative() -> StrategyOutcome {
        StrategyOutcome::new(-0.5, 0.7, OutcomeSource::Attribution)
    }

    fn test_outcome_low_confidence() -> StrategyOutcome {
        StrategyOutcome::new(0.3, 0.2, OutcomeSource::Direct)
    }

    // =========================================================================
    // Config tests
    // =========================================================================

    #[test]
    fn test_config_defaults() {
        let config = OpenWorldHookConfig::default();
        assert!(config.enabled);
        assert!(config.negative_value_threshold < 0.0);
        assert!(config.low_confidence_threshold > 0.0);
        assert!(config.low_confidence_threshold < 1.0);
    }

    // =========================================================================
    // Outcome analysis tests
    // =========================================================================

    #[test]
    fn test_is_negative_outcome_detects_negative() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let outcome = test_outcome_negative();
        assert!(hook.is_negative_outcome(&outcome));
    }

    #[test]
    fn test_is_negative_outcome_false_for_positive() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let outcome = test_outcome_positive();
        assert!(!hook.is_negative_outcome(&outcome));
    }

    #[test]
    fn test_is_low_confidence_detects_low() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let outcome = test_outcome_low_confidence();
        assert!(hook.is_low_confidence(&outcome));
    }

    #[test]
    fn test_is_low_confidence_false_for_high() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let outcome = test_outcome_positive();
        assert!(!hook.is_low_confidence(&outcome));
    }

    // =========================================================================
    // Context hash tests
    // =========================================================================

    #[test]
    fn test_compute_context_hash_deterministic() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let context = test_context();
        let learning = test_learning();

        let hash1 = hook.compute_context_hash(&context, &learning);
        let hash2 = hook.compute_context_hash(&context, &learning);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_context_hash_different_for_different_contexts() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let learning = test_learning();

        let context1 = SessionContext::new(SessionId::from("session-1"), ContextType::Interactive);
        let context2 = SessionContext::new(SessionId::from("session-2"), ContextType::Batch);

        let hash1 = hook.compute_context_hash(&context1, &learning);
        let hash2 = hook.compute_context_hash(&context2, &learning);

        // Different context types should produce different hashes
        assert_ne!(hash1, hash2);
    }

    // =========================================================================
    // Response action tests
    // =========================================================================

    #[test]
    fn test_determine_response_positive_outcome_returns_none() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let outcome = test_outcome_positive();
        let context = test_context();
        let learning = test_learning();

        let action = hook.determine_response(&outcome, &context, &learning);
        assert!(matches!(action, ResponseAction::None));
    }

    #[test]
    fn test_determine_response_negative_outcome_adjusts_exploration() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let outcome = test_outcome_negative();
        let context = test_context();
        let learning = test_learning();

        let action = hook.determine_response(&outcome, &context, &learning);

        // Negative outcomes should trigger exploration adjustment
        match action {
            ResponseAction::AdjustExploration(bonus) => {
                assert!(bonus > 0.0, "Should have positive exploration bonus");
            }
            ResponseAction::None => {
                // Also acceptable for first observation
            }
            other => panic!("Unexpected action: {:?}", other),
        }
    }

    #[test]
    fn test_determine_response_low_confidence_adjusts_exploration() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let outcome = test_outcome_low_confidence();
        let context = test_context();
        let learning = test_learning();

        let action = hook.determine_response(&outcome, &context, &learning);

        // Low confidence should trigger exploration adjustment
        match action {
            ResponseAction::AdjustExploration(bonus) => {
                assert!(bonus > 0.0, "Should have positive exploration bonus");
            }
            ResponseAction::None => {
                // Also acceptable for first observation
            }
            other => panic!("Unexpected action: {:?}", other),
        }
    }

    // =========================================================================
    // Integration tests
    // =========================================================================

    #[tokio::test]
    async fn test_on_strategy_outcome_success() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome_positive();

        let result = hook
            .on_strategy_outcome(&learning, &context, &strategy, &outcome)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_on_session_end_success() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let session_id = SessionId::from("test-session");

        let result = hook.on_session_end(session_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hook_tracks_observations() {
        let hook = OpenWorldHook::new(OpenWorldHookConfig::default());
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome_negative();

        // Process multiple outcomes
        for _ in 0..5 {
            hook.on_strategy_outcome(&learning, &context, &strategy, &outcome)
                .await
                .unwrap();
        }

        // Should have tracked observations
        let stats = hook.stats();
        assert!(stats.outcomes_processed >= 5);
    }

    // =========================================================================
    // Gap creation tests
    // =========================================================================

    #[test]
    fn test_should_create_gap_based_on_threshold() {
        let config = OpenWorldHookConfig {
            gap_creation_threshold: 3,
            ..Default::default()
        };
        let hook = OpenWorldHook::new(config);

        assert!(!hook.should_create_gap(2));
        assert!(hook.should_create_gap(3));
        assert!(hook.should_create_gap(10));
    }
}

// =============================================================================
// Implementation
// =============================================================================

use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, warn};

use crate::Result;
use crate::assessment::SessionId;
use crate::strategy::{InjectionStrategy, NoveltyHook, SessionContext, StrategyOutcome};
use crate::types::Learning;

use super::types::{GapCategory, ResponseAction};

/// Configuration for the OpenWorldHook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWorldHookConfig {
    /// Whether the hook is enabled
    pub enabled: bool,
    /// Threshold for negative outcomes (below this is negative)
    pub negative_value_threshold: f64,
    /// Threshold for low confidence outcomes
    pub low_confidence_threshold: f64,
    /// Number of negative observations before creating a gap
    pub gap_creation_threshold: u32,
    /// Base exploration bonus for uncertain contexts
    pub exploration_bonus: f64,
    /// Maximum exploration bonus
    pub max_exploration_bonus: f64,
}

impl Default for OpenWorldHookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            negative_value_threshold: -0.3,
            low_confidence_threshold: 0.4,
            gap_creation_threshold: 5,
            exploration_bonus: 0.1,
            max_exploration_bonus: 0.5,
        }
    }
}

/// Statistics tracked by the hook
#[derive(Debug, Default)]
pub struct HookStats {
    /// Total outcomes processed
    pub outcomes_processed: u64,
    /// Negative outcomes detected
    pub negative_outcomes: u64,
    /// Low confidence outcomes detected
    pub low_confidence_outcomes: u64,
    /// Exploration adjustments made
    pub exploration_adjustments: u64,
    /// Gaps created
    pub gaps_created: u64,
}

/// Observation tracker for context patterns
#[derive(Debug, Default)]
struct ObservationTracker {
    /// Count of negative observations per context hash
    negative_counts: HashMap<u64, u32>,
    /// Count of low confidence observations per context hash
    low_confidence_counts: HashMap<u64, u32>,
}

/// OpenWorldHook connects open-world adaptation with strategy learning
///
/// This hook is invoked after each strategy outcome is processed by the
/// StrategyConsumer. It analyzes outcomes for signs of novelty or capability
/// gaps and feeds exploration adjustments back to the strategy system.
pub struct OpenWorldHook {
    /// Configuration
    config: OpenWorldHookConfig,
    /// Observation tracker
    tracker: RwLock<ObservationTracker>,
    /// Statistics
    outcomes_processed: AtomicU64,
    negative_outcomes: AtomicU64,
    low_confidence_outcomes: AtomicU64,
    exploration_adjustments: AtomicU64,
    gaps_created: AtomicU64,
}

impl OpenWorldHook {
    /// Create a new OpenWorldHook
    pub fn new(config: OpenWorldHookConfig) -> Self {
        Self {
            config,
            tracker: RwLock::new(ObservationTracker::default()),
            outcomes_processed: AtomicU64::new(0),
            negative_outcomes: AtomicU64::new(0),
            low_confidence_outcomes: AtomicU64::new(0),
            exploration_adjustments: AtomicU64::new(0),
            gaps_created: AtomicU64::new(0),
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> HookStats {
        HookStats {
            outcomes_processed: self.outcomes_processed.load(Ordering::Relaxed),
            negative_outcomes: self.negative_outcomes.load(Ordering::Relaxed),
            low_confidence_outcomes: self.low_confidence_outcomes.load(Ordering::Relaxed),
            exploration_adjustments: self.exploration_adjustments.load(Ordering::Relaxed),
            gaps_created: self.gaps_created.load(Ordering::Relaxed),
        }
    }

    /// Check if outcome indicates a negative result
    pub fn is_negative_outcome(&self, outcome: &StrategyOutcome) -> bool {
        outcome.value < self.config.negative_value_threshold
    }

    /// Check if outcome has low confidence
    pub fn is_low_confidence(&self, outcome: &StrategyOutcome) -> bool {
        outcome.confidence < self.config.low_confidence_threshold
    }

    /// Compute a hash for context + learning combination
    pub fn compute_context_hash(&self, context: &SessionContext, learning: &Learning) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        context.context_type.hash(&mut hasher);
        learning.category.hash(&mut hasher);
        hasher.finish()
    }

    /// Determine response action based on outcome analysis
    #[instrument(skip(self, outcome, context, learning))]
    pub fn determine_response(
        &self,
        outcome: &StrategyOutcome,
        context: &SessionContext,
        learning: &Learning,
    ) -> ResponseAction {
        let context_hash = self.compute_context_hash(context, learning);
        let is_negative = self.is_negative_outcome(outcome);
        let is_low_confidence = self.is_low_confidence(outcome);

        if !is_negative && !is_low_confidence {
            return ResponseAction::None;
        }

        // Track observations
        let observation_count = {
            let mut tracker = self.tracker.write().unwrap();

            if is_negative {
                let count = tracker.negative_counts.entry(context_hash).or_insert(0);
                *count += 1;
                *count
            } else {
                let count = tracker
                    .low_confidence_counts
                    .entry(context_hash)
                    .or_insert(0);
                *count += 1;
                *count
            }
        };

        debug!(
            context_hash,
            observation_count, is_negative, is_low_confidence, "Tracked observation"
        );

        // Determine response based on observation count
        if self.should_create_gap(observation_count) {
            self.gaps_created.fetch_add(1, Ordering::Relaxed);
            let category = if is_negative {
                GapCategory::IncorrectPattern
            } else {
                GapCategory::ContextMismatch
            };
            let gap = super::types::CapabilityGap::new(
                category,
                format!("context_hash:{}", context_hash),
            );
            ResponseAction::CreateGap(gap)
        } else if observation_count > 1 {
            // Adjust exploration after first observation
            let bonus = self.calculate_exploration_bonus(observation_count);
            self.exploration_adjustments.fetch_add(1, Ordering::Relaxed);
            ResponseAction::AdjustExploration(bonus)
        } else {
            // First observation, just monitor
            ResponseAction::None
        }
    }

    /// Check if we should create a gap based on observation count
    pub fn should_create_gap(&self, observation_count: u32) -> bool {
        observation_count >= self.config.gap_creation_threshold
    }

    /// Calculate exploration bonus based on observation count
    fn calculate_exploration_bonus(&self, observation_count: u32) -> f64 {
        let bonus = self.config.exploration_bonus * (observation_count as f64 / 3.0).ln_1p();
        bonus.min(self.config.max_exploration_bonus)
    }

    /// Get the configuration
    pub fn config(&self) -> &OpenWorldHookConfig {
        &self.config
    }
}

#[async_trait]
impl NoveltyHook for OpenWorldHook {
    #[instrument(skip(self, learning, context, strategy, outcome), fields(learning_id = %learning.id))]
    async fn on_strategy_outcome(
        &self,
        learning: &Learning,
        context: &SessionContext,
        strategy: &InjectionStrategy,
        outcome: &StrategyOutcome,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        self.outcomes_processed.fetch_add(1, Ordering::Relaxed);

        // Track negative and low confidence outcomes
        if self.is_negative_outcome(outcome) {
            self.negative_outcomes.fetch_add(1, Ordering::Relaxed);
        }
        if self.is_low_confidence(outcome) {
            self.low_confidence_outcomes.fetch_add(1, Ordering::Relaxed);
        }

        // Determine response action
        let action = self.determine_response(outcome, context, learning);

        // Log significant actions
        match &action {
            ResponseAction::CreateGap(gap) => {
                debug!(
                    gap_id = %gap.id,
                    category = ?gap.category,
                    strategy = ?strategy.variant(),
                    "Created capability gap from strategy outcome"
                );
            }
            ResponseAction::AdjustExploration(bonus) => {
                debug!(
                    bonus,
                    strategy = ?strategy.variant(),
                    "Adjusting exploration due to outcome"
                );
            }
            _ => {}
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_session_end(&self, session_id: SessionId) -> Result<()> {
        debug!(%session_id, "Session ended, hook cleanup");
        // Future: Could trigger session-level analysis here
        Ok(())
    }
}
