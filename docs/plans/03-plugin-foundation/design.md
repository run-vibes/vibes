# Milestone 1.3: Plugin Foundation - Design Document

> Plugin trait and API crate, dynamic library loading, plugin lifecycle, CLI commands, and event subscription system.

## Overview

This milestone establishes the extensibility layer for vibes. Plugins are native Rust dynamic libraries that can react to events, register CLI commands, and integrate with the web server (future milestone).

### Key Decisions

| Decision | Choice | Notes |
|----------|--------|-------|
| Plugin format | Native Rust dynamic libraries (.so/.dylib/.dll) | ADR-002 |
| API versioning | Strict version matching (MVP), semver later | ADR-009 |
| Config storage | Per-plugin directories in `~/.config/vibes/plugins/` | Self-contained plugins |
| Binary versioning | Versioned files with symlinks (`foo.0.1.0.so`, `foo.so -> ...`) | Enables rollback |
| CLI commands | Namespaced under plugin name | `vibes <plugin> <cmd>` |
| Event subscription | Type-based handlers with default no-ops | Compile-time safety |
| Failure handling | Timeout + panic isolation | Disable faulting plugins |
| Plugin export | C ABI with helper macro | Proven pattern |

---

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              vibes binary                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                           vibes-core                                     ││
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                   ││
│  │  │   Session    │  │   EventBus   │  │ PluginHost   │◄──────────────┐   ││
│  │  │   Manager    │  │  (memory)    │  │              │               │   ││
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │   ││
│  │         │                 │                 │                        │   ││
│  │         │    ┌────────────┴────────────┐    │                        │   ││
│  │         │    │      Event Flow         │    │                        │   ││
│  │         │    │  ┌─────────────────┐    │    │                        │   ││
│  │         └────┼─►│   VibesEvent    │────┼────┘                        │   ││
│  │              │  └─────────────────┘    │                             │   ││
│  │              └─────────────────────────┘                             │   ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│         ┌──────────────────────────┼──────────────────────────┐             │
│         ▼                          ▼                          ▼             │
│  ┌─────────────┐           ┌─────────────┐           ┌─────────────┐       │
│  │  CLI Mode   │           │ Server Mode │           │  GUI Mode   │       │
│  │  (vibes-cli)│           │   (axum)    │           │  (future)   │       │
│  └─────────────┘           └─────────────┘           └─────────────┘       │
└─────────────────────────────────────────────────────────────────────────────┘
         │
         │ loads
         ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ~/.config/vibes/plugins/                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │   analytics/    │  │    history/     │  │    export/      │             │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │             │
│  │  │.0.1.0.so  │  │  │  │.0.2.0.so  │  │  │  │.0.1.0.so  │  │             │
│  │  │.so ──────►│  │  │  │.so ──────►│  │  │  │.so ──────►│  │             │
│  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │             │
│  │  config.toml    │  │  config.toml    │  │  config.toml    │             │
│  │  data/          │  │  history.db     │  │  templates/     │             │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘             │
│                                                                              │
│  registry.toml ─── enabled: [analytics, history]                            │
└─────────────────────────────────────────────────────────────────────────────┘
         │
         │ implements
         ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         vibes-plugin-api (crate)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  pub trait Plugin {                                                          │
│      fn manifest(&self) -> PluginManifest;                                  │
│      fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()>;          │
│      fn on_unload(&mut self) -> Result<()>;                                 │
│                                                                              │
│      // Type-based event handlers (default no-ops)                          │
│      fn on_turn_complete(&mut self, session_id: &str, usage: &Usage, ...);  │
│      fn on_text_delta(&mut self, session_id: &str, text: &str, ...);        │
│      fn on_session_created(&mut self, session_id: &str, name: Option<&str>);│
│      // ... more event handlers                                              │
│  }                                                                           │
│                                                                              │
│  vibes_plugin::export_plugin!(MyPlugin);  // C ABI export macro             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
┌──────────┐     spawn      ┌─────────────┐    stream-json    ┌─────────────┐
│  vibes   │───────────────►│ Claude Code │─────────────────►│  Backend    │
│  claude  │                │  (subprocess)│                  │  (parser)   │
└──────────┘                └─────────────┘                   └──────┬──────┘
                                                                     │
                                                              ClaudeEvent
                                                                     │
                                                                     ▼
