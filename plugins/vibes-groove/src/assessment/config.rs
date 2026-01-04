//! Assessment configuration types.
//!
//! Provides configuration for the assessment framework including sampling rates,
//! circuit breakers, LLM backend settings, and retention policies.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for the assessment framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AssessmentConfig {
    /// Whether assessment is enabled at all.
    pub enabled: bool,
    /// Whether interventions (suggestions) are enabled.
    pub intervention_enabled: bool,
    /// Sampling configuration for assessment collection.
    pub sampling: SamplingConfig,
    /// Session end detection configuration.
    pub session_end: SessionEndConfig,
    /// Circuit breaker to prevent assessment fatigue.
    pub circuit_breaker: CircuitBreakerConfig,
    /// LLM backend configuration for assessments.
    pub llm: LlmConfig,
    /// Pattern configuration for matching.
    pub patterns: PatternConfig,
    /// Data retention configuration.
    pub retention: RetentionConfig,
    /// Iggy server configuration.
    pub iggy: IggyServerConfig,
}

impl Default for AssessmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intervention_enabled: true,
            sampling: SamplingConfig::default(),
            session_end: SessionEndConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            llm: LlmConfig::default(),
            patterns: PatternConfig::default(),
            retention: RetentionConfig::default(),
            iggy: IggyServerConfig::default(),
        }
    }
}

/// Configuration for assessment sampling rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SamplingConfig {
    /// Base rate for sampling assessments (0.0 to 1.0).
    pub base_rate: f64,
    /// Number of sessions to collect before applying sampling.
    pub burnin_sessions: u32,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            base_rate: 0.2,
            burnin_sessions: 10,
        }
    }
}

/// Configuration for session end detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionEndConfig {
    /// Whether to use hook-based session end detection.
    pub hook_enabled: bool,
    /// Whether to use timeout-based session end detection.
    pub timeout_enabled: bool,
    /// Timeout in minutes for session end detection.
    pub timeout_minutes: u32,
}

impl Default for SessionEndConfig {
    fn default() -> Self {
        Self {
            hook_enabled: true,
            timeout_enabled: true,
            timeout_minutes: 15,
        }
    }
}

/// Circuit breaker configuration to prevent assessment fatigue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CircuitBreakerConfig {
    /// Whether circuit breaker is enabled.
    pub enabled: bool,
    /// Cooldown period in seconds after an intervention.
    pub cooldown_seconds: u32,
    /// Maximum interventions allowed per session.
    pub max_interventions_per_session: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown_seconds: 120,
            max_interventions_per_session: 3,
        }
    }
}

/// LLM backend configuration for assessments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// Whether LLM-based assessment is enabled.
    pub enabled: bool,
    /// Backend to use for LLM calls ("harness" for Claude Code subprocess).
    pub backend: String,
    /// Model to use for assessments.
    pub model: String,
    /// Timeout for LLM calls in seconds.
    pub timeout_seconds: u32,
    /// Maximum retries for failed LLM calls.
    pub max_retries: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "harness".to_string(),
            model: "claude-3-haiku".to_string(),
            timeout_seconds: 60,
            max_retries: 2,
        }
    }
}

/// Pattern configuration for matching positive and negative behaviors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PatternConfig {
    /// Patterns indicating negative behaviors to avoid.
    pub negative: Vec<String>,
    /// Patterns indicating positive behaviors to encourage.
    pub positive: Vec<String>,
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            negative: vec![
                r"(?i)\berror\b".to_string(),
                r"(?i)\bfailed\b".to_string(),
                r"(?i)\bfrustrat".to_string(),
                r"(?i)\bconfus".to_string(),
                r"(?i)\bwrong\b".to_string(),
                r"(?i)\bbroken\b".to_string(),
                r"(?i)\bnot\s+work".to_string(),
                r"(?i)\bdoesn't\s+work".to_string(),
                r"(?i)\bcan't\b".to_string(),
                r"(?i)\bwon't\b".to_string(),
            ],
            positive: vec![
                r"(?i)\bthank".to_string(),
                r"(?i)\bperfect\b".to_string(),
                r"(?i)\bexcellent\b".to_string(),
                r"(?i)\bgreat\b".to_string(),
                r"(?i)\bworks?\b".to_string(),
                r"(?i)\bsuccess".to_string(),
                r"(?i)\bcomplete".to_string(),
                r"(?i)\bdone\b".to_string(),
            ],
        }
    }
}

impl PatternConfig {
    /// Merge another pattern config into this one.
    ///
    /// Patterns from `other` are appended to existing patterns.
    pub fn merge(&mut self, other: &PatternConfig) {
        self.negative.extend(other.negative.iter().cloned());
        self.positive.extend(other.positive.iter().cloned());
    }
}

