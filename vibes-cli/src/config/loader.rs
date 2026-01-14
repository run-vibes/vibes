use super::types::{
    DEFAULT_HOST, DEFAULT_PORT, ModelsConfigSection, OllamaConfigSection, RawServerConfig,
    RawVibesConfig, ServerConfig, SessionConfig, TunnelConfigSection, VibesConfig,
};
use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;
use vibes_core::AccessConfig;

pub struct ConfigLoader;

/// Report of what was saved and any warnings
#[derive(Debug, Default)]
#[allow(dead_code)] // Will be used by setup wizards
pub struct SaveReport {
    /// Values that were saved: (key, value)
    pub saved: Vec<(String, String)>,
    /// Values that were overwritten from non-default: (key, old_value, new_value)
    pub overwritten: Vec<(String, String, String)>,
}

impl std::fmt::Display for SaveReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.saved.is_empty() {
            writeln!(f, "Configuration saved:")?;
            for (key, value) in &self.saved {
                writeln!(f, "  {} = {}", key, value)?;
            }
        }

        if !self.overwritten.is_empty() {
            writeln!(f, "\nWarning: Overwriting existing values:")?;
            for (key, old, new) in &self.overwritten {
                writeln!(f, "  {} = {} -> {}", key, old, new)?;
            }
        }

        Ok(())
    }
}

impl ConfigLoader {
    /// Load merged configuration (user + project)
    pub fn load() -> Result<VibesConfig> {
        let mut raw = RawVibesConfig::default();

        // Layer 1: User config
        if let Some(user_path) = Self::user_config_path()
            && user_path.exists()
        {
            let contents = std::fs::read_to_string(&user_path)?;
            let user_config: RawVibesConfig = toml::from_str(&contents)?;
            raw = Self::merge_raw(raw, user_config);
        }

        // Layer 2: Project config
        let project_path = Self::project_config_path();
        if project_path.exists() {
            let contents = std::fs::read_to_string(&project_path)?;
            let project_config: RawVibesConfig = toml::from_str(&contents)?;
            raw = Self::merge_raw(raw, project_config);
        }

        // Convert to final config with defaults applied
        Ok(Self::finalize(raw))
    }

