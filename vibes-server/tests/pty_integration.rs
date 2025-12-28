//! PTY integration tests
//!
//! These tests validate the PTY I/O flow:
//! - Attach creates a PTY session
//! - Input is sent to the PTY
//! - Output is received from the PTY
//!
//! Uses `cat` as the PTY command (echoes input back) for testability.

mod common;

use std::time::Duration;

use common::client::TestClient;
use uuid::Uuid;
use vibes_core::pty::PtyConfig;

/// Create a PTY config that uses `cat` instead of `claude`
fn test_pty_config() -> PtyConfig {
    PtyConfig {
        claude_path: "cat".into(),
        ..Default::default()
    }
}

#[tokio::test]
async fn attach_creates_pty_session() {
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;
    let mut client = TestClient::connect(addr).await;

    let session_id = Uuid::new_v4().to_string();
    let (cols, rows) = client.attach(&session_id).await;

    // Should get valid dimensions back
    assert!(cols > 0, "cols should be > 0");
    assert!(rows > 0, "rows should be > 0");
}

#[tokio::test]
async fn attach_with_same_id_reuses_session() {
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;
    let mut client1 = TestClient::connect(addr).await;
    let mut client2 = TestClient::connect(addr).await;

    let session_id = Uuid::new_v4().to_string();

    // First client attaches - creates session
    let (cols1, rows1) = client1.attach(&session_id).await;

    // Second client attaches with same ID - reuses session
    let (cols2, rows2) = client2.attach(&session_id).await;

    // Both should get same dimensions
    assert_eq!(cols1, cols2);
    assert_eq!(rows1, rows2);
}

#[tokio::test]
async fn pty_input_produces_output() {
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;
    let mut client = TestClient::connect(addr).await;

    let session_id = Uuid::new_v4().to_string();
    client.attach(&session_id).await;

    // Send input to the PTY (cat will echo it back)
    client.pty_input_bytes(&session_id, b"hello\n").await;

    // Should receive the echoed output
    let output = client
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;

    // Output should contain "hello" (cat echoes input)
    let output_str = String::from_utf8_lossy(&output);
    assert!(
        output_str.contains("hello"),
        "Expected output to contain 'hello', got: {:?}",
        output_str
    );
}

#[tokio::test]
async fn pty_input_with_wrong_session_id_fails_silently() {
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;
    let mut client = TestClient::connect(addr).await;

    let real_session_id = Uuid::new_v4().to_string();
    let fake_session_id = Uuid::new_v4().to_string();

    // Attach to real session
    client.attach(&real_session_id).await;

    // Send input to WRONG session ID
    client.pty_input_bytes(&fake_session_id, b"hello\n").await;

    // Should NOT receive any output (input went nowhere)
    client.expect_no_message(Duration::from_millis(200)).await;
}

#[tokio::test]
async fn multiple_clients_receive_same_pty_output() {
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;
    let mut client1 = TestClient::connect(addr).await;
    let mut client2 = TestClient::connect(addr).await;

    let session_id = Uuid::new_v4().to_string();

    // Both clients attach to the same session
    client1.attach(&session_id).await;
    client2.attach(&session_id).await;

    // Client 1 sends input
    client1.pty_input_bytes(&session_id, b"shared\n").await;

    // Both clients should receive the output
    let output1 = client1
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;
    let output2 = client2
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;

    let output1_str = String::from_utf8_lossy(&output1);
    let output2_str = String::from_utf8_lossy(&output2);

    assert!(
        output1_str.contains("shared"),
        "Client 1 should receive 'shared'"
    );
    assert!(
        output2_str.contains("shared"),
        "Client 2 should receive 'shared'"
    );
}