/// Data retention configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    /// Days to retain lightweight data. -1 means forever.
    pub lightweight_days: i32,
    /// Days to retain medium data. -1 means forever.
    pub medium_days: i32,
    /// Days to retain heavy data. -1 means forever.
    pub heavy_days: i32,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            lightweight_days: 7,
            medium_days: 30,
            heavy_days: -1,
        }
    }
}

impl RetentionConfig {
    /// Value indicating data should be retained forever.
    pub const FOREVER: i32 = -1;

    /// Check if lightweight data should be retained forever.
    pub fn lightweight_forever(&self) -> bool {
        self.lightweight_days == Self::FOREVER
    }

    /// Check if medium data should be retained forever.
    pub fn medium_forever(&self) -> bool {
        self.medium_days == Self::FOREVER
    }

    /// Check if heavy data should be retained forever.
    pub fn heavy_forever(&self) -> bool {
        self.heavy_days == Self::FOREVER
    }
}

/// Iggy server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IggyServerConfig {
    /// Directory for Iggy data storage.
    pub data_dir: Option<PathBuf>,
    /// Port for Iggy server.
    pub port: u16,
}

impl Default for IggyServerConfig {
    fn default() -> Self {
        Self {
            data_dir: None,
            port: 8090,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assessment_config_default_values() {
        let config = AssessmentConfig::default();

        assert!(config.enabled);
        assert!(config.intervention_enabled);
        assert_eq!(config.sampling.base_rate, 0.2);
        assert_eq!(config.sampling.burnin_sessions, 10);
        assert!(config.session_end.hook_enabled);
        assert!(config.session_end.timeout_enabled);
        assert_eq!(config.session_end.timeout_minutes, 15);
        assert!(config.circuit_breaker.enabled);
        assert_eq!(config.circuit_breaker.cooldown_seconds, 120);
        assert_eq!(config.circuit_breaker.max_interventions_per_session, 3);
        assert!(config.llm.enabled);
        assert_eq!(config.llm.backend, "harness");
        assert_eq!(config.llm.model, "claude-3-haiku");
        assert_eq!(config.llm.timeout_seconds, 60);
        assert_eq!(config.llm.max_retries, 2);
        // PatternConfig now has default patterns (10 negative, 8 positive)
        assert_eq!(config.patterns.negative.len(), 10);
        assert_eq!(config.patterns.positive.len(), 8);
        assert_eq!(config.retention.lightweight_days, 7);
        assert_eq!(config.retention.medium_days, 30);
        assert_eq!(config.retention.heavy_days, -1);
        assert!(config.iggy.data_dir.is_none());
        assert_eq!(config.iggy.port, 8090);
    }

    #[test]
    fn assessment_config_serialization_roundtrip() {
        let config = AssessmentConfig::default();
        let toml_str = toml::to_string(&config).expect("serialize to toml");
        let parsed: AssessmentConfig = toml::from_str(&toml_str).expect("parse from toml");

        assert_eq!(config.enabled, parsed.enabled);
        assert_eq!(config.intervention_enabled, parsed.intervention_enabled);
        assert_eq!(config.sampling.base_rate, parsed.sampling.base_rate);
        assert_eq!(
            config.circuit_breaker.cooldown_seconds,
            parsed.circuit_breaker.cooldown_seconds
        );
        assert_eq!(config.llm.model, parsed.llm.model);
        assert_eq!(config.retention.heavy_days, parsed.retention.heavy_days);
    }

    #[test]
    fn assessment_config_partial_deserialize() {
        let toml_str = r#"
            enabled = false
            [sampling]
            base_rate = 0.5
        "#;

        let config: AssessmentConfig = toml::from_str(toml_str).expect("parse partial config");

        // Explicitly set values
        assert!(!config.enabled);
        assert_eq!(config.sampling.base_rate, 0.5);

        // Default values for unspecified fields
        assert!(config.intervention_enabled);
        assert_eq!(config.sampling.burnin_sessions, 10);
        assert!(config.session_end.hook_enabled);
        assert_eq!(config.circuit_breaker.cooldown_seconds, 120);
        assert_eq!(config.llm.backend, "harness");
    }

    #[test]
    fn pattern_config_merge_semantics() {
        let mut base = PatternConfig {
            negative: vec!["error".to_string()],
            positive: vec!["success".to_string()],
        };

        let other = PatternConfig {
            negative: vec!["warning".to_string()],
            positive: vec!["complete".to_string()],
        };

        base.merge(&other);

        assert_eq!(base.negative, vec!["error", "warning"]);
        assert_eq!(base.positive, vec!["success", "complete"]);
    }

    #[test]
    fn retention_config_forever_value() {
        let mut config = RetentionConfig::default();

        // Default values
        assert!(!config.lightweight_forever());
        assert!(!config.medium_forever());
        assert!(config.heavy_forever()); // heavy_days defaults to -1

        // Test explicit forever value
        config.lightweight_days = RetentionConfig::FOREVER;
        assert!(config.lightweight_forever());

        // Verify constant value
        assert_eq!(RetentionConfig::FOREVER, -1);
    }
}
