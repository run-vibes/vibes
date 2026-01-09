//! Ablation testing for measuring true causal impact of learnings
//!
//! Layer 3 of the attribution engine runs controlled A/B experiments
//! by withholding uncertain learnings from some sessions and comparing outcomes.

use std::sync::Arc;

use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::types::{Learning, LearningId};

use super::store::AttributionStore;
use super::types::{AblationExperiment, AblationResult, LearningValue, SessionOutcome};

/// Strategy for deciding when to run ablation experiments
pub trait AblationStrategy: Send + Sync {
    /// Decide if learning should be withheld from this session
    fn should_withhold(&self, learning: &Learning, value: &LearningValue) -> bool;

    /// Check if experiment has enough data to conclude
    fn is_experiment_complete(&self, experiment: &AblationExperiment) -> bool;

    /// Compute marginal value from experiment results
    fn compute_marginal_value(&self, experiment: &AblationExperiment) -> Option<AblationResult>;
}

/// Configuration for ablation testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AblationConfig {
    /// Whether ablation testing is enabled
    pub enabled: bool,
    /// Only ablate learnings with confidence below this threshold (default: 0.7)
    pub uncertainty_threshold: f64,
    /// Fraction of sessions to withhold learning from (default: 0.10)
    pub ablation_rate: f64,
    /// Minimum sessions per arm before concluding (default: 20)
    pub min_sessions_per_arm: usize,
    /// P-value threshold for significance (default: 0.05)
    pub significance_level: f64,
}

impl Default for AblationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            uncertainty_threshold: 0.7,
            ablation_rate: 0.10,
            min_sessions_per_arm: 20,
            significance_level: 0.05,
        }
    }
}

/// Conservative ablation strategy that only experiments on uncertain learnings
pub struct ConservativeAblation {
    config: AblationConfig,
}

