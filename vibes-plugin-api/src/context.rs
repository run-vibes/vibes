//! PluginContext - Plugin's interface to vibes-core capabilities

use crate::error::PluginError;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin's interface to vibes-core capabilities.
///
/// This is passed to plugins during lifecycle events and provides access to:
/// - Plugin configuration (persistent key-value store)
/// - Plugin directory for storing data
/// - Logging utilities
/// - Command registration
pub struct PluginContext {
    plugin_name: String,
    plugin_dir: PathBuf,
    config: PluginConfig,
    registered_commands: Vec<RegisteredCommand>,
}

/// Plugin configuration - persistent key-value store backed by TOML
pub struct PluginConfig {
    values: HashMap<String, toml::Value>,
    dirty: bool,
}

/// A command registered by a plugin
pub struct RegisteredCommand {
    /// Command name
    pub name: String,
}

/// Arguments passed to a command handler
#[derive(Debug, Default)]
pub struct CommandArgs {
    /// Positional arguments
    pub args: Vec<String>,
    /// Named flags (--flag=value or --flag value)
    pub flags: HashMap<String, String>,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(plugin_name: String, plugin_dir: PathBuf) -> Self {
        Self {
            plugin_name,
            plugin_dir,
            config: PluginConfig::new(),
            registered_commands: Vec::new(),
        }
    }

    /// Create a context with a pre-loaded config
    pub fn with_config(plugin_name: String, plugin_dir: PathBuf, config: PluginConfig) -> Self {
        Self {
            plugin_name,
            plugin_dir,
            config,
            registered_commands: Vec::new(),
        }
    }

    // ─── Configuration ───────────────────────────────────────────────

    /// Get the plugin's directory (for storing data files)
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    /// Get the plugin's name
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    /// Read a configuration value
    ///
    /// # Example
    /// ```ignore
    /// let threshold: Option<u32> = ctx.config_get("threshold");
    /// ```
    pub fn config_get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.config.get(key)
    }

    /// Write a configuration value
    ///
    /// The configuration is automatically persisted when the plugin context is saved.
    ///
    /// # Example
    /// ```ignore
    /// ctx.config_set("threshold", 100)?;
    /// ```
    pub fn config_set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), PluginError> {
        self.config.set(key, value)
    }

    /// Check if the configuration has unsaved changes
    pub fn config_is_dirty(&self) -> bool {
        self.config.is_dirty()
    }

    /// Get a mutable reference to the config (for internal use by PluginHost)
    pub fn config_mut(&mut self) -> &mut PluginConfig {
        &mut self.config
    }

    // ─── Command Registration ────────────────────────────────────────

    /// Register a command that this plugin provides
    ///
    /// The command will be available as `vibes <plugin-name> <command-name>`
    pub fn register_command(&mut self, name: &str) {
        self.registered_commands.push(RegisteredCommand {
            name: name.to_string(),
        });
    }

    /// Get the list of registered commands
    pub fn registered_commands(&self) -> &[RegisteredCommand] {
        &self.registered_commands
    }

    // ─── Logging ─────────────────────────────────────────────────────

    /// Log an info message (automatically prefixed with plugin name)
    pub fn log_info(&self, message: &str) {
        tracing::info!(plugin = %self.plugin_name, "{}", message);
    }

    /// Log a warning message
    pub fn log_warn(&self, message: &str) {
        tracing::warn!(plugin = %self.plugin_name, "{}", message);
    }

    /// Log an error message
    pub fn log_error(&self, message: &str) {
        tracing::error!(plugin = %self.plugin_name, "{}", message);
    }

    /// Log a debug message
    pub fn log_debug(&self, message: &str) {
        tracing::debug!(plugin = %self.plugin_name, "{}", message);
    }
}

impl PluginConfig {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            dirty: false,
        }
    }

    /// Load configuration from a TOML file
    pub fn load(path: &Path) -> Result<Self, PluginError> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        let values: HashMap<String, toml::Value> =
            toml::from_str(&content).map_err(|e| PluginError::Config(e.to_string()))?;
        Ok(Self {
            values,
            dirty: false,
        })
    }

    /// Save configuration to a TOML file
    pub fn save(&mut self, path: &Path) -> Result<(), PluginError> {
        let content = toml::to_string_pretty(&self.values)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent().filter(|p| !p.exists()) {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        self.dirty = false;
        Ok(())
    }

    /// Get a configuration value
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.values.get(key).and_then(|v| v.clone().try_into().ok())
    }

    /// Set a configuration value
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), PluginError> {
        let toml_value =
            toml::Value::try_from(value).map_err(|e| PluginError::Serialization(e.to_string()))?;
        self.values.insert(key.to_string(), toml_value);
        self.dirty = true;
        Ok(())
    }

    /// Check if the config has been modified since loading/saving
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the config as clean (internal use after save)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_context_creation() {
        let ctx = PluginContext::new("test".to_string(), PathBuf::from("/tmp/test"));
        assert_eq!(ctx.plugin_name(), "test");
        assert_eq!(ctx.plugin_dir(), Path::new("/tmp/test"));
    }

    #[test]
    fn test_config_get_set() {
        let mut config = PluginConfig::new();

        config.set("string_key", "hello").unwrap();
        config.set("int_key", 42i64).unwrap();
        config.set("bool_key", true).unwrap();

        assert_eq!(
            config.get::<String>("string_key"),
            Some("hello".to_string())
        );
        assert_eq!(config.get::<i64>("int_key"), Some(42));
        assert_eq!(config.get::<bool>("bool_key"), Some(true));
        assert_eq!(config.get::<String>("missing"), None);
    }

    #[test]
    fn test_config_dirty_tracking() {
        let mut config = PluginConfig::new();
        assert!(!config.is_dirty());

        config.set("key", "value").unwrap();
        assert!(config.is_dirty());

        config.mark_clean();
        assert!(!config.is_dirty());
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");

        // Save config
        let mut config = PluginConfig::new();
        config.set("name", "test-plugin").unwrap();
        config.set("threshold", 100i64).unwrap();
        config.save(&config_path).unwrap();

        // Load config
        let loaded = PluginConfig::load(&config_path).unwrap();
        assert_eq!(
            loaded.get::<String>("name"),
            Some("test-plugin".to_string())
        );
        assert_eq!(loaded.get::<i64>("threshold"), Some(100));
    }

    #[test]
    fn test_config_load_missing_file() {
        let config = PluginConfig::load(Path::new("/nonexistent/path/config.toml")).unwrap();
        assert!(config.values.is_empty());
    }

    #[test]
    fn test_command_registration() {
        let mut ctx = PluginContext::new("test".to_string(), PathBuf::from("/tmp"));

        ctx.register_command("hello");
        ctx.register_command("goodbye");

        let commands = ctx.registered_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].name, "hello");
        assert_eq!(commands[1].name, "goodbye");
    }

    #[test]
    fn test_command_args() {
        let mut args = CommandArgs::default();
        args.args.push("arg1".to_string());
        args.flags.insert("verbose".to_string(), "true".to_string());

        assert_eq!(args.args.len(), 1);
        assert_eq!(args.flags.get("verbose"), Some(&"true".to_string()));
    }
}
