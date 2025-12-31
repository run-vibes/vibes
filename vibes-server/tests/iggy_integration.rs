//! Integration tests for Iggy auto-start functionality.
//!
//! These tests verify that the server properly handles Iggy availability.

use std::sync::Arc;
use std::time::Duration;
use vibes_server::AppState;

/// Time to wait for Iggy server to fully initialize after startup.
const IGGY_INIT_WAIT: Duration = Duration::from_secs(1);

/// Test that AppState::new_with_iggy() handles missing iggy-server gracefully.
///
/// This test runs unconditionally and verifies fallback behavior.
/// Success criteria: completes without panic or hang.
#[tokio::test]
async fn test_new_with_iggy_falls_back_when_binary_missing() {
    // This test expects iggy-server to NOT be available in most test environments
    // (CI, dev without `just build`). It should fall back to in-memory storage.
    let _state = AppState::new_with_iggy().await;

    // We can't directly check which EventLog implementation is used
    // without adding inspection methods, but the fact that it doesn't
    // panic or hang is the key verification.
}

/// Test that AppState::new() always uses in-memory storage.
/// Success criteria: completes without panic.
#[test]
fn test_new_uses_in_memory_storage() {
    let _state = AppState::new();
    // Success is indicated by not panicking
}

/// Test that the server can be created with Iggy when the binary is available.
///
/// This test is ignored by default because it requires iggy-server to be built.
/// Run with: cargo test -p vibes-server --test iggy_integration -- --ignored
/// Success criteria: completes without panic, server stays running.
#[tokio::test]
#[ignore]
async fn test_iggy_auto_start_when_available() {
    // This test requires iggy-server to be in the same directory as the test binary
    // or in PATH. It's ignored by default.
    let state = Arc::new(AppState::new_with_iggy().await);

    // Give Iggy a moment to fully initialize
    tokio::time::sleep(IGGY_INIT_WAIT).await;

    // Verify the server stays running for additional operations
    // (accessing uptime confirms state is valid and not dropped)
    let _ = state.uptime_seconds();
    tokio::time::sleep(Duration::from_millis(100)).await;
}
