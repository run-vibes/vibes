//! PluginContext - Plugin's interface to vibes-core capabilities

use crate::command::CommandSpec;
use crate::error::PluginError;
use crate::http::RouteSpec;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ─── Capability Enum ─────────────────────────────────────────────────

/// Capabilities that may be available to plugins through the groove/harness system.
///
/// These represent the continual learning features that can be enabled
/// when vibes-groove is integrated with the plugin system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Store and retrieve learnings from past interactions
    LearningStorage,
    /// Semantic vector search for finding similar learnings
    SemanticSearch,
    /// Adaptive parameter tuning based on feedback
    AdaptiveParams,
    /// Multi-tier storage (user/project/enterprise levels)
    MultiTierStorage,
}

// ─── Harness Trait ───────────────────────────────────────────────────

/// Harness trait for groove integration.
///
/// This is the plugin-facing interface for continual learning capabilities.
/// The concrete implementation is provided by vibes-core when groove is enabled.
///
/// Plugins can use this to:
/// - Check what capabilities are available
/// - Access learning storage and retrieval (when implemented)
/// - Integrate with adaptive parameter systems (when implemented)
///
/// # Example
///
/// ```ignore
/// fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
///     if let Some(harness) = ctx.harness() {
///         if harness.has_capability(Capability::SemanticSearch) {
///             ctx.log_info("Semantic search is available!");
///         }
///     }
///     Ok(())
/// }
/// ```
pub trait Harness: Send + Sync {
    /// Get the list of available capabilities
    fn capabilities(&self) -> &[Capability];

    /// Check if a specific capability is available
    fn has_capability(&self, cap: Capability) -> bool {
        self.capabilities().contains(&cap)
    }
}

/// Plugin's interface to vibes-core capabilities.
///
/// This is passed to plugins during lifecycle events and provides access to:
/// - Plugin configuration (persistent key-value store)
/// - Plugin directory for storing data
/// - Logging utilities
/// - Command registration
/// - Harness for groove integration (optional)
/// - Available capabilities
///
/// # Harness vs Capabilities
///
/// The context stores both a `harness` (optional) and a `capabilities` vector.
/// These serve different purposes:
///
/// - `capabilities`: The authoritative list of features available to the plugin,
///   regardless of whether a harness is present. This allows capability checks
///   without requiring the full harness implementation.
///
/// - `harness`: The runtime implementation of groove integration. When present,
///   its `capabilities()` method should return the same set as the context's
///   `capabilities` field.
///
/// When constructing a `PluginContext` with a harness, ensure both are kept in sync.
pub struct PluginContext {
    plugin_name: String,
    plugin_dir: PathBuf,
    config: PluginConfig,
    /// Commands pending registration (using CommandSpec)
    pending_commands: Vec<CommandSpec>,
    /// Routes pending registration
    pending_routes: Vec<RouteSpec>,
    /// Optional harness for groove integration.
    /// When present, its capabilities should match the `capabilities` field.
    harness: Option<Arc<dyn Harness>>,
    /// Authoritative list of capabilities available to this plugin.
    capabilities: Vec<Capability>,
}

