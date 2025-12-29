# Milestone 4.2.6: Plugin API Extension - Design Document

> Extends the plugin system so groove (and future plugins) can register CLI commands and HTTP routes.

## Overview

This milestone extends `vibes-plugin-api` to support dynamic registration of CLI commands and HTTP routes during plugin load. It then migrates the groove CLI commands and API routes from `vibes-cli`/`vibes-server` to the `vibes-groove` plugin.

### Goals

1. **Dynamic registration** - Plugins register commands/routes at runtime during `on_load`
2. **Conflict detection** - Clear errors when plugins register conflicting commands/routes
3. **Clean separation** - Plugins own their handlers; CLI/server handle dispatch
4. **Migration** - Move groove code out of core binaries into the plugin

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Registration timing | Dynamic (during `on_load`) | Plugins can adapt based on config/capabilities |
| Handler invocation | Trait dispatch methods | Avoids closure/lifetime issues with dylibs |
| Command namespacing | `vibes <plugin> <cmd...>` | Clear ownership, no conflicts between plugins |
| Route prefixing | `/api/<plugin>/...` | Consistent API structure |
| API version | Bump to v2 | ABI change requires version bump |

---

## New Types

### vibes-plugin-api Types

```rust
// ─── CLI Command Types ────────────────────────────────────────────

/// Specification for a CLI command
#[derive(Debug, Clone)]
pub struct CommandSpec {
    /// Command path, e.g., ["trust", "levels"] → `vibes groove trust levels`
    pub path: Vec<String>,
    /// Short description for help text
    pub description: String,
    /// Argument specifications
    pub args: Vec<ArgSpec>,
}

/// Specification for a command argument
#[derive(Debug, Clone)]
pub struct ArgSpec {
    /// Argument name
    pub name: String,
    /// Description for help text
    pub description: String,
    /// Whether this argument is required
    pub required: bool,
}

/// Output from a CLI command handler
#[derive(Debug)]
pub enum CommandOutput {
    /// Plain text output (printed as-is)
    Text(String),
    /// Structured data (can be formatted as table, JSON, etc.)
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    /// Success with no output
    Success,
    /// Exit with specific code
    Exit(i32),
}

// ─── HTTP Route Types ─────────────────────────────────────────────

/// Specification for an HTTP route
#[derive(Debug, Clone)]
pub struct RouteSpec {
    /// HTTP method
    pub method: HttpMethod,
    /// Path pattern, e.g., "/policy" or "/quarantine/:id"
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Incoming HTTP request passed to plugin handler
#[derive(Debug)]
pub struct RouteRequest {
    /// Path parameters extracted from route pattern (e.g., ":id" → "123")
    pub params: HashMap<String, String>,
    /// Query parameters
    pub query: HashMap<String, String>,
    /// Request body as bytes
    pub body: Vec<u8>,
    /// Request headers
    pub headers: HashMap<String, String>,
}

/// HTTP response from plugin handler
#[derive(Debug)]
pub struct RouteResponse {
    /// HTTP status code
    pub status: u16,
    /// Response body
    pub body: Vec<u8>,
    /// Content-Type header (defaults to application/json)
    pub content_type: String,
}

impl RouteResponse {
    /// Create a JSON response
    pub fn json<T: Serialize>(status: u16, data: &T) -> Result<Self, PluginError> {
        Ok(Self {
            status,
            body: serde_json::to_vec(data)?,
            content_type: "application/json".to_string(),
        })
    }
}
```

### Error Types

```rust
// vibes-plugin-api/src/error.rs additions
pub enum PluginError {
    // ... existing variants ...

    #[error("Duplicate command: {0}")]
    DuplicateCommand(String),

    #[error("Duplicate route: {0}")]
    DuplicateRoute(String),

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Unknown route: {0}")]
    UnknownRoute(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// vibes-core/src/plugins/error.rs additions
pub enum PluginHostError {
    // ... existing variants ...

    #[error("Command conflict: '{command}' already registered by plugin '{existing_plugin}'")]
    CommandConflict {
        command: String,
        existing_plugin: String,
        new_plugin: String,
    },

    #[error("Route conflict: '{route}' already registered by plugin '{existing_plugin}'")]
    RouteConflict {
        route: String,
        existing_plugin: String,
        new_plugin: String,
    },
}
```

---

## Plugin Trait Extensions

New methods added to the `Plugin` trait:

