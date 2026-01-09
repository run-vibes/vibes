//! Outcome router for combining attribution and direct signals
//!
//! Combines signals from the attribution engine and direct lightweight events
//! to compute strategy effectiveness.

use serde::{Deserialize, Serialize};

use crate::assessment::types::{LightweightEvent, LightweightSignal};
use crate::attribution::AttributionRecord;

use super::types::{InjectionStrategy, OutcomeSource, StrategyOutcome};

/// Configuration for outcome routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRouterConfig {
    /// Weight for attributed value (default 0.7)
    pub attribution_weight: f64,
    /// Weight for direct signals (default 0.3)
    pub direct_weight: f64,
    /// Minimum confidence to consider an outcome valid (default 0.3)
    pub min_outcome_confidence: f64,
}

impl Default for OutcomeRouterConfig {
    fn default() -> Self {
        Self {
            attribution_weight: 0.7,
            direct_weight: 0.3,
            min_outcome_confidence: 0.3,
        }
    }
}

/// Routes and combines outcome signals from multiple sources
pub struct OutcomeRouter {
    config: OutcomeRouterConfig,
}

impl Default for OutcomeRouter {
    fn default() -> Self {
        Self::new(OutcomeRouterConfig::default())
    }
}

impl OutcomeRouter {
    /// Create a new outcome router with the given configuration
    pub fn new(config: OutcomeRouterConfig) -> Self {
        Self { config }
    }

    /// Compute a strategy outcome from attribution and direct signals
    ///
    /// Returns None if no usable signals are available.
    pub fn compute_outcome(
        &self,
        attribution: Option<&AttributionRecord>,
        direct_events: &[LightweightEvent],
        _strategy_used: &InjectionStrategy,
    ) -> Option<StrategyOutcome> {
        // Extract attributed value if learning was activated
        let attributed_value = attribution
            .filter(|a| a.was_activated)
            .map(|a| a.attributed_value);

        // Aggregate direct signals
        let direct_value = Self::aggregate_direct_signals(direct_events);

        // Combine signals based on availability
        match (attributed_value, direct_value) {
            (Some(av), Some(dv)) => {
                // Both sources available - weighted average with high confidence
                let combined = self.config.attribution_weight * av + self.config.direct_weight * dv;
                Some(StrategyOutcome::new(combined, 0.9, OutcomeSource::Both))
            }
            (Some(av), None) => {
                // Attribution only - high confidence
                Some(StrategyOutcome::new(av, 0.8, OutcomeSource::Attribution))
            }
            (None, Some(dv)) => {
                // Direct signals only - lower confidence
                Some(StrategyOutcome::new(dv, 0.5, OutcomeSource::Direct))
            }
            (None, None) => {
                // No usable signals
                None
            }
        }
    }

    /// Aggregate direct lightweight signals into a normalized value [-1, 1]
    fn aggregate_direct_signals(events: &[LightweightEvent]) -> Option<f64> {
        if events.is_empty() {
            return None;
        }

        let mut positive_score = 0.0;
        let mut negative_score = 0.0;
        let mut signal_count = 0;

        for event in events {
            for signal in &event.signals {
                match signal {
                    LightweightSignal::Positive { confidence, .. } => {
                        positive_score += confidence;
                        signal_count += 1;
                    }
                    LightweightSignal::Negative { confidence, .. } => {
                        negative_score += confidence;
                        signal_count += 1;
                    }
                    LightweightSignal::ToolFailure { .. } => {
                        negative_score += 0.5; // Fixed penalty for tool failures
                        signal_count += 1;
                    }
                    LightweightSignal::Correction => {
                        negative_score += 0.7; // Correction indicates something was wrong
                        signal_count += 1;
                    }
                    LightweightSignal::Retry => {
                        negative_score += 0.3; // Retry is mildly negative
                        signal_count += 1;
                    }
                    LightweightSignal::BuildStatus { passed } => {
                        if *passed {
                            positive_score += 0.8;
                        } else {
                            negative_score += 0.8;
                        }
                        signal_count += 1;
                    }
                }
            }
        }

        if signal_count == 0 {
            return None;
        }

        // Normalize to [-1, 1] range
        let total = positive_score + negative_score;
        if total == 0.0 {
            return None;
        }

        Some((positive_score - negative_score) / total)
    }

    /// Get the configuration
    pub fn config(&self) -> &OutcomeRouterConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::types::AssessmentContext;
    use crate::assessment::{EventId, HarnessType, InjectionMethod, SessionId};
    use crate::strategy::types::{ContextPosition, InjectionFormat};
    use chrono::Utc;
    use uuid::Uuid;

    fn test_attribution(activated: bool, value: f64) -> AttributionRecord {
        AttributionRecord {
            learning_id: Uuid::now_v7(),
            session_id: SessionId::from("test-session"),
            timestamp: Utc::now(),
            was_activated: activated,
            activation_confidence: 0.9,
            activation_signals: vec![],
            temporal_positive: 0.5,
            temporal_negative: 0.1,
            net_temporal: 0.4,
            was_withheld: false,
            session_outcome: value,
            attributed_value: value,
        }
    }

    fn test_context() -> AssessmentContext {
        AssessmentContext {
            session_id: SessionId::from("test-session"),
            event_id: EventId::new(),
            timestamp: Utc::now(),
            active_learnings: vec![],
            injection_method: InjectionMethod::ClaudeMd,
            injection_scope: None,
            harness_type: HarnessType::ClaudeCode,
            harness_version: None,
            project_id: None,
            user_id: None,
        }
    }

    fn test_lightweight_event(
        signals: Vec<LightweightSignal>,
        frustration: f64,
    ) -> LightweightEvent {
        LightweightEvent {
            context: test_context(),
            message_idx: 0,
            signals,
            frustration_ema: frustration,
            success_ema: 1.0 - frustration,
            triggering_event_id: Uuid::now_v7(),
        }
    }

