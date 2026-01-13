//! PluginHost - manages plugin lifecycle and event dispatch

use libloading::Library;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::time::Duration;

use vibes_plugin_api::{
    API_VERSION, AssessmentQuery, AssessmentQueryResponse, CommandArgs, CommandOutput, HttpMethod,
    Plugin, PluginAssessmentResult, PluginConfig, PluginContext, PluginManifest, RawEvent,
    RouteRequest, RouteResponse,
};

use super::commands::CommandRegistry;
use super::error::PluginHostError;
use super::registry::PluginRegistry;
use super::routes::RouteRegistry;
use crate::events::{ClaudeEvent, StoredEvent, VibesEvent};

/// A loaded plugin with its runtime state
struct LoadedPlugin {
    /// Plugin manifest (metadata)
    manifest: PluginManifest,
    /// The plugin instance
    instance: Box<dyn Plugin>,
    /// Plugin context for callbacks
    pub(crate) context: PluginContext,
    /// Keep the library loaded
    _library: Library,
    /// Current plugin state
    state: PluginState,
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        // Call on_unload before the library is dropped
        // This gives plugins a chance to clean up resources (cancel tasks, etc.)
        // that hold references to types defined in the plugin library.
        if let Err(e) = self.instance.on_unload() {
            tracing::warn!(
                plugin = %self.manifest.name,
                error = %e,
                "Plugin on_unload returned error"
            );
        }
    }
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
        // Use XDG config directory helper for consistency
        let user_plugin_dir = vibes_paths::config_dir().join("plugins");

        Self {
            user_plugin_dir,
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
    fn load_plugin(&mut self, dir: &Path, name: &str) -> Result<LoadedPlugin, PluginHostError> {
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

        // 7. Validate command registrations
        for spec in context.pending_commands() {
            if let Some(existing) = self
                .command_registry
                .check_conflict(&manifest.name, &spec.path)
            {
                return Err(PluginHostError::CommandConflict {
                    command: spec.path.join(" "),
                    existing_plugin: existing.to_string(),
                    new_plugin: manifest.name.clone(),
                });
            }
        }

        // 8. Validate route registrations
        for spec in context.pending_routes() {
            if let Some(existing) = self.route_registry.check_conflict(&manifest.name, spec) {
                return Err(PluginHostError::RouteConflict {
                    route: format!("{:?} {}", spec.method, spec.path),
                    existing_plugin: existing.to_string(),
                    new_plugin: manifest.name.clone(),
                });
            }
        }

        // 9. Commit registrations
        let commands = context.take_pending_commands();
        let routes = context.take_pending_routes();

        self.command_registry.register(&manifest.name, commands);
        self.route_registry.register(&manifest.name, routes);

        Ok(LoadedPlugin {
            manifest,
            instance,
            context,
            _library: library,
            state: PluginState::Loaded,
        })
    }

    /// Unload a plugin and clean up its registrations
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginHostError> {
        if self.plugins.remove(name).is_none() {
            return Err(PluginHostError::NotFound {
                name: name.to_string(),
            });
        }

        // Clean up registrations
        self.command_registry.unregister(name);
        self.route_registry.unregister(name);

        Ok(())
    }

    /// Notify all loaded plugins that the runtime is ready.
    ///
    /// This sets the runtime handle, event log, shutdown token, and Iggy manager
    /// on each plugin's context, then calls `on_ready()` to allow plugins to start
    /// background tasks.
    ///
    /// Should be called once after the server is fully initialized.
    pub fn notify_ready(
        &mut self,
        event_log: std::sync::Arc<dyn std::any::Any + Send + Sync>,
        shutdown: tokio_util::sync::CancellationToken,
        iggy_manager: Option<std::sync::Arc<vibes_iggy::IggyManager>>,
    ) {
        // Get the tokio runtime handle from the current context.
        // This handle will be passed to plugins so they can run async operations.
        let runtime_handle = tokio::runtime::Handle::current();

        for (name, plugin) in &mut self.plugins {
            if plugin.state != PluginState::Loaded {
                continue;
            }

            // Set runtime on context - runtime_handle MUST be set first so plugins
            // can use it in on_ready() for async operations
            plugin.context.set_runtime_handle(runtime_handle.clone());
            plugin.context.set_event_log(event_log.clone());
            plugin.context.set_shutdown(shutdown.clone());
            if let Some(ref manager) = iggy_manager {
                plugin.context.set_iggy_manager(manager.clone());
            }

            // Call on_ready with panic isolation
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                plugin.instance.on_ready(&mut plugin.context)
            }));

            match result {
                Ok(Ok(())) => {
                    tracing::debug!(plugin = %name, "Plugin ready");
                }
                Ok(Err(e)) => {
                    tracing::error!(plugin = %name, error = %e, "Plugin on_ready error");
                    plugin.state = PluginState::Failed {
                        error: e.to_string(),
                    };
                }
                Err(_) => {
                    tracing::error!(plugin = %name, "Plugin panicked in on_ready, disabling");
                    plugin.state = PluginState::Failed {
                        error: "Plugin panicked in on_ready".to_string(),
                    };
                }
            }
        }
    }

    /// Get a service registered by a plugin.
    ///
    /// Services are registered by plugins during `on_ready()` and can be retrieved
    /// by the host to access shared resources like AssessmentLog.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to downcast to. For trait objects stored as `Arc<Arc<dyn Trait>>`,
    ///   specify `T = Arc<dyn Trait>`.
    pub fn get_plugin_service<T: ?Sized + 'static>(
        &self,
        plugin_name: &str,
        service_name: &str,
    ) -> Option<std::sync::Arc<T>> {
        self.plugins
            .get(plugin_name)
            .and_then(|plugin| plugin.context.get_service(service_name))
    }

    /// Check if a plugin is loaded and ready.
    ///
    /// Returns true if the plugin exists and is in the `Loaded` state.
    pub fn is_plugin_loaded(&self, plugin_name: &str) -> bool {
        self.plugins
            .get(plugin_name)
            .is_some_and(|plugin| plugin.state == PluginState::Loaded)
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

    /// Dispatch a CLI command to the appropriate plugin
    pub fn dispatch_command(
        &mut self,
        plugin_name: &str,
        path: &[&str],
        args: &CommandArgs,
    ) -> Result<CommandOutput, PluginHostError> {
        let plugin =
            self.plugins
                .get_mut(plugin_name)
                .ok_or_else(|| PluginHostError::NotFound {
                    name: plugin_name.to_string(),
                })?;

        if plugin.state != PluginState::Loaded {
            return Err(PluginHostError::NotFound {
                name: plugin_name.to_string(),
            });
        }

        plugin
            .instance
            .handle_command(path, args, &mut plugin.context)
            .map_err(PluginHostError::InitFailed)
    }

    /// Dispatch an HTTP route to the appropriate plugin
    pub fn dispatch_route(
        &mut self,
        plugin_name: &str,
        method: HttpMethod,
        path: &str,
        request: RouteRequest,
    ) -> Result<RouteResponse, PluginHostError> {
        let plugin =
            self.plugins
                .get_mut(plugin_name)
                .ok_or_else(|| PluginHostError::NotFound {
                    name: plugin_name.to_string(),
                })?;

        if plugin.state != PluginState::Loaded {
            return Err(PluginHostError::NotFound {
                name: plugin_name.to_string(),
            });
        }

        plugin
            .instance
            .handle_route(method, path, request, &mut plugin.context)
            .map_err(PluginHostError::InitFailed)
    }

    /// Dispatch a raw event to all loaded plugins and collect assessment results.
    ///
    /// This converts the StoredEvent to an FFI-safe RawEvent and calls each
    /// plugin's `on_event()` handler. Results are aggregated from all plugins.
    ///
    /// Events are dispatched with panic isolation - if a plugin panics,
    /// it is disabled and other plugins continue to receive events.
    ///
    /// # Returns
    ///
    /// A vector of assessment results from all plugins. The results are
    /// JSON-serialized and ready to be written to the AssessmentLog.
    pub fn dispatch_raw_event(&mut self, stored: &StoredEvent) -> Vec<PluginAssessmentResult> {
        let raw = Self::to_raw_event(stored);
        let mut all_results = Vec::new();

        for (name, plugin) in &mut self.plugins {
            if plugin.state != PluginState::Loaded {
                continue;
            }

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                plugin.instance.on_event(raw.clone(), &mut plugin.context)
            }));

            match result {
                Ok(results) => {
                    all_results.extend(results);
                }
                Err(_) => {
                    tracing::error!(plugin = %name, "Plugin panicked in on_event, disabling");
                    plugin.state = PluginState::Failed {
                        error: "Plugin panicked in on_event".to_string(),
                    };
                }
            }
        }

        all_results
    }

    /// Query assessment results from all loaded plugins.
    ///
    /// This calls each plugin's `query_assessment_results()` method and
    /// aggregates the results. Plugins that don't have assessment data
    /// simply return empty results.
    ///
    /// Queries are dispatched with panic isolation - if a plugin panics,
    /// it is disabled and other plugins continue to be queried.
    ///
    /// # Arguments
    ///
    /// * `query` - The query parameters (session filter, limit, pagination)
    ///
    /// # Returns
    ///
    /// Aggregated assessment results from all plugins. The results are
    /// sorted by event ID (newest first if specified in query).
    pub fn dispatch_query_assessment(&mut self, query: AssessmentQuery) -> AssessmentQueryResponse {
        let mut all_results = Vec::new();
        let mut has_more = false;

        for (name, plugin) in &mut self.plugins {
            if plugin.state != PluginState::Loaded {
                continue;
            }

            let query_clone = AssessmentQuery {
                session_id: query.session_id.clone(),
                result_types: query.result_types.clone(),
                limit: query.limit,
                after_event_id: query.after_event_id.clone(),
                newest_first: query.newest_first,
            };

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                plugin
                    .instance
                    .query_assessment_results(query_clone, &plugin.context)
            }));

            match result {
                Ok(response) => {
                    all_results.extend(response.results);
                    if response.has_more {
                        has_more = true;
                    }
                }
                Err(_) => {
                    tracing::error!(plugin = %name, "Plugin panicked in query_assessment_results, disabling");
                    plugin.state = PluginState::Failed {
                        error: "Plugin panicked in query_assessment_results".to_string(),
                    };
                }
            }
        }

        // Apply limit to aggregated results
        if all_results.len() > query.limit {
            all_results.truncate(query.limit);
            has_more = true;
        }

        let oldest_event_id = None; // Could be derived from results if needed

        AssessmentQueryResponse {
            results: all_results,
            oldest_event_id,
            has_more,
        }
    }

    /// Convert a StoredEvent to an FFI-safe RawEvent.
    fn to_raw_event(stored: &StoredEvent) -> RawEvent {
        // Serialize the VibesEvent to JSON for FFI boundary
        let payload = serde_json::to_string(&stored.event).unwrap_or_default();

        // Derive event_type from the VibesEvent variant
        let event_type = match &stored.event {
            VibesEvent::Claude { .. } => "Claude",
            VibesEvent::UserInput { .. } => "UserInput",
            VibesEvent::PermissionResponse { .. } => "PermissionResponse",
            VibesEvent::SessionCreated { .. } => "SessionCreated",
            VibesEvent::SessionStateChanged { .. } => "SessionStateChanged",
            VibesEvent::ClientConnected { .. } => "ClientConnected",
            VibesEvent::ClientDisconnected { .. } => "ClientDisconnected",
            VibesEvent::TunnelStateChanged { .. } => "TunnelStateChanged",
            VibesEvent::OwnershipTransferred { .. } => "OwnershipTransferred",
            VibesEvent::SessionRemoved { .. } => "SessionRemoved",
            VibesEvent::Hook { .. } => "Hook",
        };

        // Extract timestamp from UUIDv7 (milliseconds since Unix epoch)
        let (timestamp_secs, timestamp_subsec_nanos) = stored
            .event_id
            .get_timestamp()
            .map_or((0, 0), |ts| (ts.to_unix().0, ts.to_unix().1));
        let timestamp_ms = timestamp_secs * 1000 + u64::from(timestamp_subsec_nanos / 1_000_000);

        RawEvent::new(
            stored.event_id.into_bytes(),
            timestamp_ms,
            stored.session_id().map(|s| s.to_string()),
            event_type.to_string(),
            payload,
        )
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
        VibesEvent::Hook { session_id, event } => {
            // Dispatch hook events to plugins for processing
            let hook_type = event.hook_type().as_str();
            let project_path = event.project_path();
            let _response = plugin.on_hook(
                session_id.as_deref(),
                hook_type,
                project_path.as_deref(),
                ctx,
            );
            // Response handling will be implemented when hook responses are wired to the receiver
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

    // ─── dispatch_raw_event Tests ────────────────────────────────────

    #[test]
    fn test_to_raw_event_claude_event() {
        use crate::events::{ClaudeEvent, StoredEvent, VibesEvent};

        let event = StoredEvent::new(VibesEvent::Claude {
            session_id: "test-session".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hello world".to_string(),
            },
        });

        let raw = PluginHost::to_raw_event(&event);

        assert_eq!(raw.event_id, event.event_id.into_bytes());
        assert_eq!(raw.session_id, Some("test-session".to_string()));
        assert_eq!(raw.event_type, "Claude");
        assert!(raw.payload.contains("text_delta"));
        assert!(raw.payload.contains("Hello world"));
    }

    #[test]
    fn test_to_raw_event_without_session() {
        use crate::events::{StoredEvent, VibesEvent};

        let event = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "client-123".to_string(),
        });

        let raw = PluginHost::to_raw_event(&event);

        assert_eq!(raw.session_id, None);
        assert_eq!(raw.event_type, "ClientConnected");
    }

    #[test]
    fn test_to_raw_event_timestamp_extraction() {
        use crate::events::{StoredEvent, VibesEvent};

        let event = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });

        let raw = PluginHost::to_raw_event(&event);

        // Timestamp should be positive and reasonable (after 2024)
        assert!(raw.timestamp_ms > 1_700_000_000_000);
    }

    #[test]
    fn test_dispatch_raw_event_no_plugins() {
        use crate::events::{StoredEvent, VibesEvent};

        let config = PluginHostConfig::default();
        let mut host = PluginHost::new(config);

        let event = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });

        let results = host.dispatch_raw_event(&event);
        assert!(results.is_empty());
    }
}
