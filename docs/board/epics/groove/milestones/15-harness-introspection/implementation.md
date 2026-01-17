# Harness Introspection Implementation Plan

> Part of [vibes groove](../14-continual-learning/design.md) - The continual learning system that finds your coding rhythm.
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create `vibes-introspection` crate that discovers AI coding assistant capabilities across platform-aware config paths with real-time file watching. This is Level 0 of groove - the foundation that enables all learning.

**Architecture:** Trait-based `Harness` abstraction with `ClaudeCodeHarness` implementation. 3-tier scope hierarchy (system → user → project). `CapabilityWatcher` uses notify crate with 500ms debounce for reactive updates.

**Tech Stack:** Rust, tokio (async), notify (file watching), dirs (cross-platform paths), serde_json (settings parsing), thiserror (errors)

**Reference:** See [design.md](design.md) for full type definitions and rationale.

---

## Task 1: Scaffold vibes-introspection Crate

**Files:**
- Create: `vibes-introspection/Cargo.toml`
- Create: `vibes-introspection/src/lib.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Create crate directory**

```bash
mkdir -p vibes-introspection/src
```

**Step 2: Write Cargo.toml**

```toml
[package]
name = "vibes-introspection"
version = "0.1.0"
edition = "2021"
description = "Harness capability introspection for vibes"
license = "MIT"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["fs", "process", "sync", "time"] }
async-trait = "0.1"

# File watching
notify = "6"

# Cross-platform paths
dirs = "5"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "1"

# Logging
tracing = "0.1"

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

**Step 3: Write lib.rs stub**

```rust
//! vibes-introspection - Harness capability discovery
//!
//! This crate provides traits and implementations for discovering
//! what capabilities an AI coding assistant provides.

pub mod error;

pub use error::{IntrospectionError, Result};
```

**Step 4: Create error.rs stub**

Create `vibes-introspection/src/error.rs`:

```rust
//! Error types for introspection

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum IntrospectionError {
    #[error("Could not determine home directory")]
    NoHomeDir,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, IntrospectionError>;
```

**Step 5: Add to workspace**

Modify `Cargo.toml` (root) - add to members:

```toml
members = [
    "vibes-cli",
    "vibes-core",
    "vibes-plugin-api",
    "vibes-server",
    "vibes-introspection",
]
```

**Step 6: Verify it compiles**

Run: `cargo check -p vibes-introspection`
Expected: Compiles with no errors

**Step 7: Commit**

```bash
git add vibes-introspection/ Cargo.toml
git commit -m "feat: scaffold vibes-introspection crate"
```

---

## Task 2: ConfigPaths with Cross-Platform Support

**Files:**
- Create: `vibes-introspection/src/paths.rs`
- Modify: `vibes-introspection/src/lib.rs`
- Modify: `vibes-introspection/src/error.rs`

**Step 1: Write failing test for user config path**

Create `vibes-introspection/src/paths.rs`:

```rust
//! Cross-platform config path resolution

use crate::{IntrospectionError, Result};
use std::path::{Path, PathBuf};

/// Platform-aware config path resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPaths {
    pub system: Option<PathBuf>,
    pub user: PathBuf,
    pub project: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_returns_user_path() {
        let paths = ConfigPaths::resolve("claude", None).unwrap();

        // User path should exist and contain "claude"
        let user_str = paths.user.to_string_lossy();
        assert!(user_str.contains("claude"), "User path should contain 'claude': {}", user_str);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-introspection test_resolve_returns_user_path`
Expected: FAIL with "no function `resolve`"

**Step 3: Implement ConfigPaths::resolve**

Add to `vibes-introspection/src/paths.rs`:

```rust
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

    #[cfg(target_os = "macos")]
    fn system_config_dir(harness: &str) -> Option<PathBuf> {
        let path = PathBuf::from(format!("/Library/Application Support/{}", harness));
        path.exists().then_some(path)
    }

    #[cfg(windows)]
    fn system_config_dir(harness: &str) -> Option<PathBuf> {
        std::env::var("PROGRAMDATA")
            .ok()
            .map(|d| PathBuf::from(d).join(harness))
            .filter(|p| p.exists())
    }
}
```

