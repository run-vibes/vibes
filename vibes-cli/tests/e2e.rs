//! End-to-end integration tests for vibes-cli
//!
//! These tests require Claude CLI to be installed and are gated behind
//! the `integration` feature flag. Run with:
//!
//! ```sh
//! cargo test -p vibes-cli --features integration
//! ```

#![cfg(feature = "integration")]

use std::process::Command;

/// Test that vibes --help works
#[test]
fn vibes_help_works() {
    let output = Command::new("cargo")
        .args(["run", "-p", "vibes-cli", "--", "--help"])
        .output()
        .expect("Failed to run vibes --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Remote control for Claude Code"));
    assert!(stdout.contains("claude"));
    assert!(stdout.contains("config"));
}

/// Test that vibes claude --help shows all flags
#[test]
fn vibes_claude_help_shows_all_flags() {
    let output = Command::new("cargo")
        .args(["run", "-p", "vibes-cli", "--", "claude", "--help"])
        .output()
        .expect("Failed to run vibes claude --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--session-name"));
    assert!(stdout.contains("--no-serve"));
    assert!(stdout.contains("--continue-session"));
    assert!(stdout.contains("--resume"));
    assert!(stdout.contains("--model"));
    assert!(stdout.contains("--allowedTools"));
    assert!(stdout.contains("--system-prompt"));
}

/// Test that vibes config show works without config file
#[test]
fn vibes_config_show_works_without_config() {
    let output = Command::new("cargo")
        .args(["run", "-p", "vibes-cli", "--", "config", "show"])
        .output()
        .expect("Failed to run vibes config show");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show defaults
    assert!(stdout.contains("[server]"));
    assert!(stdout.contains("port = 7432"));
    assert!(stdout.contains("auto_start = true"));
}

/// Test that vibes config path shows paths
#[test]
fn vibes_config_path_shows_paths() {
    let output = Command::new("cargo")
        .args(["run", "-p", "vibes-cli", "--", "config", "path"])
        .output()
        .expect("Failed to run vibes config path");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("User config:"));
    assert!(stdout.contains("Project config:"));
    assert!(stdout.contains(".vibes/config.toml"));
}

/// Test that vibes claude without prompt enters interactive mode
///
/// When no prompt is provided, vibes claude enters interactive mode.
/// With EOF on stdin, it should exit gracefully with "Goodbye!".
#[test]
fn vibes_claude_without_prompt_enters_interactive_mode() {
    use std::process::Stdio;

    let mut child = Command::new("cargo")
        .args(["run", "-p", "vibes-cli", "--", "claude"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn vibes claude");

    // Close stdin immediately to trigger EOF
    drop(child.stdin.take());

    // Wait for process to exit (with timeout via test framework)
    let output = child
        .wait_with_output()
        .expect("Failed to wait for process");

    // Should exit successfully after EOF
    assert!(
        output.status.success(),
        "Expected success, got: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Goodbye"),
        "Expected 'Goodbye' in output, got: {}",
        stdout
    );
}

/// Test that vibes event send --help shows usage
#[test]
fn vibes_event_send_help_works() {
    let output = Command::new("cargo")
        .args(["run", "-p", "vibes-cli", "--", "event", "send", "--help"])
        .output()
        .expect("Failed to run vibes event send --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--type"));
    assert!(stdout.contains("--data"));
    assert!(stdout.contains("--session"));
    assert!(stdout.contains("--stream"));
    assert!(stdout.contains("--topic"));
}

/// Test that vibes event send without required args fails properly
#[test]
fn vibes_event_send_requires_type() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "vibes-cli",
            "--",
            "event",
            "send",
            "--data",
            "{}",
        ])
        .output()
        .expect("Failed to run vibes event send");

    // Should fail because --type is required
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--type") || stderr.contains("required"),
        "Expected error about --type, got: {}",
        stderr
    );
}

// Note: Tests that actually call Claude require Claude CLI to be installed
// and a valid API key. These tests are intentionally kept separate.

#[cfg(feature = "integration_claude")]
mod claude_tests {
    use super::*;

    /// Test that vibes claude produces output
    /// Requires Claude CLI and API key
    #[test]
    fn vibes_claude_hello_produces_output() {
        let output = Command::new("cargo")
            .args([
                "run",
                "-p",
                "vibes-cli",
                "--",
                "claude",
                "respond with only the word 'hello'",
            ])
            .output()
            .expect("Failed to run vibes claude");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
        assert!(stdout.contains("hello"));
    }
}

/// Tests that require a running Iggy server
#[cfg(feature = "integration_iggy")]
mod iggy_tests {
    use super::*;

    /// Test that vibes event send writes to Iggy
    ///
    /// Requires:
    /// - Iggy server running on localhost:3001
    /// - VIBES_IGGY_USERNAME and VIBES_IGGY_PASSWORD set (or defaults: iggy/iggy)
    #[test]
    fn vibes_event_send_writes_to_iggy() {
        // Send a hook event
        let output = Command::new("cargo")
            .args([
                "run",
                "-p",
                "vibes-cli",
                "--",
                "event",
                "send",
                "--type",
                "hook",
                "--session",
                "integration-test",
                "--data",
                r#"{"type":"session_start","session_id":"integration-test"}"#,
            ])
            .output()
            .expect("Failed to run vibes event send");

        assert!(
            output.status.success(),
            "Event send failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Note: Verifying the message was actually written to Iggy
        // would require reading from the topic - left as manual verification
    }

    /// Test that session-state events require --session
    #[test]
    fn vibes_event_send_session_state_requires_session() {
        let output = Command::new("cargo")
            .args([
                "run",
                "-p",
                "vibes-cli",
                "--",
                "event",
                "send",
                "--type",
                "session-state",
                "--data",
                r#"{"state":"Processing"}"#,
            ])
            .output()
            .expect("Failed to run vibes event send");

        // Should fail because session-state requires --session
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("session") || stderr.contains("required"),
            "Expected error about --session, got: {}",
            stderr
        );
    }
}
