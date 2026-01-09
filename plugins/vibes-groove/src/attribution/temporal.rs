//! Temporal correlation for weighting signals by proximity to activation
//!
//! Layer 2 of the attribution engine applies exponential decay to signals
//! based on their distance from activation points.

use serde::{Deserialize, Serialize};

use crate::assessment::LightweightSignal;

/// Result of temporal correlation analysis
#[derive(Debug, Clone)]
pub struct TemporalResult {
    /// Sum of weighted positive signals
    pub positive_score: f64,
    /// Sum of weighted negative signals
    pub negative_score: f64,
    /// Net score (positive - negative)
    pub net_score: f64,
}

/// Trait for correlating signals temporally with activation points
pub trait TemporalCorrelator: Send + Sync {
    /// Correlate signals with activation points
    ///
    /// # Arguments
    /// * `activation_points` - Message indices where learning was activated
    /// * `signals` - Pairs of (message_idx, signal) to correlate
    fn correlate(
        &self,
        activation_points: &[u32],
        signals: &[(u32, LightweightSignal)],
    ) -> TemporalResult;
}

/// Configuration for temporal correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    /// Decay rate (lambda) - higher = faster decay (default: 0.2)
    pub decay_rate: f64,
    /// Maximum distance in messages to consider (default: 10)
    pub max_distance: u32,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.2,
            max_distance: 10,
        }
    }
}

/// Temporal correlator using exponential decay
///
/// Weight = e^(-λ * distance) where λ is the decay rate
pub struct ExponentialDecayCorrelator {
    config: TemporalConfig,
}

impl ExponentialDecayCorrelator {
    /// Create a new correlator with default configuration
    pub fn new() -> Self {
        Self {
            config: TemporalConfig::default(),
        }
    }

    /// Create a new correlator with custom configuration
    pub fn with_config(config: TemporalConfig) -> Self {
        Self { config }
    }

    /// Calculate decay weight for a given distance
    pub fn weight(&self, distance: u32) -> f64 {
        if distance > self.config.max_distance {
            return 0.0;
        }
        (-self.config.decay_rate * distance as f64).exp()
    }

    /// Find the minimum distance from a message to any activation point
    fn min_distance_to_activation(&self, message_idx: u32, activation_points: &[u32]) -> u32 {
        activation_points
            .iter()
            .map(|&ap| message_idx.abs_diff(ap))
            .min()
            .unwrap_or(u32::MAX)
    }

    /// Get the confidence weight from a signal
    fn signal_confidence(signal: &LightweightSignal) -> f64 {
        match signal {
            LightweightSignal::Positive { confidence, .. } => *confidence,
            LightweightSignal::Negative { confidence, .. } => *confidence,
            LightweightSignal::ToolFailure { .. } => 0.8, // Fixed confidence for failures
            LightweightSignal::Correction => 0.9,
            LightweightSignal::Retry => 0.7,
            LightweightSignal::BuildStatus { .. } => 0.9,
        }
    }

    /// Determine if a signal is positive
    fn is_positive_signal(signal: &LightweightSignal) -> bool {
        matches!(
            signal,
            LightweightSignal::Positive { .. } | LightweightSignal::BuildStatus { passed: true }
        )
    }
}