    fn test_events_positive() -> Vec<LightweightEvent> {
        vec![test_lightweight_event(
            vec![
                LightweightSignal::Positive {
                    pattern: "thanks".into(),
                    confidence: 0.8,
                },
                LightweightSignal::BuildStatus { passed: true },
            ],
            0.1,
        )]
    }

    fn test_events_negative() -> Vec<LightweightEvent> {
        vec![test_lightweight_event(
            vec![
                LightweightSignal::Negative {
                    pattern: "frustrated".into(),
                    confidence: 0.7,
                },
                LightweightSignal::ToolFailure {
                    tool_name: "Bash".into(),
                },
            ],
            0.6,
        )]
    }

    fn test_events_mixed() -> Vec<LightweightEvent> {
        vec![test_lightweight_event(
            vec![
                LightweightSignal::Positive {
                    pattern: "good".into(),
                    confidence: 0.6,
                },
                LightweightSignal::Negative {
                    pattern: "confused".into(),
                    confidence: 0.4,
                },
            ],
            0.3,
        )]
    }

    fn test_strategy() -> InjectionStrategy {
        InjectionStrategy::MainContext {
            position: ContextPosition::Prefix,
            format: InjectionFormat::Plain,
        }
    }

    #[test]
    fn test_both_attribution_and_direct_signals() {
        let router = OutcomeRouter::default();
        let attribution = test_attribution(true, 0.8);
        let events = test_events_positive();
        let strategy = test_strategy();

        let outcome = router
            .compute_outcome(Some(&attribution), &events, &strategy)
            .expect("Should produce outcome");

        // Both sources: high confidence
        assert_eq!(outcome.confidence, 0.9);
        assert_eq!(outcome.source, OutcomeSource::Both);
        // Value should be weighted average
        assert!(outcome.value > 0.0);
    }

    #[test]
    fn test_attribution_only() {
        let router = OutcomeRouter::default();
        let attribution = test_attribution(true, 0.6);
        let strategy = test_strategy();

        let outcome = router
            .compute_outcome(Some(&attribution), &[], &strategy)
            .expect("Should produce outcome");

        assert_eq!(outcome.source, OutcomeSource::Attribution);
        assert_eq!(outcome.confidence, 0.8);
        // Value should match attribution value (clamped to valid range)
        assert!((outcome.value - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_direct_signals_only() {
        let router = OutcomeRouter::default();
        let events = test_events_positive();
        let strategy = test_strategy();

        let outcome = router
            .compute_outcome(None, &events, &strategy)
            .expect("Should produce outcome");

        assert_eq!(outcome.source, OutcomeSource::Direct);
        assert_eq!(outcome.confidence, 0.5);
        // Positive signals should produce positive value
        assert!(outcome.value > 0.0);
    }

    #[test]
    fn test_no_signals_returns_none() {
        let router = OutcomeRouter::default();
        let strategy = test_strategy();

        let outcome = router.compute_outcome(None, &[], &strategy);

        assert!(outcome.is_none());
    }

    #[test]
    fn test_inactive_attribution_ignored() {
        let router = OutcomeRouter::default();
        // Attribution exists but learning wasn't activated
        let attribution = test_attribution(false, 0.8);
        let strategy = test_strategy();

        let outcome = router.compute_outcome(Some(&attribution), &[], &strategy);

        // Should return None since attribution wasn't activated and no direct signals
        assert!(outcome.is_none());
    }

    #[test]
    fn test_direct_signal_aggregation_positive() {
        let events = test_events_positive();
        let value = OutcomeRouter::aggregate_direct_signals(&events);

        assert!(value.is_some());
        assert!(
            value.unwrap() > 0.0,
            "Positive signals should yield positive value"
        );
    }

    #[test]
    fn test_direct_signal_aggregation_negative() {
        let events = test_events_negative();
        let value = OutcomeRouter::aggregate_direct_signals(&events);

        assert!(value.is_some());
        assert!(
            value.unwrap() < 0.0,
            "Negative signals should yield negative value"
        );
    }

    #[test]
    fn test_direct_signal_aggregation_mixed() {
        let events = test_events_mixed();
        let value = OutcomeRouter::aggregate_direct_signals(&events);

        assert!(value.is_some());
        // 0.6 positive vs 0.4 negative = net positive
        let v = value.unwrap();
        assert!(
            v > 0.0 && v < 1.0,
            "Mixed signals should yield moderate value"
        );
    }

    #[test]
    fn test_empty_events_returns_none() {
        let value = OutcomeRouter::aggregate_direct_signals(&[]);
        assert!(value.is_none());
    }

    #[test]
    fn test_events_with_no_signals_returns_none() {
        let events = vec![test_lightweight_event(vec![], 0.0)];

        let value = OutcomeRouter::aggregate_direct_signals(&events);
        assert!(value.is_none());
    }

    #[test]
    fn test_config_weights_applied() {
        // Create router with custom weights
        let config = OutcomeRouterConfig {
            attribution_weight: 0.5,
            direct_weight: 0.5,
            min_outcome_confidence: 0.3,
        };
        let router = OutcomeRouter::new(config);

        let attribution = test_attribution(true, 1.0); // Max attribution value
        let events = test_events_negative(); // Negative direct signals

        let strategy = test_strategy();
        let outcome = router
            .compute_outcome(Some(&attribution), &events, &strategy)
            .expect("Should produce outcome");

        // With equal weights and opposing signals, should be moderate
        assert!(
            outcome.value.abs() < 0.9,
            "Equal weights should moderate extreme values"
        );
    }
}
