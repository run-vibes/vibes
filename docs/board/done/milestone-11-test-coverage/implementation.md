# Milestone 3.6: Test Coverage Improvement - Implementation Plan

## Overview

Add comprehensive test coverage across three layers:
- **Unit tests**: Existing 466 tests (already good)
- **Integration tests**: In-process WebSocket/server tests (new)
- **E2E tests**: Playwright browser + CLI tests (new)

## Prerequisites

- Design approved in [design.md](./design.md)
- Fixes for history bug and deadlock already merged

---

## Phase 1: Test Infrastructure

### Task 1.1: Add `run_with_listener()` to VibesServer

**File**: `vibes-server/src/lib.rs`

Add method that accepts a pre-bound listener for test control:

```rust
/// Run server with a pre-bound listener (for testing)
pub async fn run_with_listener(self, listener: TcpListener) -> Result<(), ServerError> {
    let addr = listener.local_addr()
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    tracing::info!("vibes server listening on {}", addr);

    self.start_event_forwarding();

    if let Some(notification_service) = &self.notification_service {
        self.start_notification_service(notification_service.clone());
    }

    if self.state.history.is_some() {
        self.start_history_service();
    }

    let router = create_router(self.state);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(())
}
```

**Verification**: `cargo check -p vibes-server`

---

### Task 1.2: Create test server factory

**File**: `vibes-server/tests/common/mod.rs`

```rust
//! Shared test utilities for vibes-server integration tests

pub mod client;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use vibes_core::history::{HistoryService, SqliteHistoryStore};
use vibes_server::{AppState, ServerConfig, VibesServer};

/// Creates a test server with in-memory SQLite, returns address
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
```

**Verification**: `cargo check -p vibes-server --tests`

---

### Task 1.3: Create WebSocket test client

**File**: `vibes-server/tests/common/client.rs`

```rust
//! WebSocket test client for protocol testing

use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// Low-level WebSocket connection
pub struct WsConnection {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsConnection {
    /// Connect to WebSocket endpoint
    pub async fn connect(addr: SocketAddr) -> Self {
        let url = format!("ws://{}/ws", addr);
        let (ws, _) = connect_async(&url).await.expect("Failed to connect");
        Self { ws }
    }

    /// Send raw text message
    pub async fn send_raw(&mut self, msg: &str) {
        self.ws.send(Message::Text(msg.to_string())).await.unwrap();
    }

    /// Send JSON message
    pub async fn send_json<T: Serialize>(&mut self, msg: &T) {
        let json = serde_json::to_string(msg).unwrap();
        self.send_raw(&json).await;
    }

    /// Receive raw text message
    pub async fn recv_raw(&mut self) -> String {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Text(text))) => return text,
                Some(Ok(Message::Ping(_))) => continue,
                Some(Ok(_)) => continue,
                Some(Err(e)) => panic!("WebSocket error: {}", e),
                None => panic!("WebSocket closed"),
            }
        }
    }

    /// Receive and deserialize JSON message
    pub async fn recv_json<T: DeserializeOwned>(&mut self) -> T {
        let text = self.recv_raw().await;
        serde_json::from_str(&text).expect("Failed to parse JSON")
    }

    /// Receive with timeout, returns None if timeout
    pub async fn recv_timeout(&mut self, duration: Duration) -> Option<String> {
        tokio::time::timeout(duration, self.recv_raw()).await.ok()
    }
}

/// High-level test client with helper methods
pub struct TestClient {
    conn: WsConnection,
}

impl TestClient {
    /// Connect to server
    pub async fn connect(addr: SocketAddr) -> Self {
        Self {
            conn: WsConnection::connect(addr).await,
        }
    }

    /// Create a new session, returns session ID
    pub async fn create_session(&mut self, name: Option<&str>) -> String {
        self.conn.send_json(&serde_json::json!({
            "type": "CreateSession",
            "name": name,
        })).await;

        let response: serde_json::Value = self.conn.recv_json().await;
        assert_eq!(response["type"], "SessionCreated");
        response["session_id"].as_str().unwrap().to_string()
    }

    /// Subscribe to sessions
    pub async fn subscribe(&mut self, session_ids: &[&str], catch_up: bool) {
        self.conn.send_json(&serde_json::json!({
            "type": "Subscribe",
            "session_ids": session_ids,
            "catch_up": catch_up,
        })).await;

        let response: serde_json::Value = self.conn.recv_json().await;
        assert_eq!(response["type"], "SubscribeAck");
    }

    /// Send input to session
    pub async fn send_input(&mut self, session_id: &str, content: &str) {
        self.conn.send_json(&serde_json::json!({
            "type": "Input",
            "session_id": session_id,
            "content": content,
        })).await;
    }

    /// Receive next message
    pub async fn recv(&mut self) -> serde_json::Value {
        self.conn.recv_json().await
    }

    /// Assert no message received within duration
    pub async fn expect_no_message(&mut self, duration: Duration) {
        assert!(
            self.conn.recv_timeout(duration).await.is_none(),
            "Expected no message but received one"
        );
    }
}
```