**Step 4: Export from lib.rs**

Modify `vibes-introspection/src/lib.rs`:

```rust
//! vibes-introspection - Harness capability discovery

pub mod error;
pub mod paths;

pub use error::{IntrospectionError, Result};
pub use paths::ConfigPaths;
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p vibes-introspection test_resolve_returns_user_path`
Expected: PASS

**Step 6: Add test for project path**

Add to `paths.rs` tests:

```rust
    #[test]
    fn test_resolve_with_project_root() {
        let project = PathBuf::from("/tmp/my-project");
        let paths = ConfigPaths::resolve("claude", Some(&project)).unwrap();

        assert_eq!(paths.project, Some(PathBuf::from("/tmp/my-project/.claude")));
    }

    #[test]
    fn test_resolve_without_project_root() {
        let paths = ConfigPaths::resolve("claude", None).unwrap();

        assert!(paths.project.is_none());
    }
```

**Step 7: Run all path tests**

Run: `cargo test -p vibes-introspection paths`
Expected: All 3 tests PASS

**Step 8: Commit**

```bash
git add vibes-introspection/src/paths.rs vibes-introspection/src/lib.rs
git commit -m "feat: add ConfigPaths with cross-platform resolution"
```

---

## Task 3: Capability Types

**Files:**
- Create: `vibes-introspection/src/capabilities.rs`
- Modify: `vibes-introspection/src/lib.rs`

**Step 1: Write capability types with tests**

Create `vibes-introspection/src/capabilities.rs`:

```rust
//! Harness capability types

use std::path::PathBuf;

/// Hook types that can be observed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    Stop,
    Notification,
}

/// Config file format
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
    Markdown,
}

/// Scope for injection targets
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectionScope {
    System,
    User,
    Project,
}

/// An installed hook we detected
#[derive(Debug, Clone)]
pub struct InstalledHook {
    pub hook_type: HookType,
    pub name: String,
    pub path: PathBuf,
}

/// Hook system capabilities
#[derive(Debug, Clone, Default)]
pub struct HookCapabilities {
    pub supported_types: Vec<HookType>,
    pub hooks_dir: Option<PathBuf>,
    pub installed_hooks: Vec<InstalledHook>,
}

/// A config file we detected
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub writable: bool,
}

/// A file we can inject learnings into
#[derive(Debug, Clone)]
pub struct InjectionTarget {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub writable: bool,
    pub scope: InjectionScope,
}

/// Capabilities at a single scope level
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
        self.project
            .as_ref()
            .and_then(|p| p.hooks.as_ref())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_hooks_prefers_project() {
        let project_hooks = HookCapabilities {
            supported_types: vec![HookType::Stop],
            hooks_dir: Some(PathBuf::from("/project/.claude/hooks")),
            installed_hooks: vec![],
        };

        let user_hooks = HookCapabilities {
            supported_types: vec![HookType::PreToolUse, HookType::PostToolUse],
            hooks_dir: Some(PathBuf::from("/home/user/.claude/hooks")),
            installed_hooks: vec![],
        };

        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: None,
            user: ScopedCapabilities {
                hooks: Some(user_hooks),
                ..Default::default()
            },
            project: Some(ScopedCapabilities {
                hooks: Some(project_hooks),
                ..Default::default()
            }),
        };

        let effective = caps.effective_hooks().unwrap();
        assert_eq!(effective.supported_types, vec![HookType::Stop]);
    }

    #[test]
    fn test_effective_hooks_falls_back_to_user() {
        let user_hooks = HookCapabilities {
            supported_types: vec![HookType::PreToolUse],
            hooks_dir: Some(PathBuf::from("/home/user/.claude/hooks")),
            installed_hooks: vec![],
        };

        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: None,
            user: ScopedCapabilities {
                hooks: Some(user_hooks),
                ..Default::default()
            },
            project: None,
        };

        let effective = caps.effective_hooks().unwrap();
        assert_eq!(effective.supported_types, vec![HookType::PreToolUse]);
    }

    #[test]
    fn test_effective_injection_targets_collects_all_scopes() {
        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: Some(ScopedCapabilities {
                injection_targets: vec![InjectionTarget {
                    path: PathBuf::from("/etc/claude/CLAUDE.md"),
                    format: ConfigFormat::Markdown,
                    writable: false,
                    scope: InjectionScope::System,
                }],
                ..Default::default()
            }),
            user: ScopedCapabilities {
                injection_targets: vec![InjectionTarget {
                    path: PathBuf::from("/home/user/.claude/CLAUDE.md"),
                    format: ConfigFormat::Markdown,
                    writable: true,
                    scope: InjectionScope::User,
                }],
                ..Default::default()
            },
            project: Some(ScopedCapabilities {
                injection_targets: vec![InjectionTarget {
                    path: PathBuf::from("/project/CLAUDE.md"),
                    format: ConfigFormat::Markdown,
                    writable: true,
                    scope: InjectionScope::Project,
                }],
                ..Default::default()
            }),
        };

        let targets = caps.effective_injection_targets();
        assert_eq!(targets.len(), 3);
    }
}
```