┌─────────────┐◄────────────────────────────────────────────┌─────────────┐
│  Plugins    │            VibesEvent (typed handlers)       │  EventBus   │
│  on_*()     │◄────────────────────────────────────────────│             │
└─────────────┘                                              └─────────────┘
      │                                                             ▲
      │ register_command()                                          │
      │ register_route()                                            │
      ▼                                                             │
┌─────────────┐                                              ┌─────────────┐
│PluginContext│─────────────────────────────────────────────►│   Server    │
│             │      HTTP routes, WS handlers                │   (future)  │
└─────────────┘                                              └─────────────┘
```

---

## Crate Structure

```
vibes/
├── vibes-core/              # Existing - add plugin host
│   └── src/
│       └── plugins/
│           ├── mod.rs       # Module exports
│           ├── host.rs      # PluginHost implementation
│           ├── registry.rs  # Registry parsing
│           └── error.rs     # Plugin errors
├── vibes-plugin-api/        # NEW - published crate for plugin authors
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Plugin trait, types, export macro
│       ├── context.rs       # PluginContext capabilities
│       └── error.rs         # PluginError for plugin authors
└── vibes-cli/               # Existing - add `vibes plugin` commands
    └── src/commands/
        └── plugin.rs        # Plugin management commands
```

---

## Plugin Types & Trait Design

### vibes-plugin-api Crate

This is the public API that plugin authors depend on. It must be stable.

```rust
// vibes-plugin-api/src/lib.rs

/// Plugin metadata declared in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,           // semver
    pub api_version: u32,          // must match vibes exactly (for now)
    pub description: String,
    pub author: String,
    pub license: PluginLicense,
    pub commands: Vec<CommandSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginLicense {
    Free,
    Paid { product_id: String },   // future: licensing system
    Trial { days: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    pub name: String,              // e.g., "report" -> `vibes analytics report`
    pub description: String,
    pub args: Vec<ArgSpec>,
}

/// The core plugin trait
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;

    // Lifecycle
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;

    // Type-based event handlers - default no-ops
    fn on_session_created(&mut self, _session_id: &str, _name: Option<&str>, _ctx: &mut PluginContext) {}
    fn on_session_state_changed(&mut self, _session_id: &str, _state: &SessionState, _ctx: &mut PluginContext) {}
    fn on_turn_start(&mut self, _session_id: &str, _ctx: &mut PluginContext) {}
    fn on_turn_complete(&mut self, _session_id: &str, _usage: &Usage, _ctx: &mut PluginContext) {}
    fn on_text_delta(&mut self, _session_id: &str, _text: &str, _ctx: &mut PluginContext) {}
    fn on_thinking_delta(&mut self, _session_id: &str, _text: &str, _ctx: &mut PluginContext) {}
    fn on_tool_use_start(&mut self, _session_id: &str, _tool_id: &str, _name: &str, _ctx: &mut PluginContext) {}
    fn on_tool_result(&mut self, _session_id: &str, _tool_id: &str, _output: &str, _is_error: bool, _ctx: &mut PluginContext) {}
    fn on_error(&mut self, _session_id: &str, _message: &str, _recoverable: bool, _ctx: &mut PluginContext) {}
}

/// Export macro - generates C ABI entry points
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _vibes_plugin_create() -> *mut dyn Plugin {
            let plugin = Box::new(<$plugin_type>::default());
            Box::into_raw(plugin)
        }

        #[no_mangle]
        pub extern "C" fn _vibes_plugin_api_version() -> u32 {
            vibes_plugin_api::API_VERSION
        }
    };
}

pub const API_VERSION: u32 = 1;
```

---

## PluginContext Capabilities

`PluginContext` is the plugin's interface to vibes-core. It provides access to configuration, command registration, and future server integration.

```rust
// vibes-plugin-api/src/context.rs

/// Plugin's interface to vibes-core capabilities
pub struct PluginContext<'a> {
    plugin_name: &'a str,
    plugin_dir: &'a Path,              // ~/.config/vibes/plugins/<name>/
    config: &'a mut PluginConfig,
    commands: &'a mut CommandRegistry,
    routes: &'a mut RouteRegistry,     // future: server integration
}

impl<'a> PluginContext<'a> {
    // ─── Configuration ───────────────────────────────────────────────

