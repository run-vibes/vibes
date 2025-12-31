//! Adaptive parameters with Bayesian learning

use chrono::{DateTime, Utc};
use rand_distr::{Beta, Distribution};
use serde::{Deserialize, Serialize};

/// A parameter that learns via Bayesian updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveParam {
    pub value: f64,
    pub uncertainty: f64,
    pub observations: u64,
    pub prior_alpha: f64,
    pub prior_beta: f64,
}

impl Default for AdaptiveParam {
    fn default() -> Self {
        Self::new_uninformed()
    }
}

impl AdaptiveParam {
    /// Create with uninformed (uniform) prior
    pub fn new_uninformed() -> Self {
        Self {
            value: 0.5,
            uncertainty: 1.0,
            observations: 0,
            prior_alpha: 1.0,
            prior_beta: 1.0,
        }
    }

    /// Create with informed prior
    pub fn new_with_prior(alpha: f64, beta: f64) -> Self {
        let value = alpha / (alpha + beta);
        Self {
            value,
            uncertainty: 1.0,
            observations: 0,
            prior_alpha: alpha,
            prior_beta: beta,
        }
    }

    /// Bayesian update based on outcome
    pub fn update(&mut self, outcome: f64, weight: f64) {
        self.observations += 1;
        let effective_weight = weight / (1.0 + self.uncertainty);
        self.prior_alpha += outcome * effective_weight;
        self.prior_beta += (1.0 - outcome) * effective_weight;
        self.value = self.prior_alpha / (self.prior_alpha + self.prior_beta);
        self.uncertainty = 1.0 / (1.0 + (self.observations as f64).sqrt());
    }

    /// Thompson sampling for exploration
    pub fn sample(&self) -> f64 {
        let beta = Beta::new(self.prior_alpha, self.prior_beta)
            .unwrap_or_else(|_| Beta::new(1.0, 1.0).unwrap());
        beta.sample(&mut rand::thread_rng())
    }
}

/// Named system-wide parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemParam {
    pub name: String,
    pub param: AdaptiveParam,
    pub updated_at: DateTime<Utc>,
}

impl SystemParam {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param: AdaptiveParam::new_uninformed(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_prior(name: impl Into<String>, alpha: f64, beta: f64) -> Self {
        Self {
            name: name.into(),
            param: AdaptiveParam::new_with_prior(alpha, beta),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninformed_prior() {
        let param = AdaptiveParam::new_uninformed();
        assert!((param.value - 0.5).abs() < 0.001);
        assert_eq!(param.observations, 0);
    }

    #[test]
    fn test_informed_prior() {
        // Prior of alpha=8, beta=2 should give ~0.8 value
        let param = AdaptiveParam::new_with_prior(8.0, 2.0);
        assert!((param.value - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_update_moves_toward_outcome() {
        let mut param = AdaptiveParam::new_uninformed();
        // Positive outcome (1.0) should increase value
        param.update(1.0, 1.0);
        assert!(param.value > 0.5);
        assert_eq!(param.observations, 1);
    }

    #[test]
    fn test_uncertainty_decreases_with_observations() {
        let mut param = AdaptiveParam::new_uninformed();
        let initial_uncertainty = param.uncertainty;
        param.update(0.5, 1.0);
        assert!(param.uncertainty < initial_uncertainty);
    }

    #[test]
    fn test_sample_returns_valid_probability() {
        let param = AdaptiveParam::new_uninformed();
        for _ in 0..100 {
            let sample = param.sample();
            assert!((0.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_system_param_creation() {
        let param = SystemParam::new("injection_budget");
        assert_eq!(param.name, "injection_budget");
        assert!((param.param.value - 0.5).abs() < 0.001);
    }
}