**Verification**: `cargo check -p vibes-server --tests`

---

### Task 1.4: Create SlowMockBackend

**File**: `vibes-core/src/backend/slow_mock.rs`

```rust
//! Slow mock backend for concurrency testing

use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::broadcast;

use super::mock::MockBackend;
use super::traits::{BackendState, ClaudeBackend};
use crate::error::BackendError;
use crate::events::ClaudeEvent;

/// MockBackend wrapper that adds configurable delay
pub struct SlowMockBackend {
    inner: MockBackend,
    delay: Duration,
}

impl SlowMockBackend {
    /// Create with specified delay
    pub fn new(delay: Duration) -> Self {
        Self {
            inner: MockBackend::new(),
            delay,
        }
    }

    /// Queue a response (delegates to inner)
    pub fn queue_response(&mut self, events: Vec<ClaudeEvent>) {
        self.inner.queue_response(events);
    }
}

#[async_trait]
impl ClaudeBackend for SlowMockBackend {
    async fn send(&mut self, input: &str) -> Result<(), BackendError> {
        // Delay before processing
        tokio::time::sleep(self.delay).await;
        self.inner.send(input).await
    }

    fn subscribe(&self) -> broadcast::Receiver<ClaudeEvent> {
        self.inner.subscribe()
    }

    async fn respond_permission(
        &mut self,
        request_id: &str,
        approved: bool,
    ) -> Result<(), BackendError> {
        self.inner.respond_permission(request_id, approved).await
    }

    fn claude_session_id(&self) -> &str {
        self.inner.claude_session_id()
    }

    fn state(&self) -> BackendState {
        self.inner.state()
    }

    async fn shutdown(&mut self) -> Result<(), BackendError> {
        self.inner.shutdown().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Usage;
    use std::time::Instant;

    #[tokio::test]
    async fn send_delays_by_configured_duration() {
        let mut backend = SlowMockBackend::new(Duration::from_millis(50));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);

        let start = Instant::now();
        backend.send("test").await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100));
    }
}
```

**Also update** `vibes-core/src/backend/mod.rs`:
```rust
mod slow_mock;
pub use slow_mock::SlowMockBackend;
```

**Verification**: `cargo test -p vibes-core slow_mock`

---

## Phase 2: Critical Gap Coverage

### Task 2.1: Server configuration tests

**File**: `vibes-server/tests/server_config.rs`

```rust
//! Tests that server is configured correctly with all features

mod common;

use vibes_server::{ServerConfig, VibesServer};

#[tokio::test]
async fn server_with_all_features_has_history() {
    let (state, _addr) = common::create_test_server().await;
    assert!(state.history.is_some(), "History should be enabled");
}

#[tokio::test]
async fn server_state_has_event_bus() {
    let (state, _addr) = common::create_test_server().await;
    // EventBus is always present, verify it works
    let _rx = state.event_bus.subscribe();
}

#[tokio::test]
async fn server_state_has_session_manager() {
    let (state, _addr) = common::create_test_server().await;
    let sessions = state.sessions.list_sessions().await;
    assert!(sessions.is_empty(), "No sessions initially");
}
```

**Verification**: `cargo test -p vibes-server --test server_config`

