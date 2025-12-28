# Harness Introspection Design

## Overview

Milestone 4.1 introduces harness introspection - the ability to discover what capabilities an AI coding assistant provides for capture and injection. This is **Level 0** of the continual learning system: before we can learn anything, we must know what the harness supports.

## Goals

1. **Discover capabilities** - Detect hooks, config files, injection targets
2. **Cross-platform** - Work on Linux, macOS, and Windows
3. **Multi-scope** - Detect system, user, and project-level configurations
4. **Harness-agnostic** - Trait-based design for future harness support
5. **Reactive** - File watchers detect config changes in real-time

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Crate | New `vibes-introspection` | Focused scope, reusable without learning machinery |
| Scope hierarchy | System → User → Project | Matches enterprise patterns (IT policy → user → project) |
| Platform paths | `dirs` crate | Standard cross-platform path resolution |
| Detection timing | Startup + file watchers | Accurate without per-session overhead |
| File watching | Recursive + 500ms debounce | Fewer file handles, absorbs write bursts |
| Caching | In-memory only | Defer persistence to 4.2 (CozoDB) |
| Generic harness | Deferred | YAGNI - implement when second harness needed |

## Architecture

```
vibes-introspection/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public exports
│   ├── error.rs            # IntrospectionError
│   ├── harness.rs          # Harness trait
│   ├── capabilities.rs     # Capability types (3-tier)
│   ├── paths.rs            # Cross-platform config paths
│   ├── watcher.rs          # File watcher with debounce
│   └── claude_code/        # Claude Code implementation
│       ├── mod.rs
│       ├── harness.rs      # ClaudeCodeHarness
│       └── detection.rs    # Capability detection logic
```

## Config Path Hierarchy

| Level | Linux/macOS | Windows | Purpose |
|-------|-------------|---------|---------|
| System | `/etc/claude/` | `%PROGRAMDATA%\claude` | Org-wide defaults, IT policies |
| User | `~/.claude/` | `%APPDATA%\claude` | User preferences |
| Project | `./.claude/` | `.\.claude\` | Project-specific config |

Precedence: Project overrides User overrides System.

## Capabilities Detected

| Capability | Detection Method | Purpose |
|------------|------------------|---------|
| Hooks support | Check `hooks/` directory exists | Capture tool events |
| Hook types | Parse `settings.json` hooks config | Know observable events |
| Transcript access | Stop hook provides path | Post-session analysis |
| CLAUDE.md | Check file exists + writable | Inject learnings |
| settings.json | Check file exists + writable | Configure behavior |
| config.json | Check file exists | Additional config |
| Projects dir | Check `projects/` exists | Project-scoped learnings |
| MCP servers | Parse settings for mcp_servers | Know available tools |
| Version | Run `claude --version` | Compatibility checks |

---

## Core Types

### Config Paths

```rust
/// Platform-aware config path resolution
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub system: Option<PathBuf>,   // /etc/claude, %PROGRAMDATA%\claude
    pub user: PathBuf,             // ~/.claude, %APPDATA%\claude
    pub project: Option<PathBuf>,  // .claude/ in project root
}

impl ConfigPaths {
    /// Resolve paths for a given harness type
    pub fn resolve(harness: &str, project_root: Option<&Path>) -> Result<Self> {
        let user = Self::user_config_dir(harness)?;
        let system = Self::system_config_dir(harness);
        let project = project_root.map(|p| p.join(format!(".{}", harness)));

        Ok(Self { system, user, project })
    }

    #[cfg(windows)]
    fn user_config_dir(harness: &str) -> Result<PathBuf> {
        dirs::config_dir()
            .map(|d| d.join(harness))
            .ok_or(IntrospectionError::NoHomeDir)
    }

    #[cfg(not(windows))]
    fn user_config_dir(harness: &str) -> Result<PathBuf> {
        dirs::home_dir()
            .map(|d| d.join(format!(".{}", harness)))
            .ok_or(IntrospectionError::NoHomeDir)
    }

    #[cfg(target_os = "linux")]
    fn system_config_dir(harness: &str) -> Option<PathBuf> {
        let path = PathBuf::from(format!("/etc/{}", harness));
        path.exists().then_some(path)
    }

    #[cfg(target_os = "windows")]
    fn system_config_dir(harness: &str) -> Option<PathBuf> {
        std::env::var("PROGRAMDATA").ok()
            .map(|d| PathBuf::from(d).join(harness))
            .filter(|p| p.exists())
    }

