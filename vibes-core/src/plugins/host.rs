//! PluginHost - manages plugin lifecycle and event dispatch

use libloading::Library;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::time::Duration;

use vibes_plugin_api::{API_VERSION, Plugin, PluginConfig, PluginContext, PluginManifest};

use super::commands::CommandRegistry;
use super::error::PluginHostError;
use super::registry::PluginRegistry;
use super::routes::RouteRegistry;
use crate::events::{ClaudeEvent, VibesEvent};

/// A loaded plugin with its runtime state
struct LoadedPlugin {
    /// Plugin manifest (metadata)
    manifest: PluginManifest,
    /// The plugin instance
    instance: Box<dyn Plugin>,
    /// Plugin context for callbacks
    context: PluginContext,
    /// Keep the library loaded
    _library: Library,
    /// Current plugin state
    state: PluginState,
}

/// State of a loaded plugin
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    /// Plugin is loaded and active
    Loaded,
    /// Plugin is disabled
    Disabled { reason: String },
    /// Plugin has failed (panicked or errored)
    Failed { error: String },
}

/// Configuration for PluginHost
pub struct PluginHostConfig {
    /// User plugin directory (~/.config/vibes/plugins)
    pub user_plugin_dir: PathBuf,
    /// Project-level plugin directory (.vibes/plugins)
    pub project_plugin_dir: Option<PathBuf>,
    /// Timeout for plugin handlers
    pub handler_timeout: Duration,
}

impl Default for PluginHostConfig {
    fn default() -> Self {
        Self {
            user_plugin_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from(".config"))
                .join("vibes/plugins"),
            project_plugin_dir: None,
            handler_timeout: Duration::from_secs(5),
        }
    }
}

/// Information about a plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Current state
    pub state: PluginState,
}

/// The plugin host manages loading, unloading, and dispatching events to plugins
pub struct PluginHost {
    /// Loaded plugins by name
    plugins: HashMap<String, LoadedPlugin>,
    /// Plugin directories to search (project first, then user)
    plugin_dirs: Vec<PathBuf>,
    /// Path to registry file
    registry_path: PathBuf,
    /// Handler timeout (currently unused, for future timeout implementation)
    #[allow(dead_code)]
    handler_timeout: Duration,
    /// Registry of plugin CLI commands
    command_registry: CommandRegistry,
    /// Registry of plugin HTTP routes
    route_registry: RouteRegistry,
}

impl PluginHost {
    /// Create a new plugin host with the given configuration
    pub fn new(config: PluginHostConfig) -> Self {
        let mut plugin_dirs = Vec::new();

        // Project plugins take precedence
        if let Some(project_dir) = config.project_plugin_dir {
            plugin_dirs.push(project_dir);
        }

        // Then user plugins
        plugin_dirs.push(config.user_plugin_dir.clone());

        Self {
            plugins: HashMap::new(),
            plugin_dirs,
            registry_path: config.user_plugin_dir.join("registry.toml"),
            handler_timeout: config.handler_timeout,
            command_registry: CommandRegistry::new(),
            route_registry: RouteRegistry::new(),
        }
    }

    /// Discover and load all enabled plugins
    pub fn load_all(&mut self) -> Result<(), PluginHostError> {
        let registry = PluginRegistry::load(&self.registry_path)?;

        for plugin_dir in self.discover_plugins()? {
            let name = plugin_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if name.is_empty() {
                continue;
            }

            if !registry.is_enabled(&name) {
                tracing::debug!(plugin = %name, "Plugin disabled, skipping");
                continue;
            }

            match self.load_plugin(&plugin_dir, &name) {
                Ok(plugin) => {
                    tracing::info!(
                        plugin = %name,
                        version = %plugin.manifest.version,
                        "Plugin loaded"
                    );
                    self.plugins.insert(name, plugin);
                }
                Err(e) => {
                    tracing::error!(plugin = %name, error = %e, "Failed to load plugin");
                }
            }
        }

        Ok(())
    }

