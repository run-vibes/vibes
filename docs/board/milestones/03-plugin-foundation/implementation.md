# Milestone 1.3: Plugin Foundation - Implementation Plan

> Step-by-step implementation guide for the vibes plugin system.

## Prerequisites

- Milestone 1.2 complete (vibes-cli with claude command, config system)
- Nix dev environment working (`direnv allow`)

---

## Phase 1: vibes-plugin-api Crate Setup

### 1.1 Create crate structure

```bash
mkdir -p vibes-plugin-api/src
```

Create `vibes-plugin-api/Cargo.toml`:
```toml
[package]
name = "vibes-plugin-api"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Plugin API for vibes - define plugins for Claude Code proxy"

[lib]
crate-type = ["rlib", "dylib"]

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```

### 1.2 Add to workspace

Update root `Cargo.toml`:
```toml
[workspace]
members = ["vibes-core", "vibes-cli", "vibes-plugin-api"]
```

### 1.3 Create lib.rs with API_VERSION

```rust
// vibes-plugin-api/src/lib.rs
pub mod context;
pub mod error;
pub mod types;

pub use context::PluginContext;
pub use error::PluginError;
pub use types::*;

/// Current plugin API version. Plugins must match this exactly.
pub const API_VERSION: u32 = 1;
```

### 1.4 Verify setup

```bash
just check
```

---

## Phase 2: Plugin Types

### 2.1 Implement PluginManifest

Create `vibes-plugin-api/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub description: String,
    pub author: String,
    pub license: PluginLicense,
    pub commands: Vec<CommandSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PluginLicense {
    #[default]
    Free,
    Paid { product_id: String },
    Trial { days: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    pub name: String,
    pub description: String,
    pub args: Vec<ArgSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSpec {
    pub name: String,
    pub description: String,
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
```

**Tests:**
- Default manifest has correct API_VERSION
- TOML serialization round-trips
- All fields serialize correctly

### 2.2 Implement PluginError

Create `vibes-plugin-api/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command failed: {0}")]
    Command(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("{0}")]
    Custom(String),
}
```

**Tests:**
- Error messages format correctly
- std::io::Error converts properly

---

## Phase 3: Plugin Trait

### 3.1 Define re-exported types from vibes-core

The Plugin trait needs access to `SessionState` and `Usage`. Add these to vibes-plugin-api (or re-export from vibes-core if we establish that dependency):

```rust
// vibes-plugin-api/src/types.rs (add)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Idle,
    Processing,
    WaitingForInput,
    WaitingForPermission { request_id: String, tool: String },
    Completed,
    Failed { message: String },
}
```

### 3.2 Implement Plugin trait

Add to `vibes-plugin-api/src/lib.rs`:

```rust
use context::PluginContext;
use error::PluginError;
use types::{PluginManifest, SessionState, Usage};

/// The core plugin trait - implement this to create a vibes plugin.
pub trait Plugin: Send + Sync {
    /// Return plugin metadata
    fn manifest(&self) -> PluginManifest;

    /// Called when plugin is loaded
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError>;

    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> Result<(), PluginError>;

    // ─── Event Handlers (default no-ops) ─────────────────────────────

    fn on_session_created(
        &mut self,
        _session_id: &str,
        _name: Option<&str>,
        _ctx: &mut PluginContext,
    ) {
    }

    fn on_session_state_changed(
        &mut self,
        _session_id: &str,
        _state: &SessionState,
        _ctx: &mut PluginContext,
    ) {
    }

    fn on_turn_start(&mut self, _session_id: &str, _ctx: &mut PluginContext) {}

    fn on_turn_complete(
        &mut self,
        _session_id: &str,
        _usage: &Usage,
        _ctx: &mut PluginContext,
    ) {
    }

    fn on_text_delta(&mut self, _session_id: &str, _text: &str, _ctx: &mut PluginContext) {}

    fn on_thinking_delta(&mut self, _session_id: &str, _text: &str, _ctx: &mut PluginContext) {}

    fn on_tool_use_start(
        &mut self,
        _session_id: &str,
        _tool_id: &str,
        _name: &str,
        _ctx: &mut PluginContext,
    ) {
    }

    fn on_tool_result(
        &mut self,
        _session_id: &str,
        _tool_id: &str,
        _output: &str,
        _is_error: bool,
        _ctx: &mut PluginContext,
    ) {
    }

    fn on_error(
        &mut self,
        _session_id: &str,
        _message: &str,
        _recoverable: bool,
        _ctx: &mut PluginContext,
    ) {
    }
}
```