    /// Get user config path (platform-specific)
    pub fn user_config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "vibes").map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Get project config path
    /// Can be overridden with VIBES_PROJECT_CONFIG_DIR env var (useful for isolated e2e tests)
    pub fn project_config_path() -> PathBuf {
        if let Ok(dir) = std::env::var("VIBES_PROJECT_CONFIG_DIR") {
            PathBuf::from(dir).join("config.toml")
        } else {
            PathBuf::from(".vibes/config.toml")
        }
    }

    /// Merge two raw configs (overlay values override base only if explicitly set)
    fn merge_raw(base: RawVibesConfig, overlay: RawVibesConfig) -> RawVibesConfig {
        RawVibesConfig {
            server: RawServerConfig {
                host: overlay.server.host.or(base.server.host),
                port: overlay.server.port.or(base.server.port),
                auto_start: overlay.server.auto_start.or(base.server.auto_start),
            },
            session: SessionConfig {
                default_model: overlay.session.default_model.or(base.session.default_model),
                default_allowed_tools: overlay
                    .session
                    .default_allowed_tools
                    .or(base.session.default_allowed_tools),
                working_dir: overlay.session.working_dir.or(base.session.working_dir),
            },
            tunnel: TunnelConfigSection {
                enabled: overlay.tunnel.enabled || base.tunnel.enabled,
                mode: overlay.tunnel.mode.or(base.tunnel.mode),
                name: overlay.tunnel.name.or(base.tunnel.name),
                hostname: overlay.tunnel.hostname.or(base.tunnel.hostname),
            },
            models: ModelsConfigSection {
                ollama: OllamaConfigSection {
                    enabled: overlay.models.ollama.enabled || base.models.ollama.enabled,
                    host: overlay.models.ollama.host.or(base.models.ollama.host),
                },
            },
            auth: AccessConfig {
                enabled: overlay.auth.enabled || base.auth.enabled,
                team: if overlay.auth.team.is_empty() {
                    base.auth.team
                } else {
                    overlay.auth.team
                },
                aud: if overlay.auth.aud.is_empty() {
                    base.auth.aud
                } else {
                    overlay.auth.aud
                },
                bypass_localhost: overlay.auth.bypass_localhost,
                clock_skew_seconds: {
                    let default_skew = AccessConfig::default().clock_skew_seconds;
                    if overlay.auth.clock_skew_seconds != default_skew {
                        overlay.auth.clock_skew_seconds
                    } else {
                        base.auth.clock_skew_seconds
                    }
                },
            },
        }
    }

    /// Convert raw config to final config with defaults applied
    fn finalize(raw: RawVibesConfig) -> VibesConfig {
        VibesConfig {
            server: ServerConfig {
                host: raw.server.host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
                port: raw.server.port.unwrap_or(DEFAULT_PORT),
                auto_start: raw.server.auto_start.unwrap_or(true),
            },
            session: raw.session,
            tunnel: raw.tunnel,
            models: raw.models,
            auth: raw.auth,
        }
    }

    /// Save config to user config path (~/.config/vibes/config.toml or similar)
    ///
    /// Returns a `SaveReport` showing what was saved and any warnings.
    #[allow(dead_code)] // Will be used by setup wizards
    pub fn save_to_user_config(config: &VibesConfig) -> Result<SaveReport> {
        let path = Self::user_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine user config path"))?;
        Self::save_with_report(config, &path)
    }

    /// Save config to a specific path
    ///
    /// Creates parent directories if they don't exist.
    #[allow(dead_code)] // Will be used by setup wizards
    pub fn save_to_path(config: &VibesConfig, path: &std::path::Path) -> Result<()> {
        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(config)?;
        std::fs::write(path, toml)?;

        Ok(())
    }

    /// Save config with a report of what was saved and any warnings
    ///
    /// Returns a `SaveReport` containing:
    /// - `saved`: Non-default values that were saved
    /// - `overwritten`: Warnings for overwriting existing non-default values
    pub fn save_with_report(config: &VibesConfig, path: &std::path::Path) -> Result<SaveReport> {
        let defaults = VibesConfig::default();
        let mut report = SaveReport::default();

        // Load existing config if file exists
        let existing = if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Some(toml::from_str::<VibesConfig>(&contents)?)
        } else {
            None
        };

        // Collect non-default values that will be saved
        Self::collect_saved_values(config, &defaults, &mut report.saved);

        // Check for overwrites of non-default values
        if let Some(ref existing) = existing {
            Self::collect_overwritten_values(config, existing, &defaults, &mut report.overwritten);
        }

        // Save the config
        Self::save_to_path(config, path)?;

        Ok(report)
    }

    /// Collect values that differ from defaults
    fn collect_saved_values(
        config: &VibesConfig,
        defaults: &VibesConfig,
        saved: &mut Vec<(String, String)>,
    ) {
        // Server section
        if config.server.host != defaults.server.host {
            saved.push((
                "server.host".to_string(),
                format!("\"{}\"", config.server.host),
            ));
        }
        if config.server.port != defaults.server.port {
            saved.push(("server.port".to_string(), config.server.port.to_string()));
        }
        if config.server.auto_start != defaults.server.auto_start {
            saved.push((
                "server.auto_start".to_string(),
                config.server.auto_start.to_string(),
            ));
        }

        // Tunnel section
        if config.tunnel.enabled != defaults.tunnel.enabled {
            saved.push((
                "tunnel.enabled".to_string(),
                config.tunnel.enabled.to_string(),
            ));
        }
        if config.tunnel.mode != defaults.tunnel.mode
            && let Some(ref mode) = config.tunnel.mode
        {
            saved.push(("tunnel.mode".to_string(), format!("\"{}\"", mode)));
        }
        if config.tunnel.name != defaults.tunnel.name
            && let Some(ref name) = config.tunnel.name
        {
            saved.push(("tunnel.name".to_string(), format!("\"{}\"", name)));
        }
        if config.tunnel.hostname != defaults.tunnel.hostname
            && let Some(ref hostname) = config.tunnel.hostname
        {
            saved.push(("tunnel.hostname".to_string(), format!("\"{}\"", hostname)));
        }

        // Auth section
        if config.auth.enabled != defaults.auth.enabled {
            saved.push(("auth.enabled".to_string(), config.auth.enabled.to_string()));
        }
    }

    /// Collect values being overwritten that were non-default in existing config
    fn collect_overwritten_values(
        config: &VibesConfig,
        existing: &VibesConfig,
        defaults: &VibesConfig,
        overwritten: &mut Vec<(String, String, String)>,
    ) {
        // Only warn if: existing != default AND existing != new
        // Server section
        if existing.server.port != defaults.server.port
            && existing.server.port != config.server.port
        {
            overwritten.push((
                "server.port".to_string(),
                existing.server.port.to_string(),
                config.server.port.to_string(),
            ));
        }
        if existing.server.host != defaults.server.host
            && existing.server.host != config.server.host
        {
            overwritten.push((
                "server.host".to_string(),
                existing.server.host.clone(),
                config.server.host.clone(),
            ));
        }

        // Tunnel section
        if existing.tunnel.enabled != defaults.tunnel.enabled
            && existing.tunnel.enabled != config.tunnel.enabled
        {
            overwritten.push((
                "tunnel.enabled".to_string(),
                existing.tunnel.enabled.to_string(),
                config.tunnel.enabled.to_string(),
            ));
        }
    }

    /// Load config from a specific path (for testing)
    #[cfg(test)]
    pub fn load_from_path(path: &std::path::Path) -> Result<VibesConfig> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(VibesConfig::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // ==================== Save Tests ====================

    #[test]
    fn test_save_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("config.toml");

        let config = VibesConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                auto_start: false,
            },
            ..Default::default()
        };

        ConfigLoader::save_to_path(&config, &path).unwrap();

        assert!(path.exists());
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("host = \"0.0.0.0\""));
        assert!(contents.contains("port = 8080"));
        assert!(contents.contains("auto_start = false"));
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir
            .path()
            .join("nested")
            .join("deep")
            .join("config.toml");

        let config = VibesConfig::default();

        ConfigLoader::save_to_path(&config, &path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_save_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("config.toml");

        // Create initial file
        std::fs::write(&path, "[server]\nport = 1234\n").unwrap();

        let config = VibesConfig {
            server: ServerConfig {
                port: 9999,
                ..Default::default()
            },
            ..Default::default()
        };

        ConfigLoader::save_to_path(&config, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("port = 9999"));
        assert!(!contents.contains("port = 1234"));
    }

    #[test]
    fn test_save_with_report_shows_saved_values() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("config.toml");

        let config = VibesConfig {
            tunnel: TunnelConfigSection {
                enabled: true,
                mode: Some("quick".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let report = ConfigLoader::save_with_report(&config, &path).unwrap();

        assert!(
            report
                .saved
                .contains(&("tunnel.enabled".to_string(), "true".to_string()))
        );
        assert!(
            report
                .saved
                .contains(&("tunnel.mode".to_string(), "\"quick\"".to_string()))
        );
    }

    #[test]
    fn test_save_with_report_detects_overwritten_values() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("config.toml");

        // Create initial config with non-default value
        let initial = VibesConfig {
            server: ServerConfig {
                port: 9000, // Non-default (default is 7432)
                ..Default::default()
            },
            ..Default::default()
        };
        ConfigLoader::save_to_path(&initial, &path).unwrap();

        // Save new config with different port
        let new_config = VibesConfig {
            server: ServerConfig {
                port: 8080,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = ConfigLoader::save_with_report(&new_config, &path).unwrap();

        // Should warn about overwriting the non-default port
        assert!(
            report
                .overwritten
                .iter()
                .any(|(key, old, new)| key == "server.port" && old == "9000" && new == "8080")
        );
    }

    #[test]
    fn test_save_with_report_no_warning_for_default_values() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("config.toml");

        // Create initial config with DEFAULT port
        let initial = VibesConfig::default();
        ConfigLoader::save_to_path(&initial, &path).unwrap();

        // Save new config with different port
        let new_config = VibesConfig {
            server: ServerConfig {
                port: 8080,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = ConfigLoader::save_with_report(&new_config, &path).unwrap();

        // Should NOT warn because old value was the default
        assert!(
            report.overwritten.is_empty(),
            "Expected no warnings for overwriting default values"
        );
    }

    #[test]
    fn test_save_report_display() {
        let report = SaveReport {
            saved: vec![
                ("tunnel.enabled".to_string(), "true".to_string()),
                ("tunnel.mode".to_string(), "\"quick\"".to_string()),
            ],
            overwritten: vec![(
                "server.port".to_string(),
                "9000".to_string(),
                "8080".to_string(),
            )],
        };

        let display = report.to_string();

        assert!(display.contains("tunnel.enabled = true"));
        assert!(display.contains("tunnel.mode = \"quick\""));
        assert!(display.contains("server.port"));
        assert!(display.contains("9000"));
        assert!(display.contains("8080"));
    }

    #[test]
    fn test_save_to_user_config_returns_report() {
        // This test verifies save_to_user_config exists and returns a SaveReport
        // We can't easily test the actual user config path, so we verify the method signature
        let config = VibesConfig::default();

        // The method should exist and return Result<SaveReport>
        // In actual use, it would write to ~/.config/vibes/config.toml or similar
        let result = ConfigLoader::save_to_user_config(&config);

        // Should succeed (creates the config directory if needed)
        assert!(result.is_ok());
    }

    // ==================== Load Tests ====================

    #[test]
    fn test_load_nonexistent_returns_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.toml");

        let config = ConfigLoader::load_from_path(&path).unwrap();

        assert_eq!(config.server.port, 7432);
        assert!(config.server.auto_start);
    }

    #[test]
    fn test_load_from_valid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("config.toml");

        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            r#"
[server]
port = 9999
auto_start = false

[session]
default_model = "claude-sonnet-4"
"#
        )
        .unwrap();

        let config = ConfigLoader::load_from_path(&path).unwrap();

        assert_eq!(config.server.port, 9999);
        assert!(!config.server.auto_start);
        assert_eq!(
            config.session.default_model,
            Some("claude-sonnet-4".to_string())
        );
    }

    #[test]
    fn test_load_invalid_toml_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.toml");

        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "this is not valid toml {{{{").unwrap();

        let result = ConfigLoader::load_from_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_raw_overlay_overrides_base() {
        // Test that overlay values override base, but None in overlay preserves base
        let base = RawVibesConfig {
            server: RawServerConfig {
                host: Some("127.0.0.1".to_string()),
                port: Some(7432),
                auto_start: Some(true),
            },
            session: SessionConfig {
                default_model: Some("base-model".to_string()),
                default_allowed_tools: Some(vec!["Read".to_string()]),
                working_dir: None,
            },
            tunnel: TunnelConfigSection::default(),
            models: ModelsConfigSection::default(),
            auth: AccessConfig::default(),
        };

        let overlay = RawVibesConfig {
            server: RawServerConfig {
                host: Some("0.0.0.0".to_string()),
                port: Some(8080),
                auto_start: Some(false),
            },
            session: SessionConfig {
                default_model: Some("overlay-model".to_string()),
                default_allowed_tools: None, // Should preserve base value
                working_dir: Some(PathBuf::from("/custom")),
            },
            tunnel: TunnelConfigSection::default(),
            models: ModelsConfigSection::default(),
            auth: AccessConfig::default(),
        };

        let merged = ConfigLoader::merge_raw(base, overlay);

        assert_eq!(merged.server.host, Some("0.0.0.0".to_string()));
        assert_eq!(merged.server.port, Some(8080));
        assert_eq!(merged.server.auto_start, Some(false));
        assert_eq!(
            merged.session.default_model,
            Some("overlay-model".to_string())
        );
        // overlay's None falls through to base value via .or()
        assert_eq!(
            merged.session.default_allowed_tools,
            Some(vec!["Read".to_string()])
        );
        assert_eq!(merged.session.working_dir, Some(PathBuf::from("/custom")));
    }

    #[test]
    fn test_merge_raw_none_preserves_base() {
        // Test that None values in overlay don't override base values
        let base = RawVibesConfig {
            server: RawServerConfig {
                host: Some("0.0.0.0".to_string()),
                port: Some(9000),
                auto_start: Some(false),
            },
            session: SessionConfig::default(),
            tunnel: TunnelConfigSection::default(),
            models: ModelsConfigSection::default(),
            auth: AccessConfig::default(),
        };

        let overlay = RawVibesConfig {
            server: RawServerConfig {
                host: None,       // Should preserve base
                port: None,       // Should preserve base
                auto_start: None, // Should preserve base
            },
            session: SessionConfig::default(),
            tunnel: TunnelConfigSection::default(),
            models: ModelsConfigSection::default(),
            auth: AccessConfig::default(),
        };

        let merged = ConfigLoader::merge_raw(base, overlay);

        // Base values preserved when overlay has None
        assert_eq!(merged.server.host, Some("0.0.0.0".to_string()));
        assert_eq!(merged.server.port, Some(9000));
        assert_eq!(merged.server.auto_start, Some(false));
    }

    #[test]
    fn test_user_config_path_returns_some() {
        // This should always return Some on platforms with home directories
        let path = ConfigLoader::user_config_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("vibes"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_project_config_path() {
        let path = ConfigLoader::project_config_path();
        assert_eq!(path, PathBuf::from(".vibes/config.toml"));
    }
}