**Step 2: Export from lib.rs**

Modify `vibes-introspection/src/lib.rs`:

```rust
//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod paths;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use paths::ConfigPaths;
```

**Step 3: Run tests**

Run: `cargo test -p vibes-introspection capabilities`
Expected: All 3 tests PASS

**Step 4: Commit**

```bash
git add vibes-introspection/src/capabilities.rs vibes-introspection/src/lib.rs
git commit -m "feat: add capability types with scope hierarchy"
```

---

## Task 4: Harness Trait

**Files:**
- Create: `vibes-introspection/src/harness.rs`
- Modify: `vibes-introspection/src/lib.rs`

**Step 1: Define the Harness trait**

Create `vibes-introspection/src/harness.rs`:

```rust
//! Harness trait for AI coding assistant abstraction

use crate::{ConfigPaths, HarnessCapabilities, Result};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Core trait - any AI coding assistant we can enhance
#[async_trait]
pub trait Harness: Send + Sync {
    /// Unique identifier (e.g., "claude", "cursor", "aider")
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
        "claude" => {
            #[cfg(feature = "claude-code")]
            {
                Some(Arc::new(crate::claude_code::ClaudeCodeHarness))
            }
            #[cfg(not(feature = "claude-code"))]
            {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_trait_is_object_safe() {
        // This compiles only if Harness is object-safe
        fn _takes_boxed_harness(_: Box<dyn Harness>) {}
    }
}
```

**Step 2: Export from lib.rs**

Modify `vibes-introspection/src/lib.rs`:

```rust
//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod harness;
pub mod paths;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use harness::{harness_for_command, Harness};
pub use paths::ConfigPaths;
```

**Step 3: Run test**

Run: `cargo test -p vibes-introspection harness`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-introspection/src/harness.rs vibes-introspection/src/lib.rs
git commit -m "feat: add Harness trait"
```

---

## Task 5: ClaudeCodeHarness Implementation

**Files:**
- Create: `vibes-introspection/src/claude_code/mod.rs`
- Create: `vibes-introspection/src/claude_code/harness.rs`
- Create: `vibes-introspection/src/claude_code/detection.rs`
- Modify: `vibes-introspection/src/lib.rs`
- Modify: `vibes-introspection/Cargo.toml`

**Step 1: Create module structure**

```bash
mkdir -p vibes-introspection/src/claude_code
```

**Step 2: Write detection tests first**

Create `vibes-introspection/src/claude_code/detection.rs`:

```rust
//! Claude Code capability detection logic

use crate::{
    ConfigFile, ConfigFormat, HookCapabilities, HookType, InjectionScope, InjectionTarget,
    InstalledHook, ScopedCapabilities,
};
use std::path::Path;
use tokio::fs;

/// Detect capabilities at a single scope
pub async fn detect_scope(path: &Path, scope: InjectionScope) -> Option<ScopedCapabilities> {
    if !path.exists() {
        return None;
    }

    Some(ScopedCapabilities {
        hooks: detect_hooks(path).await,
        config_files: detect_config_files(path).await,
        injection_targets: detect_injection_targets(path, scope).await,
    })
}

