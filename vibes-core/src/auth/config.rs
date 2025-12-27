//! Configuration for Cloudflare Access authentication

use serde::{Deserialize, Serialize};

/// Configuration for Cloudflare Access authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessConfig {
    /// Whether authentication is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Cloudflare Access team name (e.g., "mycompany" for mycompany.cloudflareaccess.com)
    #[serde(default)]
    pub team: String,

    /// Application audience (AUD) tag from Cloudflare Access
    #[serde(default)]
    pub aud: String,

    /// Whether to bypass authentication for localhost requests
    #[serde(default = "default_bypass_localhost")]
    pub bypass_localhost: bool,

    /// Clock skew leeway in seconds for token expiry validation
    #[serde(default = "default_clock_skew")]
    pub clock_skew_seconds: u64,
}

fn default_bypass_localhost() -> bool {
    true
}

fn default_clock_skew() -> u64 {
    60
}

impl Default for AccessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            team: String::new(),
            aud: String::new(),
            bypass_localhost: default_bypass_localhost(),
            clock_skew_seconds: default_clock_skew(),
        }
    }
}

impl AccessConfig {
    /// Create a new AccessConfig with the given team and AUD
    pub fn new(team: impl Into<String>, aud: impl Into<String>) -> Self {
        Self {
            enabled: true,
            team: team.into(),
            aud: aud.into(),
            bypass_localhost: true,
            clock_skew_seconds: 60,
        }
    }

    /// Returns the JWKS URL for this team
    pub fn jwks_url(&self) -> String {
        format!(
            "https://{}.cloudflareaccess.com/cdn-cgi/access/certs",
            self.team
        )
    }

    /// Check if the config is valid (has required fields when enabled)
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return true;
        }
        !self.team.is_empty() && !self.aud.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AccessConfig::default();
        assert!(!config.enabled);
        assert!(config.bypass_localhost);
        assert_eq!(config.clock_skew_seconds, 60);
    }

    #[test]
    fn test_new_config() {
        let config = AccessConfig::new("myteam", "aud123");
        assert!(config.enabled);
        assert_eq!(config.team, "myteam");
        assert_eq!(config.aud, "aud123");
    }

    #[test]
    fn test_jwks_url() {
        let config = AccessConfig::new("myteam", "aud123");
        assert_eq!(
            config.jwks_url(),
            "https://myteam.cloudflareaccess.com/cdn-cgi/access/certs"
        );
    }

    #[test]
    fn test_is_valid_disabled() {
        let config = AccessConfig::default();
        assert!(config.is_valid());
    }

    #[test]
    fn test_is_valid_enabled_with_fields() {
        let config = AccessConfig::new("team", "aud");
        assert!(config.is_valid());
    }

    #[test]
    fn test_is_valid_enabled_missing_fields() {
        let mut config = AccessConfig::default();
        config.enabled = true;
        assert!(!config.is_valid());
    }

    #[test]
    fn test_deserialize_toml() {
        let toml = r#"
            enabled = true
            team = "mycompany"
            aud = "abc123"
        "#;
        let config: AccessConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.team, "mycompany");
        assert_eq!(config.aud, "abc123");
        assert!(config.bypass_localhost); // default
    }
}