```rust
pub trait Plugin: Send + Sync {
    // ─── Existing methods ─────────────────────────────────────────
    fn manifest(&self) -> PluginManifest;
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;

    // ... existing event handlers ...

    // ─── NEW: Command & Route Handlers ────────────────────────────

    /// Handle a CLI command invocation.
    ///
    /// Called when a user runs a command registered by this plugin.
    /// The `path` matches what was registered in `on_load`.
    ///
    /// Default: returns UnknownCommand error
    fn handle_command(
        &mut self,
        _path: &[&str],
        _args: &CommandArgs,
        _ctx: &mut PluginContext,
    ) -> Result<CommandOutput, PluginError> {
        Err(PluginError::UnknownCommand("no commands registered".into()))
    }

    /// Handle an HTTP route invocation.
    ///
    /// Called when an HTTP request matches a route registered by this plugin.
    ///
    /// Default: returns UnknownRoute error
    fn handle_route(
        &mut self,
        _method: HttpMethod,
        _path: &str,
        _request: RouteRequest,
        _ctx: &mut PluginContext,
    ) -> Result<RouteResponse, PluginError> {
        Err(PluginError::UnknownRoute("no routes registered".into()))
    }
}
```

---

## PluginContext Extensions

Registration methods for plugins to use during `on_load`:

```rust
impl PluginContext {
    /// Register a CLI command for this plugin.
    ///
    /// The command will be namespaced under the plugin name:
    /// `vibes <plugin-name> <path...>`
    ///
    /// For example, if plugin "groove" registers `["trust", "levels"]`:
    /// → `vibes groove trust levels`
    ///
    /// Returns error if command path is duplicate within this plugin.
    pub fn register_command(&mut self, spec: CommandSpec) -> Result<(), PluginError>;

    /// Get commands pending registration (used by PluginHost)
    pub fn pending_commands(&self) -> &[CommandSpec];

    /// Take pending commands (used by PluginHost after validation)
    pub fn take_pending_commands(&mut self) -> Vec<CommandSpec>;

    /// Register an HTTP route for this plugin.
    ///
    /// Routes are automatically prefixed: `/api/<plugin-name>/...`
    ///
    /// For example, if plugin "groove" registers `/policy`:
    /// → `GET /api/groove/policy`
    ///
    /// Path parameters use `:name` syntax: `/quarantine/:id/review`
    ///
    /// Returns error if route is duplicate within this plugin.
    pub fn register_route(&mut self, spec: RouteSpec) -> Result<(), PluginError>;

    /// Get routes pending registration (used by PluginHost)
    pub fn pending_routes(&self) -> &[RouteSpec];

    /// Take pending routes (used by PluginHost after validation)
    pub fn take_pending_routes(&mut self) -> Vec<RouteSpec>;
}

// Updated PluginContext struct
pub struct PluginContext {
    plugin_name: String,
    plugin_dir: PathBuf,
    config: PluginConfig,
    pending_commands: Vec<CommandSpec>,
    pending_routes: Vec<RouteSpec>,
    harness: Option<Arc<dyn Harness>>,
    capabilities: Vec<Capability>,
}
```

**Cleanup:** Remove deprecated `registered_commands: Vec<RegisteredCommand>` field and `RegisteredCommand` type.

---

## vibes-core: Command & Route Registries

### CommandRegistry

```rust
/// Registry of all plugin commands
pub struct CommandRegistry {
    /// Map from command path to owning plugin
    /// e.g., ["groove", "trust", "levels"] → RegisteredPluginCommand
    commands: HashMap<Vec<String>, RegisteredPluginCommand>,
}

struct RegisteredPluginCommand {
    plugin_name: String,
    spec: CommandSpec,
}

impl CommandRegistry {
    /// Register commands for a plugin (called after on_load validation)
    pub fn register(&mut self, plugin_name: &str, commands: Vec<CommandSpec>);

    /// Check if a command path conflicts with existing registrations
    pub fn check_conflict(&self, plugin_name: &str, path: &[String]) -> Option<&str>;

    /// Find plugin owning a command path
    pub fn find(&self, path: &[String]) -> Option<&RegisteredPluginCommand>;

    /// Get all commands (for help text generation)
    pub fn all_commands(&self) -> impl Iterator<Item = (&[String], &CommandSpec)>;
}
```

### RouteRegistry