/// Detect hook capabilities
pub async fn detect_hooks(base: &Path) -> Option<HookCapabilities> {
    let hooks_dir = base.join("hooks");
    if !hooks_dir.exists() {
        return None;
    }

    let installed_hooks = find_installed_hooks(&hooks_dir).await;

    Some(HookCapabilities {
        supported_types: vec![
            HookType::PreToolUse,
            HookType::PostToolUse,
            HookType::Stop,
            HookType::Notification,
        ],
        hooks_dir: Some(hooks_dir),
        installed_hooks,
    })
}

/// Find config files in a directory
pub async fn detect_config_files(base: &Path) -> Vec<ConfigFile> {
    let mut files = Vec::new();

    for (name, format) in [
        ("settings.json", ConfigFormat::Json),
        ("config.json", ConfigFormat::Json),
        ("CLAUDE.md", ConfigFormat::Markdown),
    ] {
        let path = base.join(name);
        if path.exists() {
            let writable = is_writable(&path).await;
            files.push(ConfigFile {
                path,
                format,
                writable,
            });
        }
    }

    files
}

/// Find injection targets
pub async fn detect_injection_targets(base: &Path, scope: InjectionScope) -> Vec<InjectionTarget> {
    let mut targets = Vec::new();

    let claude_md = base.join("CLAUDE.md");
    let dir_writable = is_writable(base).await;

    if claude_md.exists() || dir_writable {
        targets.push(InjectionTarget {
            path: claude_md,
            format: ConfigFormat::Markdown,
            writable: if claude_md.exists() {
                is_writable(&claude_md).await
            } else {
                dir_writable
            },
            scope,
        });
    }

    targets
}

/// Find installed hook providers
async fn find_installed_hooks(hooks_dir: &Path) -> Vec<InstalledHook> {
    let mut hooks = Vec::new();

    if let Ok(mut entries) = fs::read_dir(hooks_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    hooks.push(InstalledHook {
                        hook_type: HookType::Stop, // Default; could parse scripts
                        name: name.to_string(),
                        path,
                    });
                }
            }
        }
    }

    hooks
}

/// Check if a path is writable
async fn is_writable(path: &Path) -> bool {
    fs::metadata(path)
        .await
        .map(|m| !m.permissions().readonly())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_detect_hooks_returns_none_if_no_hooks_dir() {
        let tmp = TempDir::new().unwrap();
        let result = detect_hooks(tmp.path()).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_detect_hooks_returns_some_if_hooks_dir_exists() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("hooks")).await.unwrap();

        let result = detect_hooks(tmp.path()).await;
        assert!(result.is_some());

        let caps = result.unwrap();
        assert!(caps.hooks_dir.is_some());
        assert!(!caps.supported_types.is_empty());
    }

    #[tokio::test]
    async fn test_detect_config_files_finds_settings_json() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("settings.json"), "{}").await.unwrap();

        let files = detect_config_files(tmp.path()).await;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].format, ConfigFormat::Json);
    }

    #[tokio::test]
    async fn test_detect_injection_targets_finds_claude_md() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("CLAUDE.md"), "# Test").await.unwrap();

        let targets = detect_injection_targets(tmp.path(), InjectionScope::User).await;
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].format, ConfigFormat::Markdown);
        assert_eq!(targets[0].scope, InjectionScope::User);
    }

    #[tokio::test]
    async fn test_detect_scope_returns_none_for_nonexistent_path() {
        let result = detect_scope(Path::new("/nonexistent/path"), InjectionScope::User).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_installed_hooks_finds_provider_dirs() {
        let tmp = TempDir::new().unwrap();
        let hooks_dir = tmp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::create_dir(hooks_dir.join("vibes")).await.unwrap();
        fs::create_dir(hooks_dir.join("custom")).await.unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert_eq!(hooks.len(), 2);
    }
}
```

**Step 3: Run detection tests**

Run: `cargo test -p vibes-introspection detection`
Expected: All tests PASS

**Step 4: Write harness implementation**

Create `vibes-introspection/src/claude_code/harness.rs`:

```rust
//! Claude Code harness implementation