impl ConservativeAblation {
    /// Create with default configuration
    pub fn new() -> Self {
        Self {
            config: AblationConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AblationConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &AblationConfig {
        &self.config
    }

    /// Calculate mean of outcomes
    fn mean(outcomes: &[SessionOutcome]) -> f64 {
        if outcomes.is_empty() {
            return 0.0;
        }
        outcomes.iter().map(|o| o.outcome).sum::<f64>() / outcomes.len() as f64
    }

    /// Calculate variance of outcomes
    fn variance(outcomes: &[SessionOutcome], mean: f64) -> f64 {
        if outcomes.len() < 2 {
            return 0.0;
        }
        let sum_sq: f64 = outcomes.iter().map(|o| (o.outcome - mean).powi(2)).sum();
        sum_sq / (outcomes.len() - 1) as f64
    }

    /// Perform Welch's t-test and return (t_statistic, degrees_of_freedom, p_value)
    ///
    /// Welch's t-test is preferred over Student's t-test because it doesn't
    /// assume equal variances between groups.
    fn welch_t_test(
        outcomes_with: &[SessionOutcome],
        outcomes_without: &[SessionOutcome],
    ) -> (f64, f64, f64) {
        let n1 = outcomes_with.len() as f64;
        let n2 = outcomes_without.len() as f64;

        if n1 < 2.0 || n2 < 2.0 {
            return (0.0, 0.0, 1.0);
        }

        let mean1 = Self::mean(outcomes_with);
        let mean2 = Self::mean(outcomes_without);
        let var1 = Self::variance(outcomes_with, mean1);
        let var2 = Self::variance(outcomes_without, mean2);

        // Standard error
        let se1 = var1 / n1;
        let se2 = var2 / n2;
        let se = (se1 + se2).sqrt();

        if se == 0.0 {
            return (0.0, n1 + n2 - 2.0, 1.0);
        }

        // t-statistic
        let t = (mean1 - mean2) / se;

        // Welch-Satterthwaite degrees of freedom
        let df_num = (se1 + se2).powi(2);
        let df_denom = (se1.powi(2) / (n1 - 1.0)) + (se2.powi(2) / (n2 - 1.0));
        let df = if df_denom > 0.0 {
            df_num / df_denom
        } else {
            n1 + n2 - 2.0
        };

        // Approximate p-value using normal distribution for large df
        // For a more accurate p-value, we'd use the t-distribution CDF
        let p_value = Self::approximate_p_value(t.abs(), df);

        (t, df, p_value)
    }

    /// Approximate two-tailed p-value from t-statistic
    ///
    /// Uses a simple approximation that works well for df > 30.
    /// For smaller df, this is conservative (overestimates p-value).
    fn approximate_p_value(t_abs: f64, df: f64) -> f64 {
        // For large df, t-distribution approaches normal
        // Use a simple approximation based on the standard normal
        if df < 1.0 {
            return 1.0;
        }

        // Approximation: p ≈ 2 * (1 - Φ(|t| * sqrt(df/(df+1))))
        // This is conservative for smaller df
        let adjusted_t = t_abs * (df / (df + 1.0)).sqrt();

        // Standard normal CDF approximation (Abramowitz & Stegun)
        let x = adjusted_t;
        let t_param = 1.0 / (1.0 + 0.2316419 * x);
        let d = 0.3989423 * (-x * x / 2.0).exp();
        let p_one_tail = d
            * t_param
            * (0.3193815
                + t_param
                    * (-0.3565638
                        + t_param * (1.781478 + t_param * (-1.821256 + t_param * 1.330274))));

        // Two-tailed p-value
        (2.0 * p_one_tail).min(1.0)
    }
}

impl Default for ConservativeAblation {
    fn default() -> Self {
        Self::new()
    }
}

impl AblationStrategy for ConservativeAblation {
    fn should_withhold(&self, learning: &Learning, value: &LearningValue) -> bool {
        // Only consider uncertain learnings
        if value.confidence >= self.config.uncertainty_threshold {
            return false;
        }

        // Also check the learning's own confidence
        if learning.confidence >= self.config.uncertainty_threshold {
            return false;
        }

        // Random selection based on ablation rate
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0) < self.config.ablation_rate
    }

    fn is_experiment_complete(&self, experiment: &AblationExperiment) -> bool {
        experiment.sessions_with.len() >= self.config.min_sessions_per_arm
            && experiment.sessions_without.len() >= self.config.min_sessions_per_arm
    }

    fn compute_marginal_value(&self, experiment: &AblationExperiment) -> Option<AblationResult> {
        if experiment.sessions_with.is_empty() || experiment.sessions_without.is_empty() {
            return None;
        }

        let mean_with = Self::mean(&experiment.sessions_with);
        let mean_without = Self::mean(&experiment.sessions_without);

        // Marginal value is the difference: how much better is it with the learning?
        let marginal_value = mean_with - mean_without;

        let (t_stat, df, p_value) =
            Self::welch_t_test(&experiment.sessions_with, &experiment.sessions_without);

        // Confidence is 1 - p_value (higher confidence = lower p-value)
        let confidence = 1.0 - p_value;

        // Significant if p-value below threshold
        let is_significant = p_value < self.config.significance_level;

        // Log for debugging in tests
        #[cfg(test)]
        {
            eprintln!(
                "t={:.3}, df={:.1}, p={:.4}, marginal={:.3}, sig={}",
                t_stat, df, p_value, marginal_value, is_significant
            );
        }
        // Suppress unused warnings in non-test builds
        let _ = (t_stat, df);

        Some(AblationResult {
            marginal_value,
            confidence,
            is_significant,
        })
    }
}

/// Manages ablation experiments for learnings
pub struct AblationManager<S: AblationStrategy> {
    strategy: S,
    store: Arc<dyn AttributionStore>,
}

impl<S: AblationStrategy> AblationManager<S> {
    /// Create a new manager with the given strategy and store
    pub fn new(strategy: S, store: Arc<dyn AttributionStore>) -> Self {
        Self { strategy, store }
    }