```rust
/// Registry of all plugin routes
pub struct RouteRegistry {
    /// Map from (method, path_pattern) to owning plugin
    routes: HashMap<(HttpMethod, String), RegisteredPluginRoute>,
}

struct RegisteredPluginRoute {
    plugin_name: String,
    spec: RouteSpec,
    /// Compiled path matcher for parameter extraction
    matcher: PathMatcher,
}

impl RouteRegistry {
    /// Register routes for a plugin (called after on_load validation)
    pub fn register(&mut self, plugin_name: &str, routes: Vec<RouteSpec>);

    /// Check if a route conflicts with existing registrations
    pub fn check_conflict(&self, plugin_name: &str, spec: &RouteSpec) -> Option<&str>;

    /// Find plugin route matching a request
    pub fn match_route(&self, method: HttpMethod, path: &str)
        -> Option<(&RegisteredPluginRoute, HashMap<String, String>)>;
}

/// Simple path matcher supporting :param patterns
struct PathMatcher {
    segments: Vec<PathSegment>,
}

enum PathSegment {
    Literal(String),
    Param(String),
}
```

---

## vibes-cli Integration

Plugin commands dispatched via catch-all handler:

```rust
fn dispatch_plugin_command(args: &PluginCommandArgs) -> Result<()> {
    let plugin_host = get_plugin_host()?;

    // args.path = ["groove", "trust", "levels"]
    let cmd = plugin_host.command_registry()
        .find(&args.path)
        .ok_or_else(|| anyhow!("Unknown command: {}", args.path.join(" ")))?;

    // Build CommandArgs from remaining CLI args
    let command_args = CommandArgs {
        args: args.positional.clone(),
        flags: args.flags.clone(),
    };

    // Dispatch to plugin
    let output = plugin_host.dispatch_command(
        &cmd.plugin_name,
        &args.path[1..],  // strip plugin name prefix
        &command_args,
    )?;

    // Render output
    match output {
        CommandOutput::Text(text) => println!("{}", text),
        CommandOutput::Table { headers, rows } => print_table(&headers, &rows),
        CommandOutput::Success => {}
        CommandOutput::Exit(code) => std::process::exit(code),
    }

    Ok(())
}
```

---

## vibes-server Integration

Plugin routes mounted via catch-all axum handler:

```rust
/// Create router for plugin routes
pub fn plugin_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/{*path}", any(handle_plugin_route))
        .with_state(state)
}

/// Generic handler that dispatches to plugins
async fn handle_plugin_route(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Response {
    let method = to_http_method(request.method());
    let path = request.uri().path();

    // Find matching plugin route
    let plugin_host = state.plugin_host().read().await;
    let Some((route, params)) = plugin_host.route_registry().match_route(method, path) else {
        return not_found_response();
    };

    // Extract request data
    let route_request = RouteRequest {
        params,
        query: parse_query(request.uri().query()),
        headers: extract_headers(request.headers()),
        body: to_bytes(request.into_body()).await.unwrap_or_default().to_vec(),
    };

    // Dispatch to plugin
    let result = state.plugin_host().write().await
        .dispatch_route(&route.plugin_name, method, &route.spec.path, route_request);

    // Convert plugin response to axum response
    match result {
        Ok(resp) => build_response(resp),
        Err(e) => error_response(500, &e.to_string()),
    }
}
```

---

## Groove Migration

The `vibes-groove` plugin implements `Plugin` trait with command/route registration:

