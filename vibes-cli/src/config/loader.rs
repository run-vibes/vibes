use super::types::VibesConfig;
use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

pub struct ConfigLoader;

impl ConfigLoader {
    /// Load merged configuration (user + project)
    pub fn load() -> Result<VibesConfig> {
        let mut config = VibesConfig::default();

        // Layer 1: User config
        if let Some(user_path) = Self::user_config_path()
            && user_path.exists()
        {
            let contents = std::fs::read_to_string(&user_path)?;
            let user_config: VibesConfig = toml::from_str(&contents)?;
            config = Self::merge(config, user_config);
        }

        // Layer 2: Project config
        let project_path = Self::project_config_path();
        if project_path.exists() {
            let contents = std::fs::read_to_string(&project_path)?;
            let project_config: VibesConfig = toml::from_str(&contents)?;
            config = Self::merge(config, project_config);
        }

        Ok(config)
    }

    /// Get user config path (platform-specific)
    pub fn user_config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "vibes").map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Get project config path
    pub fn project_config_path() -> PathBuf {
        PathBuf::from(".vibes/config.toml")
    }

    /// Merge two configs (overlay values override base)
    fn merge(base: VibesConfig, overlay: VibesConfig) -> VibesConfig {
        VibesConfig {
            server: super::types::ServerConfig {
                port: overlay.server.port,
                auto_start: overlay.server.auto_start,
            },
            session: super::types::SessionConfig {
                default_model: overlay.session.default_model.or(base.session.default_model),
                default_allowed_tools: overlay
                    .session
                    .default_allowed_tools
                    .or(base.session.default_allowed_tools),
                working_dir: overlay.session.working_dir.or(base.session.working_dir),
            },
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
    fn test_merge_overlay_overrides_base() {
        let base = VibesConfig {
            server: super::super::types::ServerConfig {
                port: 7432,
                auto_start: true,
            },
            session: super::super::types::SessionConfig {
                default_model: Some("base-model".to_string()),
                default_allowed_tools: Some(vec!["Read".to_string()]),
                working_dir: None,
            },
        };

        let overlay = VibesConfig {
            server: super::super::types::ServerConfig {
                port: 8080,
                auto_start: false,
            },
            session: super::super::types::SessionConfig {
                default_model: Some("overlay-model".to_string()),
                default_allowed_tools: None,
                working_dir: Some(PathBuf::from("/custom")),
            },
        };

        let merged = ConfigLoader::merge(base, overlay);

        assert_eq!(merged.server.port, 8080);
        assert!(!merged.server.auto_start);
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