use crate::claude_code::detection;
use crate::{ConfigPaths, Harness, HarnessCapabilities, InjectionScope, Result, ScopedCapabilities};
use async_trait::async_trait;
use std::path::Path;
use tokio::process::Command;

/// Claude Code harness implementation
#[derive(Debug, Default, Clone)]
pub struct ClaudeCodeHarness;

#[async_trait]
impl Harness for ClaudeCodeHarness {
    fn harness_type(&self) -> &'static str {
        "claude"
    }

    async fn version(&self) -> Option<String> {
        let output = Command::new("claude")
            .arg("--version")
            .output()
            .await
            .ok()?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        } else {
            None
        }
    }

    fn config_paths(&self, project_root: Option<&Path>) -> Result<ConfigPaths> {
        ConfigPaths::resolve("claude", project_root)
    }

    async fn introspect(&self, project_root: Option<&Path>) -> Result<HarnessCapabilities> {
        let paths = self.config_paths(project_root)?;

        let system = if let Some(ref sys_path) = paths.system {
            detection::detect_scope(sys_path, InjectionScope::System).await
        } else {
            None
        };

        let user = detection::detect_scope(&paths.user, InjectionScope::User)
            .await
            .unwrap_or_default();

        let project = if let Some(ref proj_path) = paths.project {
            detection::detect_scope(proj_path, InjectionScope::Project).await
        } else {
            None
        };

        Ok(HarnessCapabilities {
            harness_type: self.harness_type().to_string(),
            version: self.version().await,
            system,
            user,
            project,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[test]
    fn test_harness_type() {
        let harness = ClaudeCodeHarness;
        assert_eq!(harness.harness_type(), "claude");
    }

    #[tokio::test]
    async fn test_introspect_with_temp_config() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).await.unwrap();
        fs::create_dir(claude_dir.join("hooks")).await.unwrap();
        fs::write(claude_dir.join("settings.json"), "{}").await.unwrap();
        fs::write(claude_dir.join("CLAUDE.md"), "# Test").await.unwrap();

        // Create a custom harness that uses our temp dir
        struct TestHarness(std::path::PathBuf);

        #[async_trait]
        impl Harness for TestHarness {
            fn harness_type(&self) -> &'static str {
                "claude"
            }

            async fn version(&self) -> Option<String> {
                None
            }

            fn config_paths(&self, project_root: Option<&Path>) -> Result<ConfigPaths> {
                Ok(ConfigPaths {
                    system: None,
                    user: self.0.join(".claude"),
                    project: project_root.map(|p| p.join(".claude")),
                })
            }

            async fn introspect(&self, project_root: Option<&Path>) -> Result<HarnessCapabilities> {
                let paths = self.config_paths(project_root)?;

                let user = detection::detect_scope(&paths.user, InjectionScope::User)
                    .await
                    .unwrap_or_default();

                Ok(HarnessCapabilities {
                    harness_type: self.harness_type().to_string(),
                    version: self.version().await,
                    system: None,
                    user,
                    project: None,
                })
            }
        }

        let harness = TestHarness(tmp.path().to_path_buf());
        let caps = harness.introspect(None).await.unwrap();

        assert_eq!(caps.harness_type, "claude");
        assert!(caps.user.hooks.is_some());
        assert!(!caps.user.config_files.is_empty());
        assert!(!caps.user.injection_targets.is_empty());
    }
}
```

**Step 5: Create module file**

Create `vibes-introspection/src/claude_code/mod.rs`:

```rust
//! Claude Code harness implementation

mod detection;
mod harness;

pub use harness::ClaudeCodeHarness;
```

**Step 6: Add feature flag and export**

Modify `vibes-introspection/Cargo.toml`, add features section:

```toml
[features]
default = ["claude-code"]
claude-code = []
```

Modify `vibes-introspection/src/lib.rs`:

```rust
//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod harness;
pub mod paths;

#[cfg(feature = "claude-code")]
pub mod claude_code;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use harness::{harness_for_command, Harness};
pub use paths::ConfigPaths;