---

### Task 2.2: Session manager concurrency tests

**File**: `vibes-core/tests/concurrency.rs`

```rust
//! Concurrency tests for SessionManager

use std::sync::Arc;
use std::time::{Duration, Instant};

use vibes_core::backend::SlowMockBackend;
use vibes_core::events::{ClaudeEvent, Usage};
use vibes_core::session::SessionManager;

#[tokio::test]
async fn concurrent_sends_to_different_sessions_dont_block() {
    let manager = Arc::new(SessionManager::new());

    // Create two sessions with different delays
    let id1 = {
        let mut backend = SlowMockBackend::new(Duration::from_millis(100));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);
        manager.create_session_with_backend(None, backend).await.unwrap()
    };

    let id2 = {
        let mut backend = SlowMockBackend::new(Duration::from_millis(10));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);
        manager.create_session_with_backend(None, backend).await.unwrap()
    };

    let start = Instant::now();

    let m1 = Arc::clone(&manager);
    let m2 = Arc::clone(&manager);
    let id1_clone = id1.clone();
    let id2_clone = id2.clone();

    let (r1, r2) = tokio::join!(
        async move { m1.send_to_session(&id1_clone, "slow").await },
        async move { m2.send_to_session(&id2_clone, "fast").await },
    );

    let elapsed = start.elapsed();

    assert!(r1.is_ok(), "Slow session should succeed");
    assert!(r2.is_ok(), "Fast session should succeed");

    // If properly parallelized, total time should be ~100ms, not 110ms
    // Allow some margin for test flakiness
    assert!(
        elapsed < Duration::from_millis(150),
        "Concurrent sends should not block: took {:?}",
        elapsed
    );
}

#[tokio::test]
async fn list_sessions_during_active_send() {
    let manager = Arc::new(SessionManager::new());

    // Create session with slow backend
    let id = {
        let mut backend = SlowMockBackend::new(Duration::from_millis(100));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);
        manager.create_session_with_backend(None, backend).await.unwrap()
    };

    let m1 = Arc::clone(&manager);
    let m2 = Arc::clone(&manager);
    let id_clone = id.clone();

    // Start a slow send
    let send_handle = tokio::spawn(async move {
        m1.send_to_session(&id_clone, "slow").await
    });

    // Give it time to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // list_sessions should NOT block
    let start = Instant::now();
    let sessions = m2.list_sessions().await;
    let elapsed = start.elapsed();

    assert_eq!(sessions.len(), 1);
    assert!(
        elapsed < Duration::from_millis(20),
        "list_sessions blocked for {:?}",
        elapsed
    );

    // Wait for send to complete
    send_handle.await.unwrap().unwrap();
}
```

**Verification**: `cargo test -p vibes-core --test concurrency`

---

### Task 2.3: WebSocket protocol tests

**File**: `vibes-server/tests/ws_protocol.rs`

```rust
//! WebSocket protocol integration tests

mod common;

use common::client::TestClient;

#[tokio::test]
async fn create_session_returns_session_id() {
    let (_state, addr) = common::create_test_server().await;
    let mut client = TestClient::connect(addr).await;

    let session_id = client.create_session(Some("test-session")).await;

    assert!(!session_id.is_empty());
}

#[tokio::test]
async fn subscribe_receives_ack() {
    let (_state, addr) = common::create_test_server().await;
    let mut client = TestClient::connect(addr).await;

    let session_id = client.create_session(None).await;
    client.subscribe(&[&session_id], false).await;

    // subscribe() already asserts SubscribeAck
}

#[tokio::test]
async fn multiple_clients_receive_same_events() {
    let (state, addr) = common::create_test_server().await;

    let mut client1 = TestClient::connect(addr).await;
    let mut client2 = TestClient::connect(addr).await;

    let session_id = client1.create_session(None).await;

    client1.subscribe(&[&session_id], false).await;
    client2.subscribe(&[&session_id], false).await;

    // Publish event directly to event bus
    state.event_bus.publish(
        session_id.clone(),
        vibes_core::events::ClaudeEvent::TextDelta {
            text: "Hello".to_string(),
        },
    );

    // Both clients should receive it
    let msg1 = client1.recv().await;
    let msg2 = client2.recv().await;

    assert_eq!(msg1["type"], "Claude");
    assert_eq!(msg2["type"], "Claude");
}

#[tokio::test]
async fn unsubscribed_client_receives_no_events() {
    let (state, addr) = common::create_test_server().await;

    let mut client = TestClient::connect(addr).await;
    let session_id = client.create_session(None).await;

    // Don't subscribe

    // Publish event
    state.event_bus.publish(
        session_id.clone(),
        vibes_core::events::ClaudeEvent::TextDelta {
            text: "Hello".to_string(),
        },
    );

    // Client should not receive anything
    client.expect_no_message(std::time::Duration::from_millis(50)).await;
}
```