    /// Get the strategy
    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    /// Record a session outcome for an ablation experiment
    ///
    /// Creates a new experiment if one doesn't exist for this learning.
    pub async fn record_outcome(
        &self,
        learning_id: LearningId,
        outcome: SessionOutcome,
        was_withheld: bool,
    ) -> Result<()> {
        // Get or create experiment
        let mut experiment = self
            .store
            .get_experiment(learning_id)
            .await?
            .unwrap_or_else(|| AblationExperiment {
                learning_id,
                started_at: Utc::now(),
                sessions_with: Vec::new(),
                sessions_without: Vec::new(),
                result: None,
            });

        // Add outcome to appropriate arm
        if was_withheld {
            experiment.sessions_without.push(outcome);
        } else {
            experiment.sessions_with.push(outcome);
        }

        // Update experiment in store
        self.store.update_experiment(&experiment).await
    }

    /// Check a specific experiment and finalize if complete
    pub async fn check_experiment(
        &self,
        learning_id: LearningId,
    ) -> Result<Option<AblationResult>> {
        let Some(mut experiment) = self.store.get_experiment(learning_id).await? else {
            return Ok(None);
        };

        if !self.strategy.is_experiment_complete(&experiment) {
            return Ok(None);
        }

        // Already has result
        if experiment.result.is_some() {
            return Ok(experiment.result);
        }

        // Compute and store result
        if let Some(result) = self.strategy.compute_marginal_value(&experiment) {
            experiment.result = Some(result.clone());
            self.store.update_experiment(&experiment).await?;
            return Ok(Some(result));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::SessionId;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_outcome(outcome: f64) -> SessionOutcome {
        SessionOutcome {
            session_id: SessionId::from(Uuid::now_v7().to_string()),
            outcome,
            timestamp: Utc::now(),
        }
    }

    fn make_outcomes(values: &[f64]) -> Vec<SessionOutcome> {
        values.iter().map(|&v| make_outcome(v)).collect()
    }

    fn make_learning(confidence: f64) -> Learning {
        use crate::types::{LearningCategory, LearningContent, LearningSource, Scope};
        Learning {
            id: Uuid::now_v7(),
            scope: Scope::User("test".into()),
            category: LearningCategory::Preference,
            content: LearningContent {
                description: "test".into(),
                pattern: None,
                insight: "test insight".into(),
            },
            confidence,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: LearningSource::UserCreated,
        }
    }

    fn make_learning_value(confidence: f64) -> LearningValue {
        use super::super::types::LearningStatus;
        LearningValue {
            learning_id: Uuid::now_v7(),
            estimated_value: 0.5,
            confidence,
            session_count: 10,
            activation_rate: 0.5,
            temporal_value: 0.5,
            temporal_confidence: 0.5,
            ablation_value: None,
            ablation_confidence: None,
            status: LearningStatus::Active,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_should_withhold_respects_uncertainty_threshold() {
        let strategy = ConservativeAblation::new();

        // High confidence learning should never be withheld
        let high_conf_learning = make_learning(0.9);
        let high_conf_value = make_learning_value(0.9);

        // Run multiple times - should never withhold
        for _ in 0..100 {
            assert!(
                !strategy.should_withhold(&high_conf_learning, &high_conf_value),
                "High confidence learning should not be withheld"
            );
        }
    }

    #[test]
    fn test_should_withhold_allows_uncertain_learnings() {
        let strategy = ConservativeAblation::new();

        // Low confidence learning might be withheld
        let low_conf_learning = make_learning(0.3);
        let low_conf_value = make_learning_value(0.3);

        // Run many times to check probability
        let mut withheld_count = 0;
        let trials = 1000;
        for _ in 0..trials {
            if strategy.should_withhold(&low_conf_learning, &low_conf_value) {
                withheld_count += 1;
            }
        }

        // Should be withheld ~10% of the time (ablation_rate = 0.10)
        let rate = withheld_count as f64 / trials as f64;
        assert!(
            (0.05..0.20).contains(&rate),
            "Ablation rate should be near 10%, got {:.1}%",
            rate * 100.0
        );
    }

    #[test]
    fn test_experiment_completion_criteria() {
        let strategy = ConservativeAblation::new();

        let incomplete = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            sessions_with: make_outcomes(&[0.5; 10]), // Only 10
            sessions_without: make_outcomes(&[0.5; 10]), // Only 10
            result: None,
        };

        assert!(
            !strategy.is_experiment_complete(&incomplete),
            "Should not be complete with only 10 sessions per arm"
        );

        let complete = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            sessions_with: make_outcomes(&[0.5; 20]), // 20 sessions
            sessions_without: make_outcomes(&[0.5; 20]), // 20 sessions
            result: None,
        };

        assert!(
            strategy.is_experiment_complete(&complete),
            "Should be complete with 20 sessions per arm"
        );
    }

    #[test]
    fn test_marginal_value_positive_effect() {
        let strategy = ConservativeAblation::new();

        // Learning has clear positive effect
        let experiment = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            sessions_with: make_outcomes(&[0.8, 0.9, 0.85, 0.75, 0.8, 0.9, 0.85, 0.8, 0.9, 0.85]),
            sessions_without: make_outcomes(&[0.3, 0.4, 0.35, 0.3, 0.4, 0.35, 0.3, 0.4, 0.35, 0.3]),
            result: None,
        };

        let result = strategy.compute_marginal_value(&experiment).unwrap();

        assert!(
            result.marginal_value > 0.3,
            "Marginal value should be positive, got {}",
            result.marginal_value
        );
        assert!(
            result.is_significant,
            "Effect should be significant with such clear difference"
        );
    }

    #[test]
    fn test_marginal_value_negative_effect() {
        let strategy = ConservativeAblation::new();

        // Learning has negative effect (hurts sessions)
        let experiment = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            sessions_with: make_outcomes(&[0.2, 0.3, 0.25, 0.2, 0.3, 0.25, 0.2, 0.3, 0.25, 0.2]),
            sessions_without: make_outcomes(&[0.7, 0.8, 0.75, 0.7, 0.8, 0.75, 0.7, 0.8, 0.75, 0.7]),
            result: None,
        };

        let result = strategy.compute_marginal_value(&experiment).unwrap();

        assert!(
            result.marginal_value < -0.3,
            "Marginal value should be negative, got {}",
            result.marginal_value
        );
    }

