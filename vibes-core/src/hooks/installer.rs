//! Hook installer - installs vibes hooks into Claude Code configuration
//!
//! This module handles:
//! - Writing hook scripts to ~/.claude/hooks/vibes/
//! - Updating ~/.claude/settings.json to register the hooks

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tracing::{debug, info};

use super::scripts;

/// Error type for hook installation
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("Failed to determine home directory")]
    NoHomeDir,

    #[error("Failed to create hooks directory: {0}")]
    CreateDir(std::io::Error),

    #[error("Failed to write script {0}: {1}")]
    WriteScript(String, std::io::Error),

    #[error("Failed to set script permissions: {0}")]
    SetPermissions(std::io::Error),

    #[error("Failed to read settings.json: {0}")]
    ReadSettings(std::io::Error),

    #[error("Failed to parse settings.json: {0}")]
    ParseSettings(String),

    #[error("Failed to write settings.json: {0}")]
    WriteSettings(std::io::Error),
}

/// Hook installer configuration
#[derive(Debug, Clone)]
pub struct HookInstallerConfig {
    /// Claude config directory (default: ~/.claude)
    pub claude_dir: Option<PathBuf>,
    /// Whether to overwrite existing scripts
    pub overwrite: bool,
}

impl Default for HookInstallerConfig {
    fn default() -> Self {
        Self {
            claude_dir: None,
            overwrite: true,
        }
    }
}

/// Installs vibes hooks into Claude Code
pub struct HookInstaller {
    config: HookInstallerConfig,
}

impl HookInstaller {
    /// Create a new hook installer
    pub fn new(config: HookInstallerConfig) -> Self {
        Self { config }
    }

    /// Get the Claude config directory
    fn claude_dir(&self) -> Result<PathBuf, InstallError> {
        if let Some(dir) = &self.config.claude_dir {
            return Ok(dir.clone());
        }

        dirs::home_dir()
            .map(|h| h.join(".claude"))
            .ok_or(InstallError::NoHomeDir)
    }

    /// Get the hooks directory
    fn hooks_dir(&self) -> Result<PathBuf, InstallError> {
        Ok(self.claude_dir()?.join("hooks").join("vibes"))
    }

    /// Install all hook scripts
    pub fn install_scripts(&self) -> Result<PathBuf, InstallError> {
        let hooks_dir = self.hooks_dir()?;

        // Create hooks directory
        fs::create_dir_all(&hooks_dir).map_err(InstallError::CreateDir)?;
        debug!("Created hooks directory: {:?}", hooks_dir);

        // Write each script
        for (name, content) in scripts::SCRIPTS {
            let script_path = hooks_dir.join(name);

            if script_path.exists() && !self.config.overwrite {
                debug!("Skipping existing script: {:?}", script_path);
                continue;
            }

            fs::write(&script_path, content)
                .map_err(|e| InstallError::WriteScript(name.to_string(), e))?;

            // Make executable (Unix only)
            #[cfg(unix)]
            {
                let mut perms = fs::metadata(&script_path)
                    .map_err(InstallError::SetPermissions)?
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms).map_err(InstallError::SetPermissions)?;
            }

            debug!("Installed hook script: {:?}", script_path);
        }

