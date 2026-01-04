//! Shared test utilities for vibes-server integration tests

pub mod client;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use vibes_core::pty::PtyConfig;
use vibes_server::{AppState, ServerConfig, VibesServer};

/// Creates a test server with default config, returns state and address
#[allow(dead_code)]
pub async fn create_test_server() -> (Arc<AppState>, SocketAddr) {
    create_test_server_with_config(ServerConfig::default()).await
}

/// Creates a test server with custom config
#[allow(dead_code)]
pub async fn create_test_server_with_config(config: ServerConfig) -> (Arc<AppState>, SocketAddr) {
    // Use new_for_testing() to avoid loading external plugins
    // External plugins can have background tasks that outlive the test runtime
    let state = Arc::new(AppState::new_for_testing());

    let server = VibesServer::with_state(config, Arc::clone(&state));
    let addr = spawn_server(server).await;

    (state, addr)
}

/// Creates a test server with custom PTY config (for PTY integration tests)
#[allow(dead_code)]
pub async fn create_test_server_with_pty_config(
    pty_config: PtyConfig,
) -> (Arc<AppState>, SocketAddr) {
    // Use new_for_testing() to avoid loading external plugins
    let state = Arc::new(AppState::new_for_testing().with_pty_config(pty_config));

    let server = VibesServer::with_state(ServerConfig::default(), Arc::clone(&state));
    let addr = spawn_server(server).await;

    (state, addr)
}

/// Spawns server in background task, returns bound address
async fn spawn_server(server: VibesServer) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let _ = server.run_with_listener(listener).await;
    });

    // Brief delay to ensure server is accepting connections
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    addr
}
