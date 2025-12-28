//! Harness trait for AI coding assistant abstraction

use crate::{ConfigPaths, HarnessCapabilities, Result};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Core trait - any AI coding assistant we can enhance
#[async_trait]
pub trait Harness: Send + Sync {
    /// Unique identifier (e.g., "claude", "cursor", "aider")
    fn harness_type(&self) -> &'static str;

    /// Detect version from binary or config
    async fn version(&self) -> Option<String>;

    /// Platform-appropriate config paths
    fn config_paths(&self, project_root: Option<&Path>) -> Result<ConfigPaths>;

    /// Full capability introspection
    async fn introspect(&self, project_root: Option<&Path>) -> Result<HarnessCapabilities>;
}

/// Create the appropriate harness from CLI subcommand
pub fn harness_for_command(command: &str) -> Option<Arc<dyn Harness>> {
    match command {
        "claude" => {
            #[cfg(feature = "claude-code")]
            {
                Some(Arc::new(crate::claude_code::ClaudeCodeHarness))
            }
            #[cfg(not(feature = "claude-code"))]
            {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_trait_is_object_safe() {
        // This compiles only if Harness is object-safe
        fn _takes_boxed_harness(_: Box<dyn Harness>) {}
    }
}