    /// Get plugin's config directory
    pub fn plugin_dir(&self) -> &Path {
        self.plugin_dir
    }

    /// Read a config value
    pub fn config_get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.config.get(key)
    }

    /// Write a config value (persisted to config.toml)
    pub fn config_set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), PluginError> {
        self.config.set(key, value)
    }

    // ─── CLI Commands ────────────────────────────────────────────────

    /// Register a command handler
    /// Command will be available as `vibes <plugin-name> <command-name>`
    pub fn register_command<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&CommandArgs, &mut PluginContext) -> Result<(), PluginError> + Send + Sync + 'static,
    {
        self.commands.register(self.plugin_name, name, Box::new(handler));
    }

    // ─── Server Integration (Milestone 1.4) ──────────────────────────

    /// Register an HTTP route (available when server feature enabled)
    pub fn register_route(&mut self, method: Method, path: &str, handler: RouteHandler) {
        self.routes.register(self.plugin_name, method, path, handler);
    }

    /// Register a WebSocket message handler
    pub fn register_ws_handler(&mut self, msg_type: &str, handler: WsHandler) {
        self.routes.register_ws(self.plugin_name, msg_type, handler);
    }

    // ─── Logging ─────────────────────────────────────────────────────

    /// Log at info level (automatically prefixed with plugin name)
    pub fn log_info(&self, message: &str) {
        tracing::info!(plugin = self.plugin_name, "{}", message);
    }

    pub fn log_warn(&self, message: &str) {
        tracing::warn!(plugin = self.plugin_name, "{}", message);
    }

    pub fn log_error(&self, message: &str) {
        tracing::error!(plugin = self.plugin_name, "{}", message);
    }
}

/// Arguments passed to command handlers
pub struct CommandArgs {
    pub args: Vec<String>,
    pub flags: HashMap<String, String>,
}
```

---

## PluginHost (vibes-core)

The `PluginHost` lives in vibes-core and manages plugin loading, lifecycle, and event dispatch.

```rust
// vibes-core/src/plugins/host.rs

pub struct PluginHost {
    plugins: HashMap<String, LoadedPlugin>,
    plugin_dirs: Vec<PathBuf>,         // [project, user] precedence
    registry: PluginRegistry,
    command_registry: CommandRegistry,
    route_registry: RouteRegistry,
    handler_timeout: Duration,          // default 5s
}

struct LoadedPlugin {
    manifest: PluginManifest,
    instance: Box<dyn Plugin>,
    library: libloading::Library,       // keeps dylib loaded
    state: PluginState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Loaded,
    Disabled { reason: String },
    Failed { error: String },
}

impl PluginHost {
    pub fn new(config: PluginHostConfig) -> Self { /* ... */ }

    /// Discover and load all enabled plugins
    pub fn load_all(&mut self) -> Result<(), PluginHostError> {
        let registry = self.load_registry()?;

        for plugin_dir in self.discover_plugins()? {
            let name = plugin_dir.file_name().unwrap().to_string_lossy();

            if !registry.is_enabled(&name) {
                continue;
            }

            match self.load_plugin(&plugin_dir) {
                Ok(plugin) => {
                    tracing::info!(plugin = %name, "Plugin loaded");
                    self.plugins.insert(name.to_string(), plugin);
                }
                Err(e) => {
                    tracing::error!(plugin = %name, error = %e, "Failed to load plugin");
                }
            }
        }
        Ok(())
    }

    /// Load a single plugin from directory
    fn load_plugin(&mut self, dir: &Path) -> Result<LoadedPlugin, PluginHostError> {
        // 1. Find .so symlink
        let lib_path = self.find_library(dir)?;

        // 2. Load dynamic library
        let library = unsafe { libloading::Library::new(&lib_path)? };

        // 3. Check API version
        let api_version: extern "C" fn() -> u32 =
            unsafe { *library.get(b"_vibes_plugin_api_version")? };

        if api_version() != vibes_plugin_api::API_VERSION {
            return Err(PluginHostError::ApiVersionMismatch {
                expected: vibes_plugin_api::API_VERSION,
                found: api_version(),
            });
        }

        // 4. Create plugin instance
        let create: extern "C" fn() -> *mut dyn Plugin =
            unsafe { *library.get(b"_vibes_plugin_create")? };

        let instance = unsafe { Box::from_raw(create()) };
        let manifest = instance.manifest();

        // 5. Call on_load with context
        let mut ctx = self.create_context(&manifest.name, dir);
        instance.on_load(&mut ctx)?;

        Ok(LoadedPlugin {
            manifest,
            instance,
            library,
            state: PluginState::Loaded,
        })
    }