**Tests:**
- Default trait methods don't panic
- Trait is object-safe (`Box<dyn Plugin>` compiles)

### 3.3 Implement export_plugin! macro

Add to `vibes-plugin-api/src/lib.rs`:

```rust
/// Export a plugin type for dynamic loading.
///
/// Usage:
/// ```ignore
/// vibes_plugin_api::export_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _vibes_plugin_create() -> *mut dyn $crate::Plugin {
            let plugin: Box<dyn $crate::Plugin> = Box::new(<$plugin_type>::default());
            Box::into_raw(plugin)
        }

        #[no_mangle]
        pub extern "C" fn _vibes_plugin_api_version() -> u32 {
            $crate::API_VERSION
        }

        #[no_mangle]
        pub extern "C" fn _vibes_plugin_destroy(ptr: *mut dyn $crate::Plugin) {
            if !ptr.is_null() {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
        }
    };
}
```

**Tests:**
- Macro compiles with a simple plugin struct
- Generated functions have correct signatures

---

## Phase 4: PluginContext

### 4.1 Implement PluginContext structure

Create `vibes-plugin-api/src/context.rs`:

```rust
use crate::error::PluginError;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin's interface to vibes-core capabilities.
pub struct PluginContext {
    plugin_name: String,
    plugin_dir: PathBuf,
    config: PluginConfig,
    // These will be populated by PluginHost
    registered_commands: Vec<RegisteredCommand>,
}

pub struct PluginConfig {
    values: HashMap<String, toml::Value>,
    dirty: bool,
}

pub struct RegisteredCommand {
    pub name: String,
    // Handler stored separately in PluginHost
}

pub struct CommandArgs {
    pub args: Vec<String>,
    pub flags: HashMap<String, String>,
}

impl PluginContext {
    pub fn new(plugin_name: String, plugin_dir: PathBuf) -> Self {
        Self {
            plugin_name,
            plugin_dir,
            config: PluginConfig::new(),
            registered_commands: Vec::new(),
        }
    }

    // ─── Configuration ───────────────────────────────────────────────

    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    pub fn config_get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.config.get(key)
    }

    pub fn config_set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), PluginError> {
        self.config.set(key, value)
    }

    // ─── Command Registration ────────────────────────────────────────

    pub fn register_command(&mut self, name: &str) {
        self.registered_commands.push(RegisteredCommand {
            name: name.to_string(),
        });
    }

    pub fn registered_commands(&self) -> &[RegisteredCommand] {
        &self.registered_commands
    }

    // ─── Logging ─────────────────────────────────────────────────────

    pub fn log_info(&self, message: &str) {
        tracing::info!(plugin = %self.plugin_name, "{}", message);
    }

    pub fn log_warn(&self, message: &str) {
        tracing::warn!(plugin = %self.plugin_name, "{}", message);
    }

    pub fn log_error(&self, message: &str) {
        tracing::error!(plugin = %self.plugin_name, "{}", message);
    }
}