#[cfg(feature = "claude-code")]
pub use claude_code::ClaudeCodeHarness;
```

**Step 7: Run all tests**

Run: `cargo test -p vibes-introspection`
Expected: All tests PASS

**Step 8: Commit**

```bash
git add vibes-introspection/
git commit -m "feat: add ClaudeCodeHarness implementation"
```

---

## Task 6: CapabilityWatcher with Debounce

**Files:**
- Create: `vibes-introspection/src/watcher.rs`
- Modify: `vibes-introspection/src/lib.rs`
- Modify: `vibes-introspection/src/error.rs`

**Step 1: Update error types**

Modify `vibes-introspection/src/error.rs`:

```rust
//! Error types for introspection

use std::path::PathBuf;

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

**Step 2: Write watcher with tests**

Create `vibes-introspection/src/watcher.rs`:

```rust
//! File watcher with debounce for capability updates

use crate::{Harness, HarnessCapabilities, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

/// Watches config directories and triggers re-introspection on changes
pub struct CapabilityWatcher {
    harness: Arc<dyn Harness>,
    capabilities: Arc<RwLock<HarnessCapabilities>>,
    project_root: Option<PathBuf>,
    // Kept alive to maintain watch
    _watcher: RecommendedWatcher,
}

impl CapabilityWatcher {
    /// Create a new watcher with initial introspection
    ///
    /// # Arguments
    /// * `harness` - The harness to introspect
    /// * `project_root` - Optional project root for project-scoped detection
    /// * `debounce_ms` - Debounce delay in milliseconds (default: 500)
    pub async fn new(
        harness: Arc<dyn Harness>,
        project_root: Option<PathBuf>,
        debounce_ms: u64,
    ) -> Result<Self> {
        // Initial introspection
        let caps = harness.introspect(project_root.as_deref()).await?;
        let capabilities = Arc::new(RwLock::new(caps));

        // Set up event channel
        let (tx, rx) = mpsc::channel::<Event>(100);

        let tx_clone = tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: std::result::Result<Event, _>| {
            if let Ok(event) = res {
                let _ = tx_clone.blocking_send(event);
            }
        })?;

        // Watch all config directories
        let paths = harness.config_paths(project_root.as_deref())?;
        if let Some(ref sys) = paths.system {
            if sys.exists() {
                let _ = watcher.watch(sys, RecursiveMode::Recursive);
            }
        }
        if paths.user.exists() {
            let _ = watcher.watch(&paths.user, RecursiveMode::Recursive);
        }
        if let Some(ref proj) = paths.project {
            if proj.exists() {
                let _ = watcher.watch(proj, RecursiveMode::Recursive);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConfigPaths, InjectionScope, ScopedCapabilities};
    use async_trait::async_trait;
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::fs;

    struct MockHarness {
        config_dir: PathBuf,
        call_count: Arc<RwLock<u32>>,
    }

    #[async_trait]
    impl Harness for MockHarness {
        fn harness_type(&self) -> &'static str {
            "mock"
        }

        async fn version(&self) -> Option<String> {
            Some("1.0.0".to_string())
        }

        fn config_paths(&self, _project_root: Option<&Path>) -> Result<ConfigPaths> {
            Ok(ConfigPaths {
                system: None,
                user: self.config_dir.clone(),
                project: None,
            })
        }

        async fn introspect(&self, _project_root: Option<&Path>) -> Result<HarnessCapabilities> {
            *self.call_count.write().await += 1;
            Ok(HarnessCapabilities {
                harness_type: "mock".to_string(),
                version: Some("1.0.0".to_string()),
                system: None,
                user: ScopedCapabilities::default(),
                project: None,
            })
        }
    }

    #[tokio::test]
    async fn test_watcher_initial_introspection() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".mock")).await.ok();

        let call_count = Arc::new(RwLock::new(0));
        let harness = Arc::new(MockHarness {
            config_dir: tmp.path().join(".mock"),
            call_count: call_count.clone(),
        });

        let watcher = CapabilityWatcher::new(harness, None, 100).await.unwrap();
        let caps = watcher.capabilities().await;

        assert_eq!(caps.harness_type, "mock");
        assert_eq!(*call_count.read().await, 1);
    }

    #[tokio::test]
    async fn test_watcher_refresh() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".mock")).await.ok();

        let call_count = Arc::new(RwLock::new(0));
        let harness = Arc::new(MockHarness {
            config_dir: tmp.path().join(".mock"),
            call_count: call_count.clone(),
        });

        let watcher = CapabilityWatcher::new(harness, None, 100).await.unwrap();
        assert_eq!(*call_count.read().await, 1);

        watcher.refresh().await.unwrap();
        assert_eq!(*call_count.read().await, 2);
    }
}
```