/// Plugin configuration - persistent key-value store backed by TOML
pub struct PluginConfig {
    values: HashMap<String, toml::Value>,
    dirty: bool,
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
            pending_commands: Vec::new(),
            pending_routes: Vec::new(),
            harness: None,
            capabilities: Vec::new(),
        }
    }

    /// Create a context with a pre-loaded config
    pub fn with_config(plugin_name: String, plugin_dir: PathBuf, config: PluginConfig) -> Self {
        Self {
            plugin_name,
            plugin_dir,
            config,
            pending_commands: Vec::new(),
            pending_routes: Vec::new(),
            harness: None,
            capabilities: Vec::new(),
        }
    }

    /// Builder: set the harness for groove integration
    pub fn with_harness(mut self, harness: Arc<dyn Harness>) -> Self {
        self.harness = Some(harness);
        self
    }

    /// Builder: set available capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
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

    // ─── CommandSpec Registration ─────────────────────────────────────

    /// Register a CLI command for this plugin using CommandSpec.
    ///
    /// The command will be namespaced under the plugin name:
    /// `vibes <plugin-name> <path...>`
    ///
    /// Returns error if command path is duplicate within this plugin.
    pub fn register_command_spec(&mut self, spec: CommandSpec) -> Result<(), PluginError> {
        if self.pending_commands.iter().any(|c| c.path == spec.path) {
            return Err(PluginError::DuplicateCommand(spec.path.join(" ")));
        }
        self.pending_commands.push(spec);
        Ok(())
    }

    /// Get commands pending registration (used by PluginHost)
    pub fn pending_commands(&self) -> &[CommandSpec] {
        &self.pending_commands
    }

    /// Take pending commands (used by PluginHost after validation)
    pub fn take_pending_commands(&mut self) -> Vec<CommandSpec> {
        std::mem::take(&mut self.pending_commands)
    }

    // ─── Route Registration ───────────────────────────────────────

    /// Register an HTTP route for this plugin.
    ///
    /// Routes are automatically prefixed: `/api/<plugin-name>/...`
    ///
    /// Path parameters use `:name` syntax: `/quarantine/:id/review`
    ///
    /// Returns error if route (same method+path) is duplicate within this plugin.
    pub fn register_route(&mut self, spec: RouteSpec) -> Result<(), PluginError> {
        if self
            .pending_routes
            .iter()
            .any(|r| r.method == spec.method && r.path == spec.path)
        {
            return Err(PluginError::DuplicateRoute(format!(
                "{:?} {}",
                spec.method, spec.path
            )));
        }
        self.pending_routes.push(spec);
        Ok(())
    }

    /// Get routes pending registration (used by PluginHost)
    pub fn pending_routes(&self) -> &[RouteSpec] {
        &self.pending_routes
    }

    /// Take pending routes (used by PluginHost after validation)
    pub fn take_pending_routes(&mut self) -> Vec<RouteSpec> {
        std::mem::take(&mut self.pending_routes)
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

    // ─── Harness & Capabilities ─────────────────────────────────────

    /// Get the harness for groove integration (if enabled)
    ///
    /// Returns `None` if groove is not enabled or the harness has not been set.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(harness) = ctx.harness() {
    ///     if harness.has_capability(Capability::SemanticSearch) {
    ///         // Use semantic search features
    ///     }
    /// }
    /// ```
    pub fn harness(&self) -> Option<&dyn Harness> {
        self.harness.as_deref()
    }

    /// Get the list of available capabilities
    ///
    /// This returns the capabilities that are available to the plugin,
    /// which may be empty if groove is not enabled.
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
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

    // ─── CommandSpec Registration Tests ─────────────────────────────────

    #[test]
    fn test_register_command_spec() {
        use crate::command::CommandSpec;

        let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

        let result = ctx.register_command_spec(CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust levels".into(),
            args: vec![],
        });

        assert!(result.is_ok());
        assert_eq!(ctx.pending_commands().len(), 1);
    }

    #[test]
    fn test_register_command_spec_duplicate_fails() {
        use crate::command::CommandSpec;

        let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

        let spec = CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust levels".into(),
            args: vec![],
        };

        ctx.register_command_spec(spec.clone()).unwrap();
        let result = ctx.register_command_spec(spec);

        assert!(result.is_err());
    }

    #[test]
    fn test_take_pending_commands() {
        use crate::command::CommandSpec;

        let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

        ctx.register_command_spec(CommandSpec {
            path: vec!["foo".into()],
            description: "Foo".into(),
            args: vec![],
        })
        .unwrap();

        let commands = ctx.take_pending_commands();
        assert_eq!(commands.len(), 1);
        assert!(ctx.pending_commands().is_empty());
    }

    #[test]
    fn test_command_args() {
        let mut args = CommandArgs::default();
        args.args.push("arg1".to_string());
        args.flags.insert("verbose".to_string(), "true".to_string());

        assert_eq!(args.args.len(), 1);
        assert_eq!(args.flags.get("verbose"), Some(&"true".to_string()));
    }

    // ─── Capability and Harness Tests ────────────────────────────────

    #[test]
    fn test_capabilities_default_empty() {
        let ctx = PluginContext::new("test".to_string(), PathBuf::from("/tmp"));
        assert!(ctx.capabilities().is_empty());
    }

    #[test]
    fn test_harness_not_available_by_default() {
        let ctx = PluginContext::new("test".to_string(), PathBuf::from("/tmp"));
        assert!(ctx.harness().is_none());
    }

    #[test]
    fn test_with_capabilities() {
        let ctx =
            PluginContext::new("test".to_string(), PathBuf::from("/tmp")).with_capabilities(vec![
                Capability::SemanticSearch,
                Capability::LearningStorage,
            ]);
        assert!(ctx.capabilities().contains(&Capability::SemanticSearch));
        assert!(ctx.capabilities().contains(&Capability::LearningStorage));
        assert!(!ctx.capabilities().contains(&Capability::AdaptiveParams));
    }

    #[test]
    fn test_capability_equality() {
        assert_eq!(Capability::SemanticSearch, Capability::SemanticSearch);
        assert_ne!(Capability::SemanticSearch, Capability::LearningStorage);
    }

    #[test]
    fn test_harness_has_capability() {
        struct TestHarness {
            caps: Vec<Capability>,
        }
        impl Harness for TestHarness {
            fn capabilities(&self) -> &[Capability] {
                &self.caps
            }
        }

        let harness = TestHarness {
            caps: vec![Capability::SemanticSearch, Capability::LearningStorage],
        };
        assert!(harness.has_capability(Capability::SemanticSearch));
        assert!(harness.has_capability(Capability::LearningStorage));
        assert!(!harness.has_capability(Capability::AdaptiveParams));
    }

    #[test]
    fn test_with_harness() {
        use std::sync::Arc;

        struct TestHarness {
            caps: Vec<Capability>,
        }
        impl Harness for TestHarness {
            fn capabilities(&self) -> &[Capability] {
                &self.caps
            }
        }

        let harness = Arc::new(TestHarness {
            caps: vec![Capability::MultiTierStorage],
        });

        let ctx =
            PluginContext::new("test".to_string(), PathBuf::from("/tmp")).with_harness(harness);

        assert!(ctx.harness().is_some());
        let h = ctx.harness().unwrap();
        assert!(h.has_capability(Capability::MultiTierStorage));
    }

    // ─── RouteSpec Registration Tests ─────────────────────────────────

    #[test]
    fn test_register_route() {
        use crate::http::{HttpMethod, RouteSpec};

        let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

        let result = ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/policy".into(),
        });

        assert!(result.is_ok());
        assert_eq!(ctx.pending_routes().len(), 1);
    }

    #[test]
    fn test_register_route_duplicate_fails() {
        use crate::http::{HttpMethod, RouteSpec};

        let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

        let spec = RouteSpec {
            method: HttpMethod::Get,
            path: "/policy".into(),
        };

        ctx.register_route(spec.clone()).unwrap();
        let result = ctx.register_route(spec);

        assert!(result.is_err());
    }

    #[test]
    fn test_same_path_different_method_allowed() {
        use crate::http::{HttpMethod, RouteSpec};

        let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/resource".into(),
        })
        .unwrap();

        let result = ctx.register_route(RouteSpec {
            method: HttpMethod::Post,
            path: "/resource".into(),
        });

        assert!(result.is_ok());
        assert_eq!(ctx.pending_routes().len(), 2);
    }

    #[test]
    fn test_take_pending_routes() {
        use crate::http::{HttpMethod, RouteSpec};

        let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/test".into(),
        })
        .unwrap();

        let routes = ctx.take_pending_routes();
        assert_eq!(routes.len(), 1);
        assert!(ctx.pending_routes().is_empty());
    }
}
