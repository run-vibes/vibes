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

/// Helper script for sending hook data to vibes (one-way)
pub const VIBES_HOOK_SEND: &str = include_str!("scripts/vibes-hook-send.sh");

/// Helper script for sending hook data and receiving response (bidirectional)
pub const VIBES_HOOK_INJECT: &str = include_str!("scripts/vibes-hook-inject.sh");

/// Session start hook script (injection capable)
pub const SESSION_START: &str = include_str!("scripts/session-start.sh");

/// User prompt submit hook script (injection capable)
pub const USER_PROMPT_SUBMIT: &str = include_str!("scripts/user-prompt-submit.sh");

/// All scripts with their target filenames
pub const SCRIPTS: &[(&str, &str)] = &[
    ("pre-tool-use.sh", PRE_TOOL_USE),
    ("post-tool-use.sh", POST_TOOL_USE),
    ("stop.sh", STOP),
    ("vibes-hook-send.sh", VIBES_HOOK_SEND),
    ("vibes-hook-inject.sh", VIBES_HOOK_INJECT),
    ("session-start.sh", SESSION_START),
    ("user-prompt-submit.sh", USER_PROMPT_SUBMIT),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scripts_have_content() {
        // Verify scripts have meaningful content (not just shebang)
        assert!(
            PRE_TOOL_USE.len() > 50,
            "pre-tool-use.sh should have content"
        );
        assert!(
            POST_TOOL_USE.len() > 50,
            "post-tool-use.sh should have content"
        );
        assert!(STOP.len() > 50, "stop.sh should have content");
        assert!(
            VIBES_HOOK_SEND.len() > 50,
            "vibes-hook-send.sh should have content"
        );
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
        // 4 original + 3 new (session-start, user-prompt-submit, vibes-hook-inject)
        assert_eq!(SCRIPTS.len(), 7);
    }

    #[test]
    fn test_session_start_script_exists() {
        assert!(
            SESSION_START.len() > 50,
            "session-start.sh should have content"
        );
    }

    #[test]
    fn test_user_prompt_submit_script_exists() {
        assert!(
            USER_PROMPT_SUBMIT.len() > 50,
            "user-prompt-submit.sh should have content"
        );
    }

    #[test]
    fn test_hook_inject_script_exists() {
        assert!(
            VIBES_HOOK_INJECT.len() > 50,
            "vibes-hook-inject.sh should have content"
        );
    }
}