#[tokio::test]
async fn session_id_mismatch_regression() {
    // This test specifically catches the bug where the server generated
    // a new UUID instead of using the client's session ID
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;
    let mut client = TestClient::connect(addr).await;

    // Use a specific session ID
    let session_id = "my-specific-session-id-12345";

    // Attach with our specific ID
    client.attach(session_id).await;

    // Send input using the SAME session ID
    client.pty_input_bytes(session_id, b"test\n").await;

    // If session IDs match correctly, we should get output back
    // If there's a mismatch (bug), this will timeout
    let output = client
        .expect_pty_output(session_id, Duration::from_secs(2))
        .await;

    let output_str = String::from_utf8_lossy(&output);
    assert!(
        output_str.contains("test"),
        "Expected output to contain 'test', got: {:?}. \
         This may indicate a session ID mismatch bug!",
        output_str
    );
}

#[tokio::test]
async fn list_sessions_shows_pty_sessions() {
    // This test catches the bug where list_sessions queried the wrong session manager
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;

    // Client 1 creates a PTY session by attaching
    let mut client1 = TestClient::connect(addr).await;
    let session_id = Uuid::new_v4().to_string();
    client1.attach(&session_id).await;

    // Client 2 should be able to discover the session via list_sessions
    let mut client2 = TestClient::connect(addr).await;
    let sessions = client2.list_sessions().await;

    assert!(
        sessions.contains(&session_id),
        "Expected list_sessions to contain '{}', but got: {:?}. \
         This may indicate that ListSessions is querying the wrong session manager!",
        session_id,
        sessions
    );
}

#[tokio::test]
async fn second_client_can_attach_to_discovered_session() {
    // Full flow test: Client 1 creates session, Client 2 discovers and attaches
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;

    // Client 1 creates session
    let mut client1 = TestClient::connect(addr).await;
    let session_id = Uuid::new_v4().to_string();
    client1.attach(&session_id).await;

    // Client 2 discovers session
    let mut client2 = TestClient::connect(addr).await;
    let sessions = client2.list_sessions().await;
    assert!(sessions.contains(&session_id));

    // Client 2 attaches to the discovered session
    client2.attach(&session_id).await;

    // Client 1 sends input
    client1.pty_input_bytes(&session_id, b"mirrored\n").await;

    // BOTH clients should receive the output (mirroring)
    let output1 = client1
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;
    let output2 = client2
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;

    let output1_str = String::from_utf8_lossy(&output1);
    let output2_str = String::from_utf8_lossy(&output2);

    assert!(
        output1_str.contains("mirrored"),
        "Client 1 should receive 'mirrored'"
    );
    assert!(
        output2_str.contains("mirrored"),
        "Client 2 should receive 'mirrored' (session mirroring)"
    );
}

#[tokio::test]
async fn ctrl_c_terminates_pty_process() {
    // Test that Ctrl+C (byte 0x03) is correctly sent to PTY and triggers SIGINT
    // This catches a bug where PTY read() didn't distinguish EOF from no-data,
    // causing the process exit to go undetected.
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;
    let mut client = TestClient::connect(addr).await;

    let session_id = Uuid::new_v4().to_string();
    client.attach(&session_id).await;

    // Verify cat is running by sending some data first
    client.pty_input_bytes(&session_id, b"hello\n").await;
    let output = client
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;
    assert!(
        String::from_utf8_lossy(&output).contains("hello"),
        "cat should echo input"
    );

    // Send Ctrl+C (0x03 = ETX = End of Text)
    // This should trigger SIGINT on the PTY, causing cat to exit
    client.pty_input_bytes(&session_id, &[0x03]).await;

    // cat should exit - we should receive pty_exit message
    // First we may receive the ^C echo, then the exit
    let exit_code = client
        .expect_pty_exit(&session_id, Duration::from_secs(2))
        .await;

    // Exit code should be None (signal) or 130 (128 + SIGINT)
    // The exact value depends on the shell/OS
    assert!(
        exit_code.is_none() || exit_code == Some(130) || exit_code == Some(2),
        "Expected cat to be terminated by SIGINT, got exit code: {:?}",
        exit_code
    );
}
