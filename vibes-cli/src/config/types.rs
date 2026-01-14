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
    pub ollama: OllamaConfigSection,

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
    pub ollama: OllamaConfigSection,

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

/// Default Ollama host
pub const DEFAULT_OLLAMA_HOST: &str = "localhost:11434";

/// Ollama configuration for local LLM autostart
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OllamaConfigSection {
    /// Auto-start Ollama when vibes starts (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Custom host for Ollama (default: localhost:11434)
    pub host: Option<String>,
}

impl OllamaConfigSection {
    /// Get the host, using default if not specified.
    #[must_use]
    pub fn host_or_default(&self) -> &str {
        self.host.as_deref().unwrap_or(DEFAULT_OLLAMA_HOST)
    }

    /// Get the base URL for HTTP requests.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://{}", self.host_or_default())
    }
}

/// Default HTTP port for Iggy REST API
/// Uses 7431 to avoid conflicts with common dev servers (3000-3999 range)
pub const DEFAULT_IGGY_HTTP_PORT: u16 = 7431;

/// Configuration for connecting to Iggy HTTP API
#[derive(Debug, Clone, Deserialize)]
pub struct IggyClientConfig {
    /// Iggy server host
    pub host: String,

    /// Iggy HTTP port (REST API)
    pub http_port: u16,

    /// Username for Iggy authentication
    pub username: String,

    /// Password for Iggy authentication
    pub password: String,
}

impl Default for IggyClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            http_port: DEFAULT_IGGY_HTTP_PORT,
            username: "iggy".to_string(),
            password: "iggy".to_string(),
        }
    }
}

