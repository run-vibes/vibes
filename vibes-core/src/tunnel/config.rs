//! Tunnel configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the tunnel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TunnelConfig {
    /// Whether tunnel is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Tunnel mode
    #[serde(default)]
    pub mode: TunnelMode,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: TunnelMode::Quick,
        }
    }
}

/// Tunnel operating mode
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelMode {
    /// Quick tunnel with temporary URL (no account needed)
    #[default]
    Quick,
    /// Named tunnel with persistent hostname
    Named {
        /// Tunnel name from cloudflared
        name: String,
        /// Public hostname
        hostname: String,
        /// Path to credentials file (auto-detected if not specified)
        #[serde(default)]
        credentials_path: Option<PathBuf>,
    },
}

impl TunnelMode {
    /// Get the mode name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quick => "quick",
            Self::Named { .. } => "named",
        }
    }
}

impl TunnelConfig {
    /// Create a quick tunnel config
    pub fn quick() -> Self {
        Self {
            enabled: true,
            mode: TunnelMode::Quick,
        }
    }

    /// Create a named tunnel config
    pub fn named(name: String, hostname: String) -> Self {
        Self {
            enabled: true,
            mode: TunnelMode::Named {
                name,
                hostname,
                credentials_path: None,
            },
        }
    }

    /// Check if this is a quick tunnel
    pub fn is_quick(&self) -> bool {
        matches!(self.mode, TunnelMode::Quick)
    }

    /// Get the tunnel name for named tunnels
    pub fn tunnel_name(&self) -> Option<&str> {
        match &self.mode {
            TunnelMode::Named { name, .. } => Some(name),
            TunnelMode::Quick => None,
        }
    }

    /// Get the hostname for named tunnels
    pub fn hostname(&self) -> Option<&str> {
        match &self.mode {
            TunnelMode::Named { hostname, .. } => Some(hostname),
            TunnelMode::Quick => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunnel_config_default_is_disabled_quick() {
        let config = TunnelConfig::default();
        assert!(!config.enabled);
        assert!(config.is_quick());
    }

    #[test]
    fn tunnel_config_quick_constructor() {
        let config = TunnelConfig::quick();
        assert!(config.enabled);
        assert!(config.is_quick());
        assert!(config.tunnel_name().is_none());
    }

    #[test]
    fn tunnel_config_named_constructor() {
        let config = TunnelConfig::named("my-tunnel".to_string(), "vibes.example.com".to_string());
        assert!(config.enabled);
        assert!(!config.is_quick());
        assert_eq!(config.tunnel_name(), Some("my-tunnel"));
        assert_eq!(config.hostname(), Some("vibes.example.com"));
    }

    #[test]
    fn tunnel_config_quick_has_no_hostname() {
        let config = TunnelConfig::quick();
        assert!(config.hostname().is_none());
    }

    #[test]
    fn tunnel_mode_serialization_quick() {
        let mode = TunnelMode::Quick;
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("quick"));
    }

    #[test]
    fn tunnel_mode_serialization_named() {
        let mode = TunnelMode::Named {
            name: "test".to_string(),
            hostname: "test.example.com".to_string(),
            credentials_path: None,
        };
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("named"));
        assert!(json.contains("test.example.com"));
    }

    #[test]
    fn tunnel_config_toml_roundtrip() {
        let config = TunnelConfig::named("vibes-home".to_string(), "vibes.example.com".to_string());
        let toml = toml::to_string(&config).unwrap();
        let parsed: TunnelConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed, config);
    }
}
