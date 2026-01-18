# Milestone 3.6: Test Coverage Improvement

## Problem Statement

Current test coverage has significant gaps that allowed two critical bugs to ship:
1. SQLite history never enabled in production (serve.rs never called `with_all_features()`)
2. Session manager deadlock (global write lock held during async operations)

These bugs could have been caught with better integration testing.

## Current State Analysis

### What We Have (Good)

| Category | Count | Notes |
|----------|-------|-------|
| Unit tests | 466 | Good coverage for individual components |
| Mock backends | Yes | MockBackend for testing without Claude CLI |
| Integration test scaffolding | 8 | Requires Claude CLI, ignored in CI |

### What's Missing (Gaps)

| Category | Gap | Impact |
|----------|-----|--------|
| Server startup tests | No tests verify serve.rs creates server correctly | History bug shipped |
| WebSocket protocol tests | No tests for full message flow | No confidence in protocol |
| Multi-client scenarios | No tests for concurrent sessions | Deadlock shipped |
| End-to-end flows | Only CLI help tests exist | No workflow validation |
| History integration | Service tested in isolation only | History never actually used |
| Load/stress testing | None | Performance issues hidden |

## Test Categories to Add

### Priority 1: Server Configuration Tests

Test that serve.rs creates the server with correct features enabled.

```rust
// vibes-cli/tests/serve_config_test.rs
#[tokio::test]
async fn server_has_history_enabled() {
    let server = VibesServer::with_all_features(ServerConfig::default()).await.unwrap();
    assert!(server.state().history.is_some());
}

#[tokio::test]
async fn server_has_notifications_enabled() {
    let server = VibesServer::with_all_features(ServerConfig::default()).await.unwrap();
    assert!(server.state().vapid.is_some());
}
```

### Priority 2: Session Manager Concurrency Tests

Test that operations on different sessions don't block each other.

```rust
#[tokio::test]
async fn concurrent_sends_to_different_sessions_dont_block() {
    let manager = create_test_manager();
    let id1 = manager.create_session(None).await;
    let id2 = manager.create_session(None).await;

    // Configure session 1 to take 100ms to respond
    // Configure session 2 to respond immediately

    let start = Instant::now();
    let (r1, r2) = tokio::join!(
        manager.send_to_session(&id1, "slow"),
        manager.send_to_session(&id2, "fast"),
    );

    // If there's no deadlock, both should complete
    // and total time should be ~100ms, not 200ms
    assert!(start.elapsed() < Duration::from_millis(150));
}
```

### Priority 3: WebSocket Protocol Integration Tests

Test the full message flow through the WebSocket layer.

```rust
// vibes-server/tests/ws_integration.rs
#[tokio::test]
async fn websocket_subscribe_receives_events() {
    let (server, client) = create_test_server_and_client().await;

    // Create session
    client.send(CreateSession { name: None, request_id: "1" }).await;
    let session_id = client.recv().await.session_id;

    // Subscribe
    client.send(Subscribe { session_ids: vec![session_id], catch_up: false }).await;

    // Trigger event
    server.state().event_bus.publish(claude_event(session_id)).await;

    // Verify client receives it
    let msg = client.recv().await;
    assert!(matches!(msg, ServerMessage::Claude { .. }));
}
```

### Priority 4: History Catch-up Tests

Test that late joiners receive history.

```rust
#[tokio::test]
async fn subscribe_with_catchup_returns_history() {
    let (server, client1, client2) = create_test_scenario().await;

    // Client 1 creates session and sends messages
    let session_id = client1.create_session().await;
    client1.send_input("Hello").await;
    client1.wait_for_response().await;

    // Client 2 subscribes with catch_up
    client2.send(Subscribe {
        session_ids: vec![session_id],
        catch_up: true
    }).await;

    // Should receive SubscribeAck with history
    let ack = client2.recv().await;
    assert!(matches!(ack, ServerMessage::SubscribeAck { .. }));
    assert!(!ack.history.is_empty());
}
```

### Priority 5: CLI-Web Mirroring Tests

Test that input from one client appears on the other.

```rust
#[tokio::test]
async fn cli_input_appears_in_web_ui() {
    let (server, cli_client, web_client) = create_multi_client_scenario().await;

    let session_id = cli_client.create_session().await;
    web_client.subscribe(session_id, true).await;

    // CLI sends input
    cli_client.send(Input { session_id, content: "Hello" }).await;

    // Web receives UserInput broadcast
    let msg = web_client.recv().await;
    assert!(matches!(msg, ServerMessage::UserInput {
        source: InputSource::Cli,
        content: "Hello",
        ..
    }));
}
```

## Test Infrastructure Needed

### 1. In-Memory Test Server

```rust
/// Creates a test server bound to a random port
async fn create_test_server() -> (VibesServer, SocketAddr) {
    let config = ServerConfig::new("127.0.0.1", 0); // Port 0 = random
    let server = VibesServer::with_all_features(config).await.unwrap();
    let addr = server.local_addr();
    (server, addr)
}
```

### 2. WebSocket Test Client

```rust
/// Test client for WebSocket protocol testing
struct TestClient {
    ws: WebSocket,
    messages: Vec<ServerMessage>,
}

impl TestClient {
    async fn connect(addr: SocketAddr) -> Self;
    async fn send(&mut self, msg: ClientMessage);
    async fn recv(&mut self) -> ServerMessage;
    async fn recv_timeout(&mut self, duration: Duration) -> Option<ServerMessage>;
}
```

### 3. Slow Mock Backend

```rust
/// MockBackend that delays responses for concurrency testing
struct SlowMockBackend {
    delay: Duration,
    inner: MockBackend,
}
```

## Implementation Plan

### Phase 1: Test Infrastructure (3 tasks)
1. Create test server factory with in-memory SQLite
2. Create WebSocket test client utility
3. Create SlowMockBackend for concurrency tests

### Phase 2: Critical Gap Coverage (4 tasks)
4. Add server configuration tests
5. Add session manager concurrency tests
6. Add basic WebSocket protocol tests
7. Add history catch-up tests

### Phase 3: Feature Coverage (3 tasks)
8. Add CLI-Web mirroring tests
9. Add multi-client scenario tests
10. Add permission flow tests

### Phase 4: Documentation (2 tasks)
11. Document test patterns in CLAUDE.md
12. Add test coverage report to CI

## Success Criteria

- [ ] All new tests pass
- [ ] Concurrency test proves no deadlock
- [ ] Server startup test proves history is enabled
- [ ] WebSocket tests prove end-to-end message flow
- [ ] CI runs integration tests (not just unit tests)