    #[cfg(target_os = "macos")]
    fn system_config_dir(harness: &str) -> Option<PathBuf> {
        let path = PathBuf::from(format!("/Library/Application Support/{}", harness));
        path.exists().then_some(path)
    }
}
```

### Capabilities

```rust
/// What we detected at each scope level
#[derive(Debug, Clone, Default)]
pub struct ScopedCapabilities {
    pub hooks: Option<HookCapabilities>,
    pub config_files: Vec<ConfigFile>,
    pub injection_targets: Vec<InjectionTarget>,
}

/// Full harness capabilities across all scopes
#[derive(Debug, Clone)]
pub struct HarnessCapabilities {
    pub harness_type: String,
    pub version: Option<String>,
    pub system: Option<ScopedCapabilities>,
    pub user: ScopedCapabilities,
    pub project: Option<ScopedCapabilities>,
}

impl HarnessCapabilities {
    /// Get effective hooks (project → user → system precedence)
    pub fn effective_hooks(&self) -> Option<&HookCapabilities> {
        self.project.as_ref().and_then(|p| p.hooks.as_ref())
            .or(self.user.hooks.as_ref())
            .or(self.system.as_ref().and_then(|s| s.hooks.as_ref()))
    }

    /// Get all injection targets across scopes
    pub fn effective_injection_targets(&self) -> Vec<&InjectionTarget> {
        let mut targets = Vec::new();
        if let Some(sys) = &self.system {
            targets.extend(sys.injection_targets.iter());
        }
        targets.extend(self.user.injection_targets.iter());
        if let Some(proj) = &self.project {
            targets.extend(proj.injection_targets.iter());
        }
        targets
    }
}

/// Hook system capabilities
#[derive(Debug, Clone)]
pub struct HookCapabilities {
    pub supported_types: Vec<HookType>,
    pub hooks_dir: PathBuf,
    pub installed_hooks: Vec<InstalledHook>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    Stop,
    Notification,
}

#[derive(Debug, Clone)]
pub struct InstalledHook {
    pub hook_type: HookType,
    pub name: String,
    pub path: PathBuf,
}

/// A config file we detected
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub writable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
    Markdown,
}

/// A file we can inject learnings into
#[derive(Debug, Clone)]
pub struct InjectionTarget {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub writable: bool,
    pub scope: InjectionScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectionScope {
    System,
    User,
    Project,
}
```

---

## Harness Trait

```rust
use async_trait::async_trait;

/// Core trait - any AI coding assistant we can enhance
#[async_trait]
pub trait Harness: Send + Sync {
    /// Unique identifier (e.g., "claude-code", "cursor", "aider")
    fn harness_type(&self) -> &'static str;

    /// Detect version from binary or config
    async fn version(&self) -> Option<String>;

    /// Platform-appropriate config paths
    fn config_paths(&self, project_root: Option<&Path>) -> Result<ConfigPaths>;

    /// Full capability introspection
    async fn introspect(&self, project_root: Option<&Path>) -> Result<HarnessCapabilities>;
}

/// Create the appropriate harness from CLI subcommand
pub fn harness_for_command(command: &str) -> Option<Arc<dyn Harness>> {
    match command {
        "claude" => Some(Arc::new(ClaudeCodeHarness)),
        // Future: "cursor" => Some(Arc::new(CursorHarness)),
        _ => None,
    }
}
```

---

## Claude Code Harness

```rust
pub struct ClaudeCodeHarness;

#[async_trait]
impl Harness for ClaudeCodeHarness {
    fn harness_type(&self) -> &'static str {
        "claude"
    }

