---
id: BUG0001
title: Fix CWD Propagation Implementation Plan
type: bug
status: done
priority: medium
epics: [core]
depends: []
estimate:
created: 2026-01-07
updated: 2026-01-07
milestone: 
---
# Fix CWD Propagation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix bug where `vibes claude` spawns Claude in the server's directory instead of the CLI's current working directory.

**Architecture:** The CLI captures its cwd, sends it via the `Attach` WebSocket message, the server passes it through to the PTY manager, and the backend sets it on the `CommandBuilder` before spawning.

**Tech Stack:** Rust, portable_pty, serde, tokio

---

## Task 1: Add `cwd` field to protocol `Attach` message

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs:48-54`
- Test: `vibes-server/src/ws/protocol.rs` (inline tests)

**Step 1: Write the failing test**

Add test in the `#[cfg(test)]` module at the end of `protocol.rs`:

```rust
#[test]
fn test_client_message_attach_with_cwd_roundtrip() {
    let msg = ClientMessage::Attach {
        session_id: "sess-1".to_string(),
        name: Some("my-session".to_string()),
        cwd: Some("/home/user/project".to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""cwd":"/home/user/project""#));
}

#[test]
fn test_client_message_attach_without_cwd_field_deserializes() {
    // Backwards compatibility - old clients that don't send cwd field
    let json = r#"{"type":"attach","session_id":"sess-1"}"#;
    let parsed: ClientMessage = serde_json::from_str(json).unwrap();
    assert!(matches!(
        parsed,
        ClientMessage::Attach { session_id, name, cwd }
        if session_id == "sess-1" && name.is_none() && cwd.is_none()
    ));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server test_client_message_attach_with_cwd`

Expected: FAIL with compile error "no field `cwd` on type `ClientMessage::Attach`"

**Step 3: Write minimal implementation**

Modify the `Attach` variant in `ClientMessage` enum (around line 48):

```rust
/// Attach to a session (receive PTY output)
/// Creates the session if it doesn't exist.
Attach {
    /// Session ID to attach to
    session_id: String,
    /// Optional session name (used when creating new session)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Optional working directory for the spawned process
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cwd: Option<String>,
},
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-server test_client_message_attach`

Expected: All attach tests PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add cwd field to Attach message"
```

---

## Task 2: Update `PtyBackend` trait to accept `cwd`

**Files:**
- Modify: `vibes-core/src/pty/backend.rs:17-20`
- Modify: `vibes-core/src/pty/backend.rs:35-93` (RealPtyBackend)
- Modify: `vibes-core/src/pty/backend.rs:139-189` (MockPtyBackend)
- Test: `vibes-core/src/pty/backend.rs` (inline tests)

**Step 1: Write the failing test**

Add test in the `#[cfg(test)]` module:

```rust
#[test]
fn mock_backend_creates_session_with_cwd() {
    let backend = MockPtyBackend::new();
    let cwd = Some("/tmp/test-dir".to_string());
    let session = backend.create_session("test-id".to_string(), Some("test".to_string()), cwd);
    assert!(session.is_ok());
    let session = session.unwrap();
    assert_eq!(session.id, "test-id");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core mock_backend_creates_session_with_cwd`

Expected: FAIL with compile error about argument count mismatch

**Step 3: Write minimal implementation**

Update the `PtyBackend` trait (line 17-20):

```rust
/// Trait for PTY backend implementations
pub trait PtyBackend: Send + Sync {
    /// Create a new PTY session
    fn create_session(&self, id: String, name: Option<String>, cwd: Option<String>) -> Result<PtySession, PtyError>;
}
```

Update `RealPtyBackend::create_session` (line 35):

```rust
fn create_session(&self, id: String, name: Option<String>, cwd: Option<String>) -> Result<PtySession, PtyError> {
    tracing::info!(
        id = %id,
        name = ?name,
        cwd = ?cwd,
        command = %self.config.claude_path.display(),
        "Spawning real PTY session"
    );

    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows: self.config.initial_rows,
            cols: self.config.initial_cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| PtyError::CreateFailed(e.to_string()))?;

    let mut cmd = CommandBuilder::new(&self.config.claude_path);
    for arg in &self.config.claude_args {
        cmd.arg(arg);
    }

    // Set working directory if provided
    if let Some(dir) = cwd {
        cmd.cwd(dir);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

    // ... rest unchanged
```

Update `MockPtyBackend::create_session` (line 140):

