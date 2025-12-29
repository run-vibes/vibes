//! Claude Code detection utilities for discovering capabilities

use crate::{
    ConfigFile, ConfigFormat, HookCapabilities, HookType, InjectionScope, InjectionTarget,
    InstalledHook, ScopedCapabilities,
};
use std::path::Path;
use tokio::fs;

/// Detect capabilities at a specific scope level
pub async fn detect_scope(base_path: &Path, scope: InjectionScope) -> Option<ScopedCapabilities> {
    if !base_path.exists() {
        return None;
    }

    let hooks = detect_hooks(base_path).await;
    let config_files = detect_config_files(base_path).await;
    let injection_targets = detect_injection_targets(base_path, scope.clone()).await;

    // Only return Some if there's something meaningful
    if hooks.is_none() && config_files.is_empty() && injection_targets.is_empty() {
        return None;
    }

    Some(ScopedCapabilities {
        hooks,
        config_files,
        injection_targets,
    })
}

/// Detect hook capabilities at a base path
pub async fn detect_hooks(base_path: &Path) -> Option<HookCapabilities> {
    let hooks_dir = base_path.join("hooks");

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

/// Detect config files at a base path
pub async fn detect_config_files(base_path: &Path) -> Vec<ConfigFile> {
    let mut config_files = Vec::new();

    // Check for settings.json (Claude Code's main config)
    let settings_path = base_path.join("settings.json");
    if settings_path.exists() {
        config_files.push(ConfigFile {
            path: settings_path.clone(),
            format: ConfigFormat::Json,
            writable: is_writable(&settings_path).await,
        });
    }

    // Check for .clauderc (if supported)
    let clauderc_path = base_path.join(".clauderc");
    if clauderc_path.exists() {
        config_files.push(ConfigFile {
            path: clauderc_path.clone(),
            format: ConfigFormat::Json,
            writable: is_writable(&clauderc_path).await,
        });
    }

    config_files
}

/// Detect injection targets at a base path for a specific scope
pub async fn detect_injection_targets(
    base_path: &Path,
    scope: InjectionScope,
) -> Vec<InjectionTarget> {
    let mut targets = Vec::new();

    // CLAUDE.md is the primary injection target
    let claude_md_path = base_path.join("CLAUDE.md");
    if claude_md_path.exists() {
        targets.push(InjectionTarget {
            path: claude_md_path.clone(),
            format: ConfigFormat::Markdown,
            writable: is_writable(&claude_md_path).await,
            scope: scope.clone(),
        });
    }

    // Also check parent directory for project scope
    if matches!(scope, InjectionScope::Project) {
        // For project scope, also check the parent (project root) for CLAUDE.md
        if let Some(parent) = base_path.parent() {
            let project_claude_md = parent.join("CLAUDE.md");
            if project_claude_md.exists() && project_claude_md != claude_md_path {
                targets.push(InjectionTarget {
                    path: project_claude_md.clone(),
                    format: ConfigFormat::Markdown,
                    writable: is_writable(&project_claude_md).await,
                    scope: scope.clone(),
                });
            }
        }
    }

    targets
}

/// Find installed hooks in a hooks directory
pub async fn find_installed_hooks(hooks_dir: &Path) -> Vec<InstalledHook> {
    let mut hooks = Vec::new();

    let Ok(mut entries) = fs::read_dir(hooks_dir).await else {
        return hooks;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        // Skip non-files
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        // Determine hook type from filename prefix
        let hook_type = if file_name.starts_with("pre_tool_use")
            || file_name.starts_with("pre-tool-use")
        {
            Some(HookType::PreToolUse)
        } else if file_name.starts_with("post_tool_use") || file_name.starts_with("post-tool-use") {
            Some(HookType::PostToolUse)
        } else if file_name.starts_with("stop") {
            Some(HookType::Stop)
        } else if file_name.starts_with("notification") {
            Some(HookType::Notification)
        } else {
            None
        };

        if let Some(hook_type) = hook_type {
            hooks.push(InstalledHook {
                hook_type,
                name: file_name.to_string(),
                path,
            });
        }
    }

    hooks
}

/// Check if a path is writable
pub async fn is_writable(path: &Path) -> bool {
    // Try to open the file for writing without truncating
    fs::OpenOptions::new()
        .write(true)
        .create(false)
        .open(path)
        .await
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    #[tokio::test]
    async fn test_detect_scope_returns_none_for_nonexistent_path() {
        let result = detect_scope(Path::new("/nonexistent/path"), InjectionScope::User).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_detect_scope_returns_none_for_empty_dir() {
        let temp = create_test_dir().await;
        let result = detect_scope(temp.path(), InjectionScope::User).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_detect_scope_finds_hooks() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::write(hooks_dir.join("pre_tool_use.sh"), "#!/bin/bash")
            .await
            .unwrap();

        let result = detect_scope(temp.path(), InjectionScope::User).await;
        assert!(result.is_some());
        let scoped = result.unwrap();
        assert!(scoped.hooks.is_some());
        assert_eq!(scoped.hooks.as_ref().unwrap().installed_hooks.len(), 1);
    }

    #[tokio::test]
    async fn test_detect_scope_finds_config_files() {
        let temp = create_test_dir().await;
        fs::write(temp.path().join("settings.json"), "{}")
            .await
            .unwrap();

        let result = detect_scope(temp.path(), InjectionScope::User).await;
        assert!(result.is_some());
        let scoped = result.unwrap();
        assert_eq!(scoped.config_files.len(), 1);
        assert_eq!(scoped.config_files[0].format, ConfigFormat::Json);
    }

    #[tokio::test]
    async fn test_detect_scope_finds_injection_targets() {
        let temp = create_test_dir().await;
        fs::write(temp.path().join("CLAUDE.md"), "# Test")
            .await
            .unwrap();

        let result = detect_scope(temp.path(), InjectionScope::User).await;
        assert!(result.is_some());
        let scoped = result.unwrap();
        assert_eq!(scoped.injection_targets.len(), 1);
        assert_eq!(scoped.injection_targets[0].format, ConfigFormat::Markdown);
    }

    #[tokio::test]
    async fn test_detect_hooks_returns_none_without_hooks_dir() {
        let temp = create_test_dir().await;
        let result = detect_hooks(temp.path()).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_detect_hooks_returns_some_with_hooks_dir() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();

        let result = detect_hooks(temp.path()).await;
        assert!(result.is_some());
        let hooks = result.unwrap();
        assert_eq!(hooks.hooks_dir, Some(hooks_dir));
        assert_eq!(hooks.supported_types.len(), 4);
    }

    #[tokio::test]
    async fn test_detect_config_files_finds_settings_json() {
        let temp = create_test_dir().await;
        fs::write(temp.path().join("settings.json"), "{}")
            .await
            .unwrap();

        let files = detect_config_files(temp.path()).await;
        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with("settings.json"));
        assert_eq!(files[0].format, ConfigFormat::Json);
    }

    #[tokio::test]
    async fn test_detect_config_files_finds_clauderc() {
        let temp = create_test_dir().await;
        fs::write(temp.path().join(".clauderc"), "{}")
            .await
            .unwrap();

        let files = detect_config_files(temp.path()).await;
        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with(".clauderc"));
    }

    #[tokio::test]
    async fn test_detect_config_files_returns_empty_when_none() {
        let temp = create_test_dir().await;
        let files = detect_config_files(temp.path()).await;
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_detect_injection_targets_finds_claude_md() {
        let temp = create_test_dir().await;
        fs::write(temp.path().join("CLAUDE.md"), "# Test")
            .await
            .unwrap();

        let targets = detect_injection_targets(temp.path(), InjectionScope::User).await;
        assert_eq!(targets.len(), 1);
        assert!(targets[0].path.ends_with("CLAUDE.md"));
        assert_eq!(targets[0].scope, InjectionScope::User);
    }

    #[tokio::test]
    async fn test_detect_injection_targets_returns_empty_when_none() {
        let temp = create_test_dir().await;
        let targets = detect_injection_targets(temp.path(), InjectionScope::User).await;
        assert!(targets.is_empty());
    }

    #[tokio::test]
    async fn test_find_installed_hooks_empty_dir() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert!(hooks.is_empty());
    }

    #[tokio::test]
    async fn test_find_installed_hooks_finds_pre_tool_use() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::write(hooks_dir.join("pre_tool_use.sh"), "#!/bin/bash")
            .await
            .unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].hook_type, HookType::PreToolUse);
        assert_eq!(hooks[0].name, "pre_tool_use");
    }

    #[tokio::test]
    async fn test_find_installed_hooks_finds_post_tool_use() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::write(hooks_dir.join("post-tool-use.py"), "#!/usr/bin/env python")
            .await
            .unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].hook_type, HookType::PostToolUse);
        assert_eq!(hooks[0].name, "post-tool-use");
    }

    #[tokio::test]
    async fn test_find_installed_hooks_finds_stop() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::write(hooks_dir.join("stop.sh"), "#!/bin/bash")
            .await
            .unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].hook_type, HookType::Stop);
    }

    #[tokio::test]
    async fn test_find_installed_hooks_finds_notification() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::write(hooks_dir.join("notification_handler.sh"), "#!/bin/bash")
            .await
            .unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].hook_type, HookType::Notification);
    }

    #[tokio::test]
    async fn test_find_installed_hooks_ignores_unknown_files() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::write(hooks_dir.join("unknown.sh"), "#!/bin/bash")
            .await
            .unwrap();
        fs::write(hooks_dir.join("README.md"), "# Hooks")
            .await
            .unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert!(hooks.is_empty());
    }

    #[tokio::test]
    async fn test_find_installed_hooks_finds_multiple() {
        let temp = create_test_dir().await;
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).await.unwrap();
        fs::write(hooks_dir.join("pre_tool_use.sh"), "#!/bin/bash")
            .await
            .unwrap();
        fs::write(hooks_dir.join("post_tool_use.sh"), "#!/bin/bash")
            .await
            .unwrap();
        fs::write(hooks_dir.join("stop.sh"), "#!/bin/bash")
            .await
            .unwrap();

        let hooks = find_installed_hooks(&hooks_dir).await;
        assert_eq!(hooks.len(), 3);
    }

    #[tokio::test]
    async fn test_is_writable_returns_true_for_writable_file() {
        let temp = create_test_dir().await;
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test").await.unwrap();

        assert!(is_writable(&file_path).await);
    }

    #[tokio::test]
    async fn test_is_writable_returns_false_for_nonexistent_file() {
        let temp = create_test_dir().await;
        let file_path = temp.path().join("nonexistent.txt");

        assert!(!is_writable(&file_path).await);
    }
}
