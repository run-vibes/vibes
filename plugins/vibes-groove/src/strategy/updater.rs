//! Distribution updater for strategy feedback loop
//!
//! Updates category distributions and per-learning overrides based on
//! strategy outcomes, handling specialization when learnings accumulate
//! enough data.

use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::types::{Learning, LearningCategory, LearningId};

use super::learner::SessionContext;
use super::types::{
    ContextType, InjectionStrategy, LearningStrategyOverride, StrategyDistribution,
    StrategyOutcome, StrategyVariant,
};

/// Configuration for the distribution updater
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterConfig {
    /// Sessions before a learning specializes (default: 20)
    pub specialization_threshold: u32,
    /// Minimum confidence to trigger specialization (default: 0.6)
    pub specialization_confidence: f64,
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            specialization_threshold: 20,
            specialization_confidence: 0.6,
        }
    }
}

/// Updates distributions based on strategy outcomes
pub struct DistributionUpdater {
    config: UpdaterConfig,
}

impl Default for DistributionUpdater {
    fn default() -> Self {
        Self::new(UpdaterConfig::default())
    }
}

impl DistributionUpdater {
    /// Create a new distribution updater with the given configuration
    pub fn new(config: UpdaterConfig) -> Self {
        Self { config }
    }

    /// Update distributions based on a strategy outcome
    ///
    /// Updates both the category distribution and the per-learning override,
    /// potentially triggering specialization if the learning has accumulated
    /// enough data.
    pub fn update(
        &self,
        distributions: &mut HashMap<(LearningCategory, ContextType), StrategyDistribution>,
        overrides: &mut HashMap<LearningId, LearningStrategyOverride>,
        learning: &Learning,
        context: &SessionContext,
        strategy: &InjectionStrategy,
        outcome: &StrategyOutcome,
    ) {
        let variant = StrategyVariant::from(strategy);
        let key = (learning.category.clone(), context.context_type);

        // Always update category distribution
        self.update_category_distribution(distributions, &key, variant, outcome);

        // Update or create learning override
        self.update_learning_override(overrides, distributions, learning, &key, variant, outcome);
    }

    /// Update the category distribution for the given variant
    fn update_category_distribution(
        &self,
        distributions: &mut HashMap<(LearningCategory, ContextType), StrategyDistribution>,
        key: &(LearningCategory, ContextType),
        variant: StrategyVariant,
        outcome: &StrategyOutcome,
    ) {
        if let Some(dist) = distributions.get_mut(key) {
            // update_weight handles the conversion from [-1, 1] to [0, 1]
            dist.update_weight(variant, outcome.value, outcome.confidence);
            dist.session_count += 1;
        }
    }

