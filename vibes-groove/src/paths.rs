//! Cross-platform file paths for groove data
//!
//! Provides consistent paths for groove storage, transcripts, and learnings
//! across different operating systems.

use std::path::PathBuf;

/// Groove file paths with cross-platform support
#[derive(Debug, Clone)]
pub struct GroovePaths {
    /// Base data directory (e.g., ~/.local/share/vibes-groove on Linux)
    pub data_dir: PathBuf,
    /// Transcripts directory (captured session data)
    pub transcripts_dir: PathBuf,
    /// Learnings directory (extracted knowledge)
    pub learnings_dir: PathBuf,
    /// Database file path
    pub db_path: PathBuf,
}

impl GroovePaths {
    /// Create paths using platform-appropriate defaults
    ///
    /// Uses XDG on Linux, Application Support on macOS, and AppData on Windows.
    pub fn new() -> Option<Self> {
        let data_dir = Self::default_data_dir()?;
        Some(Self::from_base(data_dir))
    }

    /// Create paths from a custom base directory
    pub fn from_base(data_dir: PathBuf) -> Self {
        Self {
            transcripts_dir: data_dir.join("transcripts"),
            learnings_dir: data_dir.join("learnings"),
            db_path: data_dir.join("groove.db"),
            data_dir,
        }
    }

    /// Get the default data directory for the current platform
    fn default_data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("vibes-groove"))
    }

    /// Claude Code projects directory (where Claude stores session data)
    pub fn claude_projects_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".claude").join("projects"))
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.transcripts_dir)?;
        std::fs::create_dir_all(&self.learnings_dir)?;
        Ok(())
    }

    /// Get project-specific learnings file
    pub fn project_learnings(&self, project_id: &str) -> PathBuf {
        self.learnings_dir.join(format!("{}.md", project_id))
    }

    /// Get project-specific transcript archive
    pub fn project_transcripts(&self, project_id: &str) -> PathBuf {
        self.transcripts_dir.join(project_id)
    }
}

impl Default for GroovePaths {
    fn default() -> Self {
        Self::new().unwrap_or_else(|| {
            // Fallback to temp directory if no home
            Self::from_base(std::env::temp_dir().join("vibes-groove"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_base_creates_correct_paths() {
        let base = PathBuf::from("/tmp/test-groove");
        let paths = GroovePaths::from_base(base.clone());

        assert_eq!(paths.data_dir, base);
        assert_eq!(paths.transcripts_dir, base.join("transcripts"));
        assert_eq!(paths.learnings_dir, base.join("learnings"));
        assert_eq!(paths.db_path, base.join("groove.db"));
    }

    #[test]
    fn test_project_learnings_path() {
        let paths = GroovePaths::from_base(PathBuf::from("/data/groove"));
        let learnings = paths.project_learnings("my-project");
        assert_eq!(
            learnings,
            PathBuf::from("/data/groove/learnings/my-project.md")
        );
    }

    #[test]
    fn test_project_transcripts_path() {
        let paths = GroovePaths::from_base(PathBuf::from("/data/groove"));
        let transcripts = paths.project_transcripts("my-project");
        assert_eq!(
            transcripts,
            PathBuf::from("/data/groove/transcripts/my-project")
        );
    }

    #[test]
    fn test_default_creates_valid_paths() {
        let paths = GroovePaths::default();
        // Should have a valid data_dir
        assert!(!paths.data_dir.as_os_str().is_empty());
        // All paths should contain the data_dir
        assert!(paths.transcripts_dir.starts_with(&paths.data_dir));
        assert!(paths.learnings_dir.starts_with(&paths.data_dir));
        assert!(paths.db_path.starts_with(&paths.data_dir));
    }

    #[test]
    fn test_ensure_dirs_creates_directories() {
        let temp = tempfile::tempdir().unwrap();
        let paths = GroovePaths::from_base(temp.path().join("groove"));

        paths.ensure_dirs().unwrap();

        assert!(paths.data_dir.exists());
        assert!(paths.transcripts_dir.exists());
        assert!(paths.learnings_dir.exists());
    }
}
