//! Integration tests for the vibes-introspection crate
//!
//! These tests verify the full introspection workflow including:
//! - Creating config structures
//! - Detecting capabilities
//! - Watching for changes

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

use async_trait::async_trait;
use vibes_introspection::{
    CapabilityWatcher, ConfigFormat, ConfigPaths, Harness, HarnessCapabilities, InjectionScope,
    Result, ScopedCapabilities,
};

/// Test harness that wraps a temp directory for testing
///
/// This implements the Harness trait and uses the temp directory as the user config path.
/// It performs introspection by checking for hooks, config files, and injection targets
/// in the configured paths.
struct TestHarness {
    user_config_path: PathBuf,
}

impl TestHarness {
    fn new(user_config_path: PathBuf) -> Self {
        Self { user_config_path }
    }

    /// Detect hooks in a directory
    async fn detect_hooks(&self, base_path: &std::path::Path) -> Option<vibes_introspection::HookCapabilities> {
        let hooks_dir = base_path.join("hooks");
        if !hooks_dir.exists() {
            return None;
        }

        let mut installed_hooks = Vec::new();
        let mut entries = fs::read_dir(&hooks_dir).await.ok()?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let hook_type = if file_name.starts_with("pre_tool_use") || file_name.starts_with("pre-tool-use") {
                Some(vibes_introspection::HookType::PreToolUse)
            } else if file_name.starts_with("post_tool_use") || file_name.starts_with("post-tool-use") {
                Some(vibes_introspection::HookType::PostToolUse)
            } else if file_name.starts_with("stop") {
                Some(vibes_introspection::HookType::Stop)
            } else if file_name.starts_with("notification") {
                Some(vibes_introspection::HookType::Notification)
            } else {
                None
            };

            if let Some(hook_type) = hook_type {
                installed_hooks.push(vibes_introspection::InstalledHook {
                    hook_type,
                    name: file_name.to_string(),
                    path,
                });
            }
        }

        Some(vibes_introspection::HookCapabilities {
            supported_types: vec![
                vibes_introspection::HookType::PreToolUse,
                vibes_introspection::HookType::PostToolUse,
                vibes_introspection::HookType::Stop,
                vibes_introspection::HookType::Notification,
            ],
            hooks_dir: Some(hooks_dir),
            installed_hooks,
        })
    }

    /// Detect config files in a directory
    async fn detect_config_files(&self, base_path: &std::path::Path) -> Vec<vibes_introspection::ConfigFile> {
        let mut config_files = Vec::new();

        let settings_path = base_path.join("settings.json");
        if settings_path.exists() {
            config_files.push(vibes_introspection::ConfigFile {
                path: settings_path,
                format: ConfigFormat::Json,
                writable: true,
            });
        }

        config_files
    }

    /// Detect injection targets in a directory
    async fn detect_injection_targets(
        &self,
        base_path: &std::path::Path,
        scope: InjectionScope,
    ) -> Vec<vibes_introspection::InjectionTarget> {
        let mut targets = Vec::new();

        let claude_md_path = base_path.join("CLAUDE.md");
        if claude_md_path.exists() {
            targets.push(vibes_introspection::InjectionTarget {
                path: claude_md_path,
                format: ConfigFormat::Markdown,
                writable: true,
                scope,
            });
        }

        targets
    }

    /// Detect capabilities at a scope level
    async fn detect_scope(
        &self,
        base_path: &std::path::Path,
        scope: InjectionScope,
    ) -> Option<ScopedCapabilities> {
        if !base_path.exists() {
            return None;
        }

        let hooks = self.detect_hooks(base_path).await;
        let config_files = self.detect_config_files(base_path).await;
        let injection_targets = self.detect_injection_targets(base_path, scope).await;

        if hooks.is_none() && config_files.is_empty() && injection_targets.is_empty() {
            return None;
        }

        Some(ScopedCapabilities {
            hooks,
            config_files,
            injection_targets,
        })
    }
}