    /// Update the learning override, potentially triggering specialization
    fn update_learning_override(
        &self,
        overrides: &mut HashMap<LearningId, LearningStrategyOverride>,
        distributions: &HashMap<(LearningCategory, ContextType), StrategyDistribution>,
        learning: &Learning,
        key: &(LearningCategory, ContextType),
        variant: StrategyVariant,
        outcome: &StrategyOutcome,
    ) {
        // Create override if doesn't exist
        let override_ = overrides.entry(learning.id).or_insert_with(|| {
            LearningStrategyOverride::new(learning.id, learning.category.clone())
        });

        override_.session_count += 1;
        override_.updated_at = Utc::now();

        // Check if should specialize
        if override_.session_count >= self.config.specialization_threshold
            && override_.specialized_weights.is_none()
            && outcome.confidence >= self.config.specialization_confidence
            && let Some(dist) = distributions.get(key)
        {
            override_.specialize_from(dist);
        }

        // Update specialized weights if they exist
        if let Some(ref mut specialized) = override_.specialized_weights
            && let Some(param) = specialized.get_mut(&variant)
        {
            // Convert outcome value from [-1, 1] to [0, 1] for AdaptiveParam
            let normalized = (outcome.value.clamp(-1.0, 1.0) + 1.0) / 2.0;
            param.update(normalized, outcome.confidence);
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &UpdaterConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::SessionId;
    use crate::types::{LearningContent, LearningSource, Scope};
    use chrono::Utc;
    use uuid::Uuid;

    fn test_learning() -> Learning {
        Learning {
            id: Uuid::now_v7(),
            scope: Scope::Project("test-project".into()),
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
            position: super::super::types::ContextPosition::Prefix,
            format: super::super::types::InjectionFormat::Plain,
        }
    }

    fn test_outcome(value: f64, confidence: f64) -> StrategyOutcome {
        StrategyOutcome::new(value, confidence, super::super::types::OutcomeSource::Both)
    }

    fn setup_distributions(
        category: LearningCategory,
        context_type: ContextType,
    ) -> HashMap<(LearningCategory, ContextType), StrategyDistribution> {
        let mut distributions = HashMap::new();
        distributions.insert(
            (category.clone(), context_type),
            StrategyDistribution::new(category, context_type),
        );
        distributions
    }

    #[test]
    fn test_category_distribution_updated() {
        let updater = DistributionUpdater::default();
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(0.8, 0.9);

        let mut distributions =
            setup_distributions(learning.category.clone(), context.context_type);
        let mut overrides = HashMap::new();

        let key = (learning.category.clone(), context.context_type);
        let before = distributions.get(&key).unwrap().session_count;

        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &outcome,
        );

        let after = distributions.get(&key).unwrap().session_count;
        assert_eq!(after, before + 1, "Session count should increment");
    }

    #[test]
    fn test_category_weight_updated_on_positive_outcome() {
        let updater = DistributionUpdater::default();
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(1.0, 1.0); // Strong positive outcome

        let mut distributions =
            setup_distributions(learning.category.clone(), context.context_type);
        let mut overrides = HashMap::new();

        let key = (learning.category.clone(), context.context_type);
        let before = distributions
            .get(&key)
            .unwrap()
            .get_weight(StrategyVariant::MainContext)
            .unwrap()
            .value;

        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &outcome,
        );

        let after = distributions
            .get(&key)
            .unwrap()
            .get_weight(StrategyVariant::MainContext)
            .unwrap()
            .value;

        assert!(after > before, "Positive outcome should increase weight");
    }

    #[test]
    fn test_learning_override_created_on_first_encounter() {
        let updater = DistributionUpdater::default();
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(0.5, 0.7);

        let mut distributions =
            setup_distributions(learning.category.clone(), context.context_type);
        let mut overrides = HashMap::new();

        assert!(overrides.is_empty());

        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &outcome,
        );