    async fn version(&self) -> Option<String> {
        use tokio::process::Command;

        let output = Command::new("claude")
            .arg("--version")
            .output()
            .await
            .ok()?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    fn config_paths(&self, project_root: Option<&Path>) -> Result<ConfigPaths> {
        ConfigPaths::resolve("claude", project_root)
    }

    async fn introspect(&self, project_root: Option<&Path>) -> Result<HarnessCapabilities> {
        let paths = self.config_paths(project_root)?;

        Ok(HarnessCapabilities {
            harness_type: self.harness_type().to_string(),
            version: self.version().await,
            system: self.detect_scope(paths.system.as_deref(), InjectionScope::System).await,
            user: self.detect_scope(Some(&paths.user), InjectionScope::User).await
                      .unwrap_or_default(),
            project: self.detect_scope(paths.project.as_deref(), InjectionScope::Project).await,
        })
    }
}

impl ClaudeCodeHarness {
    async fn detect_scope(
        &self,
        path: Option<&Path>,
        scope: InjectionScope,
    ) -> Option<ScopedCapabilities> {
        let path = path?;
        if !path.exists() {
            return None;
        }

        Some(ScopedCapabilities {
            hooks: self.detect_hooks(path).await,
            config_files: self.detect_config_files(path).await,
            injection_targets: self.detect_injection_targets(path, scope).await,
        })
    }

    async fn detect_hooks(&self, base: &Path) -> Option<HookCapabilities> {
        let hooks_dir = base.join("hooks");
        if !hooks_dir.exists() {
            return None;
        }

        // Parse settings.json for hook types
        let settings_path = base.join("settings.json");
        let supported_types = self.parse_hook_types(&settings_path).await;

        // Find installed hooks
        let installed_hooks = self.find_installed_hooks(&hooks_dir).await;

        Some(HookCapabilities {
            supported_types,
            hooks_dir,
            installed_hooks,
        })
    }

    async fn detect_config_files(&self, base: &Path) -> Vec<ConfigFile> {
        let mut files = Vec::new();

        for (name, format) in [
            ("settings.json", ConfigFormat::Json),
            ("config.json", ConfigFormat::Json),
            ("CLAUDE.md", ConfigFormat::Markdown),
        ] {
            let path = base.join(name);
            if path.exists() {
                let writable = Self::is_writable(&path).await;
                files.push(ConfigFile { path, format, writable });
            }
        }

        files
    }

    async fn detect_injection_targets(
        &self,
        base: &Path,
        scope: InjectionScope,
    ) -> Vec<InjectionTarget> {
        let mut targets = Vec::new();

        // CLAUDE.md is primary injection target
        let claude_md = base.join("CLAUDE.md");
        if claude_md.exists() || Self::is_writable(&base).await {
            targets.push(InjectionTarget {
                path: claude_md,
                format: ConfigFormat::Markdown,
                writable: Self::is_writable(&base).await,
                scope: scope.clone(),
            });
        }

        targets
    }

    async fn is_writable(path: &Path) -> bool {
        tokio::fs::metadata(path)
            .await
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false)
    }

    async fn parse_hook_types(&self, settings_path: &Path) -> Vec<HookType> {
        // Default supported types for Claude Code
        vec![
            HookType::PreToolUse,
            HookType::PostToolUse,
            HookType::Stop,
            HookType::Notification,
        ]
    }

    async fn find_installed_hooks(&self, hooks_dir: &Path) -> Vec<InstalledHook> {
        let mut hooks = Vec::new();

        if let Ok(mut entries) = tokio::fs::read_dir(hooks_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_dir() {
                    // Each subdirectory is a hook provider (e.g., "vibes")
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // TODO: Parse actual hook scripts to determine types
                        hooks.push(InstalledHook {
                            hook_type: HookType::Stop, // Placeholder
                            name: name.to_string(),
                            path,
                        });
                    }
                }
            }
        }

        hooks
    }
}
```

---

## File Watcher

```rust
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use std::time::Duration;

/// Watches config directories and triggers re-introspection on changes
pub struct CapabilityWatcher {
    harness: Arc<dyn Harness>,
    capabilities: Arc<RwLock<HarnessCapabilities>>,
    project_root: Option<PathBuf>,
    _watcher: RecommendedWatcher,
}

