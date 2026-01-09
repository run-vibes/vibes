//! Hook-based intervention for injecting learnings.
//!
//! The `HookIntervention` system writes learnings to Claude Code hooks,
//! allowing learnings to influence future sessions automatically.
//!
//! ## Intervention Flow
//!
//! ```text
//! CircuitBreaker.transition_to_open()
//!     │
//!     ├─ High frustration detected
//!     │
//!     └─ HookIntervention.intervene(session, learning)
//!             │
//!             └─ Write to .claude/hooks/vibes_learning_*.sh
//! ```
//!
//! ## Hook Format
//!
//! Hooks are written as shell scripts that echo learning context:
//!
//! ```bash
//! #!/bin/bash
//! # vibes-groove learning hook
//! echo "## Learning: {title}"
//! echo "{content}"
//! ```

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::types::SessionId;

/// Configuration for the intervention system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InterventionConfig {
    /// Whether interventions are enabled.
    pub enabled: bool,
    /// Directory for hook files (defaults to .claude/hooks).
    pub hooks_dir: Option<PathBuf>,
    /// Maximum concurrent interventions per session.
    pub max_per_session: u32,
    /// Whether to use Claude Code hook format.
    pub use_claude_hooks: bool,
}

impl Default for InterventionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hooks_dir: None,
            max_per_session: 3,
            use_claude_hooks: true,
        }
    }
}

/// A learning to be injected via intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    /// Unique identifier for the learning.
    pub id: String,
    /// Short title for the learning.
    pub title: String,
    /// Full content of the learning.
    pub content: String,
    /// Tags for categorization.
    pub tags: Vec<String>,
}

impl Learning {
    /// Create a new learning.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            tags: Vec::new(),
        }
    }

    /// Add tags to the learning.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Result of an intervention attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterventionResult {
    /// Intervention was successfully applied.
    Applied {
        /// Path where the hook was written.
        hook_path: PathBuf,
    },
    /// Intervention was skipped (e.g., disabled or limit reached).
    Skipped {
        /// Reason for skipping.
        reason: String,
    },
    /// Intervention failed.
    Failed {
        /// Error message.
        error: String,
    },
}

impl InterventionResult {
    /// Check if the intervention was applied.
    pub fn is_applied(&self) -> bool {
        matches!(self, Self::Applied { .. })
    }

    /// Check if the intervention was skipped.
    pub fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped { .. })
    }

    /// Check if the intervention failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Error type for intervention operations.
#[derive(Debug, thiserror::Error)]
pub enum InterventionError {
    /// Interventions are disabled.
    #[error("interventions are disabled")]
    Disabled,
    /// Maximum interventions per session reached.
    #[error("maximum interventions per session reached ({0})")]
    LimitReached(u32),
    /// Failed to create hooks directory.
    #[error("failed to create hooks directory: {0}")]
    CreateDirFailed(std::io::Error),
    /// Failed to write hook file.
    #[error("failed to write hook file: {0}")]
    WriteFailed(std::io::Error),
}

/// Per-session intervention tracking.
#[derive(Debug, Default)]
struct SessionInterventions {
    /// Number of interventions applied to this session.
    count: u32,
    /// IDs of learnings already applied.
    applied_learnings: Vec<String>,
}

/// Hook-based intervention system.
///
/// Writes learnings to Claude Code hooks to influence future sessions.
pub struct HookIntervention {
    config: InterventionConfig,
    sessions: std::collections::HashMap<SessionId, SessionInterventions>,
}