#[async_trait]
impl Harness for TestHarness {
    fn harness_type(&self) -> &'static str {
        "claude"
    }

    async fn version(&self) -> Option<String> {
        Some("1.0.0-test".to_string())
    }

    fn config_paths(&self, project_root: Option<&std::path::Path>) -> Result<ConfigPaths> {
        Ok(ConfigPaths {
            system: None,
            user: self.user_config_path.clone(),
            project: project_root.map(|p| p.join(".claude")),
        })
    }

    async fn introspect(&self, project_root: Option<&std::path::Path>) -> Result<HarnessCapabilities> {
        let paths = self.config_paths(project_root)?;

        // Detect user scope
        let user = self
            .detect_scope(&paths.user, InjectionScope::User)
            .await
            .unwrap_or_default();

        // Detect project scope
        let project = if let Some(ref project_path) = paths.project {
            self.detect_scope(project_path, InjectionScope::Project).await
        } else {
            None
        };

        Ok(HarnessCapabilities {
            harness_type: self.harness_type().to_string(),
            version: self.version().await,
            system: None,
            user,
            project,
        })
    }
}

#[tokio::test]
async fn test_full_introspection_workflow() {
    // Create a temp directory with .claude config structure
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let user_config_path = temp_dir.path().join(".claude");

    // Create .claude directory structure
    fs::create_dir_all(&user_config_path)
        .await
        .expect("Failed to create .claude dir");

    // Create hooks directory with a "vibes" subdirectory
    let hooks_dir = user_config_path.join("hooks");
    fs::create_dir_all(&hooks_dir)
        .await
        .expect("Failed to create hooks dir");

    let vibes_hooks_dir = hooks_dir.join("vibes");
    fs::create_dir_all(&vibes_hooks_dir)
        .await
        .expect("Failed to create vibes hooks dir");

    // Create a sample hook
    fs::write(hooks_dir.join("pre_tool_use.sh"), "#!/bin/bash\necho 'hook'")
        .await
        .expect("Failed to create hook");

    // Create settings.json
    fs::write(user_config_path.join("settings.json"), "{}")
        .await
        .expect("Failed to create settings.json");

    // Create CLAUDE.md
    fs::write(user_config_path.join("CLAUDE.md"), "# User Guidelines")
        .await
        .expect("Failed to create CLAUDE.md");

    // Create the test harness
    let harness = Arc::new(TestHarness::new(user_config_path.clone()));

    // Create a CapabilityWatcher
    let watcher = CapabilityWatcher::new(harness.clone(), None, 100)
        .await
        .expect("Failed to create watcher");

    // Verify all capabilities are detected correctly
    let capabilities = watcher.capabilities().await;

    // Check harness type
    assert_eq!(capabilities.harness_type, "claude");

    // Check version
    assert_eq!(capabilities.version, Some("1.0.0-test".to_string()));

    // Check user scope hooks
    assert!(capabilities.user.hooks.is_some());
    let hooks = capabilities.user.hooks.as_ref().unwrap();
    assert!(hooks.hooks_dir.is_some());
    assert_eq!(hooks.installed_hooks.len(), 1);
    assert_eq!(hooks.installed_hooks[0].name, "pre_tool_use");

    // Check config files
    assert!(!capabilities.user.config_files.is_empty());
    let settings = capabilities
        .user
        .config_files
        .iter()
        .find(|f| f.path.ends_with("settings.json"));
    assert!(settings.is_some());
    assert_eq!(settings.unwrap().format, ConfigFormat::Json);

    // Check injection targets
    assert!(!capabilities.user.injection_targets.is_empty());
    let claude_md = capabilities
        .user
        .injection_targets
        .iter()
        .find(|t| t.path.ends_with("CLAUDE.md"));
    assert!(claude_md.is_some());
    assert_eq!(claude_md.unwrap().format, ConfigFormat::Markdown);
    assert_eq!(claude_md.unwrap().scope, InjectionScope::User);
}

#[test]
fn test_config_paths_resolve() {
    // Verify ConfigPaths::resolve returns user path containing "claude"
    let paths = ConfigPaths::resolve("claude", None).expect("Failed to resolve paths");

    let user_str = paths.user.to_string_lossy();
    assert!(
        user_str.contains("claude"),
        "User path should contain 'claude': {}",
        user_str
    );

    // Verify project is None without project_root
    assert!(
        paths.project.is_none(),
        "Project should be None without project_root"
    );
}

#[test]
fn test_config_paths_with_project() {
    // Verify project path is correctly set when project_root is provided
    let project_root = PathBuf::from("/tmp/my-test-project");
    let paths =
        ConfigPaths::resolve("claude", Some(&project_root)).expect("Failed to resolve paths");

    assert!(paths.project.is_some(), "Project should be Some with project_root");

    let project = paths.project.unwrap();
    assert_eq!(
        project,
        PathBuf::from("/tmp/my-test-project/.claude"),
        "Project path should be project_root/.claude"
    );
}
