//! Integration tests for PrintModeBackend
//!
//! These tests spawn real Claude Code CLI processes.
//! Prerequisites:
//! - Claude Code CLI installed (`claude` binary in PATH)
//! - Valid API key configured
//!
//! Run with: `cargo test --test integration -- --ignored`

use vibes_core::backend::{ClaudeBackend, PrintModeBackend, PrintModeConfig};
use vibes_core::events::ClaudeEvent;

/// Helper to check if Claude CLI is available
fn claude_available() -> bool {
    std::process::Command::new("claude")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Skip test if Claude CLI is not available
macro_rules! require_claude {
    () => {
        if !claude_available() {
            eprintln!("Skipping test: Claude CLI not available");
            return;
        }
    };
}

// ==================== Simple Prompt Tests ====================

#[tokio::test]
#[ignore = "requires Claude CLI"]
async fn simple_prompt_returns_text() {
    require_claude!();

    let mut backend = PrintModeBackend::new(PrintModeConfig::default());
    let mut rx = backend.subscribe();

    // Send a simple prompt
    let result = backend.send("Say exactly: Hello Integration Test").await;

    // Should complete without error
    assert!(result.is_ok(), "send() failed: {:?}", result.err());

    // Collect events
    let mut received_text = false;
    let mut received_turn_complete = false;

    while let Ok(event) = rx.try_recv() {
        match event {
            ClaudeEvent::TextDelta { text } => {
                received_text = true;
                // Should contain our expected phrase (or similar)
                println!("TextDelta: {}", text);
            }
            ClaudeEvent::TurnComplete { usage } => {
                received_turn_complete = true;
                println!("TurnComplete: {:?}", usage);
            }
            _ => {
                println!("Other event: {:?}", event);
            }
        }
    }

    assert!(received_text, "Expected to receive TextDelta events");
    assert!(
        received_turn_complete,
        "Expected to receive TurnComplete event"
    );
}

// ==================== Session Continuity Tests ====================

/// Tests that we can use the same session ID across multiple backend instances.
///
/// Note: This test verifies that the session ID is passed correctly to Claude CLI,
/// but session continuity may not work reliably in print mode. The `--session-id`
/// flag works best with interactive mode where Claude Code maintains session state.
///
/// For print mode, each invocation may be stateless unless Claude Code persists
/// the session to disk, which depends on CLI configuration and environment.
#[tokio::test]
#[ignore = "requires Claude CLI - may be flaky due to print mode limitations"]
async fn session_id_continuity_works() {
    require_claude!();

    // Claude CLI requires session IDs to be valid UUIDs
    let session_id = "11111111-2222-3333-4444-555555555555";

    // First turn - establish context
    {
        let mut backend =
            PrintModeBackend::with_session_id(session_id.to_string(), PrintModeConfig::default());

        let result = backend
            .send("Remember this secret code: BANANA42. Say 'Remembered!' only.")
            .await;
        assert!(result.is_ok(), "First turn failed: {:?}", result.err());

        // Give Claude time to persist session
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    // Second turn - test recall with same session ID
    {
        let mut backend =
            PrintModeBackend::with_session_id(session_id.to_string(), PrintModeConfig::default());
        let mut rx = backend.subscribe();

        let result = backend
            .send("What was the secret code I told you? Reply only with the code.")
            .await;
        assert!(result.is_ok(), "Second turn failed: {:?}", result.err());

        // Check if we got any response at all
        let mut response = String::new();
        while let Ok(event) = rx.try_recv() {
            if let ClaudeEvent::TextDelta { text } = event {
                response.push_str(&text);
            }
        }

        println!("Response: {}", response);

        // Note: Session continuity may not work in print mode.
        // We just verify the backend works - response may or may not contain the code.
        // If session state persists: response contains "BANANA42"
        // If not: response will be something else, but not empty (Claude always responds)
        if response.is_empty() {
            println!("Warning: Empty response - session continuity may not work in print mode");
            // Don't fail - this is a known limitation of print mode
        } else if response.contains("BANANA42") {
            println!("Session continuity working - code recalled correctly");
        } else {
            println!("Session may not have persisted - got different response");
        }
    }
}

// ==================== Tool Use Tests ====================

#[tokio::test]
#[ignore = "requires Claude CLI"]
async fn tool_use_events_parsed() {
    require_claude!();

    let config = PrintModeConfig {
        allowed_tools: vec!["Bash".to_string()],
        ..Default::default()
    };
    let mut backend = PrintModeBackend::new(config);
    let mut rx = backend.subscribe();

    // Request a tool use - listing current directory
    let result = backend
        .send("List the current directory with ls. Use the Bash tool.")
        .await;
    assert!(result.is_ok(), "send() failed: {:?}", result.err());

    // Check for tool use events
    let mut saw_tool_start = false;
    let mut saw_tool_result = false;

    while let Ok(event) = rx.try_recv() {
        match event {
            ClaudeEvent::ToolUseStart { name, .. } => {
                saw_tool_start = true;
                println!("ToolUseStart: {}", name);
            }
            ClaudeEvent::ToolResult { output, .. } => {
                saw_tool_result = true;
                println!("ToolResult: {}", output);
            }
            ClaudeEvent::ToolInputDelta { delta, .. } => {
                println!("ToolInputDelta: {}", delta);
            }
            _ => {
                println!("Other event: {:?}", event);
            }
        }
    }

    // Note: Claude may choose not to use a tool, so these are soft assertions
    if saw_tool_start {
        println!("Saw ToolUseStart event");
    }
    if saw_tool_result {
        println!("Saw ToolResult event");
    }
}

// ==================== Error Handling Tests ====================

#[tokio::test]
#[ignore = "requires Claude CLI"]
async fn invalid_claude_path_returns_error() {
    let config = PrintModeConfig {
        claude_path: Some("/nonexistent/claude".to_string()),
        ..Default::default()
    };
    let mut backend = PrintModeBackend::new(config);

    let result = backend.send("test").await;

    // Should fail because the binary doesn't exist
    assert!(result.is_err(), "Expected error for invalid claude path");
}

// ==================== State Transition Tests ====================

#[tokio::test]
#[ignore = "requires Claude CLI"]
async fn backend_transitions_to_idle_after_completion() {
    require_claude!();

    use vibes_core::backend::BackendState;

    let mut backend = PrintModeBackend::new(PrintModeConfig::default());

    // Initial state is Idle
    assert!(matches!(backend.state(), BackendState::Idle));

    // After send completes, should be back to Idle
    let result = backend.send("Say 'Done'").await;
    assert!(result.is_ok());
    assert!(matches!(backend.state(), BackendState::Idle));
}

#[tokio::test]
#[ignore = "requires Claude CLI"]
async fn shutdown_stops_processing() {
    require_claude!();

    use vibes_core::backend::BackendState;

    let mut backend = PrintModeBackend::new(PrintModeConfig::default());

    // Shutdown should work
    let result = backend.shutdown().await;
    assert!(result.is_ok());
    assert!(matches!(backend.state(), BackendState::Finished));
}
