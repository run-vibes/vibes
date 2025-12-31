//! Configuration for Iggy server and client.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Configuration for the Iggy server subprocess.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IggyConfig {
    /// Path to the iggy-server binary.
    #[serde(default = "default_binary_path")]
    pub binary_path: PathBuf,

    /// Directory where Iggy stores its data.
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// TCP port for Iggy server.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Interval between health checks.
    #[serde(default = "default_health_check_interval", with = "humantime_serde")]
    pub health_check_interval: Duration,

    /// Maximum number of restart attempts before giving up.
    #[serde(default = "default_max_restart_attempts")]
    pub max_restart_attempts: u32,
}

fn default_binary_path() -> PathBuf {
    PathBuf::from("iggy-server")
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("vibes").join("iggy"))
        .unwrap_or_else(|| PathBuf::from("/tmp/vibes/iggy"))
}

fn default_port() -> u16 {
    8090
}

fn default_health_check_interval() -> Duration {
    Duration::from_secs(5)
}

fn default_max_restart_attempts() -> u32 {
    3
}

impl Default for IggyConfig {
    fn default() -> Self {
        Self {
            binary_path: default_binary_path(),
            data_dir: default_data_dir(),
            port: default_port(),
            health_check_interval: default_health_check_interval(),
            max_restart_attempts: default_max_restart_attempts(),
        }
    }
}

impl IggyConfig {
    /// Create a new config with a custom binary path.
    #[must_use]
    pub fn with_binary_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = path.into();
        self
    }

    /// Create a new config with a custom data directory.
    #[must_use]
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = path.into();
        self
    }

    /// Create a new config with a custom port.
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Get the TCP connection address for clients.
    #[must_use]
    pub fn connection_address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_values() {
        let config = IggyConfig::default();

        assert_eq!(config.binary_path, PathBuf::from("iggy-server"));
        assert_eq!(config.port, 8090);
        assert_eq!(config.max_restart_attempts, 3);
        assert_eq!(config.health_check_interval, Duration::from_secs(5));
    }

    #[test]
    fn config_builder_pattern() {
        let config = IggyConfig::default()
            .with_binary_path("/usr/bin/iggy")
            .with_data_dir("/var/lib/iggy")
            .with_port(9000);

        assert_eq!(config.binary_path, PathBuf::from("/usr/bin/iggy"));
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/iggy"));
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn config_connection_address() {
        let config = IggyConfig::default().with_port(8091);
        assert_eq!(config.connection_address(), "127.0.0.1:8091");
    }
}