    /// Discover plugin directories
    fn discover_plugins(&self) -> Result<Vec<PathBuf>, PluginHostError> {
        let mut found = Vec::new();

        for base_dir in &self.plugin_dirs {
            if !base_dir.exists() {
                tracing::debug!(dir = %base_dir.display(), "Plugin directory does not exist");
                continue;
            }

            for entry in std::fs::read_dir(base_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    found.push(path);
                }
            }
        }

        Ok(found)
    }

    /// Load a single plugin from its directory
    fn load_plugin(&self, dir: &Path, name: &str) -> Result<LoadedPlugin, PluginHostError> {
        // 1. Find library file
        let lib_path = self.find_library(dir, name)?;

        // 2. Load dynamic library
        // SAFETY: We're loading a plugin that the user explicitly enabled.
        // The plugin is expected to follow the Plugin trait contract.
        let library = unsafe { Library::new(&lib_path)? };

        // 3. Check API version
        // SAFETY: We're calling a C function exported by the plugin.
        let api_version_fn: libloading::Symbol<extern "C" fn() -> u32> =
            unsafe { library.get(b"_vibes_plugin_api_version")? };

        let plugin_api_version = api_version_fn();
        if plugin_api_version != API_VERSION {
            return Err(PluginHostError::ApiVersionMismatch {
                expected: API_VERSION,
                found: plugin_api_version,
            });
        }

        // 4. Create plugin instance
        // SAFETY: We're calling the plugin's create function which returns a raw pointer
        // that we convert back to a Box<dyn Plugin>.
        let create_fn: libloading::Symbol<extern "C" fn() -> *mut dyn Plugin> =
            unsafe { library.get(b"_vibes_plugin_create")? };

        let instance = unsafe { Box::from_raw(create_fn()) };
        let manifest = instance.manifest();

        // 5. Create context and load config
        let config_path = dir.join("config.toml");
        let config = PluginConfig::load(&config_path).unwrap_or_default();
        let mut context = PluginContext::with_config(name.to_string(), dir.to_path_buf(), config);

        // 6. Call on_load
        // Note: instance is Box<dyn Plugin> which is not mutable, but the trait
        // requires &mut self. We need to use interior mutability or change the approach.
        // For now, we'll use a workaround by making the Box mutable after creation.
        let mut instance = instance;
        instance.on_load(&mut context)?;

        Ok(LoadedPlugin {
            manifest,
            instance,
            context,
            _library: library,
            state: PluginState::Loaded,
        })
    }

    /// Find the library file in a plugin directory
    fn find_library(&self, dir: &Path, name: &str) -> Result<PathBuf, PluginHostError> {
        // Look for <name>.so symlink (or .dylib on macOS, .dll on Windows)
        let extensions = if cfg!(target_os = "macos") {
            vec!["dylib", "so"]
        } else if cfg!(target_os = "windows") {
            vec!["dll"]
        } else {
            vec!["so"]
        };

        for ext in extensions {
            let lib_path = dir.join(format!("{}.{}", name, ext));
            if lib_path.exists() {
                return Ok(lib_path);
            }

            // Also try lib<name>.<ext> format
            let lib_path = dir.join(format!("lib{}.{}", name, ext));
            if lib_path.exists() {
                return Ok(lib_path);
            }
        }

        Err(PluginHostError::LibraryNotFound {
            dir: dir.to_path_buf(),
        })
    }

    /// Dispatch an event to all loaded plugins
    ///
    /// Events are dispatched with panic isolation - if a plugin panics,
    /// it is disabled and other plugins continue to receive events.
    pub fn dispatch_event(&mut self, event: &VibesEvent) {
        for (name, plugin) in &mut self.plugins {
            if plugin.state != PluginState::Loaded {
                continue;
            }

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                dispatch_to_plugin(&mut plugin.instance, &mut plugin.context, event)
            }));

            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!(plugin = %name, error = %e, "Plugin handler error");
                }
                Err(_) => {
                    tracing::error!(plugin = %name, "Plugin panicked, disabling");
                    plugin.state = PluginState::Failed {
                        error: "Plugin panicked".to_string(),
                    };
                }
            }
        }
    }

    /// List all plugins
    pub fn list_plugins(&self, _include_disabled: bool) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|(name, p)| PluginInfo {
                name: name.clone(),
                manifest: p.manifest.clone(),
                state: p.state.clone(),
            })
            .collect()
    }

    /// Enable a plugin in the registry
    pub fn enable_plugin(&mut self, name: &str) -> Result<(), PluginHostError> {
        let mut registry = PluginRegistry::load(&self.registry_path)?;
        registry.enable(name);
        registry.save(&self.registry_path)?;
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> Result<(), PluginHostError> {
        let mut registry = PluginRegistry::load(&self.registry_path)?;
        registry.disable(name);
        registry.save(&self.registry_path)?;

        // Update in-memory state if loaded
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.state = PluginState::Disabled {
                reason: "Disabled by user".to_string(),
            };
        }

        Ok(())
    }

    /// Get information about a specific plugin
    pub fn get_plugin_info(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|p| PluginInfo {
            name: name.to_string(),
            manifest: p.manifest.clone(),
            state: p.state.clone(),
        })
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Get read access to the command registry
    pub fn command_registry(&self) -> &CommandRegistry {
        &self.command_registry
    }

    /// Get read access to the route registry
    pub fn route_registry(&self) -> &RouteRegistry {
        &self.route_registry
    }
}