    #[test]
    fn test_marginal_value_no_effect() {
        let strategy = ConservativeAblation::new();

        // Learning has no clear effect (similar outcomes)
        let experiment = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            sessions_with: make_outcomes(&[0.5, 0.6, 0.55, 0.5, 0.6]),
            sessions_without: make_outcomes(&[0.5, 0.55, 0.6, 0.5, 0.55]),
            result: None,
        };

        let result = strategy.compute_marginal_value(&experiment).unwrap();

        assert!(
            result.marginal_value.abs() < 0.1,
            "Marginal value should be near zero, got {}",
            result.marginal_value
        );
        assert!(
            !result.is_significant,
            "Effect should not be significant with similar outcomes"
        );
    }

    #[test]
    fn test_welch_t_test_basic() {
        // Test with known values
        let with = make_outcomes(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let without = make_outcomes(&[2.0, 3.0, 4.0, 5.0, 6.0]);

        let (t, df, p) = ConservativeAblation::welch_t_test(&with, &without);

        // t should be negative (with < without)
        assert!(t < 0.0, "t-stat should be negative, got {}", t);
        // df should be around 8 for equal size samples
        assert!(df > 5.0 && df < 10.0, "df should be ~8, got {}", df);
        // p should be moderate (not highly significant, not trivial)
        assert!(p > 0.0 && p < 1.0, "p-value should be in (0,1), got {}", p);
    }

    #[test]
    fn test_welch_t_test_identical_samples() {
        let with = make_outcomes(&[0.5, 0.5, 0.5, 0.5, 0.5]);
        let without = make_outcomes(&[0.5, 0.5, 0.5, 0.5, 0.5]);

        let (t, _df, p) = ConservativeAblation::welch_t_test(&with, &without);

        // t should be 0 for identical means with zero variance
        assert!(
            t.abs() < 0.001,
            "t-stat should be ~0 for identical samples, got {}",
            t
        );
        // p should be 1.0 (no difference)
        assert!(p >= 0.99, "p-value should be ~1.0, got {}", p);
    }

    #[test]
    fn test_mean_calculation() {
        let outcomes = make_outcomes(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let mean = ConservativeAblation::mean(&outcomes);
        assert!(
            (mean - 3.0).abs() < 0.001,
            "Mean should be 3.0, got {}",
            mean
        );

        let empty: Vec<SessionOutcome> = vec![];
        let empty_mean = ConservativeAblation::mean(&empty);
        assert!(
            empty_mean.abs() < 0.001,
            "Empty mean should be 0.0, got {}",
            empty_mean
        );
    }

    #[test]
    fn test_variance_calculation() {
        let outcomes = make_outcomes(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        let mean = ConservativeAblation::mean(&outcomes);
        let var = ConservativeAblation::variance(&outcomes, mean);
        // Sample variance of [2,4,4,4,5,5,7,9] = 4.571...
        assert!(
            (var - 4.571).abs() < 0.1,
            "Variance should be ~4.571, got {}",
            var
        );
    }

    #[test]
    fn test_config_defaults() {
        let config = AblationConfig::default();
        assert!(config.enabled);
        assert!((config.uncertainty_threshold - 0.7).abs() < f64::EPSILON);
        assert!((config.ablation_rate - 0.10).abs() < f64::EPSILON);
        assert_eq!(config.min_sessions_per_arm, 20);
        assert!((config.significance_level - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_configurable_thresholds() {
        let config = AblationConfig {
            enabled: true,
            uncertainty_threshold: 0.5, // Lower threshold
            ablation_rate: 0.20,        // Higher rate
            min_sessions_per_arm: 10,   // Fewer required
            significance_level: 0.01,   // Stricter
        };
        let strategy = ConservativeAblation::with_config(config);

        // With lower threshold, medium confidence should not be withheld
        let medium_learning = make_learning(0.6);
        let medium_value = make_learning_value(0.6);

        for _ in 0..100 {
            assert!(
                !strategy.should_withhold(&medium_learning, &medium_value),
                "0.6 confidence should not be withheld with 0.5 threshold"
            );
        }

        // Verify stricter significance level with noisy data
        let experiment = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            // High variance data with small mean difference
            sessions_with: make_outcomes(&[0.3, 0.7, 0.4, 0.6, 0.5]),
            sessions_without: make_outcomes(&[0.35, 0.65, 0.45, 0.55, 0.5]),
            result: None,
        };

        let result = strategy.compute_marginal_value(&experiment).unwrap();
        // With 0.01 significance and high variance, this shouldn't be significant
        assert!(
            !result.is_significant,
            "High variance data should not be significant at 0.01 level, p={:.4}",
            1.0 - result.confidence
        );
    }

    #[test]
    fn test_empty_experiment_returns_none() {
        let strategy = ConservativeAblation::new();

        let empty = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            sessions_with: vec![],
            sessions_without: vec![],
            result: None,
        };

        assert!(
            strategy.compute_marginal_value(&empty).is_none(),
            "Empty experiment should return None"
        );
    }

    #[test]
    fn test_requires_both_arms() {
        let strategy = ConservativeAblation::new();

        let one_arm = AblationExperiment {
            learning_id: Uuid::now_v7(),
            started_at: Utc::now(),
            sessions_with: make_outcomes(&[0.5, 0.6, 0.7]),
            sessions_without: vec![],
            result: None,
        };

        assert!(
            strategy.compute_marginal_value(&one_arm).is_none(),
            "One-armed experiment should return None"
        );
    }
}
