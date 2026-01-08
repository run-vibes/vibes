# Milestone 4.2.6: Plugin API Extension - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend the plugin system so plugins can register CLI commands and HTTP routes, then migrate groove to use it.

**Architecture:** Dynamic registration during `on_load` with trait-based dispatch. Plugins call `ctx.register_command()` and `ctx.register_route()`, then implement `handle_command()` and `handle_route()` methods. Registries in vibes-core track ownership and detect conflicts.

**Tech Stack:** Rust, vibes-plugin-api, vibes-core, axum (server), clap (CLI)

---

## Phase 1: Plugin API Types

### Task 1: Add HTTP Method Enum

**Files:**
- Create: `vibes-plugin-api/src/http.rs`
- Modify: `vibes-plugin-api/src/lib.rs`

**Step 1: Write the failing test**

```rust
// vibes-plugin-api/src/http.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_equality() {
        assert_eq!(HttpMethod::Get, HttpMethod::Get);
        assert_ne!(HttpMethod::Get, HttpMethod::Post);
    }

    #[test]
    fn test_http_method_debug() {
        assert_eq!(format!("{:?}", HttpMethod::Get), "Get");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api http_method`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
// vibes-plugin-api/src/http.rs
//! HTTP types for plugin route registration

/// HTTP method for route registration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_equality() {
        assert_eq!(HttpMethod::Get, HttpMethod::Get);
        assert_ne!(HttpMethod::Get, HttpMethod::Post);
    }

    #[test]
    fn test_http_method_debug() {
        assert_eq!(format!("{:?}", HttpMethod::Get), "Get");
    }
}
```

**Step 4: Add module to lib.rs**

```rust
// vibes-plugin-api/src/lib.rs - add after other mod declarations
pub mod http;
pub use http::HttpMethod;
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api http_method`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-plugin-api/src/http.rs vibes-plugin-api/src/lib.rs
git commit -m "feat(plugin-api): add HttpMethod enum"
```

---

### Task 2: Add Command Types

**Files:**
- Create: `vibes-plugin-api/src/command.rs`
- Modify: `vibes-plugin-api/src/lib.rs`

**Step 1: Write the failing test**

```rust
// vibes-plugin-api/src/command.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_spec_creation() {
        let spec = CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust levels".into(),
            args: vec![],
        };
        assert_eq!(spec.path, vec!["trust", "levels"]);
        assert!(spec.args.is_empty());
    }

    #[test]
    fn test_arg_spec_required() {
        let arg = ArgSpec {
            name: "role".into(),
            description: "Role name".into(),
            required: true,
        };
        assert!(arg.required);
    }

    #[test]
    fn test_command_output_text() {
        let output = CommandOutput::Text("Hello".into());
        match output {
            CommandOutput::Text(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_command_output_table() {
        let output = CommandOutput::Table {
            headers: vec!["Name".into(), "Value".into()],
            rows: vec![vec!["foo".into(), "bar".into()]],
        };
        match output {
            CommandOutput::Table { headers, rows } => {
                assert_eq!(headers.len(), 2);
                assert_eq!(rows.len(), 1);
            }
            _ => panic!("Expected Table variant"),
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api command`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
// vibes-plugin-api/src/command.rs
//! CLI command types for plugin registration

/// Specification for a CLI command
#[derive(Debug, Clone)]
pub struct CommandSpec {
    /// Command path, e.g., ["trust", "levels"] → `vibes <plugin> trust levels`
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
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// Success with no output
    Success,
    /// Exit with specific code
    Exit(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_spec_creation() {
        let spec = CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust levels".into(),
            args: vec![],
        };
        assert_eq!(spec.path, vec!["trust", "levels"]);
        assert!(spec.args.is_empty());
    }

    #[test]
    fn test_arg_spec_required() {
        let arg = ArgSpec {
            name: "role".into(),
            description: "Role name".into(),
            required: true,
        };
        assert!(arg.required);
    }

