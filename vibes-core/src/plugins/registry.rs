//! Plugin registry - tracks enabled/disabled plugins

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use super::error::PluginHostError;

/// Registry of enabled plugins
///
/// Stored as TOML in `~/.config/vibes/plugins/registry.toml`
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PluginRegistry {
    /// Set of enabled plugin names
    #[serde(default)]
    pub enabled: HashSet<String>,
}

impl PluginRegistry {
    /// Load registry from a TOML file
    ///
    /// Returns an empty registry if the file doesn't exist.
    pub fn load(path: &Path) -> Result<Self, PluginHostError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let registry: Self =
            toml::from_str(&content).map_err(|e| PluginHostError::Registry(e.to_string()))?;
        Ok(registry)
    }

    /// Save registry to a TOML file
    pub fn save(&self, path: &Path) -> Result<(), PluginHostError> {
        let content =
            toml::to_string_pretty(self).map_err(|e| PluginHostError::Registry(e.to_string()))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent().filter(|p| !p.exists()) {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check if a plugin is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.contains(name)
    }

    /// Enable a plugin
    pub fn enable(&mut self, name: &str) {
        self.enabled.insert(name.to_string());
    }

    /// Disable a plugin
    pub fn disable(&mut self, name: &str) {
        self.enabled.remove(name);
    }

    /// Get iterator over enabled plugins
    pub fn enabled_plugins(&self) -> impl Iterator<Item = &str> {
        self.enabled.iter().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_registry_default_is_empty() {
        let registry = PluginRegistry::default();
        assert!(registry.enabled.is_empty());
    }

    #[test]
    fn test_registry_enable_disable() {
        let mut registry = PluginRegistry::default();

        registry.enable("test-plugin");
        assert!(registry.is_enabled("test-plugin"));
        assert!(!registry.is_enabled("other-plugin"));

        registry.disable("test-plugin");
        assert!(!registry.is_enabled("test-plugin"));
    }

    #[test]
    fn test_registry_load_missing_file() {
        let registry = PluginRegistry::load(Path::new("/nonexistent/path/registry.toml")).unwrap();
        assert!(registry.enabled.is_empty());
    }

    #[test]
    fn test_registry_save_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.toml");

        let mut registry = PluginRegistry::default();
        registry.enable("analytics");
        registry.enable("history");
        registry.save(&path).unwrap();

        let loaded = PluginRegistry::load(&path).unwrap();
        assert!(loaded.is_enabled("analytics"));
        assert!(loaded.is_enabled("history"));
        assert!(!loaded.is_enabled("other"));
    }

    #[test]
    fn test_registry_save_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested/dir/registry.toml");

        let registry = PluginRegistry::default();
        registry.save(&path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_registry_enabled_plugins_iterator() {
        let mut registry = PluginRegistry::default();
        registry.enable("a");
        registry.enable("b");

        let enabled: Vec<&str> = registry.enabled_plugins().collect();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&"a"));
        assert!(enabled.contains(&"b"));
    }

    #[test]
    fn test_registry_toml_format() {
        let mut registry = PluginRegistry::default();
        registry.enable("hello");

        let toml_str = toml::to_string_pretty(&registry).unwrap();
        assert!(toml_str.contains("enabled"));
        assert!(toml_str.contains("hello"));
    }
}