**Verification**: `cargo test -p vibes-server --test ws_protocol`

---

### Task 2.4: History catch-up tests

**File**: `vibes-server/tests/history_catchup.rs`

```rust
//! Tests for history catch-up on subscribe

mod common;

use common::client::TestClient;
use vibes_core::events::ClaudeEvent;
use vibes_core::history::MessageRole;

#[tokio::test]
async fn subscribe_with_catchup_returns_history() {
    let (state, addr) = common::create_test_server().await;

    let mut client1 = TestClient::connect(addr).await;
    let session_id = client1.create_session(None).await;

    // Store some history directly
    if let Some(history) = &state.history {
        history.store_message(
            &session_id,
            MessageRole::User,
            "Hello",
            None,
        ).await.unwrap();

        history.store_message(
            &session_id,
            MessageRole::Assistant,
            "Hi there!",
            None,
        ).await.unwrap();
    }

    // New client subscribes with catch_up
    let mut client2 = TestClient::connect(addr).await;
    client2.conn.send_json(&serde_json::json!({
        "type": "Subscribe",
        "session_ids": [session_id],
        "catch_up": true,
    })).await;

    let response: serde_json::Value = client2.conn.recv_json().await;

    assert_eq!(response["type"], "SubscribeAck");
    let history = response["history"].as_array().unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0]["content"], "Hello");
    assert_eq!(history[1]["content"], "Hi there!");
}

#[tokio::test]
async fn subscribe_without_catchup_returns_empty_history() {
    let (state, addr) = common::create_test_server().await;

    let mut client1 = TestClient::connect(addr).await;
    let session_id = client1.create_session(None).await;

    // Store history
    if let Some(history) = &state.history {
        history.store_message(
            &session_id,
            MessageRole::User,
            "Hello",
            None,
        ).await.unwrap();
    }

    // Subscribe without catch_up
    let mut client2 = TestClient::connect(addr).await;
    client2.conn.send_json(&serde_json::json!({
        "type": "Subscribe",
        "session_ids": [session_id],
        "catch_up": false,
    })).await;

    let response: serde_json::Value = client2.conn.recv_json().await;

    assert_eq!(response["type"], "SubscribeAck");
    let history = response["history"].as_array().unwrap();
    assert!(history.is_empty());
}
```

**Verification**: `cargo test -p vibes-server --test history_catchup`

---

## Phase 3: E2E Setup

### Task 3.1: Create e2e-tests directory structure

**Create files**:

`e2e-tests/package.json`:
```json
{
  "name": "vibes-e2e-tests",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "test": "playwright test",
    "test:headed": "playwright test --headed",
    "test:debug": "playwright test --debug"
  },
  "devDependencies": {
    "@playwright/test": "^1.40.0",
    "@types/node": "^20.10.0",
    "typescript": "^5.3.0"
  }
}
```

`e2e-tests/playwright.config.ts`:
```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 60000,
  retries: 1,
  use: {
    headless: true,
    video: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { browserName: 'chromium' },
    },
  ],
});
```

`e2e-tests/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "./dist"
  },
  "include": ["**/*.ts"]
}
```

**Verification**: `cd e2e-tests && npm install`

---

### Task 3.2: Add just commands for e2e tests

**Update** `justfile`:

