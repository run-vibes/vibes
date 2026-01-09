//! Strategy learner with Thompson sampling
//!
//! Implements lazy strategy selection via Thompson sampling from Beta posteriors.
//! Caches selections per-session for consistency.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::assessment::SessionId;
use crate::types::{Learning, LearningCategory, LearningId};

use super::types::{
    AdaptiveParam, CallbackMethod, ContextPosition, ContextType, DeferralTrigger, InjectionFormat,
    InjectionStrategy, LearningStrategyOverride, StrategyDistribution, StrategyVariant,
    SubagentType, get_effective_weights,
};

/// Context for the current session
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub session_id: SessionId,
    pub context_type: ContextType,
}

impl SessionContext {
    pub fn new(session_id: SessionId, context_type: ContextType) -> Self {
        Self {
            session_id,
            context_type,
        }
    }
}

/// Configuration for the strategy learner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyLearnerConfig {
    /// Bonus added to exploration (UCB-style)
    pub exploration_bonus: f64,
    /// Minimum samples before considering weight confident
    pub min_samples_for_confidence: u32,
    /// Sessions before a learning can specialize
    pub specialization_threshold: u32,
    /// Confidence required to specialize
    pub specialization_confidence: f64,
}

impl Default for StrategyLearnerConfig {
    fn default() -> Self {
        Self {
            exploration_bonus: 0.1,
            min_samples_for_confidence: 5,
            specialization_threshold: 20,
            specialization_confidence: 0.7,
        }
    }
}

/// Strategy learner with Thompson sampling
pub struct StrategyLearner {
    /// Category-level distributions (the priors)
    category_distributions: HashMap<(LearningCategory, ContextType), StrategyDistribution>,

    /// Per-learning overrides (specializations)
    learning_overrides: HashMap<LearningId, LearningStrategyOverride>,

    /// Session cache for lazy + consistent selection
    session_cache: HashMap<(SessionId, LearningId), InjectionStrategy>,

    /// Configuration
    config: StrategyLearnerConfig,
}

impl StrategyLearner {
    /// Create a new strategy learner
    pub fn new(config: StrategyLearnerConfig) -> Self {
        Self {
            category_distributions: HashMap::new(),
            learning_overrides: HashMap::new(),
            session_cache: HashMap::new(),
            config,
        }
    }

    /// Create with pre-loaded distributions
    pub fn with_distributions(
        config: StrategyLearnerConfig,
        distributions: HashMap<(LearningCategory, ContextType), StrategyDistribution>,
        overrides: HashMap<LearningId, LearningStrategyOverride>,
    ) -> Self {
        Self {
            category_distributions: distributions,
            learning_overrides: overrides,
            session_cache: HashMap::new(),
            config,
        }
    }

    /// Select a strategy for a learning in a session context
    ///
    /// Returns cached strategy if one exists for this session+learning,
    /// otherwise samples a new one and caches it.
    pub fn select_strategy(
        &mut self,
        learning: &Learning,
        context: &SessionContext,
    ) -> InjectionStrategy {
        let cache_key = (context.session_id.clone(), learning.id);

        // Return cached if exists
        if let Some(cached) = self.session_cache.get(&cache_key) {
            return cached.clone();
        }

        // Sample new strategy
        let strategy = self.sample_strategy(learning, context);
        self.session_cache.insert(cache_key, strategy.clone());
        strategy
    }

    /// Clear cached strategies for a session
    pub fn clear_session(&mut self, session_id: &SessionId) {
        self.session_cache.retain(|(sid, _), _| sid != session_id);
    }

    /// Get the number of cached strategies
    pub fn cache_size(&self) -> usize {
        self.session_cache.len()
    }