    #[test]
    fn test_command_output_text() {
        let output = CommandOutput::Text("Hello".into());
        match output {
            CommandOutput::Text(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_command_output_table() {
        let output = CommandOutput::Table {
            headers: vec!["Name".into(), "Value".into()],
            rows: vec![vec!["foo".into(), "bar".into()]],
        };
        match output {
            CommandOutput::Table { headers, rows } => {
                assert_eq!(headers.len(), 2);
                assert_eq!(rows.len(), 1);
            }
            _ => panic!("Expected Table variant"),
        }
    }
}
```

**Step 4: Add module to lib.rs**

```rust
// vibes-plugin-api/src/lib.rs - add after http module
pub mod command;
pub use command::{ArgSpec, CommandOutput, CommandSpec};
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api command`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-plugin-api/src/command.rs vibes-plugin-api/src/lib.rs
git commit -m "feat(plugin-api): add command types (CommandSpec, ArgSpec, CommandOutput)"
```

---

### Task 3: Add Route Types

**Files:**
- Modify: `vibes-plugin-api/src/http.rs`
- Modify: `vibes-plugin-api/src/lib.rs`
- Modify: `vibes-plugin-api/Cargo.toml`

**Step 1: Write the failing test**

```rust
// Add to vibes-plugin-api/src/http.rs tests
#[test]
fn test_route_spec_creation() {
    let spec = RouteSpec {
        method: HttpMethod::Get,
        path: "/policy".into(),
    };
    assert_eq!(spec.method, HttpMethod::Get);
    assert_eq!(spec.path, "/policy");
}

#[test]
fn test_route_request_params() {
    let request = RouteRequest {
        params: [("id".into(), "123".into())].into_iter().collect(),
        query: HashMap::new(),
        body: vec![],
        headers: HashMap::new(),
    };
    assert_eq!(request.params.get("id"), Some(&"123".to_string()));
}

#[test]
fn test_route_response_json() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Data { value: i32 }

    let resp = RouteResponse::json(200, &Data { value: 42 }).unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.content_type, "application/json");
    assert!(String::from_utf8_lossy(&resp.body).contains("42"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api route`
Expected: FAIL - RouteSpec not found

**Step 3: Add serde_json dependency**

```toml
# vibes-plugin-api/Cargo.toml - add to [dependencies]
serde_json = "1"
```

**Step 4: Write implementation**

```rust
// vibes-plugin-api/src/http.rs - add after HttpMethod
use std::collections::HashMap;
use serde::Serialize;
use crate::error::PluginError;

/// Specification for an HTTP route
#[derive(Debug, Clone)]
pub struct RouteSpec {
    /// HTTP method
    pub method: HttpMethod,
    /// Path pattern, e.g., "/policy" or "/quarantine/:id"
    pub path: String,
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
    /// Content-Type header
    pub content_type: String,
}

impl RouteResponse {
    /// Create a JSON response
    pub fn json<T: Serialize>(status: u16, data: &T) -> Result<Self, PluginError> {
        Ok(Self {
            status,
            body: serde_json::to_vec(data).map_err(|e| PluginError::Json(e.to_string()))?,
            content_type: "application/json".to_string(),
        })
    }

    /// Create a plain text response
    pub fn text(status: u16, text: impl Into<String>) -> Self {
        Self {
            status,
            body: text.into().into_bytes(),
            content_type: "text/plain".to_string(),
        }
    }

    /// Create an empty response with status code
    pub fn empty(status: u16) -> Self {
        Self {
            status,
            body: vec![],
            content_type: "application/json".to_string(),
        }
    }
}
```

**Step 5: Update exports in lib.rs**

```rust
// vibes-plugin-api/src/lib.rs - update http exports
pub use http::{HttpMethod, RouteRequest, RouteResponse, RouteSpec};
```

**Step 6: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api route`
Expected: PASS

**Step 7: Commit**

```bash
git add vibes-plugin-api/
git commit -m "feat(plugin-api): add route types (RouteSpec, RouteRequest, RouteResponse)"
```

---

### Task 4: Add New Error Variants

**Files:**
- Modify: `vibes-plugin-api/src/error.rs`

**Step 1: Write the failing test**

```rust
// Add to vibes-plugin-api/src/error.rs tests
#[test]
fn test_duplicate_command_error() {
    let err = PluginError::DuplicateCommand("trust levels".into());
    assert!(err.to_string().contains("trust levels"));
}

#[test]
fn test_unknown_command_error() {
    let err = PluginError::UnknownCommand("foo bar".into());
    assert!(err.to_string().contains("foo bar"));
}

#[test]
fn test_json_error() {
    let err = PluginError::Json("parse error".into());
    assert!(err.to_string().contains("parse error"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api error`
Expected: FAIL - DuplicateCommand variant not found

**Step 3: Add new error variants**

```rust
// vibes-plugin-api/src/error.rs - add new variants to PluginError enum
#[derive(Error, Debug)]
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
    Json(String),
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api error`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-plugin-api/src/error.rs
git commit -m "feat(plugin-api): add command/route error variants"
```

---

## Phase 2: Plugin Trait Extensions

### Task 5: Add handle_command to Plugin Trait

**Files:**
- Modify: `vibes-plugin-api/src/lib.rs`

**Step 1: Write the failing test**

```rust
// Add to vibes-plugin-api/src/lib.rs tests
#[test]
fn test_plugin_handle_command_default_returns_error() {
    use crate::context::{CommandArgs, PluginContext};
    use std::path::PathBuf;

    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn manifest(&self) -> PluginManifest {
            PluginManifest::default()
        }
        fn on_load(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {
            Ok(())
        }
        fn on_unload(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
    }

    let mut plugin = TestPlugin;
    let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));
    let args = CommandArgs::default();

    let result = plugin.handle_command(&["foo"], &args, &mut ctx);
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api handle_command`
Expected: FAIL - handle_command method not found

**Step 3: Add handle_command to Plugin trait**

```rust
// vibes-plugin-api/src/lib.rs - add to Plugin trait after existing methods

/// Handle a CLI command invocation.
///
/// Called when a user runs a command registered by this plugin.
/// The `path` matches what was registered in `on_load`.
///
/// Default: returns UnknownCommand error (override if registering commands)
fn handle_command(
    &mut self,
    _path: &[&str],
    _args: &CommandArgs,
    _ctx: &mut PluginContext,
) -> Result<CommandOutput, PluginError> {
    Err(PluginError::UnknownCommand("no commands registered".into()))
}
```

**Step 4: Add CommandArgs import**

```rust
// vibes-plugin-api/src/lib.rs - update context exports
pub use context::{Capability, CommandArgs, Harness, PluginConfig, PluginContext};
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api handle_command`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-plugin-api/src/lib.rs
git commit -m "feat(plugin-api): add handle_command to Plugin trait"
```

---

### Task 6: Add handle_route to Plugin Trait

**Files:**
- Modify: `vibes-plugin-api/src/lib.rs`

**Step 1: Write the failing test**

```rust
// Add to vibes-plugin-api/src/lib.rs tests
#[test]
fn test_plugin_handle_route_default_returns_error() {
    use crate::context::PluginContext;
    use crate::http::{HttpMethod, RouteRequest};
    use std::collections::HashMap;
    use std::path::PathBuf;

    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn manifest(&self) -> PluginManifest {
            PluginManifest::default()
        }
        fn on_load(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {
            Ok(())
        }
        fn on_unload(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
    }

    let mut plugin = TestPlugin;
    let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));
    let request = RouteRequest {
        params: HashMap::new(),
        query: HashMap::new(),
        body: vec![],
        headers: HashMap::new(),
    };

    let result = plugin.handle_route(HttpMethod::Get, "/foo", request, &mut ctx);
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api handle_route`
Expected: FAIL - handle_route method not found

**Step 3: Add handle_route to Plugin trait**

```rust
// vibes-plugin-api/src/lib.rs - add to Plugin trait after handle_command

/// Handle an HTTP route invocation.
///
/// Called when an HTTP request matches a route registered by this plugin.
///
/// Default: returns UnknownRoute error (override if registering routes)
fn handle_route(
    &mut self,
    _method: HttpMethod,
    _path: &str,
    _request: RouteRequest,
    _ctx: &mut PluginContext,
) -> Result<RouteResponse, PluginError> {
    Err(PluginError::UnknownRoute("no routes registered".into()))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api handle_route`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-plugin-api/src/lib.rs
git commit -m "feat(plugin-api): add handle_route to Plugin trait"
```

---

## Phase 3: PluginContext Registration

### Task 7: Add Command Registration to PluginContext

**Files:**
- Modify: `vibes-plugin-api/src/context.rs`

**Step 1: Write the failing test**

```rust
// Add to vibes-plugin-api/src/context.rs tests
#[test]
fn test_register_command() {
    use crate::command::{CommandSpec, ArgSpec};

    let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

    let result = ctx.register_command(CommandSpec {
        path: vec!["trust".into(), "levels".into()],
        description: "Show trust levels".into(),
        args: vec![],
    });

    assert!(result.is_ok());
    assert_eq!(ctx.pending_commands().len(), 1);
}

#[test]
fn test_register_command_duplicate_fails() {
    use crate::command::CommandSpec;

    let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

    let spec = CommandSpec {
        path: vec!["trust".into(), "levels".into()],
        description: "Show trust levels".into(),
        args: vec![],
    };

    ctx.register_command(spec.clone()).unwrap();
    let result = ctx.register_command(spec);

    assert!(result.is_err());
}

#[test]
fn test_take_pending_commands() {
    use crate::command::CommandSpec;

    let mut ctx = PluginContext::new("test".into(), PathBuf::from("/tmp"));

    ctx.register_command(CommandSpec {
        path: vec!["foo".into()],
        description: "Foo".into(),
        args: vec![],
    }).unwrap();

    let commands = ctx.take_pending_commands();
    assert_eq!(commands.len(), 1);
    assert!(ctx.pending_commands().is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api register_command`
Expected: FAIL - register_command method not found

**Step 3: Add pending_commands field to PluginContext**

```rust
// vibes-plugin-api/src/context.rs - update struct
use crate::command::CommandSpec;

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

**Step 4: Update constructors**

```rust
// vibes-plugin-api/src/context.rs - update new() and with_config()
impl PluginContext {
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
}
```

**Step 5: Add registration methods**

```rust
// vibes-plugin-api/src/context.rs - add to impl PluginContext

// ─── Command Registration ─────────────────────────────────────

/// Register a CLI command for this plugin.
///
/// The command will be namespaced under the plugin name:
/// `vibes <plugin-name> <path...>`
///
/// Returns error if command path is duplicate within this plugin.
pub fn register_command(&mut self, spec: CommandSpec) -> Result<(), PluginError> {
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
```

**Step 6: Add import for RouteSpec**

```rust
// vibes-plugin-api/src/context.rs - add to imports
use crate::http::RouteSpec;
```

**Step 7: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api register_command`
Expected: PASS

**Step 8: Commit**

```bash
git add vibes-plugin-api/src/context.rs
git commit -m "feat(plugin-api): add command registration to PluginContext"
```

---

### Task 8: Add Route Registration to PluginContext

**Files:**
- Modify: `vibes-plugin-api/src/context.rs`

**Step 1: Write the failing test**

```rust
// Add to vibes-plugin-api/src/context.rs tests
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
    }).unwrap();

    let result = ctx.register_route(RouteSpec {
        method: HttpMethod::Post,
        path: "/resource".into(),
    });

    assert!(result.is_ok());
    assert_eq!(ctx.pending_routes().len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-plugin-api register_route`
Expected: FAIL - register_route method not found

**Step 3: Add route registration methods**

```rust
// vibes-plugin-api/src/context.rs - add to impl PluginContext

// ─── Route Registration ───────────────────────────────────────

/// Register an HTTP route for this plugin.
///
/// Routes are automatically prefixed: `/api/<plugin-name>/...`
///
/// Path parameters use `:name` syntax: `/quarantine/:id/review`
///
/// Returns error if route is duplicate within this plugin.
pub fn register_route(&mut self, spec: RouteSpec) -> Result<(), PluginError> {
    if self.pending_routes.iter().any(|r| r.method == spec.method && r.path == spec.path) {
        return Err(PluginError::DuplicateRoute(format!("{:?} {}", spec.method, spec.path)));
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api register_route`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-plugin-api/src/context.rs
git commit -m "feat(plugin-api): add route registration to PluginContext"
```

---

### Task 9: Bump API Version

**Files:**
- Modify: `vibes-plugin-api/src/lib.rs`

**Step 1: Update API version constant**

```rust
// vibes-plugin-api/src/lib.rs - update version
pub const API_VERSION: u32 = 2;
```

**Step 2: Update test**

```rust
// vibes-plugin-api/src/lib.rs - update test
#[test]
fn test_api_version_is_set() {
    assert_eq!(API_VERSION, 2);
}
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p vibes-plugin-api api_version`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-plugin-api/src/lib.rs
git commit -m "feat(plugin-api)!: bump API version to 2"
```

---

## Phase 4: vibes-core Registries

### Task 10: Add CommandRegistry

**Files:**
- Create: `vibes-core/src/plugins/commands.rs`
- Modify: `vibes-core/src/plugins/mod.rs`

**Step 1: Write the failing test**

```rust
// vibes-core/src/plugins/commands.rs
#[cfg(test)]
mod tests {
    use super::*;
    use vibes_plugin_api::CommandSpec;

    #[test]
    fn test_register_commands() {
        let mut registry = CommandRegistry::new();

        let commands = vec![
            CommandSpec {
                path: vec!["trust".into(), "levels".into()],
                description: "Show levels".into(),
                args: vec![],
            },
        ];

        registry.register("groove", commands);

        let found = registry.find(&["groove".into(), "trust".into(), "levels".into()]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().plugin_name, "groove");
    }

    #[test]
    fn test_check_conflict() {
        let mut registry = CommandRegistry::new();

        let commands = vec![
            CommandSpec {
                path: vec!["foo".into()],
                description: "Foo".into(),
                args: vec![],
            },
        ];

        registry.register("plugin-a", commands.clone());

        // Same command from different plugin should conflict
        let conflict = registry.check_conflict("plugin-b", &["foo".into()]);
        assert_eq!(conflict, Some("plugin-a"));
    }

    #[test]
    fn test_no_conflict_with_self() {
        let registry = CommandRegistry::new();

        // No conflict if plugin doesn't exist yet
        let conflict = registry.check_conflict("new-plugin", &["foo".into()]);
        assert!(conflict.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core commands`
Expected: FAIL - module not found

**Step 3: Write implementation**

```rust
// vibes-core/src/plugins/commands.rs
//! Command registry for plugin CLI commands

use std::collections::HashMap;
use vibes_plugin_api::CommandSpec;

/// Registry of all plugin commands
pub struct CommandRegistry {
    /// Map from full command path to registration info
    commands: HashMap<Vec<String>, RegisteredPluginCommand>,
}

/// A command registered by a plugin
pub struct RegisteredPluginCommand {
    /// Name of the plugin that owns this command
    pub plugin_name: String,
    /// Command specification
    pub spec: CommandSpec,
}

impl CommandRegistry {
    /// Create a new empty command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register commands for a plugin
    ///
    /// Commands are stored with full path including plugin name prefix
    pub fn register(&mut self, plugin_name: &str, commands: Vec<CommandSpec>) {
        for spec in commands {
            let mut full_path = vec![plugin_name.to_string()];
            full_path.extend(spec.path.clone());

            self.commands.insert(
                full_path,
                RegisteredPluginCommand {
                    plugin_name: plugin_name.to_string(),
                    spec,
                },
            );
        }
    }

    /// Check if a command path would conflict with existing registrations
    ///
    /// Returns the name of the plugin that owns the conflicting command, if any
    pub fn check_conflict(&self, plugin_name: &str, path: &[String]) -> Option<&str> {
        let mut full_path = vec![plugin_name.to_string()];
        full_path.extend(path.iter().cloned());

        self.commands
            .get(&full_path)
            .map(|c| c.plugin_name.as_str())
    }

    /// Find a command by its full path
    pub fn find(&self, path: &[String]) -> Option<&RegisteredPluginCommand> {
        self.commands.get(path)
    }

    /// Get all registered commands
    pub fn all_commands(&self) -> impl Iterator<Item = (&[String], &RegisteredPluginCommand)> {
        self.commands.iter().map(|(k, v)| (k.as_slice(), v))
    }

    /// Unregister all commands for a plugin
    pub fn unregister(&mut self, plugin_name: &str) {
        self.commands.retain(|_, v| v.plugin_name != plugin_name);
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_commands() {
        let mut registry = CommandRegistry::new();

        let commands = vec![
            CommandSpec {
                path: vec!["trust".into(), "levels".into()],
                description: "Show levels".into(),
                args: vec![],
            },
        ];

        registry.register("groove", commands);

        let found = registry.find(&["groove".into(), "trust".into(), "levels".into()]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().plugin_name, "groove");
    }

    #[test]
    fn test_check_conflict() {
        let mut registry = CommandRegistry::new();

        let commands = vec![
            CommandSpec {
                path: vec!["foo".into()],
                description: "Foo".into(),
                args: vec![],
            },
        ];

        registry.register("plugin-a", commands);

        let conflict = registry.check_conflict("plugin-b", &["foo".into()]);
        assert_eq!(conflict, Some("plugin-a"));
    }

    #[test]
    fn test_no_conflict_with_self() {
        let registry = CommandRegistry::new();

        let conflict = registry.check_conflict("new-plugin", &["foo".into()]);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_unregister() {
        let mut registry = CommandRegistry::new();

        registry.register("plugin-a", vec![
            CommandSpec {
                path: vec!["cmd".into()],
                description: "Cmd".into(),
                args: vec![],
            },
        ]);

        assert!(registry.find(&["plugin-a".into(), "cmd".into()]).is_some());

        registry.unregister("plugin-a");

        assert!(registry.find(&["plugin-a".into(), "cmd".into()]).is_none());
    }
}
```

**Step 4: Add module to mod.rs**

```rust
// vibes-core/src/plugins/mod.rs - add
pub mod commands;
pub use commands::CommandRegistry;
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p vibes-core commands`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-core/src/plugins/commands.rs vibes-core/src/plugins/mod.rs
git commit -m "feat(core): add CommandRegistry for plugin CLI commands"
```

---

### Task 11: Add RouteRegistry

**Files:**
- Create: `vibes-core/src/plugins/routes.rs`
- Modify: `vibes-core/src/plugins/mod.rs`

**Step 1: Write the failing test**

```rust
// vibes-core/src/plugins/routes.rs
#[cfg(test)]
mod tests {
    use super::*;
    use vibes_plugin_api::{HttpMethod, RouteSpec};

    #[test]
    fn test_register_routes() {
        let mut registry = RouteRegistry::new();

        let routes = vec![
            RouteSpec {
                method: HttpMethod::Get,
                path: "/policy".into(),
            },
        ];

        registry.register("groove", routes);

        let (route, params) = registry.match_route(HttpMethod::Get, "/api/groove/policy").unwrap();
        assert_eq!(route.plugin_name, "groove");
        assert!(params.is_empty());
    }

    #[test]
    fn test_path_parameter_extraction() {
        let mut registry = RouteRegistry::new();

        registry.register("groove", vec![
            RouteSpec {
                method: HttpMethod::Get,
                path: "/quarantine/:id".into(),
            },
        ]);

        let (route, params) = registry.match_route(HttpMethod::Get, "/api/groove/quarantine/123").unwrap();
        assert_eq!(route.plugin_name, "groove");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_no_match_wrong_method() {
        let mut registry = RouteRegistry::new();

        registry.register("groove", vec![
            RouteSpec {
                method: HttpMethod::Get,
                path: "/policy".into(),
            },
        ]);

        let result = registry.match_route(HttpMethod::Post, "/api/groove/policy");
        assert!(result.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core routes`
Expected: FAIL - module not found

**Step 3: Write implementation**

```rust
// vibes-core/src/plugins/routes.rs
//! Route registry for plugin HTTP routes

use std::collections::HashMap;
use vibes_plugin_api::{HttpMethod, RouteSpec};

/// Registry of all plugin HTTP routes
pub struct RouteRegistry {
    /// Registered routes with compiled path matchers
    routes: Vec<RegisteredPluginRoute>,
}

/// A route registered by a plugin
pub struct RegisteredPluginRoute {
    /// Name of the plugin that owns this route
    pub plugin_name: String,
    /// Route specification
    pub spec: RouteSpec,
    /// Full path including /api/<plugin>/ prefix
    pub full_path: String,
    /// Compiled path matcher
    matcher: PathMatcher,
}

/// Simple path matcher supporting :param patterns
struct PathMatcher {
    segments: Vec<PathSegment>,
}

enum PathSegment {
    Literal(String),
    Param(String),
}

impl PathMatcher {
    fn new(path: &str) -> Self {
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if let Some(name) = s.strip_prefix(':') {
                    PathSegment::Param(name.to_string())
                } else {
                    PathSegment::Literal(s.to_string())
                }
            })
            .collect();

        Self { segments }
    }

    fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if path_parts.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (segment, part) in self.segments.iter().zip(path_parts.iter()) {
            match segment {
                PathSegment::Literal(expected) => {
                    if expected != *part {
                        return None;
                    }
                }
                PathSegment::Param(name) => {
                    params.insert(name.clone(), (*part).to_string());
                }
            }
        }

        Some(params)
    }
}

impl RouteRegistry {
    /// Create a new empty route registry
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register routes for a plugin
    ///
    /// Routes are prefixed with /api/<plugin>/
    pub fn register(&mut self, plugin_name: &str, routes: Vec<RouteSpec>) {
        for spec in routes {
            let full_path = format!("/api/{}{}", plugin_name, spec.path);
            let matcher = PathMatcher::new(&full_path);

            self.routes.push(RegisteredPluginRoute {
                plugin_name: plugin_name.to_string(),
                spec,
                full_path,
                matcher,
            });
        }
    }

    /// Check if a route would conflict with existing registrations
    ///
    /// Returns the name of the plugin that owns the conflicting route, if any
    pub fn check_conflict(&self, plugin_name: &str, spec: &RouteSpec) -> Option<&str> {
        let full_path = format!("/api/{}{}", plugin_name, spec.path);

        self.routes
            .iter()
            .find(|r| r.spec.method == spec.method && r.full_path == full_path)
            .map(|r| r.plugin_name.as_str())
    }

    /// Find a route matching the given method and path
    ///
    /// Returns the route and extracted path parameters
    pub fn match_route(
        &self,
        method: HttpMethod,
        path: &str,
    ) -> Option<(&RegisteredPluginRoute, HashMap<String, String>)> {
        for route in &self.routes {
            if route.spec.method == method {
                if let Some(params) = route.matcher.match_path(path) {
                    return Some((route, params));
                }
            }
        }
        None
    }

    /// Unregister all routes for a plugin
    pub fn unregister(&mut self, plugin_name: &str) {
        self.routes.retain(|r| r.plugin_name != plugin_name);
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_routes() {
        let mut registry = RouteRegistry::new();

        let routes = vec![
            RouteSpec {
                method: HttpMethod::Get,
                path: "/policy".into(),
            },
        ];

        registry.register("groove", routes);

        let (route, params) = registry.match_route(HttpMethod::Get, "/api/groove/policy").unwrap();
        assert_eq!(route.plugin_name, "groove");
        assert!(params.is_empty());
    }

    #[test]
    fn test_path_parameter_extraction() {
        let mut registry = RouteRegistry::new();

        registry.register("groove", vec![
            RouteSpec {
                method: HttpMethod::Get,
                path: "/quarantine/:id".into(),
            },
        ]);

        let (route, params) = registry.match_route(HttpMethod::Get, "/api/groove/quarantine/123").unwrap();
        assert_eq!(route.plugin_name, "groove");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_no_match_wrong_method() {
        let mut registry = RouteRegistry::new();

        registry.register("groove", vec![
            RouteSpec {
                method: HttpMethod::Get,
                path: "/policy".into(),
            },
        ]);

        let result = registry.match_route(HttpMethod::Post, "/api/groove/policy");
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_params() {
        let mut registry = RouteRegistry::new();

        registry.register("test", vec![
            RouteSpec {
                method: HttpMethod::Post,
                path: "/users/:user_id/items/:item_id".into(),
            },
        ]);

        let (_, params) = registry.match_route(HttpMethod::Post, "/api/test/users/alice/items/42").unwrap();
        assert_eq!(params.get("user_id"), Some(&"alice".to_string()));
        assert_eq!(params.get("item_id"), Some(&"42".to_string()));
    }

    #[test]
    fn test_unregister() {
        let mut registry = RouteRegistry::new();

        registry.register("plugin-a", vec![
            RouteSpec {
                method: HttpMethod::Get,
                path: "/route".into(),
            },
        ]);

        assert!(registry.match_route(HttpMethod::Get, "/api/plugin-a/route").is_some());

        registry.unregister("plugin-a");

        assert!(registry.match_route(HttpMethod::Get, "/api/plugin-a/route").is_none());
    }
}
```

**Step 4: Add module to mod.rs**

```rust
// vibes-core/src/plugins/mod.rs - add
pub mod routes;
pub use routes::RouteRegistry;
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p vibes-core routes`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-core/src/plugins/routes.rs vibes-core/src/plugins/mod.rs
git commit -m "feat(core): add RouteRegistry for plugin HTTP routes"
```

---

### Task 12: Add Conflict Error Types to PluginHostError

**Files:**
- Modify: `vibes-core/src/plugins/error.rs`

**Step 1: Write the failing test**

```rust
// Add to vibes-core/src/plugins/error.rs tests
#[test]
fn test_command_conflict_error() {
    let err = PluginHostError::CommandConflict {
        command: "trust levels".into(),
        existing_plugin: "groove".into(),
        new_plugin: "other".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("trust levels"));
    assert!(msg.contains("groove"));
}

#[test]
fn test_route_conflict_error() {
    let err = PluginHostError::RouteConflict {
        route: "GET /api/groove/policy".into(),
        existing_plugin: "groove".into(),
        new_plugin: "other".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("/api/groove/policy"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core conflict_error`
Expected: FAIL - CommandConflict variant not found

**Step 3: Add new error variants**

```rust
// vibes-core/src/plugins/error.rs - add to PluginHostError enum

#[error("Command conflict: '{command}' already registered by plugin '{existing_plugin}', cannot register for '{new_plugin}'")]
CommandConflict {
    command: String,
    existing_plugin: String,
    new_plugin: String,
},

#[error("Route conflict: '{route}' already registered by plugin '{existing_plugin}', cannot register for '{new_plugin}'")]
RouteConflict {
    route: String,
    existing_plugin: String,
    new_plugin: String,
},
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core conflict_error`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/plugins/error.rs
git commit -m "feat(core): add command/route conflict errors"
```

---

## Phase 5: PluginHost Integration

### Task 13: Add Registries to PluginHost

**Files:**
- Modify: `vibes-core/src/plugins/host.rs`

**Step 1: Add registry fields to PluginHost**

```rust
// vibes-core/src/plugins/host.rs - add to PluginHost struct
use super::commands::CommandRegistry;
use super::routes::RouteRegistry;

pub struct PluginHost {
    // ... existing fields ...
    command_registry: CommandRegistry,
    route_registry: RouteRegistry,
}
```

**Step 2: Initialize registries in new()**

```rust
// vibes-core/src/plugins/host.rs - update new() to initialize registries
impl PluginHost {
    pub fn new(config: PluginHostConfig) -> Self {
        Self {
            // ... existing initialization ...
            command_registry: CommandRegistry::new(),
            route_registry: RouteRegistry::new(),
        }
    }
}
```

**Step 3: Add accessor methods**

```rust
// vibes-core/src/plugins/host.rs - add to impl PluginHost

/// Get read access to the command registry
pub fn command_registry(&self) -> &CommandRegistry {
    &self.command_registry
}

/// Get read access to the route registry
pub fn route_registry(&self) -> &RouteRegistry {
    &self.route_registry
}
```

**Step 4: Run tests to verify nothing broke**

Run: `cargo test -p vibes-core plugins`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/plugins/host.rs
git commit -m "feat(core): add command/route registries to PluginHost"
```

---

### Task 14: Validate Registrations on Plugin Load

**Files:**
- Modify: `vibes-core/src/plugins/host.rs`

**Step 1: Update load_plugin to validate and register commands/routes**

```rust
// vibes-core/src/plugins/host.rs - update load_plugin method

fn load_plugin(&mut self, dir: &Path) -> Result<LoadedPlugin, PluginHostError> {
    // ... existing library loading code ...

    // Call on_load with context
    let mut ctx = self.create_context(&manifest.name, dir);
    instance.on_load(&mut ctx).map_err(PluginHostError::InitFailed)?;

    // Validate command registrations
    for spec in ctx.pending_commands() {
        if let Some(existing) = self.command_registry.check_conflict(&manifest.name, &spec.path) {
            return Err(PluginHostError::CommandConflict {
                command: spec.path.join(" "),
                existing_plugin: existing.to_string(),
                new_plugin: manifest.name.clone(),
            });
        }
    }

    // Validate route registrations
    for spec in ctx.pending_routes() {
        if let Some(existing) = self.route_registry.check_conflict(&manifest.name, spec) {
            return Err(PluginHostError::RouteConflict {
                route: format!("{:?} {}", spec.method, spec.path),
                existing_plugin: existing.to_string(),
                new_plugin: manifest.name.clone(),
            });
        }
    }

    // Commit registrations
    let commands = ctx.take_pending_commands();
    let routes = ctx.take_pending_routes();

    self.command_registry.register(&manifest.name, commands);
    self.route_registry.register(&manifest.name, routes);

    Ok(LoadedPlugin {
        manifest,
        instance,
        library,
        state: PluginState::Loaded,
    })
}
```

**Step 2: Update unload_plugin to clean up registrations**

```rust
// vibes-core/src/plugins/host.rs - add/update unload method

pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginHostError> {
    // ... existing unload logic ...

    // Clean up registrations
    self.command_registry.unregister(name);
    self.route_registry.unregister(name);

    // ... rest of unload logic ...
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-core plugins`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-core/src/plugins/host.rs
git commit -m "feat(core): validate and commit plugin registrations on load"
```

---

### Task 15: Add Command Dispatch to PluginHost

**Files:**
- Modify: `vibes-core/src/plugins/host.rs`

**Step 1: Write the failing test**

```rust
// Add to vibes-core/src/plugins/host.rs tests
#[test]
fn test_dispatch_command() {
    // This test requires a mock plugin - we'll test via integration test
}
```

**Step 2: Add dispatch method**

```rust
// vibes-core/src/plugins/host.rs - add to impl PluginHost

/// Dispatch a CLI command to the appropriate plugin
pub fn dispatch_command(
    &mut self,
    plugin_name: &str,
    path: &[&str],
    args: &vibes_plugin_api::CommandArgs,
) -> Result<vibes_plugin_api::CommandOutput, PluginHostError> {
    let plugin = self.plugins.get_mut(plugin_name)
        .ok_or_else(|| PluginHostError::NotFound { name: plugin_name.to_string() })?;

    if plugin.state != PluginState::Loaded {
        return Err(PluginHostError::NotFound { name: plugin_name.to_string() });
    }

    let mut ctx = self.create_context(&plugin.manifest.name, &self.get_plugin_dir(plugin_name)?);

    plugin.instance
        .handle_command(path, args, &mut ctx)
        .map_err(PluginHostError::InitFailed)
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-core dispatch`
Expected: PASS (or skipped if mock required)

**Step 4: Commit**

```bash
git add vibes-core/src/plugins/host.rs
git commit -m "feat(core): add command dispatch to PluginHost"
```

---

### Task 16: Add Route Dispatch to PluginHost

**Files:**
- Modify: `vibes-core/src/plugins/host.rs`

**Step 1: Add dispatch method**

```rust
// vibes-core/src/plugins/host.rs - add to impl PluginHost

/// Dispatch an HTTP route to the appropriate plugin
pub fn dispatch_route(
    &mut self,
    plugin_name: &str,
    method: vibes_plugin_api::HttpMethod,
    path: &str,
    request: vibes_plugin_api::RouteRequest,
) -> Result<vibes_plugin_api::RouteResponse, PluginHostError> {
    let plugin = self.plugins.get_mut(plugin_name)
        .ok_or_else(|| PluginHostError::NotFound { name: plugin_name.to_string() })?;

    if plugin.state != PluginState::Loaded {
        return Err(PluginHostError::NotFound { name: plugin_name.to_string() });
    }

    let mut ctx = self.create_context(&plugin.manifest.name, &self.get_plugin_dir(plugin_name)?);

    plugin.instance
        .handle_route(method, path, request, &mut ctx)
        .map_err(PluginHostError::InitFailed)
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core dispatch`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-core/src/plugins/host.rs
git commit -m "feat(core): add route dispatch to PluginHost"
```

---

## Phase 6: CLI Integration

### Task 17: Add Plugin Command Dispatch to vibes-cli

**Files:**
- Create: `vibes-cli/src/commands/plugin_dispatch.rs`
- Modify: `vibes-cli/src/commands/mod.rs`
- Modify: `vibes-cli/src/main.rs`

**Step 1: Create plugin dispatch module**

```rust
// vibes-cli/src/commands/plugin_dispatch.rs
//! Dispatch commands to plugins

use anyhow::{anyhow, Result};
use vibes_core::plugins::PluginHost;
use vibes_plugin_api::{CommandArgs, CommandOutput};

/// Dispatch a command to a plugin
pub fn dispatch(
    plugin_host: &mut PluginHost,
    path: &[String],
    positional: Vec<String>,
    flags: std::collections::HashMap<String, String>,
) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("No command specified"));
    }

    let plugin_name = &path[0];
    let cmd_path: Vec<&str> = path[1..].iter().map(|s| s.as_str()).collect();

    let args = CommandArgs {
        args: positional,
        flags,
    };

    let output = plugin_host.dispatch_command(plugin_name, &cmd_path, &args)?;

    render_output(output);

    Ok(())
}

fn render_output(output: CommandOutput) {
    match output {
        CommandOutput::Text(text) => println!("{}", text),
        CommandOutput::Table { headers, rows } => {
            // Simple table rendering
            println!("{}", headers.join("\t"));
            for row in rows {
                println!("{}", row.join("\t"));
            }
        }
        CommandOutput::Success => {}
        CommandOutput::Exit(code) => std::process::exit(code),
    }
}
```

**Step 2: Add module to mod.rs**

```rust
// vibes-cli/src/commands/mod.rs - add
pub mod plugin_dispatch;
```

**Step 3: Update main.rs to handle plugin commands**

This requires modifying the CLI to detect unknown subcommands and route them to plugins. The exact implementation depends on the current CLI structure.

**Step 4: Run tests**

Run: `cargo build -p vibes-cli`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-cli/src/commands/plugin_dispatch.rs vibes-cli/src/commands/mod.rs
git commit -m "feat(cli): add plugin command dispatch"
```

---

## Phase 7: Server Integration

### Task 18: Add Plugin Route Handler to vibes-server

**Files:**
- Create: `vibes-server/src/http/plugins.rs`
- Modify: `vibes-server/src/http/mod.rs`

**Step 1: Create plugin route handler**

```rust
// vibes-server/src/http/plugins.rs
//! Plugin HTTP route handler

use std::sync::Arc;
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use vibes_plugin_api::{HttpMethod, RouteRequest};
use std::collections::HashMap;

use crate::AppState;

/// Convert axum Method to plugin HttpMethod
fn to_http_method(method: &axum::http::Method) -> Option<HttpMethod> {
    match *method {
        axum::http::Method::GET => Some(HttpMethod::Get),
        axum::http::Method::POST => Some(HttpMethod::Post),
        axum::http::Method::PUT => Some(HttpMethod::Put),
        axum::http::Method::DELETE => Some(HttpMethod::Delete),
        axum::http::Method::PATCH => Some(HttpMethod::Patch),
        _ => None,
    }
}

/// Plugin route handler
pub async fn handle_plugin_route(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Response {
    let method = match to_http_method(request.method()) {
        Some(m) => m,
        None => {
            return (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed").into_response();
        }
    };

    let path = request.uri().path().to_string();
    let query = parse_query(request.uri().query());
    let headers = extract_headers(request.headers());

    // Get plugin host and find matching route
    let plugin_host = state.plugin_host().read().await;
    let Some((route, params)) = plugin_host.route_registry().match_route(method, &path) else {
        return (StatusCode::NOT_FOUND, r#"{"error":"Not found"}"#).into_response();
    };

    let plugin_name = route.plugin_name.clone();
    let route_path = route.spec.path.clone();
    drop(plugin_host);

    // Extract body
    let body = match axum::body::to_bytes(request.into_body(), 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Failed to read body").into_response();
        }
    };

    let route_request = RouteRequest {
        params,
        query,
        body,
        headers,
    };

    // Dispatch to plugin
    let mut plugin_host = state.plugin_host().write().await;
    match plugin_host.dispatch_route(&plugin_name, method, &route_path, route_request) {
        Ok(resp) => Response::builder()
            .status(resp.status)
            .header("Content-Type", resp.content_type)
            .body(Body::from(resp.body))
            .unwrap(),
        Err(e) => Response::builder()
            .status(500)
            .header("Content-Type", "application/json")
            .body(Body::from(format!(r#"{{"error":"{}"}}"#, e)))
            .unwrap(),
    }
}

fn parse_query(query: Option<&str>) -> HashMap<String, String> {
    query
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_headers(headers: &axum::http::HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str().ok().map(|val| (k.to_string(), val.to_string()))
        })
        .collect()
}

/// Create router for plugin routes
pub fn plugin_router() -> Router<Arc<AppState>> {
    Router::new()
        .fallback(handle_plugin_route)
}
```

**Step 2: Update mod.rs to include plugin routes**

```rust
// vibes-server/src/http/mod.rs - add
pub mod plugins;
```

**Step 3: Mount plugin routes in main router**

The plugin routes should be mounted as a fallback or nested under `/api/`.

**Step 4: Run tests**

Run: `cargo build -p vibes-server`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/http/plugins.rs vibes-server/src/http/mod.rs
git commit -m "feat(server): add plugin HTTP route handler"
```

---

## Phase 8: Groove Migration

### Task 19: Implement Plugin Trait for GroovePlugin

**Files:**
- Create: `vibes-groove/src/plugin.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Create plugin implementation**

```rust
// vibes-groove/src/plugin.rs
//! Groove plugin implementation

use vibes_plugin_api::{
    ArgSpec, CommandArgs, CommandOutput, CommandSpec, HttpMethod,
    Plugin, PluginContext, PluginError, PluginManifest, RouteRequest, RouteResponse, RouteSpec,
};

use crate::security::{OrgRole, TrustLevel, load_policy_or_default};

/// The groove continual learning plugin
#[derive(Default)]
pub struct GroovePlugin;

impl Plugin for GroovePlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "groove".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Continual learning system".to_string(),
            ..Default::default()
        }
    }

    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        // Register CLI commands
        ctx.register_command(CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust level hierarchy".into(),
            args: vec![],
        })?;
        ctx.register_command(CommandSpec {
            path: vec!["trust".into(), "role".into()],
            description: "Show permissions for a role".into(),
            args: vec![ArgSpec {
                name: "role".into(),
                description: "Role name (admin, curator, member, viewer)".into(),
                required: true,
            }],
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

        // Register HTTP routes
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/policy".into(),
        })?;
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/trust/levels".into(),
        })?;
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/trust/role/:role".into(),
        })?;
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/quarantine".into(),
        })?;
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/quarantine/stats".into(),
        })?;
        ctx.register_route(RouteSpec {
            method: HttpMethod::Post,
            path: "/quarantine/:id/review".into(),
        })?;

        ctx.log_info("groove plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn handle_command(
        &mut self,
        path: &[&str],
        args: &CommandArgs,
        _ctx: &mut PluginContext,
    ) -> Result<CommandOutput, PluginError> {
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

    fn handle_route(
        &mut self,
        method: HttpMethod,
        path: &str,
        request: RouteRequest,
        _ctx: &mut PluginContext,
    ) -> Result<RouteResponse, PluginError> {
        match (method, path) {
            (HttpMethod::Get, "/policy") => self.route_get_policy(),
            (HttpMethod::Get, "/trust/levels") => self.route_get_trust_levels(),
            (HttpMethod::Get, "/trust/role/:role") => {
                let role = request.params.get("role")
                    .ok_or_else(|| PluginError::InvalidInput("missing role parameter".into()))?;
                self.route_get_role_permissions(role)
            }
            (HttpMethod::Get, "/quarantine") => self.route_list_quarantined(),
            (HttpMethod::Get, "/quarantine/stats") => self.route_get_quarantine_stats(),
            (HttpMethod::Post, "/quarantine/:id/review") => {
                let id = request.params.get("id")
                    .ok_or_else(|| PluginError::InvalidInput("missing id parameter".into()))?;
                self.route_review_quarantined(id, &request.body)
            }
            _ => Err(PluginError::UnknownRoute(format!("{:?} {}", method, path))),
        }
    }
}

// Command implementations
impl GroovePlugin {
    fn cmd_trust_levels(&self) -> Result<CommandOutput, PluginError> {
        let text = format!(
            "Trust Level Hierarchy (highest to lowest):\n\n\
             {:<24} {:>6}  Description\n\
             {:24} {:>6}  {}\n\
             {:<24} {:>6}  Locally created content (full trust)\n\
             {:<24} {:>6}  Synced from user's own cloud\n\
             {:<24} {:>6}  Enterprise content, curator approved\n\
             {:<24} {:>6}  Enterprise content, not yet approved\n\
             {:<24} {:>6}  Community content, verified\n\
             {:<24} {:>6}  Community content, unverified\n\
             {:<24} {:>6}  Quarantined (blocked)",
            "Level", "Score",
            "─".repeat(24), "─".repeat(6), "─".repeat(36),
            "Local", TrustLevel::Local as u8,
            "PrivateCloud", TrustLevel::PrivateCloud as u8,
            "OrganizationVerified", TrustLevel::OrganizationVerified as u8,
            "OrganizationUnverified", TrustLevel::OrganizationUnverified as u8,
            "PublicVerified", TrustLevel::PublicVerified as u8,
            "PublicUnverified", TrustLevel::PublicUnverified as u8,
            "Quarantined", TrustLevel::Quarantined as u8,
        );
        Ok(CommandOutput::Text(text))
    }

    fn cmd_trust_role(&self, args: &CommandArgs) -> Result<CommandOutput, PluginError> {
        let role_str = args.args.first()
            .ok_or_else(|| PluginError::InvalidInput("role argument required".into()))?;

        let role: OrgRole = role_str.parse()
            .map_err(|_| PluginError::InvalidInput(
                format!("Invalid role: {}. Use: admin, curator, member, viewer", role_str)
            ))?;

        let perms = role.permissions();
        let text = format!(
            "Role: {}\n\n\
             Permissions:\n\
               Create:   {}\n\
               Read:     {}\n\
               Modify:   {}\n\
               Delete:   {}\n\
               Publish:  {}\n\
               Review:   {}\n\
               Admin:    {}",
            role.as_str(),
            if perms.can_create { "✓" } else { "✗" },
            if perms.can_read { "✓" } else { "✗" },
            if perms.can_modify { "✓" } else { "✗" },
            if perms.can_delete { "✓" } else { "✗" },
            if perms.can_publish { "✓" } else { "✗" },
            if perms.can_review { "✓" } else { "✗" },
            if perms.can_admin { "✓" } else { "✗" },
        );
        Ok(CommandOutput::Text(text))
    }

    fn cmd_policy_show(&self) -> Result<CommandOutput, PluginError> {
        let policy = load_policy_or_default("groove-policy.toml");
        let text = format!(
            "Current Security Policy:\n\n\
             Injection Policy:\n\
               Block quarantined:       {}\n\
               Allow personal:          {}\n\
               Allow unverified:        {}\n\n\
             Quarantine Policy:\n\
               Reviewers:               {:?}\n\
               Visible to:              {:?}\n\
               Auto-delete after days:  {:?}\n\n\
             Import/Export Policy:\n\
               Allow import from file:  {}\n\
               Allow import from URL:   {}\n\
               Allowed import sources:  {:?}\n\
               Allow export personal:   {}\n\
               Allow export enterprise: {}\n\n\
             Audit Policy:\n\
               Enabled:                 {}\n\
               Retention days:          {:?}",
            policy.injection.block_quarantined,
            policy.injection.allow_personal_injection,
            policy.injection.allow_unverified_injection,
            policy.quarantine.reviewers,
            policy.quarantine.visible_to,
            policy.quarantine.auto_delete_after_days,
            policy.import_export.allow_import_from_file,
            policy.import_export.allow_import_from_url,
            policy.import_export.allowed_import_sources,
            policy.import_export.allow_export_personal,
            policy.import_export.allow_export_enterprise,
            policy.audit.enabled,
            policy.audit.retention_days,
        );
        Ok(CommandOutput::Text(text))
    }

    fn cmd_policy_path(&self) -> Result<CommandOutput, PluginError> {
        let text = "Policy search paths:\n\
                    1. ./groove-policy.toml\n\
                    2. ~/.config/vibes/groove-policy.toml\n\
                    3. /etc/vibes/groove-policy.toml\n\n\
                    If no policy file is found, defaults are used.";
        Ok(CommandOutput::Text(text.to_string()))
    }

    fn cmd_quarantine_list(&self) -> Result<CommandOutput, PluginError> {
        Ok(CommandOutput::Text(
            "Quarantine queue listing not yet implemented.\n\
             This will show learnings pending review.".to_string()
        ))
    }

    fn cmd_quarantine_stats(&self) -> Result<CommandOutput, PluginError> {
        Ok(CommandOutput::Text(
            "Quarantine statistics not yet implemented.\n\
             This will show quarantine queue metrics.".to_string()
        ))
    }
}

// Route implementations (reuse logic from vibes-server/src/http/groove.rs)
impl GroovePlugin {
    fn route_get_policy(&self) -> Result<RouteResponse, PluginError> {
        // Implementation migrated from vibes-server
        todo!("Migrate from vibes-server/src/http/groove.rs")
    }

    fn route_get_trust_levels(&self) -> Result<RouteResponse, PluginError> {
        todo!("Migrate from vibes-server/src/http/groove.rs")
    }

    fn route_get_role_permissions(&self, _role: &str) -> Result<RouteResponse, PluginError> {
        todo!("Migrate from vibes-server/src/http/groove.rs")
    }

    fn route_list_quarantined(&self) -> Result<RouteResponse, PluginError> {
        todo!("Migrate from vibes-server/src/http/groove.rs")
    }

    fn route_get_quarantine_stats(&self) -> Result<RouteResponse, PluginError> {
        todo!("Migrate from vibes-server/src/http/groove.rs")
    }

    fn route_review_quarantined(&self, _id: &str, _body: &[u8]) -> Result<RouteResponse, PluginError> {
        todo!("Migrate from vibes-server/src/http/groove.rs")
    }
}

vibes_plugin_api::export_plugin!(GroovePlugin);
```

**Step 2: Add module to lib.rs**

```rust
// vibes-groove/src/lib.rs - add
pub mod plugin;
pub use plugin::GroovePlugin;
```

**Step 3: Complete route implementations**

Migrate the actual route handler logic from `vibes-server/src/http/groove.rs` to the `GroovePlugin` route methods.

**Step 4: Run tests**

Run: `cargo build -p vibes-groove`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-groove/src/plugin.rs vibes-groove/src/lib.rs
git commit -m "feat(groove): implement Plugin trait with commands and routes"
```

---

## Phase 9: Cleanup

### Task 20: Remove Deprecated Code

**Files:**
- Delete: `vibes-cli/src/commands/groove.rs`
- Delete: `vibes-server/src/http/groove.rs`
- Modify: `vibes-cli/src/commands/mod.rs`
- Modify: `vibes-cli/src/main.rs`
- Modify: `vibes-server/src/http/mod.rs`
- Modify: `vibes-plugin-api/src/context.rs`

**Step 1: Remove groove command from CLI**

```rust
// vibes-cli/src/main.rs - remove Commands::Groove variant
// vibes-cli/src/commands/mod.rs - remove pub mod groove;
```

**Step 2: Delete groove CLI file**

```bash
rm vibes-cli/src/commands/groove.rs
```

**Step 3: Remove groove routes from server**

```rust
// vibes-server/src/http/mod.rs - remove groove module and routes
```

**Step 4: Delete groove HTTP file**

```bash
rm vibes-server/src/http/groove.rs
```

**Step 5: Remove deprecated RegisteredCommand from plugin-api**

```rust
// vibes-plugin-api/src/context.rs - remove:
// - registered_commands: Vec<RegisteredCommand> field
// - RegisteredCommand struct
// - register_command (old version that just stores name)
// - registered_commands() method
```

**Step 6: Run tests to ensure nothing broke**

Run: `cargo test --workspace`
Expected: PASS

**Step 7: Commit**

```bash
git add -A
git commit -m "chore: remove deprecated groove CLI/server code and old RegisteredCommand"
```

---

### Task 21: Update Documentation

**Files:**
- Modify: `docs/PROGRESS.md`
- Modify: `docs/PLAN.md` (if needed)

**Step 1: Mark milestone 4.2.6 complete in PROGRESS.md**

**Step 2: Add changelog entry**

**Step 3: Commit**

```bash
git add docs/
git commit -m "docs: mark milestone 4.2.6 complete, update progress"
```

---

## Final Verification

### Task 22: Integration Test

**Step 1: Build everything**

Run: `just build`
Expected: PASS

**Step 2: Run all tests**

Run: `just test`
Expected: PASS

**Step 3: Manual verification**

```bash
# Test groove CLI commands work via plugin
vibes groove trust levels
vibes groove policy show

# Test groove HTTP routes work via plugin
curl http://localhost:8080/api/groove/policy
curl http://localhost:8080/api/groove/trust/levels
```

**Step 4: Run pre-commit checks**

Run: `just pre-commit`
Expected: PASS

**Step 5: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: address integration test issues"
```

---

## Summary

This plan implements milestone 4.2.6 in 22 tasks across 9 phases:

1. **Phase 1**: Add plugin API types (HttpMethod, commands, routes)
2. **Phase 2**: Extend Plugin trait with handle_command/handle_route
3. **Phase 3**: Add registration methods to PluginContext
4. **Phase 4**: Add CommandRegistry and RouteRegistry to vibes-core
5. **Phase 5**: Integrate registries with PluginHost
6. **Phase 6**: Add plugin command dispatch to vibes-cli
7. **Phase 7**: Add plugin route handler to vibes-server
8. **Phase 8**: Migrate groove to use the plugin system
9. **Phase 9**: Cleanup deprecated code and update docs

Each task follows TDD: write failing test → run to verify failure → implement → verify pass → commit.
