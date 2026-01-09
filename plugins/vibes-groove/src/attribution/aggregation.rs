//! Value aggregation for combining attribution signals
//!
//! Layer 4 of the attribution engine aggregates temporal correlation and
//! ablation results into a single estimated value with confidence.

use serde::{Deserialize, Serialize};

use super::temporal::TemporalResult;
use super::types::{LearningStatus, LearningValue};

/// Configuration for value aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Weight for temporal correlation signal (default: 0.6)
    pub temporal_weight: f64,
    /// Weight for ablation signal when available (default: 0.4)
    pub ablation_weight: f64,
    /// Value threshold below which learning is deprecated (default: -0.3)
    pub deprecation_threshold: f64,
    /// Confidence required to trigger deprecation (default: 0.8)
    pub deprecation_confidence: f64,
    /// Minimum sessions before deprecation can trigger (default: 10)
    pub min_sessions_for_deprecation: u32,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            temporal_weight: 0.6,
            ablation_weight: 0.4,
            deprecation_threshold: -0.3,
            deprecation_confidence: 0.8,
            min_sessions_for_deprecation: 10,
        }
    }
}

/// Aggregates attribution signals into final learning value
pub struct ValueAggregator {
    config: AggregationConfig,
}

impl ValueAggregator {
    /// Create with default configuration
    pub fn new() -> Self {
        Self {
            config: AggregationConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AggregationConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &AggregationConfig {
        &self.config
    }

    /// Update temporal value with new observation using running weighted average
    ///
    /// Returns (new_temporal_value, new_temporal_confidence)
    pub fn update_temporal_value(
        &self,
        current: &LearningValue,
        new_temporal: &TemporalResult,
    ) -> (f64, f64) {
        let new_count = current.session_count + 1;

        // Weight for new observation decreases as we accumulate more data
        // This creates a running average that stabilizes over time
        let learning_rate = 1.0 / new_count as f64;

        let new_value = weighted_update(
            current.temporal_value,
            new_temporal.net_score,
            learning_rate,
        );

        let new_confidence = confidence_from_count(new_count);

        (new_value, new_confidence)
    }

    /// Combine temporal and ablation values into final estimated value
    ///
    /// Returns (estimated_value, confidence)
    pub fn aggregate_value(&self, value: &LearningValue) -> (f64, f64) {
        let temporal = (value.temporal_value, value.temporal_confidence);

        let ablation = match (value.ablation_value, value.ablation_confidence) {
            (Some(v), Some(c)) if c > 0.5 => Some((v, c)), // Only use if reasonably confident
            _ => None,
        };

        combine_estimates(
            temporal,
            ablation,
            self.config.temporal_weight,
            self.config.ablation_weight,
        )
    }

    /// Check if a learning should be deprecated based on its value
    ///
    /// Returns Some(reason) if should deprecate, None otherwise
    pub fn should_deprecate(&self, value: &LearningValue) -> Option<String> {
        // Need enough sessions to be confident
        if value.session_count < self.config.min_sessions_for_deprecation {
            return None;
        }

        // Need high confidence in our estimate
        if value.confidence < self.config.deprecation_confidence {
            return None;
        }

        // Check if value is below threshold
        if value.estimated_value < self.config.deprecation_threshold {
            return Some(format!(
                "Value {:.2} below threshold {:.2} with {:.0}% confidence",
                value.estimated_value,
                self.config.deprecation_threshold,
                value.confidence * 100.0
            ));
        }

        None
    }

    /// Update a learning value with new temporal result and recompute aggregate
    ///
    /// Returns the updated LearningValue
    pub fn update_learning_value(
        &self,
        mut value: LearningValue,
        temporal_result: &TemporalResult,
        activation_rate: f64,
    ) -> LearningValue {
        // Update session count
        value.session_count += 1;
        value.activation_rate = weighted_update(
            value.activation_rate,
            activation_rate,
            1.0 / value.session_count as f64,
        );

        // Update temporal values
        let (new_temporal, new_temporal_conf) = self.update_temporal_value(&value, temporal_result);
        value.temporal_value = new_temporal;
        value.temporal_confidence = new_temporal_conf;

        // Recompute aggregate
        let (estimated, confidence) = self.aggregate_value(&value);
        value.estimated_value = estimated;
        value.confidence = confidence;

        // Check for deprecation
        if let Some(reason) = self.should_deprecate(&value) {
            value.status = LearningStatus::Deprecated { reason };
        }

        value.updated_at = chrono::Utc::now();

        value
    }

    /// Incorporate ablation result into learning value
    pub fn update_with_ablation(
        &self,
        mut value: LearningValue,
        ablation_value: f64,
        ablation_confidence: f64,
    ) -> LearningValue {
        value.ablation_value = Some(ablation_value);
        value.ablation_confidence = Some(ablation_confidence);

        // Recompute aggregate with new ablation data
        let (estimated, confidence) = self.aggregate_value(&value);
        value.estimated_value = estimated;
        value.confidence = confidence;

        // Check for deprecation
        if let Some(reason) = self.should_deprecate(&value) {
            value.status = LearningStatus::Deprecated { reason };
        }

        value.updated_at = chrono::Utc::now();

        value
    }
}

impl Default for ValueAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Weighted update for running average
///
/// `weight` is the weight for the new value (typically 1/n for n observations)
fn weighted_update(old_value: f64, new_value: f64, weight: f64) -> f64 {
    old_value * (1.0 - weight) + new_value * weight
}

/// Convert session count to confidence score
///
/// Uses a logarithmic function that approaches 1.0 asymptotically
fn confidence_from_count(count: u32) -> f64 {
    if count == 0 {
        return 0.0;
    }
    // Confidence grows logarithmically: starts at ~0.3, reaches ~0.9 at 100 sessions
    // Formula: 1 - 1/(1 + ln(count))
    let ln_count = (count as f64).ln();
    1.0 - 1.0 / (1.0 + ln_count)
}

/// Combine temporal and ablation estimates using confidence-weighted average
///
/// When ablation data is available and confident, it gets weighted more heavily
/// because it provides causal evidence.
fn combine_estimates(
    temporal: (f64, f64),
    ablation: Option<(f64, f64)>,
    temporal_weight: f64,
    ablation_weight: f64,
) -> (f64, f64) {
    let (temporal_value, temporal_conf) = temporal;

    match ablation {
        Some((ablation_value, ablation_conf)) => {
            // Confidence-weighted combination
            // Higher confidence source gets more weight
            let temporal_effective_weight = temporal_weight * temporal_conf;
            let ablation_effective_weight = ablation_weight * ablation_conf;
            let total_weight = temporal_effective_weight + ablation_effective_weight;

            if total_weight == 0.0 {
                return (0.0, 0.0);
            }

            let combined_value = (temporal_value * temporal_effective_weight
                + ablation_value * ablation_effective_weight)
                / total_weight;

            // Combined confidence is weighted average of confidences
            let combined_conf = (temporal_conf * temporal_effective_weight
                + ablation_conf * ablation_effective_weight)
                / total_weight;

            (combined_value, combined_conf)
        }
        None => {
            // No ablation data, use temporal only
            (temporal_value, temporal_conf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_learning_value(
        temporal_value: f64,
        temporal_confidence: f64,
        session_count: u32,
    ) -> LearningValue {
        LearningValue {
            learning_id: Uuid::now_v7(),
            estimated_value: temporal_value,
            confidence: temporal_confidence,
            session_count,
            activation_rate: 0.5,
            temporal_value,
            temporal_confidence,
            ablation_value: None,
            ablation_confidence: None,
            status: LearningStatus::Active,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_weighted_update_basic() {
        // With weight 0.5, should be average
        let result = weighted_update(0.0, 1.0, 0.5);
        assert!((result - 0.5).abs() < 0.001);

        // With weight 1.0, should be new value
        let result = weighted_update(0.0, 1.0, 1.0);
        assert!((result - 1.0).abs() < 0.001);

        // With weight 0.0, should be old value
        let result = weighted_update(0.5, 1.0, 0.0);
        assert!((result - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_confidence_from_count() {
        assert_eq!(confidence_from_count(0), 0.0);

        let c1 = confidence_from_count(1);
        let c10 = confidence_from_count(10);
        let c100 = confidence_from_count(100);

        // Confidence should increase with count
        assert!(c1 < c10);
        assert!(c10 < c100);

        // Should approach but not exceed 1.0
        assert!(c100 < 1.0);
        assert!(c100 > 0.8);
    }

    #[test]
    fn test_running_average_converges() {
        let aggregator = ValueAggregator::new();
        let mut value = make_learning_value(0.0, 0.0, 0);

        // Simulate 10 sessions all with net_score = 0.5
        for _ in 0..10 {
            let temporal = TemporalResult {
                positive_score: 0.7,
                negative_score: 0.2,
                net_score: 0.5,
            };
            let (new_val, _) = aggregator.update_temporal_value(&value, &temporal);
            value.temporal_value = new_val;
            value.session_count += 1;
        }

        // Should converge to 0.5
        assert!(
            (value.temporal_value - 0.5).abs() < 0.1,
            "Running average should converge to 0.5, got {}",
            value.temporal_value
        );
    }

    #[test]
    fn test_confidence_increases_with_sessions() {
        let aggregator = ValueAggregator::new();
        let value = make_learning_value(0.5, 0.3, 5);

        let temporal = TemporalResult {
            positive_score: 0.6,
            negative_score: 0.1,
            net_score: 0.5,
        };

        let (_, conf1) = aggregator.update_temporal_value(&value, &temporal);

        // Update again with more sessions
        let value2 = make_learning_value(0.5, conf1, 20);
        let (_, conf2) = aggregator.update_temporal_value(&value2, &temporal);

        assert!(
            conf2 > conf1,
            "Confidence should increase with more sessions"
        );
    }

    #[test]
    fn test_temporal_only_aggregation() {
        let aggregator = ValueAggregator::new();
        let value = make_learning_value(0.6, 0.8, 10);

        let (estimated, confidence) = aggregator.aggregate_value(&value);

        // Without ablation, should use temporal values directly
        assert!((estimated - 0.6).abs() < 0.001, "Should use temporal value");
        assert!(
            (confidence - 0.8).abs() < 0.001,
            "Should use temporal confidence"
        );
    }

    #[test]
    fn test_temporal_plus_ablation_aggregation() {
        let aggregator = ValueAggregator::new();
        let mut value = make_learning_value(0.5, 0.7, 20);
        value.ablation_value = Some(0.8);
        value.ablation_confidence = Some(0.9);

        let (estimated, _confidence) = aggregator.aggregate_value(&value);

        // Should be weighted combination
        // Ablation should pull estimate higher since it has higher value
        assert!(
            estimated > 0.5,
            "With positive ablation, estimate should be higher than temporal alone"
        );
        assert!(estimated < 0.8, "Estimate should be weighted average");
    }

    #[test]
    fn test_weight_balancing() {
        // Custom weights favoring ablation
        let config = AggregationConfig {
            temporal_weight: 0.2,
            ablation_weight: 0.8,
            ..Default::default()
        };
        let aggregator = ValueAggregator::with_config(config);

        let mut value = make_learning_value(0.3, 0.8, 20);
        value.ablation_value = Some(0.9);
        value.ablation_confidence = Some(0.8);

        let (estimated, _) = aggregator.aggregate_value(&value);

        // With heavy ablation weight, should be closer to ablation value
        assert!(
            estimated > 0.6,
            "With high ablation weight, should be closer to ablation value (0.9)"
        );
    }

    #[test]
    fn test_deprecation_threshold_detection() {
        let aggregator = ValueAggregator::new();

        // Not enough sessions
        let value = make_learning_value(-0.5, 0.9, 5);
        assert!(
            aggregator.should_deprecate(&value).is_none(),
            "Should not deprecate with few sessions"
        );

        // Not enough confidence
        let value = make_learning_value(-0.5, 0.5, 20);
        assert!(
            aggregator.should_deprecate(&value).is_none(),
            "Should not deprecate with low confidence"
        );

        // Value not below threshold
        let value = make_learning_value(0.1, 0.9, 20);
        assert!(
            aggregator.should_deprecate(&value).is_none(),
            "Should not deprecate positive value"
        );

        // Should deprecate: low value, high confidence, enough sessions
        let mut value = make_learning_value(-0.5, 0.9, 20);
        value.estimated_value = -0.5;
        value.confidence = 0.9;
        assert!(
            aggregator.should_deprecate(&value).is_some(),
            "Should deprecate harmful learning"
        );
    }

    #[test]
    fn test_update_learning_value_full_cycle() {
        let aggregator = ValueAggregator::new();
        let value = make_learning_value(0.0, 0.0, 0);

        let temporal = TemporalResult {
            positive_score: 0.8,
            negative_score: 0.1,
            net_score: 0.7,
        };

        let updated = aggregator.update_learning_value(value, &temporal, 1.0);

        assert_eq!(updated.session_count, 1);
        assert!(updated.temporal_value > 0.0);
        assert!(updated.temporal_confidence > 0.0);
        assert!(matches!(updated.status, LearningStatus::Active));
    }

    #[test]
    fn test_update_with_ablation() {
        let aggregator = ValueAggregator::new();
        let value = make_learning_value(0.5, 0.7, 20);

        let updated = aggregator.update_with_ablation(value, 0.8, 0.9);

        assert_eq!(updated.ablation_value, Some(0.8));
        assert_eq!(updated.ablation_confidence, Some(0.9));
        // Estimate should now incorporate ablation
        assert!(updated.estimated_value > 0.5);
    }

    #[test]
    fn test_combine_estimates_confidence_weighting() {
        // Equal values but different confidences
        let temporal = (0.5, 0.3); // low confidence
        let ablation = Some((0.5, 0.9)); // high confidence

        let (value, conf) = combine_estimates(temporal, ablation, 0.6, 0.4);

        // Value should be 0.5 (same from both sources)
        assert!((value - 0.5).abs() < 0.01);

        // Confidence should be weighted toward ablation's higher confidence
        assert!(conf > 0.5);
    }

    #[test]
    fn test_low_confidence_ablation_ignored() {
        let aggregator = ValueAggregator::new();
        let mut value = make_learning_value(0.5, 0.8, 20);
        // Ablation with very low confidence
        value.ablation_value = Some(0.9);
        value.ablation_confidence = Some(0.3); // Below 0.5 threshold

        let (estimated, _) = aggregator.aggregate_value(&value);

        // Should ignore low-confidence ablation and use temporal only
        assert!(
            (estimated - 0.5).abs() < 0.01,
            "Should use temporal when ablation confidence is low"
        );
    }

    #[test]
    fn test_deprecation_updates_status() {
        let aggregator = ValueAggregator::new();
        // Use 100 sessions so confidence is high enough (>0.8)
        let mut value = make_learning_value(-0.5, 0.9, 100);
        value.estimated_value = -0.5;
        value.confidence = 0.9;

        let temporal = TemporalResult {
            positive_score: 0.1,
            negative_score: 0.6,
            net_score: -0.5,
        };

        let updated = aggregator.update_learning_value(value, &temporal, 0.5);

        // With 101 sessions, confidence should be high enough
        assert!(
            updated.confidence >= 0.8,
            "Confidence should be >= 0.8 with 101 sessions, got {}",
            updated.confidence
        );
        assert!(
            matches!(updated.status, LearningStatus::Deprecated { .. }),
            "Should be deprecated after update confirms negative value"
        );
    }

    #[test]
    fn test_config_defaults() {
        let config = AggregationConfig::default();
        assert!((config.temporal_weight - 0.6).abs() < f64::EPSILON);
        assert!((config.ablation_weight - 0.4).abs() < f64::EPSILON);
        assert!((config.deprecation_threshold - (-0.3)).abs() < f64::EPSILON);
        assert!((config.deprecation_confidence - 0.8).abs() < f64::EPSILON);
        assert_eq!(config.min_sessions_for_deprecation, 10);
    }
}
