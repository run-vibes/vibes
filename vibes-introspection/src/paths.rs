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

    #[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
    fn system_config_dir(_harness: &str) -> Option<PathBuf> {
        None // No standard system config location for other Unix systems
    }

    #[cfg(windows)]
    fn system_config_dir(harness: &str) -> Option<PathBuf> {
        std::env::var("PROGRAMDATA")
            .ok()
            .map(|d| PathBuf::from(d).join(harness))
            .filter(|p| p.exists())
    }
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

    #[test]
    fn test_resolve_system_path_none_when_not_exists() {
        // System path should be None when /etc/claude doesn't exist
        let paths = ConfigPaths::resolve("nonexistent_harness_xyz", None).unwrap();
        assert!(paths.system.is_none(), "System path should be None for non-existent harness");
    }
}