```rust
fn create_session(&self, id: String, name: Option<String>, cwd: Option<String>) -> Result<PtySession, PtyError> {
    tracing::info!(
        id = %id,
        name = ?name,
        cwd = ?cwd,
        "Creating mock PTY session (no real process)"
    );
    // ... rest unchanged
```

Update the existing test to pass the new argument:

```rust
#[test]
fn mock_backend_creates_session() {
    let backend = MockPtyBackend::new();
    let session = backend.create_session("test-id".to_string(), Some("test".to_string()), None);
    assert!(session.is_ok());
    let session = session.unwrap();
    assert_eq!(session.id, "test-id");
    assert_eq!(session.name, Some("test".to_string()));
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core backend`

Expected: All backend tests PASS

**Step 5: Commit**

```bash
git add vibes-core/src/pty/backend.rs
git commit -m "feat(pty): add cwd parameter to PtyBackend trait"
```

---

## Task 3: Update `PtyManager` to pass `cwd` through

**Files:**
- Modify: `vibes-core/src/pty/manager.rs:44-59`
- Test: `vibes-core/src/pty/manager.rs` (inline tests)

**Step 1: Write the failing test**

Add test:

```rust
#[test]
fn create_session_with_cwd() {
    let mut manager = PtyManager::new(test_config());

    let id = manager
        .create_session_with_cwd(Some("my-session".to_string()), Some("/tmp".to_string()))
        .unwrap();
    let session = manager.get_session(&id).unwrap();
    assert_eq!(session.name, Some("my-session".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core create_session_with_cwd`

Expected: FAIL with "no method named `create_session_with_cwd`"

**Step 3: Write minimal implementation**

Update `PtyManager` methods:

```rust
/// Create a new PTY session with auto-generated ID
pub fn create_session(&mut self, name: Option<String>) -> Result<String, PtyError> {
    self.create_session_with_cwd(name, None)
}

/// Create a new PTY session with auto-generated ID and optional cwd
pub fn create_session_with_cwd(&mut self, name: Option<String>, cwd: Option<String>) -> Result<String, PtyError> {
    let id = Uuid::new_v4().to_string();
    self.create_session_with_id_and_cwd(id, name, cwd)
}

/// Create a new PTY session with a specific ID
pub fn create_session_with_id(
    &mut self,
    id: String,
    name: Option<String>,
) -> Result<String, PtyError> {
    self.create_session_with_id_and_cwd(id, name, None)
}

/// Create a new PTY session with a specific ID and optional cwd
pub fn create_session_with_id_and_cwd(
    &mut self,
    id: String,
    name: Option<String>,
    cwd: Option<String>,
) -> Result<String, PtyError> {
    let session = self.backend.create_session(id.clone(), name, cwd)?;
    self.sessions.insert(id.clone(), session);
    Ok(id)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core manager`

Expected: All manager tests PASS

**Step 5: Commit**

```bash
git add vibes-core/src/pty/manager.rs
git commit -m "feat(pty): add cwd support to PtyManager"
```

---

## Task 4: Update server connection handler to pass `cwd`

**Files:**
- Modify: `vibes-server/src/ws/connection.rs:303-353`

**Step 1: Write the failing test** (integration test)

Create or update `vibes-server/tests/ws_protocol.rs`:

```rust
#[tokio::test]
async fn attach_with_cwd_creates_session() {
    use vibes_server::ws::ClientMessage;

    let pty_config = PtyConfig {
        mock_mode: true,
        ..Default::default()
    };
    let (_state, addr) = common::create_test_server_with_pty_config(pty_config).await;
    let mut client = TestClient::connect(addr).await;

    // Send attach with cwd
    let msg = ClientMessage::Attach {
        session_id: "test-cwd-session".to_string(),
        name: Some("test".to_string()),
        cwd: Some("/tmp".to_string()),
    };
    client.send_raw(&msg).await;

    // Should receive attach ack
    let ack = client.recv_attach_ack().await;
    assert_eq!(ack.session_id, "test-cwd-session");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server attach_with_cwd`

Expected: May compile but test confirms message handling works

**Step 3: Write minimal implementation**

Update the `ClientMessage::Attach` handler in `connection.rs` (around line 303):