impl HookIntervention {
    /// Create a new hook intervention system.
    pub fn new(config: InterventionConfig) -> Self {
        Self {
            config,
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Check if interventions are enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the hooks directory path.
    pub fn hooks_dir(&self) -> PathBuf {
        self.config
            .hooks_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(".claude/hooks"))
    }

    /// Apply an intervention for a session (synchronous version).
    ///
    /// Writes a hook file containing the learning if:
    /// - Interventions are enabled
    /// - Session hasn't exceeded the intervention limit
    /// - The learning hasn't already been applied to this session
    ///
    /// This is the synchronous version for use in `SyncAssessmentProcessor`.
    pub fn intervene_sync(
        &mut self,
        session_id: &SessionId,
        learning: &Learning,
    ) -> Result<InterventionResult, InterventionError> {
        if !self.config.enabled {
            return Err(InterventionError::Disabled);
        }

        // Check session state
        {
            let session = self.sessions.entry(session_id.clone()).or_default();

            // Check intervention limit
            if session.count >= self.config.max_per_session {
                return Ok(InterventionResult::Skipped {
                    reason: format!(
                        "session intervention limit reached ({})",
                        self.config.max_per_session
                    ),
                });
            }

            // Check if learning already applied
            if session.applied_learnings.contains(&learning.id) {
                return Ok(InterventionResult::Skipped {
                    reason: format!("learning '{}' already applied to session", learning.id),
                });
            }
        }

        // Write the hook synchronously
        let hook_path = self.write_hook_sync(session_id, learning)?;

        // Track the intervention
        let session = self.sessions.entry(session_id.clone()).or_default();
        session.count += 1;
        session.applied_learnings.push(learning.id.clone());

        Ok(InterventionResult::Applied { hook_path })
    }

    /// Apply an intervention for a session (async version).
    ///
    /// Writes a hook file containing the learning if:
    /// - Interventions are enabled
    /// - Session hasn't exceeded the intervention limit
    /// - The learning hasn't already been applied to this session
    pub async fn intervene(
        &mut self,
        session_id: &SessionId,
        learning: &Learning,
    ) -> Result<InterventionResult, InterventionError> {
        if !self.config.enabled {
            return Err(InterventionError::Disabled);
        }

        // Check session state without holding borrow across await
        {
            let session = self.sessions.entry(session_id.clone()).or_default();

            // Check intervention limit
            if session.count >= self.config.max_per_session {
                return Ok(InterventionResult::Skipped {
                    reason: format!(
                        "session intervention limit reached ({})",
                        self.config.max_per_session
                    ),
                });
            }

            // Check if learning already applied
            if session.applied_learnings.contains(&learning.id) {
                return Ok(InterventionResult::Skipped {
                    reason: format!("learning '{}' already applied to session", learning.id),
                });
            }
        } // Mutable borrow ends here

        // Write the hook (no longer borrows sessions)
        let hook_path = self.write_hook(session_id, learning).await?;

        // Track the intervention
        let session = self.sessions.entry(session_id.clone()).or_default();
        session.count += 1;
        session.applied_learnings.push(learning.id.clone());

        Ok(InterventionResult::Applied { hook_path })
    }

    /// Write a hook file for a learning (async version).
    async fn write_hook(
        &self,
        session_id: &SessionId,
        learning: &Learning,
    ) -> Result<PathBuf, InterventionError> {
        let hooks_dir = self.hooks_dir();

        // Ensure hooks directory exists
        tokio::fs::create_dir_all(&hooks_dir)
            .await
            .map_err(InterventionError::CreateDirFailed)?;

        // Generate hook filename
        let filename = format!(
            "vibes_learning_{}_{}.sh",
            sanitize_filename(session_id.as_str()),
            sanitize_filename(&learning.id)
        );
        let hook_path = hooks_dir.join(&filename);

        // Generate hook content
        let content = if self.config.use_claude_hooks {
            generate_claude_hook(learning)
        } else {
            generate_simple_hook(learning)
        };

        // Write the hook file
        tokio::fs::write(&hook_path, content)
            .await
            .map_err(InterventionError::WriteFailed)?;

        Ok(hook_path)
    }

    /// Write a hook file for a learning (synchronous version).
    fn write_hook_sync(
        &self,
        session_id: &SessionId,
        learning: &Learning,
    ) -> Result<PathBuf, InterventionError> {
        let hooks_dir = self.hooks_dir();

        // Ensure hooks directory exists
        std::fs::create_dir_all(&hooks_dir).map_err(InterventionError::CreateDirFailed)?;

        // Generate hook filename
        let filename = format!(
            "vibes_learning_{}_{}.sh",
            sanitize_filename(session_id.as_str()),
            sanitize_filename(&learning.id)
        );
        let hook_path = hooks_dir.join(&filename);

        // Generate hook content
        let content = if self.config.use_claude_hooks {
            generate_claude_hook(learning)
        } else {
            generate_simple_hook(learning)
        };

        // Write the hook file
        std::fs::write(&hook_path, content).map_err(InterventionError::WriteFailed)?;

        Ok(hook_path)
    }

    /// Get the number of interventions for a session.
    pub fn intervention_count(&self, session_id: &SessionId) -> u32 {
        self.sessions.get(session_id).map(|s| s.count).unwrap_or(0)
    }

    /// Get the total number of interventions across all sessions.
    pub fn total_intervention_count(&self) -> u32 {
        self.sessions.values().map(|s| s.count).sum()
    }

    /// Check if a learning has been applied to a session.
    pub fn learning_applied(&self, session_id: &SessionId, learning_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .map(|s| s.applied_learnings.contains(&learning_id.to_string()))
            .unwrap_or(false)
    }

    /// Remove a session from tracking.
    pub fn remove_session(&mut self, session_id: &SessionId) {
        self.sessions.remove(session_id);
    }
}

impl Default for HookIntervention {
    fn default() -> Self {
        Self::new(InterventionConfig::default())
    }
}

impl std::fmt::Debug for HookIntervention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookIntervention")
            .field("enabled", &self.config.enabled)
            .field("max_per_session", &self.config.max_per_session)
            .field("session_count", &self.sessions.len())
            .finish()
    }
}

