//! Metric definitions for evaluation.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// A time period with start and end timestamps.
///
/// Uses a half-open interval `[start, end)` - start is inclusive, end is exclusive.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Unit of measurement for a metric.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricUnit {
    /// Raw count (e.g., tasks completed)
    Count,
    /// Percentage value (0.0 - 100.0)
    Percentage,
    /// Time duration
    Duration,
    /// Token count (for LLM usage)
    Tokens,
    /// Monetary value
    Currency,
    /// Custom unit with a description
    Custom(String),
}

/// Longitudinal metrics for AI assistant performance evaluation.
///
/// Captures performance measurements across sessions, tasks, agents,
/// and learning effectiveness over a time period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LongitudinalMetrics {
    // Session-level metrics
    /// Number of sessions completed in this period
    pub sessions_completed: u64,
    /// Percentage of sessions that completed successfully (0.0 - 1.0)
    pub session_success_rate: f64,
    /// Average duration of sessions
    pub avg_session_duration: Duration,

    // Task-level metrics
    /// Number of tasks completed in this period
    pub tasks_completed: u64,
    /// Percentage of tasks successful on first attempt (0.0 - 1.0)
    pub first_attempt_success_rate: f64,
    /// Average number of iterations needed to complete a task
    pub avg_iterations_to_success: f64,

    // Agent-level metrics
    /// Overall agent efficiency score (0.0 - 1.0)
    pub agent_efficiency: f64,
    /// Percentage of tool calls that succeeded (0.0 - 1.0)
    pub tool_success_rate: f64,
    /// Rate at which agent corrects its own mistakes (0.0 - 1.0)
    pub self_correction_rate: f64,

    // Swarm-level metrics (for future multi-agent use)
    /// Overhead from coordinating multiple agents (0.0 - 1.0)
    pub swarm_coordination_overhead: f64,
    /// Efficiency of parallel task execution (0.0 - 1.0)
    pub parallelism_efficiency: f64,

    // Learning integration (groove plugin)
    /// Number of learnings applied from the learning system
    pub learnings_applied: u64,
    /// Effectiveness of applied learnings (0.0 - 1.0)
    pub learning_effectiveness: f64,

    // Cost metrics
    /// Total tokens consumed
    pub total_tokens: u64,
    /// Total monetary cost
    pub total_cost: f64,
    /// Average cost per successful task
    pub cost_per_successful_task: f64,

    // Time window
    /// The time period these metrics cover
    pub period: TimePeriod,
}

impl Default for LongitudinalMetrics {
    fn default() -> Self {
        Self {
            sessions_completed: 0,
            session_success_rate: 0.0,
            avg_session_duration: Duration::zero(),
            tasks_completed: 0,
            first_attempt_success_rate: 0.0,
            avg_iterations_to_success: 0.0,
            agent_efficiency: 0.0,
            tool_success_rate: 0.0,
            self_correction_rate: 0.0,
            swarm_coordination_overhead: 0.0,
            parallelism_efficiency: 0.0,
            learnings_applied: 0,
            learning_effectiveness: 0.0,
            total_tokens: 0,
            total_cost: 0.0,
            cost_per_successful_task: 0.0,
            period: TimePeriod::default(),
        }
    }
}

/// Definition of a custom metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// Name of the metric (e.g., "task_success_rate")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Unit of measurement
    pub unit: MetricUnit,
    /// How to aggregate values over time
    pub aggregation: AggregationType,
}

/// How values should be aggregated over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum of all values
    Sum,
    /// Arithmetic mean
    Average,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// 50th percentile (median)
    P50,
    /// 95th percentile
    P95,
    /// 99th percentile
    P99,
}