```rust
impl Plugin for GroovePlugin {
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        // CLI Commands
        ctx.register_command(CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust level hierarchy".into(),
            args: vec![],
        })?;
        ctx.register_command(CommandSpec {
            path: vec!["trust".into(), "role".into()],
            description: "Show permissions for a role".into(),
            args: vec![ArgSpec { name: "role".into(), description: "Role name".into(), required: true }],
        })?;
        ctx.register_command(CommandSpec {
            path: vec!["policy".into(), "show".into()],
            description: "Show current security policy".into(),
            args: vec![],
        })?;
        ctx.register_command(CommandSpec {
            path: vec!["policy".into(), "path".into()],
            description: "Show policy file search paths".into(),
            args: vec![],
        })?;
        ctx.register_command(CommandSpec {
            path: vec!["quarantine".into(), "list".into()],
            description: "List quarantined learnings".into(),
            args: vec![],
        })?;
        ctx.register_command(CommandSpec {
            path: vec!["quarantine".into(), "stats".into()],
            description: "Show quarantine statistics".into(),
            args: vec![],
        })?;

        // HTTP Routes
        ctx.register_route(RouteSpec { method: HttpMethod::Get, path: "/policy".into() })?;
        ctx.register_route(RouteSpec { method: HttpMethod::Get, path: "/trust/levels".into() })?;
        ctx.register_route(RouteSpec { method: HttpMethod::Get, path: "/trust/role/:role".into() })?;
        ctx.register_route(RouteSpec { method: HttpMethod::Get, path: "/quarantine".into() })?;
        ctx.register_route(RouteSpec { method: HttpMethod::Get, path: "/quarantine/stats".into() })?;
        ctx.register_route(RouteSpec { method: HttpMethod::Post, path: "/quarantine/:id/review".into() })?;

        ctx.log_info("groove plugin loaded");
        Ok(())
    }

    fn handle_command(&mut self, path: &[&str], args: &CommandArgs, _ctx: &mut PluginContext)
        -> Result<CommandOutput, PluginError>
    {
        match path {
            ["trust", "levels"] => self.cmd_trust_levels(),
            ["trust", "role"] => self.cmd_trust_role(args),
            ["policy", "show"] => self.cmd_policy_show(),
            ["policy", "path"] => self.cmd_policy_path(),
            ["quarantine", "list"] => self.cmd_quarantine_list(),
            ["quarantine", "stats"] => self.cmd_quarantine_stats(),
            _ => Err(PluginError::UnknownCommand(path.join(" "))),
        }
    }

    fn handle_route(&mut self, method: HttpMethod, path: &str, request: RouteRequest, _ctx: &mut PluginContext)
        -> Result<RouteResponse, PluginError>
    {
        match (method, path) {
            (HttpMethod::Get, "/policy") => self.route_get_policy(),
            (HttpMethod::Get, "/trust/levels") => self.route_get_trust_levels(),
            (HttpMethod::Get, "/trust/role/:role") => {
                let role = request.params.get("role").ok_or_else(||
                    PluginError::InvalidInput("missing role parameter".into()))?;
                self.route_get_role_permissions(role)
            }
            (HttpMethod::Get, "/quarantine") => self.route_list_quarantined(),
            (HttpMethod::Get, "/quarantine/stats") => self.route_get_quarantine_stats(),
            (HttpMethod::Post, "/quarantine/:id/review") => {
                let id = request.params.get("id").ok_or_else(||
                    PluginError::InvalidInput("missing id parameter".into()))?;
                self.route_review_quarantined(id, &request.body)
            }
            _ => Err(PluginError::UnknownRoute(format!("{:?} {}", method, path))),
        }
    }
}
```

### Migration Cleanup

Files to delete after migration:
- `vibes-cli/src/commands/groove.rs`
- `vibes-server/src/http/groove.rs`

Code to remove:
- `Commands::Groove` variant from CLI enum
- Groove routes from `vibes-server/src/http/mod.rs`
- Deprecated `RegisteredCommand` type and `registered_commands` field from plugin-api

---

## API Version

Bump API version to indicate breaking change:

```rust
// vibes-plugin-api/src/lib.rs
pub const API_VERSION: u32 = 2;  // Was 1
```

Plugins compiled against API v1 will fail to load with a clear version mismatch error.

---

## Testing Strategy

1. **Unit tests** in `vibes-plugin-api` for new types (`CommandSpec`, `RouteSpec`, `RouteResponse::json`)
2. **Unit tests** in `vibes-core` for `CommandRegistry` and `RouteRegistry`
3. **Integration test** with a mock plugin that registers commands/routes, verifying:
   - Successful registration
   - Conflict detection across plugins
   - Command dispatch
   - Route dispatch with path parameters
4. **Migrate existing groove tests** from `vibes-server` to `vibes-groove` crate

---

## Implementation Order

1. Extend `vibes-plugin-api` with new types and trait methods
2. Add `CommandRegistry` and `RouteRegistry` to `vibes-core`
3. Integrate with `vibes-cli` (plugin command dispatch)
4. Integrate with `vibes-server` (plugin route dispatch)
5. Migrate groove code to `vibes-groove` plugin
6. Clean up deprecated code
7. Update documentation

---

## Deliverables

| Component | Description |
|-----------|-------------|
| `vibes-plugin-api` v2 | New types, trait methods, registration API |
| `vibes-core/plugins/commands.rs` | CommandRegistry implementation |
| `vibes-core/plugins/routes.rs` | RouteRegistry implementation |
| `vibes-cli` integration | Plugin command dispatch |
| `vibes-server` integration | Plugin route dispatch |
| `vibes-groove` migration | Commands and routes moved to plugin |
| Cleanup | Deprecated code removed |
| Tests | Unit and integration tests |