impl Default for ExponentialDecayCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalCorrelator for ExponentialDecayCorrelator {
    fn correlate(
        &self,
        activation_points: &[u32],
        signals: &[(u32, LightweightSignal)],
    ) -> TemporalResult {
        let mut positive_score = 0.0;
        let mut negative_score = 0.0;

        // No activation points means no correlation
        if activation_points.is_empty() {
            return TemporalResult {
                positive_score: 0.0,
                negative_score: 0.0,
                net_score: 0.0,
            };
        }

        for (message_idx, signal) in signals {
            let distance = self.min_distance_to_activation(*message_idx, activation_points);
            let decay_weight = self.weight(distance);

            if decay_weight == 0.0 {
                continue;
            }

            let confidence = Self::signal_confidence(signal);
            let weighted_value = confidence * decay_weight;

            if Self::is_positive_signal(signal) {
                positive_score += weighted_value;
            } else {
                negative_score += weighted_value;
            }
        }

        TemporalResult {
            positive_score,
            negative_score,
            net_score: positive_score - negative_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_weight_at_zero_distance() {
        let correlator = ExponentialDecayCorrelator::new();
        // e^0 = 1.0
        assert!((correlator.weight(0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_decay_weight_decreases_with_distance() {
        let correlator = ExponentialDecayCorrelator::new();
        let w0 = correlator.weight(0);
        let w1 = correlator.weight(1);
        let w5 = correlator.weight(5);
        let w10 = correlator.weight(10);

        assert!(w0 > w1);
        assert!(w1 > w5);
        assert!(w5 > w10);
    }

    #[test]
    fn test_decay_weight_beyond_max_distance() {
        let correlator = ExponentialDecayCorrelator::new();
        assert_eq!(correlator.weight(11), 0.0);
        assert_eq!(correlator.weight(100), 0.0);
    }

    #[test]
    fn test_decay_weight_at_max_distance() {
        let correlator = ExponentialDecayCorrelator::new();
        // e^(-0.2 * 10) = e^(-2) ≈ 0.135
        let w = correlator.weight(10);
        assert!(w > 0.0);
        assert!(w < 0.2);
    }

    #[test]
    fn test_configurable_decay_rate() {
        let fast_decay = ExponentialDecayCorrelator::with_config(TemporalConfig {
            decay_rate: 0.5,
            max_distance: 10,
        });
        let slow_decay = ExponentialDecayCorrelator::with_config(TemporalConfig {
            decay_rate: 0.1,
            max_distance: 10,
        });

        // At distance 5, fast decay should be lower
        assert!(fast_decay.weight(5) < slow_decay.weight(5));
    }

    #[test]
    fn test_positive_signal_accumulation() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5];
        let signals = vec![
            (
                5,
                LightweightSignal::Positive {
                    pattern: "thanks".into(),
                    confidence: 0.8,
                },
            ),
            (
                6,
                LightweightSignal::Positive {
                    pattern: "great".into(),
                    confidence: 0.9,
                },
            ),
        ];

        let result = correlator.correlate(&activation_points, &signals);

        assert!(result.positive_score > 0.0);
        assert_eq!(result.negative_score, 0.0);
        assert!(result.net_score > 0.0);
    }

    #[test]
    fn test_negative_signal_accumulation() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5];
        let signals = vec![
            (
                5,
                LightweightSignal::Negative {
                    pattern: "wrong".into(),
                    confidence: 0.8,
                },
            ),
            (6, LightweightSignal::Retry),
        ];

        let result = correlator.correlate(&activation_points, &signals);

        assert_eq!(result.positive_score, 0.0);
        assert!(result.negative_score > 0.0);
        assert!(result.net_score < 0.0);
    }

    #[test]
    fn test_net_score_calculation() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5];
        let signals = vec![
            (
                5,
                LightweightSignal::Positive {
                    pattern: "good".into(),
                    confidence: 0.8,
                },
            ), // Full weight
            (
                5,
                LightweightSignal::Negative {
                    pattern: "bad".into(),
                    confidence: 0.3,
                },
            ), // Full weight
        ];

        let result = correlator.correlate(&activation_points, &signals);

        // Net should be positive - negative = 0.8 - 0.3 = 0.5
        assert!((result.net_score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_signals_beyond_max_distance_ignored() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5];
        let signals = vec![(
            20, // Distance of 15, beyond max_distance of 10
            LightweightSignal::Positive {
                pattern: "good".into(),
                confidence: 1.0,
            },
        )];

        let result = correlator.correlate(&activation_points, &signals);

        assert_eq!(result.positive_score, 0.0);
        assert_eq!(result.negative_score, 0.0);
        assert_eq!(result.net_score, 0.0);
    }

    #[test]
    fn test_multiple_activation_points() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5, 15];
        let signals = vec![
            (
                10, // Equidistant from both, distance = 5
                LightweightSignal::Positive {
                    pattern: "good".into(),
                    confidence: 1.0,
                },
            ),
            (
                20, // Distance 5 from activation point 15
                LightweightSignal::Positive {
                    pattern: "nice".into(),
                    confidence: 1.0,
                },
            ),
        ];

        let result = correlator.correlate(&activation_points, &signals);

        // Both signals should have same weight (distance 5)
        let w5 = correlator.weight(5);
        assert!((result.positive_score - 2.0 * w5).abs() < 0.001);
    }

    #[test]
    fn test_no_activation_points_returns_zero() {
        let correlator = ExponentialDecayCorrelator::new();
        let signals = vec![(
            5,
            LightweightSignal::Positive {
                pattern: "good".into(),
                confidence: 1.0,
            },
        )];

        let result = correlator.correlate(&[], &signals);

        assert_eq!(result.positive_score, 0.0);
        assert_eq!(result.negative_score, 0.0);
        assert_eq!(result.net_score, 0.0);
    }

    #[test]
    fn test_build_status_signals() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5];
        let signals = vec![
            (5, LightweightSignal::BuildStatus { passed: true }),
            (6, LightweightSignal::BuildStatus { passed: false }),
        ];

        let result = correlator.correlate(&activation_points, &signals);

        assert!(result.positive_score > 0.0); // passed = true is positive
        assert!(result.negative_score > 0.0); // passed = false is negative
    }

    #[test]
    fn test_tool_failure_is_negative() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5];
        let signals = vec![(
            5,
            LightweightSignal::ToolFailure {
                tool_name: "compile".into(),
            },
        )];

        let result = correlator.correlate(&activation_points, &signals);

        assert_eq!(result.positive_score, 0.0);
        assert!(result.negative_score > 0.0);
    }

    #[test]
    fn test_correction_is_negative() {
        let correlator = ExponentialDecayCorrelator::new();
        let activation_points = vec![5];
        let signals = vec![(5, LightweightSignal::Correction)];

        let result = correlator.correlate(&activation_points, &signals);

        assert_eq!(result.positive_score, 0.0);
        assert!(result.negative_score > 0.0);
    }
}
