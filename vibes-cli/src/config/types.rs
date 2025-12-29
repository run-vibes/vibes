use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use vibes_core::AccessConfig;

/// Default host for the vibes server
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Default port for the vibes server
pub const DEFAULT_PORT: u16 = 7432;

/// Configuration as stored in TOML files (with optional fields for merging)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawVibesConfig {
    #[serde(default)]
    pub server: RawServerConfig,

    #[serde(default)]
    pub session: SessionConfig,

    #[serde(default)]
    pub tunnel: TunnelConfigSection,

    #[serde(default)]
    pub auth: AccessConfig,
}

/// Server config as stored in TOML (optional fields for proper merging)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawServerConfig {
    /// Host to bind to (default: 127.0.0.1)
    pub host: Option<String>,

    /// Port for the vibes server
    pub port: Option<u16>,

    /// Auto-start server with vibes claude
    pub auto_start: Option<bool>,
}

/// Final configuration with defaults applied
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VibesConfig {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub session: SessionConfig,

    #[serde(default)]
    pub tunnel: TunnelConfigSection,

    #[serde(default)]
    pub auth: AccessConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port for the vibes server
    #[serde(default = "default_port")]
    pub port: u16,

    /// Auto-start server with vibes claude
    #[serde(default = "default_true")]
    pub auto_start: bool,
}

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_true() -> bool {
    true
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            auto_start: true,
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

/// Tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TunnelConfigSection {
    /// Auto-start tunnel with serve
    #[serde(default)]
    pub enabled: bool,

    /// Tunnel mode: "quick" or "named" (defaults to "quick" when None)
    pub mode: Option<String>,

    /// Tunnel name (for named mode)
    pub name: Option<String>,

    /// Public hostname (for named mode)
    pub hostname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = VibesConfig::default();
        assert_eq!(config.server.host, DEFAULT_HOST);
        assert_eq!(config.server.port, DEFAULT_PORT);
        assert!(config.server.auto_start);
        assert!(config.session.default_model.is_none());
        assert!(config.session.default_allowed_tools.is_none());
        assert!(config.session.working_dir.is_none());
    }

    #[test]
    fn test_toml_round_trip() {
        let config = VibesConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                auto_start: false,
            },
            session: SessionConfig {
                default_model: Some("claude-opus-4-5".to_string()),
                default_allowed_tools: Some(vec!["Read".to_string(), "Write".to_string()]),
                working_dir: Some(PathBuf::from("/tmp")),
            },
            tunnel: TunnelConfigSection::default(),
            auth: AccessConfig::default(),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: VibesConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.server.host, "0.0.0.0");
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
    fn test_raw_config_partial_parsing() {
        let toml_str = r#"
[server]
port = 9000
"#;
        let raw: RawVibesConfig = toml::from_str(toml_str).unwrap();

        // Only port was set, host and auto_start should be None
        assert!(raw.server.host.is_none());
        assert_eq!(raw.server.port, Some(9000));
        assert!(raw.server.auto_start.is_none());
        assert!(raw.session.default_model.is_none());
    }

    #[test]
    fn test_raw_config_empty_uses_none() {
        let raw: RawVibesConfig = toml::from_str("").unwrap();

        // Empty config should have all None values
        assert!(raw.server.host.is_none());
        assert!(raw.server.port.is_none());
        assert!(raw.server.auto_start.is_none());
    }

    #[test]
    fn test_tunnel_config_defaults() {
        let config = TunnelConfigSection::default();
        assert!(!config.enabled);
        assert!(config.mode.is_none());
        assert!(config.name.is_none());
    }

    #[test]
    fn test_tunnel_config_parsing() {
        let toml_str = r#"
[tunnel]
enabled = true
mode = "named"
name = "vibes-home"
hostname = "vibes.example.com"
"#;
        let config: RawVibesConfig = toml::from_str(toml_str).unwrap();
        assert!(config.tunnel.enabled);
        assert_eq!(config.tunnel.mode, Some("named".to_string()));
        assert_eq!(config.tunnel.name, Some("vibes-home".to_string()));
    }
}