    /// Dispatch event to all plugins with timeout + panic isolation
    pub fn dispatch_event(&mut self, event: &VibesEvent) {
        for (name, plugin) in &mut self.plugins {
            if plugin.state != PluginState::Loaded {
                continue;
            }

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                self.dispatch_to_plugin(plugin, event)
            }));

            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!(plugin = %name, error = %e, "Plugin handler error");
                }
                Err(_) => {
                    tracing::error!(plugin = %name, "Plugin panicked, disabling");
                    plugin.state = PluginState::Failed {
                        error: "Plugin panicked".to_string()
                    };
                }
            }
        }
    }

    /// Route event to appropriate typed handler
    fn dispatch_to_plugin(&self, plugin: &mut LoadedPlugin, event: &VibesEvent)
        -> Result<(), PluginError>
    {
        let mut ctx = self.create_context(&plugin.manifest.name, ...);

        match event {
            VibesEvent::SessionCreated { session_id, name } => {
                plugin.instance.on_session_created(session_id, name.as_deref(), &mut ctx);
            }
            VibesEvent::Claude { session_id, event: ClaudeEvent::TurnComplete { usage } } => {
                plugin.instance.on_turn_complete(session_id, usage, &mut ctx);
            }
            // ... dispatch other event types
            _ => {}
        }
        Ok(())
    }
}
```

---

## CLI Commands

New commands under `vibes plugin` for managing plugins.

```
vibes plugin <COMMAND>

Commands:
  list      List installed plugins and their status
  enable    Enable a disabled plugin
  disable   Disable a plugin without removing it
  info      Show detailed plugin information
  reload    Reload a plugin (useful during development)

Examples:
  vibes plugin list
  vibes plugin enable analytics
  vibes plugin disable history
  vibes plugin info analytics
  vibes plugin reload analytics
```

### Implementation

```rust
// vibes-cli/src/commands/plugin.rs