```just
# E2E tests with Playwright
test-e2e:
    cd e2e-tests && npm test

# E2E tests in headed mode (visible browser)
test-e2e-headed:
    cd e2e-tests && npm run test:headed

# Install Playwright browsers
e2e-setup:
    cd e2e-tests && npm install && npx playwright install chromium
```

**Verification**: `just --list | grep e2e`

---

### Task 3.3: Create Vibes fixture

**File**: `e2e-tests/fixtures/vibes.ts`

```typescript
import { test as base } from '@playwright/test';
import { spawn, spawnSync, ChildProcess } from 'child_process';
import { Readable } from 'stream';

export type VibesFixture = {
  serverUrl: string;
  cli: (...args: string[]) => ChildProcess;
};

export const test = base.extend<VibesFixture>({
  serverUrl: async ({}, use) => {
    // Build first to ensure we have the latest
    const buildResult = spawnSync('cargo', ['build', '--release'], {
      cwd: '..',
      stdio: 'inherit'
    });

    if (buildResult.status !== 0) {
      throw new Error('Failed to build vibes');
    }

    // Start server on random port
    const server = spawn('../target/release/vibes', ['serve', '--port', '0'], {
      cwd: __dirname,
    });

    const port = await waitForPort(server.stdout!);
    const url = `http://127.0.0.1:${port}`;

    await use(url);

    server.kill('SIGTERM');
  },

  cli: async ({}, use) => {
    const processes: ChildProcess[] = [];

    const spawnCli = (...args: string[]) => {
      const proc = spawn('../target/release/vibes', args, {
        cwd: __dirname,
      });
      processes.push(proc);
      return proc;
    };

    await use(spawnCli);

    // Cleanup all spawned processes
    processes.forEach(p => p.kill('SIGTERM'));
  },
});

async function waitForPort(stdout: Readable): Promise<number> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('Timeout waiting for server')), 10000);

    let buffer = '';
    stdout.on('data', (data) => {
      buffer += data.toString();
      // Look for "listening on 127.0.0.1:XXXX"
      const match = buffer.match(/listening on 127\.0\.0\.1:(\d+)/);
      if (match) {
        clearTimeout(timeout);
        resolve(parseInt(match[1], 10));
      }
    });
  });
}

export { expect } from '@playwright/test';
```

**File**: `e2e-tests/helpers/cli.ts`

```typescript
import { ChildProcess } from 'child_process';
import { Readable } from 'stream';

export async function readUntil(
  stream: Readable,
  text: string,
  timeoutMs = 10000
): Promise<string> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error(`Timeout waiting for "${text}"`)),
      timeoutMs
    );

    let buffer = '';
    const onData = (data: Buffer) => {
      buffer += data.toString();
      if (buffer.includes(text)) {
        clearTimeout(timeout);
        stream.removeListener('data', onData);
        resolve(buffer);
      }
    };

    stream.on('data', onData);
  });
}

export async function waitForSessionId(proc: ChildProcess): Promise<string> {
  const output = await readUntil(proc.stdout!, 'Session:');
  const match = output.match(/Session:\s*(\S+)/);
  if (!match) throw new Error('Could not find session ID in output');
  return match[1];
}
```

**Verification**: `cd e2e-tests && npx tsc --noEmit`

---

## Phase 4: E2E Mirroring Tests

### Task 4.1: Session flow test

**File**: `e2e-tests/tests/session-flow.spec.ts`

```typescript
import { test, expect } from '../fixtures/vibes';

test('can create session via web UI', async ({ page, serverUrl }) => {
  await page.goto(serverUrl);

  // Should show session list or empty state
  await expect(page.locator('body')).toBeVisible();

  // Create new session
  await page.getByRole('button', { name: /new session/i }).click();

  // Session should appear
  await expect(page.getByTestId('session-item')).toBeVisible();
});

test('web UI shows session list', async ({ page, serverUrl, cli }) => {
  // Create session via CLI
  const cliProc = cli('claude', '--session-name', 'e2e-test');

  // Wait for session to be created
  await new Promise(r => setTimeout(r, 1000));

  await page.goto(serverUrl);

  // Should see the session
  await expect(page.getByText('e2e-test')).toBeVisible();

  cliProc.kill();
});
```

**Verification**: `just test-e2e`

---

### Task 4.2: CLI → Web mirroring test

**File**: `e2e-tests/tests/cli-to-web.spec.ts`

```typescript
import { test, expect } from '../fixtures/vibes';
import { waitForSessionId } from '../helpers/cli';