    /// Sample a strategy using Thompson sampling
    fn sample_strategy(&self, learning: &Learning, context: &SessionContext) -> InjectionStrategy {
        let weights = self.get_effective_weights_for(learning, context);

        // Thompson sampling: sample from each Beta posterior, pick highest
        let selected_variant = weights
            .iter()
            .map(|(variant, param)| (*variant, param.sample()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(v, _)| v)
            .unwrap_or(StrategyVariant::Deferred);

        // Sample parameters for the selected strategy
        self.sample_params(selected_variant, learning, context)
    }

    /// Get effective weights for a learning, resolving hierarchy
    fn get_effective_weights_for(
        &self,
        learning: &Learning,
        context: &SessionContext,
    ) -> &HashMap<StrategyVariant, AdaptiveParam> {
        let override_ = self.learning_overrides.get(&learning.id);
        let key = (learning.category.clone(), context.context_type);

        if let Some(dist) = self.category_distributions.get(&key) {
            get_effective_weights(override_, dist)
        } else {
            // No distribution for this category/context - return empty or default
            // For now, we'll panic in tests but could return a static default
            static EMPTY: std::sync::OnceLock<HashMap<StrategyVariant, AdaptiveParam>> =
                std::sync::OnceLock::new();
            EMPTY.get_or_init(|| {
                let mut map = HashMap::new();
                map.insert(
                    StrategyVariant::Deferred,
                    AdaptiveParam::new_with_prior(1.0, 1.0),
                );
                map
            })
        }
    }

    /// Sample parameters for a strategy variant
    fn sample_params(
        &self,
        variant: StrategyVariant,
        _learning: &Learning,
        _context: &SessionContext,
    ) -> InjectionStrategy {
        match variant {
            StrategyVariant::MainContext => InjectionStrategy::MainContext {
                position: self.sample_position(),
                format: self.sample_format(),
            },
            StrategyVariant::Subagent => InjectionStrategy::Subagent {
                agent_type: self.sample_agent_type(),
                blocking: self.sample_blocking(),
                prompt_template: None,
            },
            StrategyVariant::BackgroundSubagent => InjectionStrategy::BackgroundSubagent {
                agent_type: self.sample_agent_type(),
                callback: CallbackMethod::Poll,
                timeout_ms: 30_000,
            },
            StrategyVariant::Deferred => InjectionStrategy::Deferred {
                trigger: DeferralTrigger::Explicit,
                max_wait_ms: None,
            },
        }
    }

    /// Sample context position
    fn sample_position(&self) -> ContextPosition {
        // Simple random selection for now
        use rand::Rng;
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => ContextPosition::Prefix,
            1 => ContextPosition::Suffix,
            _ => ContextPosition::Contextual,
        }
    }