impl PluginConfig {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            dirty: false,
        }
    }

    pub fn load(path: &Path) -> Result<Self, PluginError> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        let values: HashMap<String, toml::Value> = toml::from_str(&content)
            .map_err(|e| PluginError::Config(e.to_string()))?;
        Ok(Self { values, dirty: false })
    }

    pub fn save(&self, path: &Path) -> Result<(), PluginError> {
        let content = toml::to_string_pretty(&self.values)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.values.get(key).and_then(|v| v.clone().try_into().ok())
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), PluginError> {
        let toml_value = toml::Value::try_from(value)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;
        self.values.insert(key.to_string(), toml_value);
        self.dirty = true;
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
```

**Tests:**
- Config get/set works for basic types
- Config serializes to TOML correctly
- load() returns empty config for missing file

---

## Phase 5: PluginHost in vibes-core

### 5.1 Add libloading dependency

Update `vibes-core/Cargo.toml`:
```toml
[dependencies]
libloading = "0.8"
vibes-plugin-api = { path = "../vibes-plugin-api" }
```

### 5.2 Create plugins module structure

```bash
mkdir -p vibes-core/src/plugins
```

Create module files:
```
vibes-core/src/plugins/
├── mod.rs
├── host.rs
├── registry.rs
└── error.rs
```

### 5.3 Implement PluginHostError

Create `vibes-core/src/plugins/error.rs`:

```rust
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginHostError {
    #[error("Plugin directory not found: {path}")]
    PluginDirNotFound { path: PathBuf },

    #[error("Plugin library not found in {dir}")]
    LibraryNotFound { dir: PathBuf },

    #[error("API version mismatch: vibes expects {expected}, plugin has {found}")]
    ApiVersionMismatch { expected: u32, found: u32 },

    #[error("Failed to load plugin library: {0}")]
    LibraryLoad(#[from] libloading::Error),

    #[error("Plugin initialization failed: {0}")]
    InitFailed(#[from] vibes_plugin_api::PluginError),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Plugin '{name}' not found")]
    NotFound { name: String },

    #[error("Plugin '{name}' timed out after {timeout:?}")]
    Timeout { name: String, timeout: Duration },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 5.4 Implement PluginRegistry

Create `vibes-core/src/plugins/registry.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use super::error::PluginHostError;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PluginRegistry {
    pub enabled: HashSet<String>,
}

impl PluginRegistry {
    pub fn load(path: &Path) -> Result<Self, PluginHostError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let registry: Self = toml::from_str(&content)
            .map_err(|e| PluginHostError::Registry(e.to_string()))?;
        Ok(registry)
    }

    pub fn save(&self, path: &Path) -> Result<(), PluginHostError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| PluginHostError::Registry(e.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.contains(name)
    }

    pub fn enable(&mut self, name: &str) {
        self.enabled.insert(name.to_string());
    }

    pub fn disable(&mut self, name: &str) {
        self.enabled.remove(name);
    }
}
```

**Tests:**
- Load returns empty registry for missing file
- Enable/disable modifies set correctly
- Save/load round-trips

### 5.5 Implement PluginHost

Create `vibes-core/src/plugins/host.rs`:

```rust
use libloading::Library;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::time::Duration;

use vibes_plugin_api::{Plugin, PluginContext, PluginManifest, API_VERSION};

use super::error::PluginHostError;
use super::registry::PluginRegistry;
use crate::events::VibesEvent;

pub struct PluginHost {
    plugins: HashMap<String, LoadedPlugin>,
    plugin_dirs: Vec<PathBuf>,
    registry_path: PathBuf,
    handler_timeout: Duration,
}

struct LoadedPlugin {
    manifest: PluginManifest,
    instance: Box<dyn Plugin>,
    context: PluginContext,
    _library: Library, // Keep library loaded
    state: PluginState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Loaded,
    Disabled { reason: String },
    Failed { error: String },
}

pub struct PluginHostConfig {
    pub user_plugin_dir: PathBuf,
    pub project_plugin_dir: Option<PathBuf>,
    pub handler_timeout: Duration,
}

impl Default for PluginHostConfig {
    fn default() -> Self {
        Self {
            user_plugin_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("vibes/plugins"),
            project_plugin_dir: None,
            handler_timeout: Duration::from_secs(5),
        }
    }
}

impl PluginHost {
    pub fn new(config: PluginHostConfig) -> Self {
        let mut plugin_dirs = Vec::new();
        if let Some(project_dir) = config.project_plugin_dir {
            plugin_dirs.push(project_dir);
        }
        plugin_dirs.push(config.user_plugin_dir.clone());

        Self {
            plugins: HashMap::new(),
            plugin_dirs,
            registry_path: config.user_plugin_dir.join("registry.toml"),
            handler_timeout: config.handler_timeout,
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

            if !registry.is_enabled(&name) {
                tracing::debug!(plugin = %name, "Plugin disabled, skipping");
                continue;
            }

            match self.load_plugin(&plugin_dir) {
                Ok(plugin) => {
                    tracing::info!(plugin = %name, version = %plugin.manifest.version, "Plugin loaded");
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
    fn load_plugin(&self, dir: &Path) -> Result<LoadedPlugin, PluginHostError> {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 1. Find library symlink
        let lib_path = self.find_library(dir, &name)?;

        // 2. Load dynamic library
        let library = unsafe { Library::new(&lib_path)? };

        // 3. Check API version
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
        let create_fn: libloading::Symbol<extern "C" fn() -> *mut dyn Plugin> =
            unsafe { library.get(b"_vibes_plugin_create")? };

        let instance = unsafe { Box::from_raw(create_fn()) };
        let manifest = instance.manifest();

        // 5. Create context and call on_load
        let mut context = PluginContext::new(name.clone(), dir.to_path_buf());

        // Load plugin config if exists
        let config_path = dir.join("config.toml");
        if config_path.exists() {
            // Context will load config internally
        }

        instance.on_load(&mut context)?;

        Ok(LoadedPlugin {
            manifest,
            instance,
            context,
            _library: library,
            state: PluginState::Loaded,
        })
    }

    /// Find the library file (follows symlink convention)
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
        }

        Err(PluginHostError::LibraryNotFound { dir: dir.to_path_buf() })
    }

    /// Dispatch event to all loaded plugins with panic isolation
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

    /// List all plugins (loaded and discovered)
    pub fn list_plugins(&self, include_disabled: bool) -> Vec<PluginInfo> {
        // Implementation: list loaded plugins + discover disabled ones
        self.plugins
            .iter()
            .map(|(name, p)| PluginInfo {
                name: name.clone(),
                manifest: p.manifest.clone(),
                state: p.state.clone(),
            })
            .collect()
    }

    /// Enable a plugin
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

        // Also update in-memory state if loaded
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.state = PluginState::Disabled {
                reason: "Disabled by user".to_string(),
            };
        }
        Ok(())
    }

    /// Get plugin info
    pub fn get_plugin(&self, name: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub manifest: PluginManifest,
    pub state: PluginState,
}

/// Route event to appropriate typed handler
fn dispatch_to_plugin(
    plugin: &mut Box<dyn Plugin>,
    ctx: &mut PluginContext,
    event: &VibesEvent,
) -> Result<(), vibes_plugin_api::PluginError> {
    use crate::events::{ClaudeEvent, VibesEvent};

    match event {
        VibesEvent::SessionCreated { session_id, name } => {
            plugin.on_session_created(session_id, name.as_deref(), ctx);
        }
        VibesEvent::SessionStateChanged { session_id, state } => {
            // Convert state types
            let api_state = convert_session_state(state);
            plugin.on_session_state_changed(session_id, &api_state, ctx);
        }
        VibesEvent::Claude { session_id, event: claude_event } => {
            match claude_event {
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
                ClaudeEvent::ToolResult { id, output, is_error } => {
                    plugin.on_tool_result(session_id, id, output, *is_error, ctx);
                }
                ClaudeEvent::Error { message, recoverable } => {
                    plugin.on_error(session_id, message, *recoverable, ctx);
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn convert_session_state(state: &crate::session::SessionState) -> vibes_plugin_api::SessionState {
    // Convert between vibes-core and vibes-plugin-api SessionState
    // They should have the same structure
    match state {
        crate::session::SessionState::Idle => vibes_plugin_api::SessionState::Idle,
        crate::session::SessionState::Processing => vibes_plugin_api::SessionState::Processing,
        // ... etc
        _ => vibes_plugin_api::SessionState::Idle,
    }
}
```

**Tests:**
- load_plugin fails gracefully for missing library
- API version mismatch returns clear error
- dispatch_event catches panics
- list_plugins returns correct info

### 5.6 Export plugins module

Update `vibes-core/src/lib.rs`:
```rust
pub mod plugins;
pub use plugins::{PluginHost, PluginHostConfig, PluginHostError, PluginInfo, PluginState};
```

---

## Phase 6: CLI Commands

### 6.1 Add plugin command to CLI

Update `vibes-cli/src/main.rs`:
```rust
#[derive(Subcommand)]
enum Commands {
    Claude(commands::claude::ClaudeArgs),
    Config(commands::config::ConfigArgs),
    Plugin(commands::plugin::PluginArgs),  // NEW
}
```

### 6.2 Implement plugin commands

Create `vibes-cli/src/commands/plugin.rs`:

```rust
use anyhow::Result;
use clap::{Args, Subcommand};
use vibes_core::{PluginHost, PluginHostConfig, PluginState};

#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommands,
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// List installed plugins
    List {
        /// Include disabled plugins
        #[arg(long)]
        all: bool,
    },
    /// Enable a plugin
    Enable { name: String },
    /// Disable a plugin
    Disable { name: String },
    /// Show plugin details
    Info { name: String },
    /// Reload a plugin
    Reload { name: String },
}

pub fn run(args: PluginArgs) -> Result<()> {
    let config = PluginHostConfig::default();
    let mut host = PluginHost::new(config);

    match args.command {
        PluginCommands::List { all } => {
            host.load_all()?;
            let plugins = host.list_plugins(all);

            if plugins.is_empty() {
                println!("No plugins installed");
                println!("Plugin directory: ~/.config/vibes/plugins/");
                return Ok(());
            }

            for p in plugins {
                let status = match &p.state {
                    PluginState::Loaded => "✓".to_string(),
                    PluginState::Disabled { .. } => "○".to_string(),
                    PluginState::Failed { .. } => "✗".to_string(),
                };
                println!(
                    "{} {} v{}    {}",
                    status, p.name, p.manifest.version, p.manifest.description
                );
            }
        }

        PluginCommands::Enable { name } => {
            host.enable_plugin(&name)?;
            println!("Enabled plugin: {}", name);
        }

        PluginCommands::Disable { name } => {
            host.disable_plugin(&name)?;
            println!("Disabled plugin: {}", name);
        }

        PluginCommands::Info { name } => {
            host.load_all()?;
            if let Some(plugin) = host.get_plugin(&name) {
                let m = &plugin.manifest;
                println!("Name:        {}", m.name);
                println!("Version:     {}", m.version);
                println!("API Version: {}", m.api_version);
                println!("Author:      {}", m.author);
                println!("Description: {}", m.description);
                println!();
                if !m.commands.is_empty() {
                    println!("Commands:");
                    for cmd in &m.commands {
                        println!("  vibes {} {}    {}", name, cmd.name, cmd.description);
                    }
                }
            } else {
                println!("Plugin '{}' not found", name);
            }
        }

        PluginCommands::Reload { name } => {
            // Unload and reload the plugin
            println!("Reloading plugin: {}", name);
            // TODO: implement reload
        }
    }

    Ok(())
}
```

### 6.3 Update module exports

Update `vibes-cli/src/commands/mod.rs`:
```rust
pub mod claude;
pub mod config;
pub mod plugin;
```

---

## Phase 7: Integration with Session

### 7.1 Add PluginHost to SessionManager

Update `vibes-core/src/session/manager.rs` to optionally hold a PluginHost and dispatch events:

```rust
pub struct SessionManager {
    // ... existing fields
    plugin_host: Option<PluginHost>,
}

impl SessionManager {
    pub fn with_plugins(mut self, plugin_host: PluginHost) -> Self {
        self.plugin_host = Some(plugin_host);
        self
    }

    // In event handling, dispatch to plugins
    async fn handle_event(&mut self, event: VibesEvent) {
        if let Some(ref mut host) = self.plugin_host {
            host.dispatch_event(&event);
        }
        // ... rest of handling
    }
}
```

### 7.2 Initialize plugins in CLI

Update `vibes-cli/src/commands/claude.rs`:

```rust
pub async fn run(args: ClaudeArgs) -> Result<()> {
    // ... existing setup

    // Load plugins
    let mut plugin_host = PluginHost::new(PluginHostConfig::default());
    if let Err(e) = plugin_host.load_all() {
        tracing::warn!("Failed to load plugins: {}", e);
    }

    // Create session manager with plugins
    let manager = SessionManager::new(factory, event_bus.clone())
        .with_plugins(plugin_host);

    // ... rest of run
}
```

---

## Phase 8: Example Plugin

### 8.1 Create example plugin crate

Create `examples/plugins/hello-plugin/`:

```toml
# Cargo.toml
[package]
name = "hello-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
vibes-plugin-api = { path = "../../../vibes-plugin-api" }
```

```rust
// src/lib.rs
use vibes_plugin_api::{
    export_plugin, Plugin, PluginContext, PluginError, PluginManifest, Usage,
};

#[derive(Default)]
pub struct HelloPlugin {
    turn_count: u32,
}

impl Plugin for HelloPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "hello".to_string(),
            version: "0.1.0".to_string(),
            description: "A simple example plugin".to_string(),
            author: "vibes-team".to_string(),
            ..Default::default()
        }
    }

    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        ctx.log_info("Hello plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_turn_complete(&mut self, session_id: &str, usage: &Usage, ctx: &mut PluginContext) {
        self.turn_count += 1;
        ctx.log_info(&format!(
            "Turn {} complete. Tokens: {} in, {} out",
            self.turn_count, usage.input_tokens, usage.output_tokens
        ));
    }
}

export_plugin!(HelloPlugin);
```

### 8.2 Build and install example

```bash
cd examples/plugins/hello-plugin
cargo build --release

# Install to plugin directory
mkdir -p ~/.config/vibes/plugins/hello
cp target/release/libhello_plugin.so ~/.config/vibes/plugins/hello/hello.0.1.0.so
ln -sf hello.0.1.0.so ~/.config/vibes/plugins/hello/hello.so

# Enable in registry
echo 'enabled = ["hello"]' > ~/.config/vibes/plugins/registry.toml
```

---

## Phase 9: Testing

### 9.1 Unit tests for vibes-plugin-api

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_default_api_version() {
        let manifest = PluginManifest::default();
        assert_eq!(manifest.api_version, API_VERSION);
    }

    #[test]
    fn test_plugin_trait_object_safe() {
        // This compiles only if Plugin is object-safe
        fn _takes_boxed_plugin(_: Box<dyn Plugin>) {}
    }

    #[test]
    fn test_config_roundtrip() {
        let mut config = PluginConfig::new();
        config.set("key", "value").unwrap();
        let val: Option<String> = config.get("key");
        assert_eq!(val, Some("value".to_string()));
    }
}
```

### 9.2 Unit tests for PluginHost

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_registry_enable_disable() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.toml");

        let mut registry = PluginRegistry::default();
        registry.enable("test-plugin");
        assert!(registry.is_enabled("test-plugin"));

        registry.save(&path).unwrap();
        let loaded = PluginRegistry::load(&path).unwrap();
        assert!(loaded.is_enabled("test-plugin"));
    }

    #[test]
    fn test_api_version_mismatch() {
        // Create a mock plugin with wrong version
        // Verify PluginHostError::ApiVersionMismatch is returned
    }
}
```

### 9.3 Integration test with example plugin

```rust
#[cfg(feature = "integration")]
#[test]
fn test_load_hello_plugin() {
    // Build hello-plugin
    // Install to temp directory
    // Create PluginHost with temp dir
    // Verify plugin loads and on_load is called
}
```

---

## Phase 10: Documentation & Polish

### 10.1 Update PROGRESS.md

Mark milestone 1.3 items as complete:
- [x] Plugin trait and API crate (vibes-plugin-api)
- [x] Dynamic library loading
- [x] Plugin lifecycle (load, unload, enable, disable)
- [x] vibes plugin CLI commands
- [x] Event subscription system

### 10.2 Update README

Add plugin section:
```markdown
## Plugins

vibes supports native Rust plugins for extending functionality.

### Installing Plugins

```bash
# List installed plugins
vibes plugin list

# Enable/disable plugins
vibes plugin enable analytics
vibes plugin disable history
```

### Plugin Directory

Plugins are installed to `~/.config/vibes/plugins/`:
```
~/.config/vibes/plugins/
├── registry.toml
└── my-plugin/
    ├── my-plugin.0.1.0.so
    ├── my-plugin.so -> ./my-plugin.0.1.0.so
    └── config.toml
```
```

### 10.3 Add plugin authoring docs

Create `docs/plugin-authoring.md` with:
- Getting started
- Plugin trait reference
- Event handlers
- Configuration
- CLI commands
- Publishing plugins

---

## Verification Checklist

Before marking 1.3 complete:

- [ ] `just pre-commit` passes (fmt, clippy, test)
- [ ] `vibes plugin --help` shows all commands
- [ ] `vibes plugin list` works with no plugins
- [ ] Example plugin builds and loads
- [ ] Plugin events are dispatched correctly
- [ ] Plugin panics don't crash vibes
- [ ] API version mismatch shows clear error
- [ ] Enable/disable persists to registry.toml
- [ ] Plugin config loads from plugin directory