impl CapabilityWatcher {
    pub async fn new(
        harness: Arc<dyn Harness>,
        project_root: Option<PathBuf>,
        debounce_ms: u64,
    ) -> Result<Self> {
        // Initial introspection
        let caps = harness.introspect(project_root.as_deref()).await?;
        let capabilities = Arc::new(RwLock::new(caps));

        // Set up debounced watcher
        let (tx, rx) = mpsc::channel::<Event>(100);

        let tx_clone = tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let _ = tx_clone.blocking_send(event);
            }
        })?;

        // Watch all config directories
        let paths = harness.config_paths(project_root.as_deref())?;
        if let Some(ref sys) = paths.system {
            let _ = watcher.watch(sys, RecursiveMode::Recursive);
        }
        let _ = watcher.watch(&paths.user, RecursiveMode::Recursive);
        if let Some(ref proj) = paths.project {
            let _ = watcher.watch(proj, RecursiveMode::Recursive);
        }

        // Spawn debounce handler
        let caps_clone = capabilities.clone();
        let harness_clone = harness.clone();
        let project_clone = project_root.clone();
        tokio::spawn(async move {
            Self::debounce_loop(rx, harness_clone, caps_clone, project_clone, debounce_ms).await;
        });

        Ok(Self {
            harness,
            capabilities,
            project_root,
            _watcher: watcher,
        })
    }

    /// Get current capabilities
    pub async fn capabilities(&self) -> HarnessCapabilities {
        self.capabilities.read().await.clone()
    }

    /// Force re-introspection
    pub async fn refresh(&self) -> Result<()> {
        let caps = self.harness.introspect(self.project_root.as_deref()).await?;
        *self.capabilities.write().await = caps;
        Ok(())
    }

    async fn debounce_loop(
        mut rx: mpsc::Receiver<Event>,
        harness: Arc<dyn Harness>,
        capabilities: Arc<RwLock<HarnessCapabilities>>,
        project_root: Option<PathBuf>,
        debounce_ms: u64,
    ) {
        let debounce = Duration::from_millis(debounce_ms);

        loop {
            // Wait for first event
            if rx.recv().await.is_none() {
                break; // Channel closed
            }

            // Debounce: wait for quiet period
            loop {
                match tokio::time::timeout(debounce, rx.recv()).await {
                    Ok(Some(_)) => continue, // More events, keep waiting
                    Ok(None) => return,      // Channel closed
                    Err(_) => break,         // Timeout - quiet period elapsed
                }
            }

            // Re-introspect
            tracing::debug!("Config changed, re-introspecting harness capabilities");
            if let Ok(caps) = harness.introspect(project_root.as_deref()).await {
                *capabilities.write().await = caps;
                tracing::info!(
                    harness = %caps.harness_type,
                    version = ?caps.version,
                    "Harness capabilities refreshed"
                );
            }
        }
    }
}
```

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntrospectionError {
    #[error("Could not determine home directory")]
    NoHomeDir,

    #[error("Failed to read config file {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("File watcher error: {0}")]
    Watcher(#[from] notify::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, IntrospectionError>;
```

---

## Integration with vibes-cli

```rust
// In vibes-cli, when starting a session:

use vibes_introspection::{harness_for_command, CapabilityWatcher};

async fn start_session(cmd: &str, project_root: Option<PathBuf>) -> anyhow::Result<()> {
    // Get harness for this command
    let harness = harness_for_command(cmd)
        .ok_or_else(|| anyhow::anyhow!("Unknown harness: {}", cmd))?;

    // Start watcher (introspects on creation)
    let watcher = CapabilityWatcher::new(harness, project_root, 500).await?;

    // Log detected capabilities
    let caps = watcher.capabilities().await;
    tracing::info!(
        harness = %caps.harness_type,
        version = ?caps.version,
        hooks = ?caps.effective_hooks().map(|h| h.supported_types.len()),
        injection_targets = caps.effective_injection_targets().len(),
        "Harness capabilities detected"
    );

    // Pass watcher to session manager for later milestones (4.3+)
    // session_manager.set_capability_watcher(watcher);

    Ok(())
}
```

---

## Dependencies

```toml
[package]
name = "vibes-introspection"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["fs", "process", "sync", "time"] }
async-trait = "0.1"

# File watching
notify = "6"

# Cross-platform paths
dirs = "5"

# Serialization (for parsing settings.json)
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "1"

# Logging
tracing = "0.1"
```

---

## Testing Strategy

1. **Unit tests** for path resolution across platforms (mock `dirs` functions)
2. **Integration tests** with temp directories simulating `.claude/` structure
3. **File watcher tests** verifying debounce behavior
4. **Platform-specific CI** on Linux, macOS, Windows

---

## Deliverables

- [ ] `vibes-introspection` crate with public API
- [ ] `Harness` trait and `ClaudeCodeHarness` implementation
- [ ] `ConfigPaths` with cross-platform support
- [ ] `HarnessCapabilities` with 3-tier scope hierarchy
- [ ] `CapabilityWatcher` with debounced file watching
- [ ] Integration in `vibes-cli` session startup
- [ ] Unit and integration tests
