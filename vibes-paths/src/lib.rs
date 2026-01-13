//! XDG Base Directory paths for vibes.
//!
//! CLI tools should use XDG paths for cross-platform consistency,
//! not platform-native paths. This matches tools like gh, docker, kubectl.

use std::path::PathBuf;

/// Get the vibes config directory.
///
/// Returns `$XDG_CONFIG_HOME/vibes` if set, otherwise `~/.config/vibes`.
/// This is where config files, plugins, and registries are stored.
///
/// # Examples
///
/// ```
/// use vibes_paths::config_dir;
///
/// let config = config_dir();
/// let plugin_dir = config.join("plugins");
/// ```
pub fn config_dir() -> PathBuf {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("vibes")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".config/vibes")
    } else {
        PathBuf::from(".config/vibes")
    }
}

/// Get the vibes data directory.
///
/// Returns `$XDG_DATA_HOME/vibes` if set, otherwise `~/.local/share/vibes`.
/// This is where persistent data like Iggy streams are stored.
///
/// # Examples
///
/// ```
/// use vibes_paths::data_dir;
///
/// let data = data_dir();
/// let iggy_dir = data.join("iggy");
/// ```
pub fn data_dir() -> PathBuf {
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data).join("vibes")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".local/share/vibes")
    } else {
        PathBuf::from(".local/share/vibes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_ends_with_vibes() {
        let path = config_dir();
        assert!(
            path.ends_with("vibes"),
            "config_dir should end with 'vibes'"
        );
    }

    #[test]
    fn test_data_dir_ends_with_vibes() {
        let path = data_dir();
        assert!(path.ends_with("vibes"), "data_dir should end with 'vibes'");
    }

    #[test]
    fn test_config_dir_respects_xdg_env() {
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/tmp/test-config");
        }
        let path = config_dir();
        assert_eq!(path, PathBuf::from("/tmp/test-config/vibes"));
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn test_data_dir_respects_xdg_env() {
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "/tmp/test-data");
        }
        let path = data_dir();
        assert_eq!(path, PathBuf::from("/tmp/test-data/vibes"));
        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
        }
    }
}