    /// Sample injection format
    fn sample_format(&self) -> InjectionFormat {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => InjectionFormat::Plain,
            1 => InjectionFormat::Tagged,
            _ => InjectionFormat::SystemInstruction,
        }
    }

    /// Sample subagent type
    fn sample_agent_type(&self) -> SubagentType {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => SubagentType::General,
            1 => SubagentType::Explorer,
            _ => SubagentType::Planner,
        }
    }

    /// Sample blocking preference
    fn sample_blocking(&self) -> bool {
        use rand::Rng;
        rand::thread_rng().gen_bool(0.5)
    }

    /// Ensure a distribution exists for a category/context pair
    pub fn ensure_distribution(&mut self, category: LearningCategory, context_type: ContextType) {
        let key = (category.clone(), context_type);
        self.category_distributions
            .entry(key)
            .or_insert_with(|| StrategyDistribution::new(category, context_type));
    }

    /// Get distributions (for persistence)
    pub fn distributions(&self) -> &HashMap<(LearningCategory, ContextType), StrategyDistribution> {
        &self.category_distributions
    }

    /// Get mutable distributions
    pub fn distributions_mut(
        &mut self,
    ) -> &mut HashMap<(LearningCategory, ContextType), StrategyDistribution> {
        &mut self.category_distributions
    }

    /// Get overrides (for persistence)
    pub fn overrides(&self) -> &HashMap<LearningId, LearningStrategyOverride> {
        &self.learning_overrides
    }

    /// Get mutable overrides
    pub fn overrides_mut(&mut self) -> &mut HashMap<LearningId, LearningStrategyOverride> {
        &mut self.learning_overrides
    }

    /// Get configuration
    pub fn config(&self) -> &StrategyLearnerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn test_learning() -> Learning {
        Learning {
            id: Uuid::now_v7(),
            scope: crate::types::Scope::Project("test-project".into()),
            category: LearningCategory::CodePattern,
            content: crate::types::LearningContent {
                description: "Test learning".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            confidence: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: crate::types::LearningSource::UserCreated,
        }
    }

    fn test_context() -> SessionContext {
        SessionContext::new(SessionId::from("test-session"), ContextType::Interactive)
    }

    #[test]
    fn test_cache_returns_same_strategy_within_session() {
        let config = StrategyLearnerConfig::default();
        let mut learner = StrategyLearner::new(config);

        let learning = test_learning();
        let context = test_context();

        // Ensure distribution exists
        learner.ensure_distribution(learning.category.clone(), context.context_type);

        // First selection
        let strategy1 = learner.select_strategy(&learning, &context);

        // Second selection should return the same strategy (cached)
        let strategy2 = learner.select_strategy(&learning, &context);

        assert_eq!(strategy1.variant(), strategy2.variant());
        // The full strategy should be identical since it's cached
        assert_eq!(learner.cache_size(), 1);
    }

    #[test]
    fn test_different_sessions_get_different_cache_entries() {
        let config = StrategyLearnerConfig::default();
        let mut learner = StrategyLearner::new(config);

        let learning = test_learning();
        let context1 = SessionContext::new(SessionId::from("session-1"), ContextType::Interactive);
        let context2 = SessionContext::new(SessionId::from("session-2"), ContextType::Interactive);

        learner.ensure_distribution(learning.category.clone(), ContextType::Interactive);

        let _strategy1 = learner.select_strategy(&learning, &context1);
        let _strategy2 = learner.select_strategy(&learning, &context2);

        // Each session gets its own cache entry
        assert_eq!(learner.cache_size(), 2);
    }

    #[test]
    fn test_clear_session_removes_cached_strategies() {
        let config = StrategyLearnerConfig::default();
        let mut learner = StrategyLearner::new(config);

        let learning = test_learning();
        let context = test_context();

        learner.ensure_distribution(learning.category.clone(), context.context_type);

        let _strategy = learner.select_strategy(&learning, &context);
        assert_eq!(learner.cache_size(), 1);

        learner.clear_session(&context.session_id);
        assert_eq!(learner.cache_size(), 0);
    }

    #[test]
    fn test_thompson_sampling_returns_valid_variant() {
        let config = StrategyLearnerConfig::default();
        let mut learner = StrategyLearner::new(config);

        let learning = test_learning();
        let context = test_context();

        learner.ensure_distribution(learning.category.clone(), context.context_type);

        // Sample many times to exercise Thompson sampling
        for _ in 0..10 {
            learner.clear_session(&context.session_id);
            let strategy = learner.select_strategy(&learning, &context);

            // Should return one of the valid variants
            let variant = strategy.variant();
            assert!(
                variant == StrategyVariant::MainContext
                    || variant == StrategyVariant::Subagent
                    || variant == StrategyVariant::BackgroundSubagent
                    || variant == StrategyVariant::Deferred
            );
        }
    }

    #[test]
    fn test_hierarchy_uses_override_when_specialized() {
        let config = StrategyLearnerConfig::default();

        let learning = test_learning();
        let context = test_context();

        // Create a category distribution
        let dist = StrategyDistribution::new(learning.category.clone(), context.context_type);

        // Create a specialized override with heavily biased weights toward Deferred
        let mut override_ = LearningStrategyOverride::new(learning.id, learning.category.clone());
        override_.specialize_from(&dist);

        // Modify specialized weights to heavily favor Deferred
        if let Some(ref mut weights) = override_.specialized_weights {
            weights.insert(
                StrategyVariant::Deferred,
                AdaptiveParam::new_with_prior(100.0, 1.0),
            );
            weights.insert(
                StrategyVariant::MainContext,
                AdaptiveParam::new_with_prior(1.0, 100.0),
            );
            weights.insert(
                StrategyVariant::Subagent,
                AdaptiveParam::new_with_prior(1.0, 100.0),
            );
            weights.insert(
                StrategyVariant::BackgroundSubagent,
                AdaptiveParam::new_with_prior(1.0, 100.0),
            );
        }

        let mut distributions = HashMap::new();
        distributions.insert((learning.category.clone(), context.context_type), dist);

        let mut overrides = HashMap::new();
        overrides.insert(learning.id, override_);

        let mut learner = StrategyLearner::with_distributions(config, distributions, overrides);

        // With heavily biased weights, should almost always select Deferred
        let mut deferred_count = 0;
        for _ in 0..20 {
            learner.clear_session(&context.session_id);
            let strategy = learner.select_strategy(&learning, &context);
            if strategy.variant() == StrategyVariant::Deferred {
                deferred_count += 1;
            }
        }

        // With 100:1 prior ratio, should see Deferred most of the time
        assert!(
            deferred_count > 15,
            "Expected mostly Deferred, got {deferred_count}/20"
        );
    }

    #[test]
    fn test_ensure_distribution_creates_if_missing() {
        let config = StrategyLearnerConfig::default();
        let mut learner = StrategyLearner::new(config);

        assert!(learner.distributions().is_empty());

        learner.ensure_distribution(LearningCategory::CodePattern, ContextType::Interactive);

        assert_eq!(learner.distributions().len(), 1);
        let key = (LearningCategory::CodePattern, ContextType::Interactive);
        assert!(learner.distributions().contains_key(&key));
    }

    #[test]
    fn test_ensure_distribution_does_not_overwrite() {
        let config = StrategyLearnerConfig::default();
        let mut learner = StrategyLearner::new(config);

        learner.ensure_distribution(LearningCategory::CodePattern, ContextType::Interactive);

        // Modify the distribution
        let key = (LearningCategory::CodePattern, ContextType::Interactive);
        learner
            .distributions_mut()
            .get_mut(&key)
            .unwrap()
            .session_count = 42;

        // Ensure again - should not overwrite
        learner.ensure_distribution(LearningCategory::CodePattern, ContextType::Interactive);

        assert_eq!(learner.distributions().get(&key).unwrap().session_count, 42);
    }
}