/// Dispatch a VibesEvent to the appropriate plugin handler
fn dispatch_to_plugin(
    plugin: &mut Box<dyn Plugin>,
    ctx: &mut PluginContext,
    event: &VibesEvent,
) -> Result<(), vibes_plugin_api::PluginError> {
    match event {
        VibesEvent::SessionCreated { session_id, name } => {
            plugin.on_session_created(session_id, name.as_deref(), ctx);
        }
        VibesEvent::SessionStateChanged { session_id, state } => {
            // Parse state string back to SessionState for API
            let api_state = parse_session_state(state);
            plugin.on_session_state_changed(session_id, &api_state, ctx);
        }
        VibesEvent::Claude {
            session_id,
            event: claude_event,
        } => {
            dispatch_claude_event(plugin, ctx, session_id, claude_event);
        }
        VibesEvent::UserInput { .. }
        | VibesEvent::PermissionResponse { .. }
        | VibesEvent::ClientConnected { .. }
        | VibesEvent::ClientDisconnected { .. }
        | VibesEvent::TunnelStateChanged { .. }
        | VibesEvent::OwnershipTransferred { .. }
        | VibesEvent::SessionRemoved { .. } => {
            // These events are not dispatched to plugins (they're client -> server or system events)
        }
    }
    Ok(())
}

/// Dispatch a ClaudeEvent to the appropriate plugin handler
fn dispatch_claude_event(
    plugin: &mut Box<dyn Plugin>,
    ctx: &mut PluginContext,
    session_id: &str,
    event: &ClaudeEvent,
) {
    match event {
        ClaudeEvent::TurnStart => {
            plugin.on_turn_start(session_id, ctx);
        }
        ClaudeEvent::TurnComplete { usage } => {
            let api_usage = vibes_plugin_api::Usage {
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
            };
            plugin.on_turn_complete(session_id, &api_usage, ctx);
        }
        ClaudeEvent::TextDelta { text } => {
            plugin.on_text_delta(session_id, text, ctx);
        }
        ClaudeEvent::ThinkingDelta { text } => {
            plugin.on_thinking_delta(session_id, text, ctx);
        }
        ClaudeEvent::ToolUseStart { id, name } => {
            plugin.on_tool_use_start(session_id, id, name, ctx);
        }
        ClaudeEvent::ToolResult {
            id,
            output,
            is_error,
        } => {
            plugin.on_tool_result(session_id, id, output, *is_error, ctx);
        }
        ClaudeEvent::Error {
            message,
            recoverable,
        } => {
            plugin.on_error(session_id, message, *recoverable, ctx);
        }
        ClaudeEvent::ToolInputDelta { .. } | ClaudeEvent::PermissionRequest { .. } => {
            // Not dispatched to plugins
        }
    }
}

