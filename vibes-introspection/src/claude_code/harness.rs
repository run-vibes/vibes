//! Claude Code harness implementation

use crate::{
    ConfigPaths, Harness, HarnessCapabilities, InjectionScope, Result, ScopedCapabilities,
};
use async_trait::async_trait;
use std::path::Path;
use tokio::process::Command;

use super::detection::{detect_injection_targets, detect_scope};

/// Claude Code harness implementation
#[derive(Debug, Clone, Default)]
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

        if !output.status.success() {
            return None;
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        // Parse version from output like "claude version 1.2.3"
        let version = version_str
            .trim()
            .strip_prefix("claude version ")
            .or_else(|| version_str.trim().strip_prefix("claude-code "))
            .or_else(|| version_str.trim().strip_prefix("claude "))
            .unwrap_or(version_str.trim());

        Some(version.to_string())
    }

    fn config_paths(&self, project_root: Option<&Path>) -> Result<ConfigPaths> {
        ConfigPaths::resolve("claude", project_root)
    }

    async fn introspect(&self, project_root: Option<&Path>) -> Result<HarnessCapabilities> {
        let paths = self.config_paths(project_root)?;
        let version = self.version().await;

        // Detect system scope
        let system = if let Some(ref system_path) = paths.system {
            detect_scope(system_path, InjectionScope::System).await
        } else {
            None
        };

        // Detect user scope
        let user = detect_scope(&paths.user, InjectionScope::User)
            .await
            .unwrap_or_default();

        // Detect project scope
        let project = if let Some(ref project_path) = paths.project {
            let mut scoped = detect_scope(project_path, InjectionScope::Project).await;

            // Also check for CLAUDE.md in project root (parent of .claude)
            if let Some(parent) = project_path.parent() {
                let root_targets = detect_injection_targets(parent, InjectionScope::Project).await;
                if !root_targets.is_empty() {
                    let scoped = scoped.get_or_insert_with(ScopedCapabilities::default);
                    for target in root_targets {
                        // Avoid duplicates
                        if !scoped
                            .injection_targets
                            .iter()
                            .any(|t| t.path == target.path)
                        {
                            scoped.injection_targets.push(target);
                        }
                    }
                }
            }

            scoped
        } else {
            None
        };

        Ok(HarnessCapabilities {
            harness_type: self.harness_type().to_string(),
            version,
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

    /// Test harness helper for creating temp directory structures
    struct TestHarness {
        _temp_dir: TempDir,
        project_root: std::path::PathBuf,
    }

    impl TestHarness {
        async fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_root = temp_dir.path().to_path_buf();
            Self {
                _temp_dir: temp_dir,
                project_root,
            }
        }

        fn project_root(&self) -> &Path {
            &self.project_root
        }

        async fn create_claude_dir(&self) {
            let claude_dir = self.project_root.join(".claude");
            fs::create_dir_all(&claude_dir).await.unwrap();
        }

        async fn create_hooks_dir(&self) {
            let hooks_dir = self.project_root.join(".claude").join("hooks");
            fs::create_dir_all(&hooks_dir).await.unwrap();
        }

        async fn create_hook(&self, name: &str, content: &str) {
            let hooks_dir = self.project_root.join(".claude").join("hooks");
            fs::create_dir_all(&hooks_dir).await.unwrap();
            fs::write(hooks_dir.join(name), content).await.unwrap();
        }

        async fn create_claude_md(&self, content: &str) {
            fs::write(self.project_root.join("CLAUDE.md"), content)
                .await
                .unwrap();
        }

        async fn create_settings_json(&self, content: &str) {
            let claude_dir = self.project_root.join(".claude");
            fs::create_dir_all(&claude_dir).await.unwrap();
            fs::write(claude_dir.join("settings.json"), content)
                .await
                .unwrap();
        }
    }

    #[test]
    fn test_harness_type_returns_claude() {
        let harness = ClaudeCodeHarness;
        assert_eq!(harness.harness_type(), "claude");
    }

    #[tokio::test]
    async fn test_config_paths_returns_paths() {
        let harness = ClaudeCodeHarness;
        let paths = harness.config_paths(None).unwrap();

        // User path should contain "claude"
        let user_str = paths.user.to_string_lossy();
        assert!(
            user_str.contains("claude"),
            "User path should contain 'claude': {}",
            user_str
        );
    }

    #[tokio::test]
    async fn test_config_paths_with_project_root() {
        let test = TestHarness::new().await;
        let harness = ClaudeCodeHarness;
        let paths = harness.config_paths(Some(test.project_root())).unwrap();

        assert!(paths.project.is_some());
        let project = paths.project.unwrap();
        assert!(project.ends_with(".claude"));
    }

    #[tokio::test]
    async fn test_introspect_without_project() {
        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(None).await.unwrap();

        assert_eq!(caps.harness_type, "claude");
        // Project should be None when no project root
        assert!(caps.project.is_none());
    }

    #[tokio::test]
    async fn test_introspect_with_empty_project() {
        let test = TestHarness::new().await;
        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        assert_eq!(caps.harness_type, "claude");
        // No .claude dir exists, so project scope should be None
        assert!(caps.project.is_none());
    }

    #[tokio::test]
    async fn test_introspect_finds_hooks() {
        let test = TestHarness::new().await;
        test.create_hook("pre_tool_use.sh", "#!/bin/bash\necho 'test'")
            .await;

        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        assert!(caps.project.is_some());
        let project = caps.project.unwrap();
        assert!(project.hooks.is_some());
        let hooks = project.hooks.unwrap();
        assert_eq!(hooks.installed_hooks.len(), 1);
    }

    #[tokio::test]
    async fn test_introspect_finds_claude_md() {
        let test = TestHarness::new().await;
        test.create_claude_md("# Project Guidelines").await;

        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        assert!(caps.project.is_some());
        let project = caps.project.unwrap();
        assert!(!project.injection_targets.is_empty());
        assert!(project.injection_targets[0].path.ends_with("CLAUDE.md"));
    }

    #[tokio::test]
    async fn test_introspect_finds_settings_json() {
        let test = TestHarness::new().await;
        test.create_settings_json("{}").await;

        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        assert!(caps.project.is_some());
        let project = caps.project.unwrap();
        assert!(!project.config_files.is_empty());
        assert!(project.config_files[0].path.ends_with("settings.json"));
    }

    #[tokio::test]
    async fn test_introspect_finds_all_capabilities() {
        let test = TestHarness::new().await;
        test.create_claude_md("# Guidelines").await;
        test.create_settings_json("{}").await;
        test.create_hook("pre_tool_use.sh", "#!/bin/bash").await;
        test.create_hook("post_tool_use.sh", "#!/bin/bash").await;

        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        assert!(caps.project.is_some());
        let project = caps.project.unwrap();

        // Should have hooks
        assert!(project.hooks.is_some());
        let hooks = project.hooks.unwrap();
        assert_eq!(hooks.installed_hooks.len(), 2);

        // Should have config files
        assert!(!project.config_files.is_empty());

        // Should have injection targets
        assert!(!project.injection_targets.is_empty());
    }

    #[tokio::test]
    async fn test_introspect_claude_md_in_project_root() {
        let test = TestHarness::new().await;
        // Create CLAUDE.md in project root (not in .claude dir)
        test.create_claude_md("# Guidelines").await;
        // Also create .claude dir so project scope is detected
        test.create_claude_dir().await;

        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        assert!(caps.project.is_some());
        let project = caps.project.unwrap();
        assert!(!project.injection_targets.is_empty());
    }

    #[tokio::test]
    async fn test_introspect_empty_claude_dir() {
        let test = TestHarness::new().await;
        test.create_claude_dir().await;

        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        // Empty .claude dir should result in None project scope
        assert!(caps.project.is_none());
    }

    #[tokio::test]
    async fn test_introspect_empty_hooks_dir() {
        let test = TestHarness::new().await;
        test.create_hooks_dir().await;

        let harness = ClaudeCodeHarness;
        let caps = harness.introspect(Some(test.project_root())).await.unwrap();

        // Empty hooks dir should still be detected
        assert!(caps.project.is_some());
        let project = caps.project.unwrap();
        assert!(project.hooks.is_some());
        assert!(project.hooks.unwrap().installed_hooks.is_empty());
    }
}