        info!(
            "Installed {} hook scripts to {:?}",
            scripts::SCRIPTS.len(),
            hooks_dir
        );
        Ok(hooks_dir)
    }

    /// Update settings.json to register hooks
    ///
    /// Claude Code's settings.json uses hooks as an object where:
    /// - Each key is a hook type (e.g., "PreToolUse", "SessionStart")
    /// - Each value is an array of hook configurations
    pub fn update_settings(&self, hooks_dir: &Path) -> Result<(), InstallError> {
        let settings_path = self.claude_dir()?.join("settings.json");

        // Read existing settings or create new
        let mut settings: serde_json::Value = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path).map_err(InstallError::ReadSettings)?;
            serde_json::from_str(&content)
                .map_err(|e| InstallError::ParseSettings(e.to_string()))?
        } else {
            serde_json::json!({})
        };

        // Ensure settings is an object
        let settings_obj = settings.as_object_mut().ok_or_else(|| {
            InstallError::ParseSettings("settings.json is not an object".to_string())
        })?;

        // Get or create hooks object (Claude Code uses object format, not array)
        let hooks = settings_obj
            .entry("hooks")
            .or_insert_with(|| serde_json::json!({}));

        let hooks_obj = hooks
            .as_object_mut()
            .ok_or_else(|| InstallError::ParseSettings("hooks is not an object".to_string()))?;

        // Hook configurations to add
        let vibes_hooks = [
            ("PreToolUse", "pre-tool-use.sh"),
            ("PostToolUse", "post-tool-use.sh"),
            ("Stop", "stop.sh"),
            ("SessionStart", "session-start.sh"),
            ("UserPromptSubmit", "user-prompt-submit.sh"),
        ];

        for (hook_type, script_name) in vibes_hooks {
            let script_path = hooks_dir.join(script_name);
            let script_path_str = script_path.to_string_lossy().to_string();

            // Get or create array for this hook type
            let hook_type_array = hooks_obj
                .entry(hook_type)
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .ok_or_else(|| {
                    InstallError::ParseSettings(format!("hooks.{} is not an array", hook_type))
                })?;

            // Check if vibes hook already exists for this type
            let vibes_exists = hook_type_array.iter().any(|h| {
                h.get("hooks")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| {
                        arr.iter().any(|cmd| {
                            cmd.get("command")
                                .and_then(|c| c.as_str())
                                .map(|c| c.contains("vibes"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            });

            if vibes_exists {
                debug!("Hook {} already registered for vibes", hook_type);
                continue;
            }

            // Add new hook configuration
            let hook_config = serde_json::json!({
                "hooks": [{
                    "type": "command",
                    "command": script_path_str
                }]
            });

            hook_type_array.push(hook_config);
            debug!("Added hook configuration for {}", hook_type);
        }

        // Write updated settings
        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| InstallError::ParseSettings(e.to_string()))?;
        fs::write(&settings_path, content).map_err(InstallError::WriteSettings)?;

        info!("Updated settings.json with vibes hooks");
        Ok(())
    }

    /// Install hooks and update settings
    pub fn install(&self) -> Result<(), InstallError> {
        let hooks_dir = self.install_scripts()?;
        self.update_settings(&hooks_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_install_scripts() {
        let temp_dir = TempDir::new().unwrap();
        let claude_dir = temp_dir.path().join(".claude");

        let installer = HookInstaller::new(HookInstallerConfig {
            claude_dir: Some(claude_dir.clone()),
            overwrite: true,
        });

        let hooks_dir = installer.install_scripts().unwrap();
        assert!(hooks_dir.exists());

        // Check all scripts exist
        for (name, _) in scripts::SCRIPTS {
            let script_path = hooks_dir.join(name);
            assert!(script_path.exists(), "Script {} should exist", name);

            // Check executable on Unix
            #[cfg(unix)]
            {
                let perms = fs::metadata(&script_path).unwrap().permissions();
                assert!(
                    perms.mode() & 0o111 != 0,
                    "Script {} should be executable",
                    name
                );
            }
        }
    }

    #[test]
    fn test_update_settings_creates_new() {
        let temp_dir = TempDir::new().unwrap();
        let claude_dir = temp_dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let installer = HookInstaller::new(HookInstallerConfig {
            claude_dir: Some(claude_dir.clone()),
            overwrite: true,
        });

        let hooks_dir = claude_dir.join("hooks").join("vibes");
        fs::create_dir_all(&hooks_dir).unwrap();

        installer.update_settings(&hooks_dir).unwrap();

        let settings_path = claude_dir.join("settings.json");
        assert!(settings_path.exists());

        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();

        // hooks is an object with 5 hook types as keys
        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        assert_eq!(hooks.len(), 5); // PreToolUse, PostToolUse, Stop, SessionStart, UserPromptSubmit
        assert!(hooks.contains_key("PreToolUse"));
        assert!(hooks.contains_key("PostToolUse"));
        assert!(hooks.contains_key("Stop"));
        assert!(hooks.contains_key("SessionStart"));
        assert!(hooks.contains_key("UserPromptSubmit"));
    }

    #[test]
    fn test_update_settings_preserves_existing() {
        let temp_dir = TempDir::new().unwrap();
        let claude_dir = temp_dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        // Write existing settings with hooks as object (Claude Code's real format)
        let existing = serde_json::json!({
            "some_setting": "value",
            "hooks": {
                "OtherHook": [{
                    "matcher": "some_matcher",
                    "hooks": [{"type": "command", "command": "other-script.sh"}]
                }]
            }
        });
        fs::write(
            claude_dir.join("settings.json"),
            serde_json::to_string(&existing).unwrap(),
        )
        .unwrap();

        let installer = HookInstaller::new(HookInstallerConfig {
            claude_dir: Some(claude_dir.clone()),
            overwrite: true,
        });

        let hooks_dir = claude_dir.join("hooks").join("vibes");
        fs::create_dir_all(&hooks_dir).unwrap();

        installer.update_settings(&hooks_dir).unwrap();

        let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Check existing setting preserved
        assert_eq!(settings.get("some_setting").unwrap(), "value");

        // Check hooks object now has 6 keys (1 existing OtherHook + 5 vibes)
        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        assert_eq!(hooks.len(), 6);
        assert!(hooks.contains_key("OtherHook"));
        assert!(hooks.contains_key("PreToolUse"));
    }

    #[test]
    fn test_install_skips_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        let claude_dir = temp_dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let installer = HookInstaller::new(HookInstallerConfig {
            claude_dir: Some(claude_dir.clone()),
            overwrite: true,
        });

        let hooks_dir = claude_dir.join("hooks").join("vibes");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Install twice
        installer.update_settings(&hooks_dir).unwrap();
        installer.update_settings(&hooks_dir).unwrap();

        let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();

        // hooks is an object with hook types as keys
        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        // Each hook type should have exactly 1 entry (no duplicates)
        for (_hook_type, entries) in hooks {
            assert_eq!(entries.as_array().unwrap().len(), 1);
        }
    }

    #[test]
    fn test_update_settings_handles_object_hooks_format() {
        // Claude Code's actual settings.json uses hooks as an OBJECT, not an array
        // Each key is a hook type, each value is an array of hook configs
        let temp_dir = TempDir::new().unwrap();
        let claude_dir = temp_dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        // Write existing settings with hooks as object (Claude Code's real format)
        let existing = serde_json::json!({
            "some_setting": "value",
            "hooks": {
                "Notification": [{
                    "matcher": "idle_prompt",
                    "hooks": [{"type": "command", "command": "notify-send 'test'"}]
                }]
            }
        });
        fs::write(
            claude_dir.join("settings.json"),
            serde_json::to_string(&existing).unwrap(),
        )
        .unwrap();

        let installer = HookInstaller::new(HookInstallerConfig {
            claude_dir: Some(claude_dir.clone()),
            overwrite: true,
        });

        let hooks_dir = claude_dir.join("hooks").join("vibes");
        fs::create_dir_all(&hooks_dir).unwrap();

        // This should NOT fail - it should handle object format
        installer.update_settings(&hooks_dir).unwrap();

        let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Check existing setting preserved
        assert_eq!(settings.get("some_setting").unwrap(), "value");

        // hooks should still be an object
        let hooks = settings.get("hooks").unwrap().as_object().unwrap();

        // Original Notification hook should be preserved
        assert!(hooks.contains_key("Notification"));
        let notification = hooks.get("Notification").unwrap().as_array().unwrap();
        assert_eq!(notification.len(), 1);

        // vibes hooks should be added under their respective types
        assert!(hooks.contains_key("PreToolUse"), "PreToolUse should exist");
        assert!(
            hooks.contains_key("SessionStart"),
            "SessionStart should exist"
        );
    }
}