impl TimePeriod {
    /// Returns the duration of this time period.
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }

    /// Returns true if the given instant falls within this period.
    ///
    /// Uses half-open interval semantics: `[start, end)`
    pub fn contains(&self, instant: DateTime<Utc>) -> bool {
        instant >= self.start && instant < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn time_period_duration_returns_difference_between_start_and_end() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap();
        let period = TimePeriod { start, end };

        let duration = period.duration();

        assert_eq!(duration, Duration::hours(1));
    }

    #[test]
    fn time_period_contains_returns_true_for_instant_within_period() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();
        let period = TimePeriod { start, end };
        let instant = Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap();

        assert!(period.contains(instant));
    }

    #[test]
    fn time_period_contains_returns_false_for_instant_before_period() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();
        let period = TimePeriod { start, end };
        let instant = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        assert!(!period.contains(instant));
    }

    #[test]
    fn time_period_contains_returns_false_for_instant_after_period() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap();
        let period = TimePeriod { start, end };
        let instant = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();

        assert!(!period.contains(instant));
    }

    #[test]
    fn time_period_contains_returns_true_for_instant_at_start_boundary() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap();
        let period = TimePeriod { start, end };

        assert!(period.contains(start));
    }

    #[test]
    fn time_period_contains_returns_false_for_instant_at_end_boundary() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap();
        let period = TimePeriod { start, end };

        // End is exclusive (half-open interval [start, end))
        assert!(!period.contains(end));
    }

    #[test]
    fn metric_unit_variants_are_comparable() {
        assert_eq!(MetricUnit::Count, MetricUnit::Count);
        assert_eq!(MetricUnit::Percentage, MetricUnit::Percentage);
        assert_eq!(MetricUnit::Duration, MetricUnit::Duration);
        assert_eq!(MetricUnit::Tokens, MetricUnit::Tokens);
        assert_eq!(MetricUnit::Currency, MetricUnit::Currency);
        assert_eq!(
            MetricUnit::Custom("requests/sec".to_string()),
            MetricUnit::Custom("requests/sec".to_string())
        );
        assert_ne!(MetricUnit::Count, MetricUnit::Percentage);
    }

    #[test]
    fn aggregation_type_variants_are_comparable() {
        assert_eq!(AggregationType::Sum, AggregationType::Sum);
        assert_eq!(AggregationType::Average, AggregationType::Average);
        assert_eq!(AggregationType::Min, AggregationType::Min);
        assert_eq!(AggregationType::Max, AggregationType::Max);
        assert_eq!(AggregationType::P50, AggregationType::P50);
        assert_eq!(AggregationType::P95, AggregationType::P95);
        assert_eq!(AggregationType::P99, AggregationType::P99);
        assert_ne!(AggregationType::Sum, AggregationType::Average);
    }

    #[test]
    fn metric_definition_stores_name_description_unit_and_aggregation() {
        let definition = MetricDefinition {
            name: "task_success_rate".to_string(),
            description: "Percentage of tasks completed successfully".to_string(),
            unit: MetricUnit::Percentage,
            aggregation: AggregationType::Average,
        };

        assert_eq!(definition.name, "task_success_rate");
        assert_eq!(
            definition.description,
            "Percentage of tasks completed successfully"
        );
        assert_eq!(definition.unit, MetricUnit::Percentage);
        assert_eq!(definition.aggregation, AggregationType::Average);
    }

    #[test]
    fn longitudinal_metrics_stores_all_metric_categories() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();

        let metrics = LongitudinalMetrics {
            // Session-level
            sessions_completed: 100,
            session_success_rate: 0.95,
            avg_session_duration: Duration::hours(2),
            // Task-level
            tasks_completed: 500,
            first_attempt_success_rate: 0.80,
            avg_iterations_to_success: 1.5,
            // Agent-level
            agent_efficiency: 0.90,
            tool_success_rate: 0.95,
            self_correction_rate: 0.10,
            // Swarm-level
            swarm_coordination_overhead: 0.05,
            parallelism_efficiency: 0.85,
            // Learning integration
            learnings_applied: 50,
            learning_effectiveness: 0.75,
            // Cost
            total_tokens: 1_000_000,
            total_cost: 50.0,
            cost_per_successful_task: 0.10,
            // Time window
            period: TimePeriod { start, end },
        };

        assert_eq!(metrics.sessions_completed, 100);
        assert_eq!(metrics.tasks_completed, 500);
        assert_eq!(metrics.total_tokens, 1_000_000);
        assert_eq!(metrics.period.duration(), Duration::days(1));
    }

    #[test]
    fn time_period_default_uses_unix_epoch() {
        let period = TimePeriod::default();
        // Default should be a zero-duration period at Unix epoch
        assert_eq!(period.start, period.end);
    }

    #[test]
    fn longitudinal_metrics_default_has_sensible_zero_values() {
        let metrics = LongitudinalMetrics::default();

        assert_eq!(metrics.sessions_completed, 0);
        assert_eq!(metrics.tasks_completed, 0);
        assert_eq!(metrics.total_tokens, 0);
        assert_eq!(metrics.total_cost, 0.0);
        assert_eq!(metrics.session_success_rate, 0.0);
    }

    #[test]
    fn time_period_serializes_and_deserializes() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        let period = TimePeriod { start, end };

        let json = serde_json::to_string(&period).unwrap();
        let deserialized: TimePeriod = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.start, start);
        assert_eq!(deserialized.end, end);
    }

    #[test]
    fn metric_unit_serializes_and_deserializes() {
        let units = vec![
            MetricUnit::Count,
            MetricUnit::Percentage,
            MetricUnit::Duration,
            MetricUnit::Tokens,
            MetricUnit::Currency,
            MetricUnit::Custom("requests/sec".to_string()),
        ];

        for unit in units {
            let json = serde_json::to_string(&unit).unwrap();
            let deserialized: MetricUnit = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, unit);
        }
    }

    #[test]
    fn aggregation_type_serializes_and_deserializes() {
        let types = vec![
            AggregationType::Sum,
            AggregationType::Average,
            AggregationType::Min,
            AggregationType::Max,
            AggregationType::P50,
            AggregationType::P95,
            AggregationType::P99,
        ];

        for agg_type in types {
            let json = serde_json::to_string(&agg_type).unwrap();
            let deserialized: AggregationType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, agg_type);
        }
    }

    #[test]
    fn metric_definition_serializes_and_deserializes() {
        let definition = MetricDefinition {
            name: "test_metric".to_string(),
            description: "A test metric".to_string(),
            unit: MetricUnit::Count,
            aggregation: AggregationType::Sum,
        };

        let json = serde_json::to_string(&definition).unwrap();
        let deserialized: MetricDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, definition.name);
        assert_eq!(deserialized.description, definition.description);
    }

    #[test]
    fn longitudinal_metrics_serializes_and_deserializes() {
        let metrics = LongitudinalMetrics::default();

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: LongitudinalMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sessions_completed, 0);
        assert_eq!(deserialized.total_cost, 0.0);
    }
}
