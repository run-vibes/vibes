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

use base64::Engine;
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

/// Test that output produced immediately when PTY spawns is received by client.
///
/// This test catches a race condition where:
/// 1. Session is created, PTY process starts outputting immediately
/// 2. pty_output_reader task is spawned
/// 3. Reader broadcasts PtyEvent::Output
/// 4. BUT conn_state.attach_pty() hasn't been called yet!
/// 5. handle_pty_event checks is_attached_to_pty() -> FALSE
/// 6. Event is DROPPED, client never receives initial output
///
/// The fix: call conn_state.attach_pty() BEFORE spawning the reader task.
///
/// Note: This race is timing-dependent. We run multiple iterations to increase
/// the probability of catching it. Even if individual runs pass, the race exists
/// in the code when attach_pty is called after spawning the reader.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn immediate_pty_output_is_received_on_new_session() {
    // Run multiple iterations to increase chance of hitting the race
    for iteration in 0..5 {
        // Use `echo` which outputs immediately when spawned and then exits
        let pty_config = PtyConfig {
            claude_path: "echo".into(),
            claude_args: vec![format!("welcome-{}", iteration)],
            ..Default::default()
        };
        let (_state, addr) = common::create_test_server_with_pty_config(pty_config).await;
        let mut client = TestClient::connect(addr).await;

        let session_id = Uuid::new_v4().to_string();
        let expected_output = format!("welcome-{}", iteration);

        // Send attach - this creates a NEW session
        // The echo command runs immediately and outputs
        client
            .conn
            .send_json(&serde_json::json!({
                "type": "attach",
                "session_id": session_id,
            }))
            .await;

        // Collect all messages until we get attach_ack
        // We should receive the echo output either before or after attach_ack
        let mut received_output = false;
        let mut received_ack = false;

        let timeout = Duration::from_secs(5);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout && !(received_ack && received_output) {
            if let Some(text) = client.conn.recv_timeout(Duration::from_millis(50)).await {
                let msg: serde_json::Value = serde_json::from_str(&text).unwrap();

                match msg["type"].as_str() {
                    Some("attach_ack") if msg["session_id"] == session_id => {
                        received_ack = true;
                    }
                    Some("pty_output") | Some("pty_replay") if msg["session_id"] == session_id => {
                        // Decode base64 data
                        let data = msg["data"].as_str().unwrap();
                        let decoded = base64::engine::general_purpose::STANDARD
                            .decode(data)
                            .unwrap();
                        let output_str = String::from_utf8_lossy(&decoded);
                        if output_str.contains(&expected_output) {
                            received_output = true;
                        }
                    }
                    _ => {} // Ignore other messages
                }
            }
        }

        assert!(
            received_ack,
            "Iteration {}: Should have received attach_ack",
            iteration
        );
        assert!(
            received_output,
            "Iteration {}: Should have received '{}' output from echo command. \
             This failure indicates the attach race condition: output was broadcast \
             before conn_state.attach_pty() was called, causing it to be dropped.",
            iteration, expected_output
        );
    }
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

#[tokio::test]
async fn replay_excludes_content_received_after_attach() {
    // This test verifies that when a client attaches and then receives pty_output
    // before sending resize, the subsequent pty_replay does NOT include that content.
    // This prevents duplicate content on mobile clients where:
    // 1. Client attaches
    // 2. Content arrives as pty_output (client displays it)
    // 3. Client sends resize
    // 4. Server sends pty_replay - should NOT include content from step 2
    let (_state, addr) = common::create_test_server_with_pty_config(test_pty_config()).await;

    // Client 1 creates session and generates some initial content
    let mut client1 = TestClient::connect(addr).await;
    let session_id = Uuid::new_v4().to_string();
    client1.attach(&session_id).await;

    // Generate initial content that goes to scrollback
    client1.pty_input_bytes(&session_id, b"initial\n").await;
    let _ = client1
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;

    // Small delay to ensure scrollback is populated
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Client 2 attaches - scrollback now contains "initial"
    let mut client2 = TestClient::connect(addr).await;
    client2.attach(&session_id).await;

    // Client 1 generates MORE content AFTER client 2 attached
    client1
        .pty_input_bytes(&session_id, b"after_attach\n")
        .await;

    // Client 2 receives this as pty_output (real-time)
    let realtime_output = client2
        .expect_pty_output(&session_id, Duration::from_secs(2))
        .await;
    let realtime_str = String::from_utf8_lossy(&realtime_output);
    assert!(
        realtime_str.contains("after_attach"),
        "Client 2 should receive 'after_attach' as pty_output"
    );

    // Now client 2 sends resize - this triggers replay
    client2.pty_resize(&session_id, 80, 24).await;

    // Client 2 should receive replay with ONLY "initial", NOT "after_attach"
    let replay = client2
        .expect_pty_replay(&session_id, Duration::from_secs(2))
        .await;
    let replay_str = String::from_utf8_lossy(&replay);

    assert!(
        replay_str.contains("initial"),
        "Replay should contain 'initial' (content from before attach)"
    );
    assert!(
        !replay_str.contains("after_attach"),
        "Replay should NOT contain 'after_attach' (content received as pty_output after attach). \
         Got replay: {:?}",
        replay_str
    );
}