/// Parse a session state string back to plugin API SessionState
fn parse_session_state(state: &str) -> vibes_plugin_api::SessionState {
    // The state string comes from format!("{:?}", state) so we need to parse it
    match state {
        "Idle" => vibes_plugin_api::SessionState::Idle,
        "Processing" => vibes_plugin_api::SessionState::Processing,
        "Finished" => vibes_plugin_api::SessionState::Completed,
        s if s.starts_with("WaitingPermission") => vibes_plugin_api::SessionState::WaitingForInput,
        s if s.starts_with("Failed") => vibes_plugin_api::SessionState::Failed {
            message: "Unknown error".to_string(),
        },
        _ => vibes_plugin_api::SessionState::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_host_config_default() {
        let config = PluginHostConfig::default();
        assert!(config.user_plugin_dir.ends_with("vibes/plugins"));
        assert!(config.project_plugin_dir.is_none());
        assert_eq!(config.handler_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_plugin_host_new() {
        let config = PluginHostConfig::default();
        let host = PluginHost::new(config);
        assert_eq!(host.plugin_count(), 0);
    }

    #[test]
    fn test_plugin_host_list_plugins_empty() {
        let config = PluginHostConfig::default();
        let host = PluginHost::new(config);
        let plugins = host.list_plugins(true);
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_plugin_host_load_all_no_plugins() {
        let dir = TempDir::new().unwrap();
        let config = PluginHostConfig {
            user_plugin_dir: dir.path().to_path_buf(),
            project_plugin_dir: None,
            handler_timeout: Duration::from_secs(5),
        };
        let mut host = PluginHost::new(config);
        host.load_all().unwrap();
        assert_eq!(host.plugin_count(), 0);
    }

    #[test]
    fn test_plugin_host_enable_disable() {
        let dir = TempDir::new().unwrap();
        let config = PluginHostConfig {
            user_plugin_dir: dir.path().to_path_buf(),
            project_plugin_dir: None,
            handler_timeout: Duration::from_secs(5),
        };
        let mut host = PluginHost::new(config);

        // Enable a plugin
        host.enable_plugin("test-plugin").unwrap();

        // Check registry was updated
        let registry = PluginRegistry::load(&dir.path().join("registry.toml")).unwrap();
        assert!(registry.is_enabled("test-plugin"));

        // Disable the plugin
        host.disable_plugin("test-plugin").unwrap();

        // Check registry was updated
        let registry = PluginRegistry::load(&dir.path().join("registry.toml")).unwrap();
        assert!(!registry.is_enabled("test-plugin"));
    }

    #[test]
    fn test_find_library_not_found() {
        let dir = TempDir::new().unwrap();
        let config = PluginHostConfig::default();
        let host = PluginHost::new(config);

        let result = host.find_library(dir.path(), "nonexistent");
        assert!(matches!(
            result,
            Err(PluginHostError::LibraryNotFound { .. })
        ));
    }

    #[test]
    fn test_parse_session_state_strings() {
        assert!(matches!(
            parse_session_state("Idle"),
            vibes_plugin_api::SessionState::Idle
        ));
        assert!(matches!(
            parse_session_state("Processing"),
            vibes_plugin_api::SessionState::Processing
        ));
        assert!(matches!(
            parse_session_state("Finished"),
            vibes_plugin_api::SessionState::Completed
        ));
    }

    #[test]
    fn test_plugin_state_clone() {
        let state = PluginState::Loaded;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_plugin_state_disabled() {
        let state = PluginState::Disabled {
            reason: "User disabled".to_string(),
        };
        assert!(matches!(state, PluginState::Disabled { .. }));
    }

    #[test]
    fn test_plugin_state_failed() {
        let state = PluginState::Failed {
            error: "Panicked".to_string(),
        };
        assert!(matches!(state, PluginState::Failed { .. }));
    }
}
