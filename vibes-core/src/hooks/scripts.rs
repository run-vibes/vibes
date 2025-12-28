//! Embedded hook scripts
//!
//! These scripts are embedded in the binary and installed to ~/.claude/hooks/
//! when the daemon starts.

/// Pre-tool-use hook script
pub const PRE_TOOL_USE: &str = include_str!("scripts/pre-tool-use.sh");

/// Post-tool-use hook script
pub const POST_TOOL_USE: &str = include_str!("scripts/post-tool-use.sh");

/// Stop hook script
pub const STOP: &str = include_str!("scripts/stop.sh");

/// Helper script for sending hook data to vibes
pub const VIBES_HOOK_SEND: &str = include_str!("scripts/vibes-hook-send.sh");

/// All scripts with their target filenames
pub const SCRIPTS: &[(&str, &str)] = &[
    ("pre-tool-use.sh", PRE_TOOL_USE),
    ("post-tool-use.sh", POST_TOOL_USE),
    ("stop.sh", STOP),
    ("vibes-hook-send.sh", VIBES_HOOK_SEND),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scripts_not_empty() {
        assert!(!PRE_TOOL_USE.is_empty());
        assert!(!POST_TOOL_USE.is_empty());
        assert!(!STOP.is_empty());
        assert!(!VIBES_HOOK_SEND.is_empty());
    }

    #[test]
    fn test_scripts_are_shell() {
        for (name, content) in SCRIPTS {
            assert!(
                content.starts_with("#!/bin/bash"),
                "{} should start with shebang",
                name
            );
        }
    }

    #[test]
    fn test_script_count() {
        assert_eq!(SCRIPTS.len(), 4);
    }
}