test('CLI input appears in Web UI', async ({ page, serverUrl, cli }) => {
  // Start CLI session
  const cliProc = cli('claude', '--session-name', 'mirror-test');
  const sessionId = await waitForSessionId(cliProc);

  // Open session in browser
  await page.goto(`${serverUrl}/session/${sessionId}`);

  // Send input from CLI
  cliProc.stdin!.write('Hello from CLI\n');

  // Should appear in Web UI
  await expect(page.getByText('Hello from CLI')).toBeVisible({ timeout: 5000 });

  // Should show CLI source attribution
  await expect(page.getByTestId('source-cli')).toBeVisible();

  cliProc.kill();
});
```

---

### Task 4.3: Web → CLI mirroring test

**File**: `e2e-tests/tests/web-to-cli.spec.ts`

```typescript
import { test, expect } from '../fixtures/vibes';
import { readUntil, waitForSessionId } from '../helpers/cli';

test('Web UI input appears in CLI with prefix', async ({ page, serverUrl, cli }) => {
  // Start CLI session
  const cliProc = cli('claude', '--session-name', 'web-input-test');
  const sessionId = await waitForSessionId(cliProc);

  // Open in browser
  await page.goto(`${serverUrl}/session/${sessionId}`);

  // Send from Web UI
  await page.getByRole('textbox').fill('Hello from Web');
  await page.getByRole('button', { name: /send/i }).click();

  // CLI should show prefixed message
  const output = await readUntil(cliProc.stdout!, 'Hello from Web', 5000);
  expect(output).toContain('[Web UI]:');

  cliProc.kill();
});

test('late joiner receives history', async ({ page, serverUrl, cli }) => {
  // Start CLI session and send messages
  const cliProc = cli('claude', '--session-name', 'history-test');
  const sessionId = await waitForSessionId(cliProc);

  cliProc.stdin!.write('First message\n');
  await readUntil(cliProc.stdout!, 'First message');

  // Wait for history to be stored
  await new Promise(r => setTimeout(r, 500));

  // Open in browser (late joiner)
  await page.goto(`${serverUrl}/session/${sessionId}`);

  // Should see previous message via catch-up
  await expect(page.getByText('First message')).toBeVisible({ timeout: 5000 });

  cliProc.kill();
});
```

**Verification**: `just test-e2e`

---

## Phase 5: CI Integration

### Task 5.1: Add integration tests to CI

**Update** `.github/workflows/ci.yml`:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: DeterminateSystems/nix-installer-action@main
      - uses: DeterminateSystems/magic-nix-cache-action@main

      - name: Run checks
        run: nix develop --command just check

      - name: Run tests (unit + integration)
        run: nix develop --command just test
```

**Verification**: Push to branch, verify CI runs integration tests

---

### Task 5.2: Add E2E tests to CI

**Update** `.github/workflows/ci.yml`:

```yaml
  e2e:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request' || github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - uses: DeterminateSystems/nix-installer-action@main
      - uses: DeterminateSystems/magic-nix-cache-action@main

      - name: Build release binary
        run: nix develop --command cargo build --release

      - name: Install Playwright
        run: |
          cd e2e-tests
          npm ci
          npx playwright install chromium --with-deps

      - name: Run E2E tests
        run: nix develop --command just test-e2e

      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report
          path: e2e-tests/playwright-report/
```

**Verification**: Create PR, verify E2E tests run

---

## Success Criteria

- [ ] All 16 tasks complete
- [ ] `just test` runs unit + integration tests
- [ ] `just test-e2e` runs Playwright tests
- [ ] CI runs integration tests on every push
- [ ] CI runs E2E tests on PR/main
- [ ] Concurrency test proves no deadlock
- [ ] Server config test proves history is enabled
- [ ] Mirroring tests prove CLI ↔ Web sync works
