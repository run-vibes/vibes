//! Configuration types for vibes-groove storage tiers.
//!
//! Provides configuration for the three-tier storage architecture:
//! - User tier: Personal preferences and history
//! - Project tier: Project-specific settings
//! - Enterprise tier: Organization-wide policies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::extraction::DEFAULT_SIMILARITY_THRESHOLD;
use crate::extraction::patterns::CorrectionConfig;

/// Configuration for the groove storage system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrooveConfig {
    /// Path to user tier database
    pub user_db_path: PathBuf,
    /// Configured enterprises
    pub enterprises: HashMap<String, EnterpriseConfig>,
    /// Deduplication settings
    #[serde(default)]
    pub deduplication: DeduplicationConfig,
    /// Correction pattern detection settings
    #[serde(default)]
    pub correction: CorrectionConfig,
}

/// Configuration for semantic deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationConfig {
    /// Whether deduplication is enabled
    pub enabled: bool,
    /// Similarity threshold (0.0-1.0) for considering learnings as duplicates
    pub similarity_threshold: f64,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            similarity_threshold: DEFAULT_SIMILARITY_THRESHOLD,
        }
    }
}

/// Configuration for an enterprise tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Organization identifier
    pub org_id: String,
    /// Path to enterprise database
    pub db_path: PathBuf,
    /// Optional sync URL for remote synchronization
    pub sync_url: Option<String>,
}

/// Context for determining which storage tier to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectContext {
    /// Personal project, uses user tier only
    Personal,
    /// Enterprise project, uses enterprise tier
    Enterprise {
        /// Organization identifier
        org_id: String,
    },
}

impl Default for GrooveConfig {
    fn default() -> Self {
        let user_db_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vibes/groove/user");

        Self {
            user_db_path,
            enterprises: HashMap::new(),
            deduplication: DeduplicationConfig::default(),
            correction: CorrectionConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GrooveConfig::default();
        assert!(config.user_db_path.to_string_lossy().contains("vibes"));
        assert!(config.enterprises.is_empty());
    }

    #[test]
    fn test_project_context_personal() {
        let ctx = ProjectContext::Personal;
        assert!(matches!(ctx, ProjectContext::Personal));
    }

    #[test]
    fn test_project_context_enterprise() {
        let ctx = ProjectContext::Enterprise {
            org_id: "acme".into(),
        };
        if let ProjectContext::Enterprise { org_id } = ctx {
            assert_eq!(org_id, "acme");
        } else {
            panic!("Expected Enterprise variant");
        }
    }

    #[test]
    fn test_config_serialization() {
        let config = GrooveConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: GrooveConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.user_db_path, config.user_db_path);
    }

    #[test]
    fn test_deduplication_config_defaults() {
        let config = DeduplicationConfig::default();
        assert!(config.enabled);
        assert!((config.similarity_threshold - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deduplication_config_serialization() {
        let config = DeduplicationConfig {
            enabled: false,
            similarity_threshold: 0.85,
        };
        let toml = toml::to_string(&config).unwrap();
        let parsed: DeduplicationConfig = toml::from_str(&toml).unwrap();
        assert!(!parsed.enabled);
        assert!((parsed.similarity_threshold - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_groove_config_includes_deduplication() {
        let config = GrooveConfig::default();
        assert!(config.deduplication.enabled);
    }
}
