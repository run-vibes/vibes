//! Usage statistics and outcomes for the continual learning system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Per-learning usage statistics (updated frequently)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub times_injected: u32,
    pub times_helpful: u32,
    pub times_ignored: u32,
    pub times_contradicted: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
    pub confidence_alpha: f64,
    pub confidence_beta: f64,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            times_injected: 0,
            times_helpful: 0,
            times_ignored: 0,
            times_contradicted: 0,
            last_used: None,
            confidence_alpha: 1.0, // Uniform prior
            confidence_beta: 1.0,
        }
    }
}

impl UsageStats {
    /// Calculate current confidence from Bayesian priors
    pub fn confidence(&self) -> f64 {
        self.confidence_alpha / (self.confidence_alpha + self.confidence_beta)
    }

    /// Update confidence based on outcome
    pub fn record_outcome(&mut self, outcome: Outcome) {
        self.times_injected += 1;
        self.last_used = Some(Utc::now());

        match outcome {
            Outcome::Helpful => {
                self.times_helpful += 1;
                self.confidence_alpha += 1.0;
            }
            Outcome::Ignored => {
                self.times_ignored += 1;
                // Neutral - slight decay
                self.confidence_beta += 0.1;
            }
            Outcome::Contradicted => {
                self.times_contradicted += 1;
                self.confidence_beta += 1.5; // Strong negative signal
            }
        }
    }
}

/// Outcome of a learning injection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Helpful,
    Ignored,
    Contradicted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_usage_stats() {
        let stats = UsageStats::default();
        assert_eq!(stats.times_injected, 0);
        assert_eq!(stats.confidence_alpha, 1.0);
        assert_eq!(stats.confidence_beta, 1.0);
        // Uniform prior = 0.5 confidence
        assert!((stats.confidence() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_helpful_outcome_increases_confidence() {
        let mut stats = UsageStats::default();
        let initial = stats.confidence();
        stats.record_outcome(Outcome::Helpful);
        assert!(stats.confidence() > initial);
        assert_eq!(stats.times_helpful, 1);
        assert_eq!(stats.times_injected, 1);
    }

    #[test]
    fn test_contradicted_outcome_decreases_confidence() {
        let mut stats = UsageStats::default();
        let initial = stats.confidence();
        stats.record_outcome(Outcome::Contradicted);
        assert!(stats.confidence() < initial);
        assert_eq!(stats.times_contradicted, 1);
    }

    #[test]
    fn test_ignored_outcome_slight_decay() {
        let mut stats = UsageStats::default();
        let initial = stats.confidence();
        stats.record_outcome(Outcome::Ignored);
        // Ignored causes slight decay
        assert!(stats.confidence() < initial);
        assert_eq!(stats.times_ignored, 1);
    }

    #[test]
    fn test_last_used_updated_on_outcome() {
        let mut stats = UsageStats::default();
        assert!(stats.last_used.is_none());
        stats.record_outcome(Outcome::Helpful);
        assert!(stats.last_used.is_some());
    }
}