**Step 3: Export from lib.rs**

Modify `vibes-introspection/src/lib.rs`:

```rust
//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod harness;
pub mod paths;
pub mod watcher;

#[cfg(feature = "claude-code")]
pub mod claude_code;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use harness::{harness_for_command, Harness};
pub use paths::ConfigPaths;
pub use watcher::CapabilityWatcher;

#[cfg(feature = "claude-code")]
pub use claude_code::ClaudeCodeHarness;
```

**Step 4: Run tests**

Run: `cargo test -p vibes-introspection watcher`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add vibes-introspection/
git commit -m "feat: add CapabilityWatcher with debounced file watching"
```

---

## Task 7: Integration Test

**Files:**
- Create: `vibes-introspection/tests/integration.rs`

**Step 1: Write integration test**

Create `vibes-introspection/tests/integration.rs`:

```rust
//! Integration tests for vibes-introspection

use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use vibes_introspection::{
    CapabilityWatcher, ClaudeCodeHarness, ConfigFormat, ConfigPaths, Harness, HarnessCapabilities,
    InjectionScope, ScopedCapabilities,
};

#[tokio::test]
async fn test_full_introspection_workflow() {
    // Create a fake Claude config directory
    let tmp = TempDir::new().unwrap();
    let claude_dir = tmp.path().join(".claude");
    fs::create_dir(&claude_dir).await.unwrap();
    fs::create_dir(claude_dir.join("hooks")).await.unwrap();
    fs::create_dir(claude_dir.join("hooks").join("vibes")).await.unwrap();
    fs::write(claude_dir.join("settings.json"), r#"{"model": "opus"}"#)
        .await
        .unwrap();
    fs::write(claude_dir.join("CLAUDE.md"), "# Project Instructions")
        .await
        .unwrap();

    // Create a mock harness that uses our temp directory
    struct TestHarness(std::path::PathBuf);

    #[async_trait::async_trait]
    impl Harness for TestHarness {
        fn harness_type(&self) -> &'static str {
            "claude"
        }

        async fn version(&self) -> Option<String> {
            Some("test-1.0".to_string())
        }

        fn config_paths(
            &self,
            project_root: Option<&std::path::Path>,
        ) -> vibes_introspection::Result<ConfigPaths> {
            Ok(ConfigPaths {
                system: None,
                user: self.0.join(".claude"),
                project: project_root.map(|p| p.join(".claude")),
            })
        }

        async fn introspect(
            &self,
            project_root: Option<&std::path::Path>,
        ) -> vibes_introspection::Result<HarnessCapabilities> {
            let paths = self.config_paths(project_root)?;

            let user =
                vibes_introspection::claude_code::ClaudeCodeHarness::default();

            // Use detection module directly
            let user_caps = if paths.user.exists() {
                Some(ScopedCapabilities {
                    hooks: Some(vibes_introspection::HookCapabilities {
                        supported_types: vec![
                            vibes_introspection::HookType::PreToolUse,
                            vibes_introspection::HookType::PostToolUse,
                            vibes_introspection::HookType::Stop,
                        ],
                        hooks_dir: Some(paths.user.join("hooks")),
                        installed_hooks: vec![vibes_introspection::InstalledHook {
                            hook_type: vibes_introspection::HookType::Stop,
                            name: "vibes".to_string(),
                            path: paths.user.join("hooks").join("vibes"),
                        }],
                    }),
                    config_files: vec![vibes_introspection::ConfigFile {
                        path: paths.user.join("settings.json"),
                        format: ConfigFormat::Json,
                        writable: true,
                    }],
                    injection_targets: vec![vibes_introspection::InjectionTarget {
                        path: paths.user.join("CLAUDE.md"),
                        format: ConfigFormat::Markdown,
                        writable: true,
                        scope: InjectionScope::User,
                    }],
                })
            } else {
                None
            };

            Ok(HarnessCapabilities {
                harness_type: self.harness_type().to_string(),
                version: self.version().await,
                system: None,
                user: user_caps.unwrap_or_default(),
                project: None,
            })
        }
    }

    let harness = Arc::new(TestHarness(tmp.path().to_path_buf()));

    // Create watcher
    let watcher = CapabilityWatcher::new(harness, None, 100).await.unwrap();
    let caps = watcher.capabilities().await;

    // Verify capabilities
    assert_eq!(caps.harness_type, "claude");
    assert_eq!(caps.version, Some("test-1.0".to_string()));

    // Check hooks
    let hooks = caps.effective_hooks().expect("Should have hooks");
    assert!(hooks.hooks_dir.is_some());
    assert_eq!(hooks.installed_hooks.len(), 1);
    assert_eq!(hooks.installed_hooks[0].name, "vibes");

    // Check config files
    assert_eq!(caps.user.config_files.len(), 1);
    assert_eq!(caps.user.config_files[0].format, ConfigFormat::Json);

    // Check injection targets
    let targets = caps.effective_injection_targets();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].format, ConfigFormat::Markdown);
    assert_eq!(targets[0].scope, InjectionScope::User);
}