impl IggyClientConfig {
    /// Load from environment variables, falling back to defaults.
    ///
    /// Environment variables:
    /// - `VIBES_IGGY_HOST` (default: `127.0.0.1`)
    /// - `VIBES_IGGY_HTTP_PORT` (default: `3001`)
    /// - `VIBES_IGGY_USERNAME` (default: `iggy`)
    /// - `VIBES_IGGY_PASSWORD` (default: `iggy`)
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("VIBES_IGGY_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            http_port: std::env::var("VIBES_IGGY_HTTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_IGGY_HTTP_PORT),
            username: std::env::var("VIBES_IGGY_USERNAME").unwrap_or_else(|_| "iggy".to_string()),
            password: std::env::var("VIBES_IGGY_PASSWORD").unwrap_or_else(|_| "iggy".to_string()),
        }
    }

    /// Get the base URL for HTTP requests
    #[must_use]
    #[allow(dead_code)]
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.http_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== IggyClientConfig Tests ====================

    #[test]
    fn iggy_config_default_values() {
        let config = IggyClientConfig::default();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.http_port, DEFAULT_IGGY_HTTP_PORT);
        assert_eq!(config.http_port, 7431);
        assert_eq!(config.username, "iggy");
        assert_eq!(config.password, "iggy");
    }

    #[test]
    fn iggy_config_base_url() {
        let config = IggyClientConfig::default();
        assert_eq!(config.base_url(), "http://127.0.0.1:7431");

        let custom = IggyClientConfig {
            host: "iggy.example.com".to_string(),
            http_port: 8080,
            ..Default::default()
        };
        assert_eq!(custom.base_url(), "http://iggy.example.com:8080");
    }

    #[test]
    fn iggy_config_from_env_uses_defaults_when_no_env() {
        // Clear any existing env vars (they shouldn't be set in test env)
        // SAFETY: Tests run with --test-threads=1 to prevent concurrent access
        unsafe {
            std::env::remove_var("VIBES_IGGY_HOST");
            std::env::remove_var("VIBES_IGGY_HTTP_PORT");
            std::env::remove_var("VIBES_IGGY_USERNAME");
            std::env::remove_var("VIBES_IGGY_PASSWORD");
        }

        let config = IggyClientConfig::from_env();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.http_port, 7431);
        assert_eq!(config.username, "iggy");
        assert_eq!(config.password, "iggy");
    }

    #[test]
    fn iggy_config_from_env_reads_host() {
        // SAFETY: Tests run with --test-threads=1 to prevent concurrent access
        unsafe {
            std::env::set_var("VIBES_IGGY_HOST", "remote.iggy.io");
        }
        let config = IggyClientConfig::from_env();
        unsafe {
            std::env::remove_var("VIBES_IGGY_HOST");
        }

        assert_eq!(config.host, "remote.iggy.io");
    }

    #[test]
    fn iggy_config_from_env_reads_http_port() {
        // SAFETY: Tests run with --test-threads=1 to prevent concurrent access
        unsafe {
            std::env::set_var("VIBES_IGGY_HTTP_PORT", "9000");
        }
        let config = IggyClientConfig::from_env();
        unsafe {
            std::env::remove_var("VIBES_IGGY_HTTP_PORT");
        }

        assert_eq!(config.http_port, 9000);
    }

    #[test]
    fn iggy_config_from_env_invalid_port_uses_default() {
        // SAFETY: Tests run with --test-threads=1 to prevent concurrent access
        unsafe {
            std::env::set_var("VIBES_IGGY_HTTP_PORT", "not-a-number");
        }
        let config = IggyClientConfig::from_env();
        unsafe {
            std::env::remove_var("VIBES_IGGY_HTTP_PORT");
        }

        assert_eq!(config.http_port, DEFAULT_IGGY_HTTP_PORT);
    }

    #[test]
    fn iggy_config_from_env_reads_credentials() {
        // SAFETY: Tests run with --test-threads=1 to prevent concurrent access
        unsafe {
            std::env::set_var("VIBES_IGGY_USERNAME", "admin");
            std::env::set_var("VIBES_IGGY_PASSWORD", "secret123");
        }
        let config = IggyClientConfig::from_env();
        unsafe {
            std::env::remove_var("VIBES_IGGY_USERNAME");
            std::env::remove_var("VIBES_IGGY_PASSWORD");
        }

        assert_eq!(config.username, "admin");
        assert_eq!(config.password, "secret123");
    }

    // ==================== VibesConfig Tests ====================

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
            ollama: OllamaConfigSection::default(),
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

    // ==================== OllamaConfigSection Tests ====================

    #[test]
    fn ollama_config_defaults() {
        let config = OllamaConfigSection::default();
        assert!(!config.enabled);
        assert!(config.host.is_none());
        assert_eq!(config.host_or_default(), DEFAULT_OLLAMA_HOST);
    }

    #[test]
    fn ollama_config_parsing() {
        let toml_str = r#"
[ollama]
enabled = true
host = "192.168.1.100:11434"
"#;
        let config: RawVibesConfig = toml::from_str(toml_str).unwrap();
        assert!(config.ollama.enabled);
        assert_eq!(config.ollama.host, Some("192.168.1.100:11434".to_string()));
    }

    #[test]
    fn ollama_config_empty_uses_defaults() {
        let config: RawVibesConfig = toml::from_str("").unwrap();
        assert!(!config.ollama.enabled);
        assert!(config.ollama.host.is_none());
    }

    #[test]
    fn ollama_config_host_or_default() {
        let default_config = OllamaConfigSection::default();
        assert_eq!(default_config.host_or_default(), "localhost:11434");

        let custom_config = OllamaConfigSection {
            enabled: true,
            host: Some("custom:8080".to_string()),
        };
        assert_eq!(custom_config.host_or_default(), "custom:8080");
    }

    #[test]
    fn ollama_config_base_url() {
        let default_config = OllamaConfigSection::default();
        assert_eq!(default_config.base_url(), "http://localhost:11434");

        let custom_config = OllamaConfigSection {
            enabled: true,
            host: Some("192.168.1.100:11434".to_string()),
        };
        assert_eq!(custom_config.base_url(), "http://192.168.1.100:11434");
    }
}
