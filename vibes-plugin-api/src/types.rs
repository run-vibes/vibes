//! Plugin types and metadata structures

use serde::{Deserialize, Serialize};

/// Plugin manifest containing metadata about the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (used for CLI commands and identification)
    pub name: String,
    /// Plugin version (semver)
    pub version: String,
    /// API version this plugin was built against
    pub api_version: u32,
    /// Human-readable description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// License type
    pub license: PluginLicense,
    /// Commands this plugin provides
    pub commands: Vec<CommandSpec>,
}

/// Plugin license type
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginLicense {
    /// Free to use
    #[default]
    Free,
    /// Requires purchase
    Paid {
        /// Product identifier for licensing system
        product_id: String,
    },
    /// Trial period
    Trial {
        /// Number of days in trial period
        days: u32,
    },
}

/// Specification for a command this plugin provides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    /// Command name (e.g., "report" -> `vibes <plugin> report`)
    pub name: String,
    /// Command description
    pub description: String,
    /// Command arguments
    pub args: Vec<ArgSpec>,
}

/// Specification for a command argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSpec {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: String,
    /// Whether this argument is required
    pub required: bool,
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.0.1".to_string(),
            api_version: crate::API_VERSION,
            description: String::new(),
            author: String::new(),
            license: PluginLicense::default(),
            commands: Vec::new(),
        }
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Number of input tokens
    pub input_tokens: u32,
    /// Number of output tokens
    pub output_tokens: u32,
}

/// Session state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SessionState {
    /// Session is idle, waiting for input
    Idle,
    /// Session is processing a request
    Processing,
    /// Session is waiting for user input
    WaitingForInput,
    /// Session is waiting for permission to use a tool
    WaitingForPermission {
        /// Request identifier
        request_id: String,
        /// Tool name requiring permission
        tool: String,
    },
    /// Session has completed
    Completed,
    /// Session has failed
    Failed {
        /// Error message
        message: String,
    },
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_default_api_version() {
        let manifest = PluginManifest::default();
        assert_eq!(manifest.api_version, crate::API_VERSION);
    }

    #[test]
    fn test_manifest_toml_roundtrip() {
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            api_version: 1,
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            license: PluginLicense::Free,
            commands: vec![CommandSpec {
                name: "hello".to_string(),
                description: "Say hello".to_string(),
                args: vec![ArgSpec {
                    name: "name".to_string(),
                    description: "Name to greet".to_string(),
                    required: false,
                }],
            }],
        };

        let toml_str = toml::to_string(&manifest).expect("Failed to serialize");
        let parsed: PluginManifest = toml::from_str(&toml_str).expect("Failed to parse");

        assert_eq!(manifest.name, parsed.name);
        assert_eq!(manifest.version, parsed.version);
        assert_eq!(manifest.commands.len(), parsed.commands.len());
    }

    #[test]
    fn test_license_variants_serialize() {
        let free = PluginLicense::Free;
        let paid = PluginLicense::Paid {
            product_id: "prod_123".to_string(),
        };
        let trial = PluginLicense::Trial { days: 30 };

        // Just verify they serialize without error
        let _ = toml::to_string(&free).unwrap();
        let _ = toml::to_string(&paid).unwrap();
        let _ = toml::to_string(&trial).unwrap();
    }

    #[test]
    fn test_session_state_variants() {
        let states = vec![
            SessionState::Idle,
            SessionState::Processing,
            SessionState::WaitingForInput,
            SessionState::WaitingForPermission {
                request_id: "req_1".to_string(),
                tool: "bash".to_string(),
            },
            SessionState::Completed,
            SessionState::Failed {
                message: "Something went wrong".to_string(),
            },
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let parsed: SessionState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, parsed);
        }
    }

    #[test]
    fn test_usage_default() {
        let usage = Usage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
    }
}
