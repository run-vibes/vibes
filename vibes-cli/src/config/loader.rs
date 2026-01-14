use super::types::{
    DEFAULT_HOST, DEFAULT_PORT, ModelsConfigSection, OllamaConfigSection, RawServerConfig,
    RawVibesConfig, ServerConfig, SessionConfig, TunnelConfigSection, VibesConfig,
};
use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;
use vibes_core::AccessConfig;

pub struct ConfigLoader;

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