#[derive(Subcommand)]
pub enum PluginCommand {
    /// List installed plugins
    List {
        #[arg(long)]
        all: bool,  // include disabled
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

pub fn handle_plugin_command(cmd: PluginCommand) -> Result<()> {
    match cmd {
        PluginCommand::List { all } => {
            let host = PluginHost::new(config)?;
            let plugins = host.list_plugins(all)?;

            for p in plugins {
                let status = match p.state {
                    PluginState::Loaded => "✓".green(),
                    PluginState::Disabled { .. } => "○".dim(),
                    PluginState::Failed { .. } => "✗".red(),
                };
                println!("{} {} v{}", status, p.manifest.name, p.manifest.version);
            }
        }
        // ... other commands
    }
}
```

### Example Output

```bash
$ vibes plugin list
✓ analytics v0.1.0    Track token usage and session stats
✓ history v0.2.0      Searchable session history
○ export v0.1.0       Export sessions to markdown/JSON (disabled)

$ vibes plugin info analytics
Name:        analytics
Version:     0.1.0
API Version: 1
Author:      vibes-team
License:     Free
Status:      Loaded

Commands:
  vibes analytics report    Generate usage report
  vibes analytics summary   Show quick summary

Config: ~/.config/vibes/plugins/analytics/config.toml
```

---

## Registry & Configuration

### registry.toml

Central file tracking plugin enable/disable state:

```toml
# ~/.config/vibes/plugins/registry.toml

# Enabled plugins (loaded on startup)
enabled = ["analytics", "history"]

# Plugin-specific overrides (optional)
[analytics]
# pin to specific version instead of following symlink
# version = "0.1.0"

[history]
# no overrides, uses defaults
```

### Plugin config.toml

Each plugin has its own config in its directory:

```toml
# ~/.config/vibes/plugins/analytics/config.toml

track_tokens = true
report_frequency = "daily"
export_format = "json"
```

### Complete Plugin Directory Structure

```
~/.config/vibes/
├── config.toml                      # vibes core config (from 1.2)
└── plugins/
    ├── registry.toml                # enabled/disabled state
    ├── analytics/
    │   ├── analytics.0.1.0.so       # versioned binary
    │   ├── analytics.so -> ./analytics.0.1.0.so
    │   ├── config.toml              # plugin settings
    │   └── data/                    # plugin data
    └── history/
        ├── history.0.2.0.so
        ├── history.so -> ./history.0.2.0.so
        ├── config.toml
        └── history.db

.vibes/plugins/                      # project-level (optional)
├── registry.toml
└── custom-plugin/
    └── ...
```

### Loading Precedence

1. Project plugins (`.vibes/plugins/`) - can override user plugins
2. User plugins (`~/.config/vibes/plugins/`)

---

## Error Types

```rust
// vibes-core/src/plugins/error.rs

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
    InitFailed(#[from] PluginError),

    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    #[error("Plugin '{name}' not found")]
    NotFound { name: String },

    #[error("Plugin '{name}' timed out after {timeout:?}")]
    Timeout { name: String, timeout: Duration },
}

// vibes-plugin-api/src/error.rs (plugin authors use this)

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command failed: {0}")]
    Command(String),

    #[error("{0}")]
    Custom(String),
}
```

---

## Testing Strategy

### Test Organization

```
vibes-core/
└── src/plugins/
    ├── mod.rs
    ├── host.rs
    ├── registry.rs
    └── tests/
        ├── host_test.rs        # PluginHost unit tests
        ├── registry_test.rs    # Registry parsing tests
        └── mock_plugin.rs      # Test plugin for integration tests

vibes-plugin-api/
└── tests/
    ├── macro_test.rs           # export_plugin! macro tests
    └── context_test.rs         # PluginContext tests
```

### Mock Plugin for Testing

```rust
// vibes-core/src/plugins/tests/mock_plugin.rs

/// A test plugin that records all events it receives
pub struct MockPlugin {
    pub events_received: Vec<String>,
    pub should_panic: bool,
    pub should_hang: bool,
}

impl Plugin for MockPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "mock".to_string(),
            version: "0.0.1".to_string(),
            api_version: API_VERSION,
            ..Default::default()
        }
    }

    fn on_turn_complete(&mut self, session_id: &str, usage: &Usage, _ctx: &mut PluginContext) {
        if self.should_panic {
            panic!("Mock plugin panic!");
        }
        if self.should_hang {
            std::thread::sleep(Duration::from_secs(60));
        }
        self.events_received.push(format!("turn_complete:{}", session_id));
    }
}
```

### Key Test Cases

```rust
#[tokio::test]
async fn test_plugin_loading() {
    // Plugin loads successfully and on_load is called
}

#[tokio::test]
async fn test_api_version_mismatch_rejected() {
    // Plugin with wrong API version is not loaded
}

#[tokio::test]
async fn test_event_dispatch_to_plugins() {
    // Events are routed to correct typed handlers
}

#[tokio::test]
async fn test_plugin_panic_isolation() {
    // Panicking plugin is disabled, others continue
}

#[tokio::test]
async fn test_plugin_timeout_isolation() {
    // Hanging plugin is killed after timeout
}

#[tokio::test]
async fn test_registry_enable_disable() {
    // Enable/disable persists to registry.toml
}

#[tokio::test]
async fn test_command_registration() {
    // Plugin-registered commands are callable
}

#[tokio::test]
async fn test_config_persistence() {
    // config_set persists to plugin's config.toml
}
```

---

## Dependencies

```toml
# vibes-core/Cargo.toml additions
[dependencies]
libloading = "0.8"        # Dynamic library loading

# vibes-plugin-api/Cargo.toml
[package]
name = "vibes-plugin-api"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "1"

[dev-dependencies]
vibes-plugin-api = { path = "../vibes-plugin-api" }
```

---

## Milestone 1.3 Deliverables

| Component | Description |
|-----------|-------------|
| `vibes-plugin-api` crate | Plugin trait, types, export macro |
| `vibes-core/plugins/` module | PluginHost, registry, loading |
| `vibes plugin` CLI | list, enable, disable, info, reload |
| ADR-009 | Plugin API versioning strategy |
| Tests | Unit + integration tests for plugin system |