/// Sanitize a string for use in filenames.
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' => c,
            _ => '_',
        })
        .take(32) // Limit filename length
        .collect()
}

/// Escape a string for safe use in a shell script.
///
/// Uses single quotes which prevent interpretation of all special characters
/// except single quotes themselves. Embedded single quotes are escaped using
/// the standard shell idiom: end the single-quoted string, add an escaped
/// single quote, and start a new single-quoted string.
///
/// Example: "don't" -> 'don'\''t'
fn shell_escape(s: &str) -> String {
    // Replace ' with '\'' (end quote, escaped quote, start quote)
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Generate a Claude Code hook script.
fn generate_claude_hook(learning: &Learning) -> String {
    let tags = if learning.tags.is_empty() {
        String::new()
    } else {
        format!("\n# Tags: {}", learning.tags.join(", "))
    };

    // Use shell_escape for safe content injection
    format!(
        "#!/bin/bash\n\
         # vibes-groove learning hook\n\
         # Learning ID: {id}{tags}\n\
         \n\
         echo {title}\n\
         echo \"\"\n\
         echo {content}\n",
        id = learning.id,
        tags = tags,
        title = shell_escape(&format!("## Learning: {}", learning.title)),
        content = shell_escape(&learning.content),
    )
}

/// Generate a simple hook script.
fn generate_simple_hook(learning: &Learning) -> String {
    // Use shell_escape for safe content injection
    format!(
        "#!/bin/bash\n# Learning: {title}\necho {content}\n",
        title = learning.title,
        content = shell_escape(&learning.content),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intervention_config_defaults() {
        let config = InterventionConfig::default();

        assert!(config.enabled);
        assert!(config.hooks_dir.is_none());
        assert_eq!(config.max_per_session, 3);
        assert!(config.use_claude_hooks);
    }

    #[test]
    fn test_learning_builder() {
        let learning = Learning::new("learn-1", "Test Title", "Test content")
            .with_tags(vec!["rust".to_string(), "testing".to_string()]);

        assert_eq!(learning.id, "learn-1");
        assert_eq!(learning.title, "Test Title");
        assert_eq!(learning.content, "Test content");
        assert_eq!(learning.tags, vec!["rust", "testing"]);
    }

    #[test]
    fn test_intervention_result_variants() {
        let applied = InterventionResult::Applied {
            hook_path: PathBuf::from("/test"),
        };
        assert!(applied.is_applied());
        assert!(!applied.is_skipped());
        assert!(!applied.is_failed());

        let skipped = InterventionResult::Skipped {
            reason: "test".to_string(),
        };
        assert!(!skipped.is_applied());
        assert!(skipped.is_skipped());
        assert!(!skipped.is_failed());

        let failed = InterventionResult::Failed {
            error: "test".to_string(),
        };
        assert!(!failed.is_applied());
        assert!(!failed.is_skipped());
        assert!(failed.is_failed());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test"), "test");
        assert_eq!(sanitize_filename("test/path"), "test_path");
        assert_eq!(sanitize_filename("test:name"), "test_name");
        assert_eq!(sanitize_filename("test<>name"), "test__name");

        // Test length limit
        let long = "a".repeat(100);
        assert_eq!(sanitize_filename(&long).len(), 32);
    }

    #[test]
    fn test_generate_claude_hook() {
        let learning = Learning::new("test-id", "Test Title", "Test content");
        let hook = generate_claude_hook(&learning);

        assert!(hook.contains("#!/bin/bash"));
        assert!(hook.contains("vibes-groove learning hook"));
        assert!(hook.contains("Learning ID: test-id"));
        assert!(hook.contains("## Learning: Test Title"));
        assert!(hook.contains("Test content"));
    }

    #[test]
    fn test_generate_claude_hook_with_tags() {
        let learning =
            Learning::new("test-id", "Title", "Content").with_tags(vec!["tag1".to_string()]);
        let hook = generate_claude_hook(&learning);

        assert!(hook.contains("# Tags: tag1"));
    }

    #[test]
    fn test_generate_simple_hook() {
        let learning = Learning::new("test-id", "Test Title", "Test content");
        let hook = generate_simple_hook(&learning);

        assert!(hook.contains("#!/bin/bash"));
        assert!(hook.contains("# Learning: Test Title"));
        assert!(hook.contains("echo 'Test content'"));
    }

    #[test]
    fn test_intervention_hooks_dir() {
        let config = InterventionConfig::default();
        let intervention = HookIntervention::new(config);

        assert_eq!(intervention.hooks_dir(), PathBuf::from(".claude/hooks"));

        let config_custom = InterventionConfig {
            hooks_dir: Some(PathBuf::from("/custom/hooks")),
            ..Default::default()
        };
        let intervention_custom = HookIntervention::new(config_custom);

        assert_eq!(
            intervention_custom.hooks_dir(),
            PathBuf::from("/custom/hooks")
        );
    }

    #[tokio::test]
    async fn test_intervention_disabled() {
        let config = InterventionConfig {
            enabled: false,
            ..Default::default()
        };
        let mut intervention = HookIntervention::new(config);
        let session_id = SessionId::from("test-session");
        let learning = Learning::new("test-id", "Title", "Content");

        let result = intervention.intervene(&session_id, &learning).await;
        assert!(matches!(result, Err(InterventionError::Disabled)));
    }

    #[tokio::test]
    async fn test_intervention_writes_hook() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks_dir = temp_dir.path().join("hooks");

        let config = InterventionConfig {
            enabled: true,
            hooks_dir: Some(hooks_dir.clone()),
            max_per_session: 3,
            use_claude_hooks: true,
        };
        let mut intervention = HookIntervention::new(config);
        let session_id = SessionId::from("test-session");
        let learning = Learning::new("test-id", "Test Title", "Test content");

        let result = intervention
            .intervene(&session_id, &learning)
            .await
            .expect("intervene should succeed");

        assert!(result.is_applied());
        if let InterventionResult::Applied { hook_path } = result {
            assert!(hook_path.exists());
            let content = std::fs::read_to_string(&hook_path).expect("read hook");
            assert!(content.contains("vibes-groove learning hook"));
            assert!(content.contains("Test Title"));
        }

        // Verify tracking
        assert_eq!(intervention.intervention_count(&session_id), 1);
        assert!(intervention.learning_applied(&session_id, "test-id"));
    }

    #[tokio::test]
    async fn test_intervention_skips_duplicate() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let config = InterventionConfig {
            enabled: true,
            hooks_dir: Some(temp_dir.path().join("hooks")),
            max_per_session: 10,
            use_claude_hooks: true,
        };
        let mut intervention = HookIntervention::new(config);
        let session_id = SessionId::from("test-session");
        let learning = Learning::new("test-id", "Title", "Content");

        // First intervention should succeed
        let result1 = intervention
            .intervene(&session_id, &learning)
            .await
            .unwrap();
        assert!(result1.is_applied());

        // Second intervention with same learning should be skipped
        let result2 = intervention
            .intervene(&session_id, &learning)
            .await
            .unwrap();
        assert!(result2.is_skipped());
        if let InterventionResult::Skipped { reason } = result2 {
            assert!(reason.contains("already applied"));
        }
    }

    #[tokio::test]
    async fn test_intervention_limit() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let config = InterventionConfig {
            enabled: true,
            hooks_dir: Some(temp_dir.path().join("hooks")),
            max_per_session: 2, // Low limit for testing
            use_claude_hooks: true,
        };
        let mut intervention = HookIntervention::new(config);
        let session_id = SessionId::from("test-session");

        // Apply two interventions
        for i in 0..2 {
            let learning = Learning::new(format!("learn-{i}"), "Title", "Content");
            let result = intervention
                .intervene(&session_id, &learning)
                .await
                .unwrap();
            assert!(result.is_applied());
        }

        // Third should be skipped
        let learning = Learning::new("learn-3", "Title", "Content");
        let result = intervention
            .intervene(&session_id, &learning)
            .await
            .unwrap();
        assert!(result.is_skipped());
        if let InterventionResult::Skipped { reason } = result {
            assert!(reason.contains("limit reached"));
        }
    }

    #[test]
    fn test_intervention_debug() {
        let intervention = HookIntervention::default();
        let debug = format!("{intervention:?}");
        assert!(debug.contains("HookIntervention"));
        assert!(debug.contains("enabled"));
        assert!(debug.contains("max_per_session"));
    }

    #[test]
    fn test_intervention_error_display() {
        assert_eq!(
            InterventionError::Disabled.to_string(),
            "interventions are disabled"
        );
        assert!(InterventionError::LimitReached(3).to_string().contains("3"));
    }

    #[test]
    fn test_shell_escape_basic() {
        // Simple string
        assert_eq!(shell_escape("hello"), "'hello'");
    }

    #[test]
    fn test_shell_escape_single_quotes() {
        // Single quotes are escaped
        assert_eq!(shell_escape("don't"), "'don'\\''t'");
    }

    #[test]
    fn test_shell_escape_special_chars() {
        // All special characters are safe in single quotes
        assert_eq!(shell_escape("$HOME"), "'$HOME'");
        assert_eq!(shell_escape("`ls`"), "'`ls`'");
        assert_eq!(shell_escape("$(whoami)"), "'$(whoami)'");
        assert_eq!(shell_escape("a\nb"), "'a\nb'");
        assert_eq!(shell_escape("\"double\""), "'\"double\"'");
    }

    #[test]
    fn test_shell_escape_empty() {
        assert_eq!(shell_escape(""), "''");
    }

    // ─── Sync intervention tests ─────────────────────────────────────────

    #[test]
    fn test_intervention_sync_writes_hook() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks_dir = temp_dir.path().join("hooks");

        let config = InterventionConfig {
            enabled: true,
            hooks_dir: Some(hooks_dir.clone()),
            max_per_session: 3,
            use_claude_hooks: true,
        };
        let mut intervention = HookIntervention::new(config);
        let session_id = SessionId::from("sync-test-session");
        let learning = Learning::new("sync-test-id", "Sync Test Title", "Sync test content");

        let result = intervention
            .intervene_sync(&session_id, &learning)
            .expect("intervene_sync should succeed");

        assert!(result.is_applied());
        if let InterventionResult::Applied { hook_path } = result {
            assert!(hook_path.exists());
            let content = std::fs::read_to_string(&hook_path).expect("read hook");
            assert!(content.contains("vibes-groove learning hook"));
            assert!(content.contains("Sync Test Title"));
        }

        // Verify tracking works same as async version
        assert_eq!(intervention.intervention_count(&session_id), 1);
        assert!(intervention.learning_applied(&session_id, "sync-test-id"));
    }

    #[test]
    fn test_intervention_sync_respects_disabled() {
        let config = InterventionConfig {
            enabled: false,
            ..Default::default()
        };
        let mut intervention = HookIntervention::new(config);
        let session_id = SessionId::from("disabled-session");
        let learning = Learning::new("test-id", "Title", "Content");

        let result = intervention.intervene_sync(&session_id, &learning);
        assert!(matches!(result, Err(InterventionError::Disabled)));
    }

    #[test]
    fn test_intervention_sync_respects_limit() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");
        let config = InterventionConfig {
            enabled: true,
            hooks_dir: Some(temp_dir.path().join("hooks")),
            max_per_session: 2,
            use_claude_hooks: true,
        };
        let mut intervention = HookIntervention::new(config);
        let session_id = SessionId::from("limit-session");

        // Apply two interventions
        for i in 0..2 {
            let learning = Learning::new(format!("learn-{i}"), "Title", "Content");
            let result = intervention.intervene_sync(&session_id, &learning).unwrap();
            assert!(result.is_applied());
        }

        // Third should be skipped
        let learning = Learning::new("learn-3", "Title", "Content");
        let result = intervention.intervene_sync(&session_id, &learning).unwrap();
        assert!(result.is_skipped());
    }
}