```rust
ClientMessage::Attach { session_id, name, cwd } => {
    debug!("PTY attach requested for session: {}, cwd: {:?}", session_id, cwd);

    let mut pty_manager = state.pty_manager.write().await;

    conn_state.attach_pty(&session_id);

    let (cols, rows) = if pty_manager.get_session(&session_id).is_some() {
        (120, 40)
    } else {
        // Create new PTY session with the client's session ID and cwd
        match pty_manager.create_session_with_id_and_cwd(session_id.clone(), name, cwd) {
            Ok(created_id) => {
                // ... rest unchanged
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-server`

Expected: All server tests PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): pass cwd from Attach message to PtyManager"
```

---

## Task 5: Update CLI client to send `cwd`

**Files:**
- Modify: `vibes-cli/src/client/connection.rs:109-115`
- Modify: `vibes-cli/src/commands/claude.rs:95-96`
- Test: `vibes-cli/src/client/connection.rs` (inline tests)

**Step 1: Write the failing test**

Add test in `connection.rs`:

```rust
#[test]
fn test_attach_message_includes_cwd() {
    let msg = ClientMessage::Attach {
        session_id: "sess-1".to_string(),
        name: Some("test".to_string()),
        cwd: Some("/home/user/project".to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("/home/user/project"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-cli test_attach_message_includes_cwd`

Expected: FAIL with compile error (field doesn't exist yet in this crate's view)

**Step 3: Write minimal implementation**

Update `VibesClient::attach` method:

```rust
/// Attach to a PTY session to receive output
///
/// If `name` is provided and the session doesn't exist, a new session will be
/// created with this human-readable name.
/// If `cwd` is provided, the spawned process will use it as its working directory.
pub async fn attach(&self, session_id: &str, name: Option<String>, cwd: Option<String>) -> Result<()> {
    self.send(ClientMessage::Attach {
        session_id: session_id.to_string(),
        name,
        cwd,
    })
    .await
}
```

Update the call site in `claude.rs`:

```rust
// Get current working directory
let cwd = std::env::current_dir()
    .ok()
    .and_then(|p| p.to_str().map(String::from));

// Send attach request with optional session name and cwd
client.attach(&session_id, session_name, cwd).await?;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-cli`

Expected: All CLI tests PASS

**Step 5: Commit**

```bash
git add vibes-cli/src/client/connection.rs vibes-cli/src/commands/claude.rs
git commit -m "feat(cli): capture and send cwd in attach request"
```

---

## Task 6: Update test client helper

**Files:**
- Modify: `vibes-server/tests/common/client.rs`

**Step 1: Identify needed changes**

The test client's `create_session` method needs to handle the new cwd field.

**Step 2: Update test client**

Update `TestClient::create_session` and add helper for attach with cwd:

```rust
pub async fn create_session(&mut self, name: Option<&str>) -> String {
    self.create_session_with_cwd(name, None).await
}

pub async fn create_session_with_cwd(&mut self, name: Option<&str>, cwd: Option<&str>) -> String {
    let session_id = uuid::Uuid::new_v4().to_string();
    let msg = ClientMessage::Attach {
        session_id: session_id.clone(),
        name: name.map(String::from),
        cwd: cwd.map(String::from),
    };
    self.send_raw(&msg).await;
    // ... wait for ack
    session_id
}
```

**Step 3: Run all tests**

Run: `cargo test`

Expected: All tests PASS

**Step 4: Commit**

```bash
git add vibes-server/tests/common/client.rs
git commit -m "test: update test client for cwd support"
```

---

## Task 7: Final integration verification

**Step 1: Build the project**

Run: `just build`

Expected: Build succeeds

**Step 2: Run all tests**

Run: `just test`

Expected: All tests pass

**Step 3: Run pre-commit checks**

Run: `just pre-commit`

Expected: All checks pass

**Step 4: Manual test (optional)**

```bash
cd /tmp/test-dir
vibes claude
# Verify Claude starts in /tmp/test-dir
```

**Step 5: Final commit and PR**

```bash
git push -u origin fix-cwd-propagation
gh pr create --title "fix: propagate working directory from CLI to spawned Claude process" --body "$(cat <<'EOF'
## Summary
- Add `cwd` field to WebSocket `Attach` protocol message
- CLI captures current working directory and sends it with attach request
- Server passes cwd through to PTY manager
- PTY backend sets working directory on spawned process

## Test Plan
- [x] Unit tests for protocol serialization with cwd field
- [x] Unit tests for PTY backend with cwd parameter
- [x] Unit tests for PTY manager with cwd support
- [x] Integration test for attach with cwd
- [ ] Manual test: run `vibes claude` from a non-home directory

Fixes working directory bug where Claude always started in server's cwd.
EOF
)"
```
