use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VibesConfig {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Port for the vibes server
    #[serde(default = "default_port")]
    pub port: u16,

    /// Auto-start server with vibes claude
    #[serde(default = "default_true")]
    pub auto_start: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            auto_start: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionConfig {
    /// Default model for new sessions
    pub default_model: Option<String>,

    /// Default allowed tools
    pub default_allowed_tools: Option<Vec<String>>,

    /// Default working directory
    pub working_dir: Option<PathBuf>,
}

fn default_port() -> u16 {
    7432
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = VibesConfig::default();
        assert_eq!(config.server.port, 7432);
        assert!(config.server.auto_start);
        assert!(config.session.default_model.is_none());
        assert!(config.session.default_allowed_tools.is_none());
        assert!(config.session.working_dir.is_none());
    }

    #[test]
    fn test_toml_round_trip() {
        let config = VibesConfig {
            server: ServerConfig {
                port: 8080,
                auto_start: false,
            },
            session: SessionConfig {
                default_model: Some("claude-opus-4-5".to_string()),
                default_allowed_tools: Some(vec!["Read".to_string(), "Write".to_string()]),
                working_dir: Some(PathBuf::from("/tmp")),
            },
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: VibesConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.server.port, 8080);
        assert!(!parsed.server.auto_start);
        assert_eq!(
            parsed.session.default_model,
            Some("claude-opus-4-5".to_string())
        );
        assert_eq!(
            parsed.session.default_allowed_tools,
            Some(vec!["Read".to_string(), "Write".to_string()])
        );
    }

    #[test]
    fn test_partial_config_with_defaults() {
        let toml_str = r#"
[server]
port = 9000
"#;
        let config: VibesConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.server.port, 9000);
        assert!(config.server.auto_start); // default
        assert!(config.session.default_model.is_none()); // default
    }

    #[test]
    fn test_empty_config_uses_all_defaults() {
        let config: VibesConfig = toml::from_str("").unwrap();

        assert_eq!(config.server.port, 7432);
        assert!(config.server.auto_start);
    }
}
