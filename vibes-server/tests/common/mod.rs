//! Shared test utilities for vibes-server integration tests

pub mod client;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use vibes_core::history::{HistoryService, SqliteHistoryStore};
use vibes_server::{AppState, ServerConfig, VibesServer};

/// Creates a test server with in-memory SQLite, returns state and address
pub async fn create_test_server() -> (Arc<AppState>, SocketAddr) {
    create_test_server_with_config(ServerConfig::default()).await
}

/// Creates a test server with custom config
pub async fn create_test_server_with_config(config: ServerConfig) -> (Arc<AppState>, SocketAddr) {
    // In-memory SQLite for speed and isolation
    let store = SqliteHistoryStore::open(":memory:").unwrap();
    let history = Arc::new(HistoryService::new(Arc::new(store)));
    let state = Arc::new(AppState::new().with_history(history));

    let server = VibesServer::with_state(config, Arc::clone(&state));
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