#[test]
fn test_config_paths_resolve() {
    let paths = ConfigPaths::resolve("claude", None).unwrap();

    // User path should be set
    let user_str = paths.user.to_string_lossy();
    assert!(
        user_str.contains("claude"),
        "User path should contain 'claude': {}",
        user_str
    );

    // Project should be None without project_root
    assert!(paths.project.is_none());
}

#[test]
fn test_config_paths_with_project() {
    let project_root = std::path::PathBuf::from("/tmp/my-project");
    let paths = ConfigPaths::resolve("claude", Some(&project_root)).unwrap();

    assert_eq!(
        paths.project,
        Some(std::path::PathBuf::from("/tmp/my-project/.claude"))
    );
}
```

**Step 2: Run integration tests**

Run: `cargo test -p vibes-introspection --test integration`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add vibes-introspection/tests/
git commit -m "test: add integration tests for introspection workflow"
```

---

## Task 8: Final Verification & Documentation

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Run all checks**

Run: `just pre-commit` (or equivalent)
Expected: All checks pass

**Step 2: Run full test suite**

Run: `cargo test -p vibes-introspection`
Expected: All tests PASS

**Step 3: Update progress tracker**

Update `docs/PROGRESS.md` - change milestone 4.1 status:

```markdown
| 4.1 Harness Introspection | Complete | [design](plans/15-harness-introspection/design.md) | [implementation](plans/15-harness-introspection/implementation.md) |
```

And update the Phase 4 section:

```markdown
### Milestone 4.1: Harness Introspection
- [x] `vibes-introspection` crate with public API
- [x] `Harness` trait and `ClaudeCodeHarness` implementation
- [x] `ConfigPaths` with cross-platform support
- [x] `HarnessCapabilities` with 3-tier scope hierarchy
- [x] `CapabilityWatcher` with debounced file watching
- [x] Integration tests
```

**Step 4: Commit progress update**

```bash
git add docs/PROGRESS.md
git commit -m "docs: mark milestone 4.1 complete"
```

**Step 5: Final commit summary**

Run: `git log --oneline -10`
Expected: ~8 commits for this milestone

---

## Summary

| Task | Description | Commits |
|------|-------------|---------|
| 1 | Scaffold crate | 1 |
| 2 | ConfigPaths | 1 |
| 3 | Capability types | 1 |
| 4 | Harness trait | 1 |
| 5 | ClaudeCodeHarness | 1 |
| 6 | CapabilityWatcher | 1 |
| 7 | Integration tests | 1 |
| 8 | Final verification | 1 |

**Total: 8 tasks, ~8 commits**
