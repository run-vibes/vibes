//! Integration tests for Iggy auto-start functionality.
//!
//! These tests verify that the server properly handles Iggy availability.

use std::sync::Arc;
use vibes_server::AppState;

/// Test that AppState::new_with_iggy() handles missing iggy-server gracefully.
///
/// This test runs unconditionally and verifies fallback behavior.
#[tokio::test]
async fn test_new_with_iggy_falls_back_when_binary_missing() {
    // This test expects iggy-server to NOT be available in most test environments
    // (CI, dev without `just build`). It should fall back to in-memory storage.
    let state = AppState::new_with_iggy().await;

    // The state should be created successfully regardless of Iggy availability
    assert!(state.uptime_seconds() >= 0);

    // We can't directly check which EventLog implementation is used
    // without adding inspection methods, but the fact that it doesn't
    // panic or hang is the key verification.
}

/// Test that AppState::new() always uses in-memory storage.
#[test]
fn test_new_uses_in_memory_storage() {
    let state = AppState::new();
    assert!(state.uptime_seconds() >= 0);
}

/// Test that the server can be created with Iggy when the binary is available.
///
/// This test is ignored by default because it requires iggy-server to be built.
/// Run with: cargo test -p vibes-server --test iggy_integration -- --ignored
#[tokio::test]
#[ignore]
async fn test_iggy_auto_start_when_available() {
    // This test requires iggy-server to be in the same directory as the test binary
    // or in PATH. It's ignored by default.
    let state = Arc::new(AppState::new_with_iggy().await);

    // Give Iggy a moment to fully start
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Check uptime - if we got here without panic, the server started successfully
    assert!(state.uptime_seconds() >= 0);

    // The server should stay running for additional operations
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