        assert!(overrides.contains_key(&learning.id));
        assert_eq!(overrides.get(&learning.id).unwrap().session_count, 1);
    }

    #[test]
    fn test_session_count_increments_on_each_update() {
        let updater = DistributionUpdater::default();
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(0.5, 0.7);

        let mut distributions =
            setup_distributions(learning.category.clone(), context.context_type);
        let mut overrides = HashMap::new();

        for i in 1..=5 {
            updater.update(
                &mut distributions,
                &mut overrides,
                &learning,
                &context,
                &strategy,
                &outcome,
            );

            assert_eq!(overrides.get(&learning.id).unwrap().session_count, i);
        }
    }

    #[test]
    fn test_specialization_triggers_at_threshold() {
        let config = UpdaterConfig {
            specialization_threshold: 5, // Lower threshold for testing
            specialization_confidence: 0.6,
        };
        let updater = DistributionUpdater::new(config);
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(0.5, 0.8); // Confidence > 0.6

        let mut distributions =
            setup_distributions(learning.category.clone(), context.context_type);
        let mut overrides = HashMap::new();

        // Update 4 times - should not specialize
        for _ in 0..4 {
            updater.update(
                &mut distributions,
                &mut overrides,
                &learning,
                &context,
                &strategy,
                &outcome,
            );
        }

        assert!(
            !overrides.get(&learning.id).unwrap().is_specialized(),
            "Should not specialize before threshold"
        );

        // Update 5th time - should trigger specialization
        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &outcome,
        );

        assert!(
            overrides.get(&learning.id).unwrap().is_specialized(),
            "Should specialize at threshold"
        );
    }

    #[test]
    fn test_specialization_requires_minimum_confidence() {
        let config = UpdaterConfig {
            specialization_threshold: 3,
            specialization_confidence: 0.8, // High confidence requirement
        };
        let updater = DistributionUpdater::new(config);
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let low_confidence_outcome = test_outcome(0.5, 0.5); // Below threshold

        let mut distributions =
            setup_distributions(learning.category.clone(), context.context_type);
        let mut overrides = HashMap::new();

        // Update past threshold with low confidence
        for _ in 0..5 {
            updater.update(
                &mut distributions,
                &mut overrides,
                &learning,
                &context,
                &strategy,
                &low_confidence_outcome,
            );
        }

        assert!(
            !overrides.get(&learning.id).unwrap().is_specialized(),
            "Low confidence should not trigger specialization"
        );

        // Update with high confidence - should now specialize
        let high_confidence_outcome = test_outcome(0.5, 0.9);
        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &high_confidence_outcome,
        );

        assert!(
            overrides.get(&learning.id).unwrap().is_specialized(),
            "High confidence should trigger specialization"
        );
    }

    #[test]
    fn test_specialized_weights_updated_independently() {
        let config = UpdaterConfig {
            specialization_threshold: 2,
            specialization_confidence: 0.5,
        };
        let updater = DistributionUpdater::new(config);
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(0.5, 0.7);

        let mut distributions =
            setup_distributions(learning.category.clone(), context.context_type);
        let mut overrides = HashMap::new();

        // Trigger specialization
        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &outcome,
        );
        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &outcome,
        );

        assert!(overrides.get(&learning.id).unwrap().is_specialized());

        // Get specialized weight before update
        let before = overrides
            .get(&learning.id)
            .unwrap()
            .specialized_weights
            .as_ref()
            .unwrap()
            .get(&StrategyVariant::MainContext)
            .unwrap()
            .value;

        // Update with strong positive outcome
        let strong_outcome = test_outcome(1.0, 1.0);
        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &strong_outcome,
        );

        let after = overrides
            .get(&learning.id)
            .unwrap()
            .specialized_weights
            .as_ref()
            .unwrap()
            .get(&StrategyVariant::MainContext)
            .unwrap()
            .value;

        assert!(
            after > before,
            "Specialized weight should update on positive outcome"
        );
    }

    #[test]
    fn test_no_update_when_distribution_missing() {
        let updater = DistributionUpdater::default();
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(0.5, 0.7);

        let mut distributions = HashMap::new(); // Empty - no distribution for this category
        let mut overrides = HashMap::new();

        // Should not panic, just skip category update
        updater.update(
            &mut distributions,
            &mut overrides,
            &learning,
            &context,
            &strategy,
            &outcome,
        );

        // Override should still be created
        assert!(overrides.contains_key(&learning.id));
    }

    #[test]
    fn test_multiple_learnings_independent() {
        let updater = DistributionUpdater::default();
        let learning1 = test_learning();
        let mut learning2 = test_learning();
        learning2.id = Uuid::now_v7(); // Different ID

        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome(0.5, 0.7);

        let mut distributions =
            setup_distributions(LearningCategory::CodePattern, context.context_type);
        let mut overrides = HashMap::new();

        updater.update(
            &mut distributions,
            &mut overrides,
            &learning1,
            &context,
            &strategy,
            &outcome,
        );
        updater.update(
            &mut distributions,
            &mut overrides,
            &learning2,
            &context,
            &strategy,
            &outcome,
        );

        assert_eq!(overrides.len(), 2);
        assert_eq!(overrides.get(&learning1.id).unwrap().session_count, 1);
        assert_eq!(overrides.get(&learning2.id).unwrap().session_count, 1);
    }
}
