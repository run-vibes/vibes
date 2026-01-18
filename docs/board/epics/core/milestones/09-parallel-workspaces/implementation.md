# Multi-Session Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable multiple concurrent Claude sessions with ownership tracking, lifecycle management, and improved UX.

**Architecture:** Extend existing `SessionManager` with `SessionOwnership` for tracking owner/subscribers. Add `SessionLifecycleManager` for handling client disconnects and ownership transfer. Update WebSocket protocol for session list and lifecycle events. Add CLI `sessions` commands and responsive Web UI.

**Tech Stack:** Rust (vibes-core, vibes-server, vibes-cli), TypeScript/React (web-ui), WebSocket protocol

---

## Phase 1: Core Ownership Types

### Task 1.1: Create SessionOwnership struct

**Files:**
- Create: `vibes-core/src/session/ownership.rs`
- Modify: `vibes-core/src/session/mod.rs`

**Step 1: Write the failing test**

Create `vibes-core/src/session/ownership.rs`:

```rust
//! Session ownership and subscription tracking

use std::collections::HashSet;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// Unique identifier for a connected client
pub type ClientId = String;

/// Session ownership and subscription tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOwnership {
    /// The client that created/owns this session
    pub owner_id: ClientId,
    /// All clients currently subscribed to this session
    pub subscriber_ids: HashSet<ClientId>,
    /// When ownership was last transferred
    pub owned_since: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ownership_includes_owner_as_subscriber() {
        let ownership = SessionOwnership::new("client-1".to_string());

        assert_eq!(ownership.owner_id, "client-1");
        assert!(ownership.subscriber_ids.contains("client-1"));
        assert_eq!(ownership.subscriber_ids.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core ownership::tests::new_ownership --no-default-features`
Expected: FAIL with "function `new` not found"

**Step 3: Write minimal implementation**

Add to `vibes-core/src/session/ownership.rs`:

```rust
impl SessionOwnership {
    pub fn new(owner_id: ClientId) -> Self {
        Self {
            owner_id: owner_id.clone(),
            subscriber_ids: HashSet::from([owner_id]),
            owned_since: SystemTime::now(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core ownership::tests::new_ownership --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/ownership.rs
git commit -m "feat(session): add SessionOwnership struct with new()"
```

---

### Task 1.2: Add subscriber management methods

**Files:**
- Modify: `vibes-core/src/session/ownership.rs`

**Step 1: Write the failing tests**

Add to `ownership.rs` tests module:

```rust
#[test]
fn add_subscriber_adds_to_set() {
    let mut ownership = SessionOwnership::new("owner".to_string());

    ownership.add_subscriber("subscriber-1".to_string());

    assert!(ownership.subscriber_ids.contains("subscriber-1"));
    assert_eq!(ownership.subscriber_ids.len(), 2);
}

#[test]
fn remove_subscriber_removes_from_set() {
    let mut ownership = SessionOwnership::new("owner".to_string());
    ownership.add_subscriber("subscriber-1".to_string());

    let was_owner = ownership.remove_subscriber("subscriber-1");

    assert!(!ownership.subscriber_ids.contains("subscriber-1"));
    assert!(!was_owner);
}

#[test]
fn remove_subscriber_returns_true_if_owner() {
    let mut ownership = SessionOwnership::new("owner".to_string());

    let was_owner = ownership.remove_subscriber("owner");

    assert!(was_owner);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p vibes-core ownership::tests --no-default-features`
Expected: FAIL with methods not found

**Step 3: Write minimal implementation**

Add to `SessionOwnership` impl:

```rust
/// Add a subscriber
pub fn add_subscriber(&mut self, client_id: ClientId) {
    self.subscriber_ids.insert(client_id);
}

/// Remove a subscriber, returns true if was the owner
pub fn remove_subscriber(&mut self, client_id: &ClientId) -> bool {
    self.subscriber_ids.remove(client_id);
    &self.owner_id == client_id
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core ownership::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/ownership.rs
git commit -m "feat(session): add subscriber management to SessionOwnership"
```

---

### Task 1.3: Add ownership transfer methods

**Files:**
- Modify: `vibes-core/src/session/ownership.rs`

**Step 1: Write the failing tests**

Add to tests module:

```rust
#[test]
fn transfer_to_subscriber_succeeds() {
    let mut ownership = SessionOwnership::new("owner".to_string());
    ownership.add_subscriber("new-owner".to_string());

    let success = ownership.transfer_to(&"new-owner".to_string());

    assert!(success);
    assert_eq!(ownership.owner_id, "new-owner");
}

#[test]
fn transfer_to_non_subscriber_fails() {
    let mut ownership = SessionOwnership::new("owner".to_string());

    let success = ownership.transfer_to(&"not-subscribed".to_string());

    assert!(!success);
    assert_eq!(ownership.owner_id, "owner");
}

#[test]
fn pick_next_owner_returns_subscriber() {
    let mut ownership = SessionOwnership::new("owner".to_string());
    ownership.add_subscriber("candidate".to_string());
    ownership.remove_subscriber(&"owner".to_string());

    let next = ownership.pick_next_owner();

    assert!(next.is_some());
    assert_eq!(next.unwrap(), "candidate");
}

#[test]
fn pick_next_owner_returns_none_when_empty() {
    let mut ownership = SessionOwnership::new("owner".to_string());
    ownership.remove_subscriber(&"owner".to_string());

    let next = ownership.pick_next_owner();

    assert!(next.is_none());
}

#[test]
fn should_cleanup_when_no_subscribers() {
    let mut ownership = SessionOwnership::new("owner".to_string());
    ownership.remove_subscriber(&"owner".to_string());

    assert!(ownership.should_cleanup());
}

#[test]
fn should_not_cleanup_when_subscribers_exist() {
    let ownership = SessionOwnership::new("owner".to_string());

    assert!(!ownership.should_cleanup());
}

#[test]
fn is_owner_returns_true_for_owner() {
    let ownership = SessionOwnership::new("owner".to_string());

    assert!(ownership.is_owner(&"owner".to_string()));
    assert!(!ownership.is_owner(&"other".to_string()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p vibes-core ownership::tests --no-default-features`
Expected: FAIL

**Step 3: Write minimal implementation**

Add to `SessionOwnership` impl:

```rust
/// Transfer ownership to another subscriber
pub fn transfer_to(&mut self, new_owner: &ClientId) -> bool {
    if self.subscriber_ids.contains(new_owner) {
        self.owner_id = new_owner.clone();
        self.owned_since = SystemTime::now();
        true
    } else {
        false
    }
}

/// Pick next owner from subscribers (if any)
pub fn pick_next_owner(&self) -> Option<&ClientId> {
    self.subscriber_ids.iter().next()
}

/// Returns true if session should be cleaned up
pub fn should_cleanup(&self) -> bool {
    self.subscriber_ids.is_empty()
}

/// Check if client is the owner
pub fn is_owner(&self, client_id: &ClientId) -> bool {
    &self.owner_id == client_id
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core ownership::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/ownership.rs
git commit -m "feat(session): add ownership transfer and cleanup methods"
```

---

### Task 1.4: Export ownership module

**Files:**
- Modify: `vibes-core/src/session/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Update session/mod.rs**

Add to `vibes-core/src/session/mod.rs`:

```rust
mod ownership;

pub use ownership::{ClientId, SessionOwnership};
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-core`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-core/src/session/mod.rs
git commit -m "feat(session): export SessionOwnership from session module"
```

---

## Phase 2: Update Session with Ownership

### Task 2.1: Add ownership field to Session

**Files:**
- Modify: `vibes-core/src/session/state.rs`

**Step 1: Write the failing test**

Add to `vibes-core/src/session/state.rs` tests:

```rust
#[tokio::test]
async fn new_session_with_owner_has_ownership() {
    let backend = MockBackend::new();
    let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
    let session = Session::new_with_owner(
        "test-session",
        None,
        "client-1".to_string(),
        Box::new(backend),
        event_bus,
    );

    assert_eq!(session.ownership().owner_id, "client-1");
    assert!(session.ownership().subscriber_ids.contains("client-1"));
}

#[tokio::test]
async fn session_ownership_can_be_mutated() {
    let backend = MockBackend::new();
    let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
    let mut session = Session::new_with_owner(
        "test-session",
        None,
        "client-1".to_string(),
        Box::new(backend),
        event_bus,
    );

    session.ownership_mut().add_subscriber("client-2".to_string());

    assert!(session.ownership().subscriber_ids.contains("client-2"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p vibes-core session::state::tests::new_session_with_owner --no-default-features`
Expected: FAIL with `new_with_owner` not found

**Step 3: Write minimal implementation**

Add to `Session` struct in `state.rs`:

```rust
use super::ownership::{ClientId, SessionOwnership};

pub struct Session {
    id: String,
    name: Option<String>,
    backend: Box<dyn ClaudeBackend>,
    event_bus: Arc<dyn EventBus>,
    state: SessionState,
    created_at: SystemTime,
    last_activity_at: SystemTime,     // NEW
    ownership: SessionOwnership,       // NEW
}
```

Add new constructor and accessor methods:

```rust
/// Create a new session with owner
pub fn new_with_owner(
    id: impl Into<String>,
    name: Option<String>,
    owner_id: ClientId,
    backend: Box<dyn ClaudeBackend>,
    event_bus: Arc<dyn EventBus>,
) -> Self {
    let now = SystemTime::now();
    Self {
        id: id.into(),
        name,
        backend,
        event_bus,
        state: SessionState::Idle,
        created_at: now,
        last_activity_at: now,
        ownership: SessionOwnership::new(owner_id),
    }
}

pub fn ownership(&self) -> &SessionOwnership {
    &self.ownership
}

pub fn ownership_mut(&mut self) -> &mut SessionOwnership {
    &mut self.ownership
}

pub fn last_activity_at(&self) -> SystemTime {
    self.last_activity_at
}

pub fn touch(&mut self) {
    self.last_activity_at = SystemTime::now();
}
```

Update existing `new()` to use a default owner (for backwards compatibility):

```rust
pub fn new(
    id: impl Into<String>,
    name: Option<String>,
    backend: Box<dyn ClaudeBackend>,
    event_bus: Arc<dyn EventBus>,
) -> Self {
    Self::new_with_owner(id, name, "local".to_string(), backend, event_bus)
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core session::state::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/state.rs
git commit -m "feat(session): add ownership and last_activity_at to Session"
```

---

### Task 2.2: Update SessionManager to accept owner_id

**Files:**
- Modify: `vibes-core/src/session/manager.rs`

**Step 1: Write the failing test**

Add to `manager.rs` tests:

```rust
#[tokio::test]
async fn create_session_with_owner_sets_ownership() {
    let manager = create_test_manager();

    let id = manager.create_session_with_owner(
        Some("test".to_string()),
        "client-123".to_string(),
    ).await;

    let owner = manager.with_session(&id, |s| {
        s.ownership().owner_id.clone()
    }).await.unwrap();

    assert_eq!(owner, "client-123");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core manager::tests::create_session_with_owner --no-default-features`
Expected: FAIL

**Step 3: Write minimal implementation**

Add to `SessionManager`:

```rust
use super::ownership::ClientId;

/// Create a new session with an optional name and owner
pub async fn create_session_with_owner(
    &self,
    name: Option<String>,
    owner_id: ClientId,
) -> String {
    let id = Uuid::new_v4().to_string();
    let backend = self.backend_factory.create(None);
    let session = Session::new_with_owner(
        id.clone(),
        name,
        owner_id,
        backend,
        self.event_bus.clone(),
    );

    self.sessions.write().await.insert(id.clone(), session);
    id
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core manager::tests::create_session_with_owner --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/manager.rs
git commit -m "feat(session): add create_session_with_owner to SessionManager"
```

---

### Task 2.3: Add helper methods to SessionManager

**Files:**
- Modify: `vibes-core/src/session/manager.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn get_sessions_owned_by_returns_matching() {
    let manager = create_test_manager();

    let id1 = manager.create_session_with_owner(None, "client-a".to_string()).await;
    let _id2 = manager.create_session_with_owner(None, "client-b".to_string()).await;
    let id3 = manager.create_session_with_owner(None, "client-a".to_string()).await;

    let owned = manager.get_sessions_owned_by("client-a").await;

    assert_eq!(owned.len(), 2);
    assert!(owned.contains(&id1));
    assert!(owned.contains(&id3));
}

#[tokio::test]
async fn get_sessions_subscribed_by_returns_matching() {
    let manager = create_test_manager();

    let id1 = manager.create_session_with_owner(None, "client-a".to_string()).await;
    let id2 = manager.create_session_with_owner(None, "client-b".to_string()).await;

    // Subscribe client-a to session 2
    manager.with_session(&id2, |s| {
        s.ownership_mut().add_subscriber("client-a".to_string());
    }).await.unwrap();

    let subscribed = manager.get_sessions_subscribed_by("client-a").await;

    assert_eq!(subscribed.len(), 2);
    assert!(subscribed.contains(&id1));
    assert!(subscribed.contains(&id2));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p vibes-core manager::tests::get_sessions --no-default-features`
Expected: FAIL

**Step 3: Write minimal implementation**

Add to `SessionManager`:

```rust
/// Get all sessions owned by a client
pub async fn get_sessions_owned_by(&self, client_id: &str) -> Vec<String> {
    self.sessions
        .read()
        .await
        .iter()
        .filter(|(_, session)| session.ownership().is_owner(&client_id.to_string()))
        .map(|(id, _)| id.clone())
        .collect()
}

/// Get all sessions a client is subscribed to
pub async fn get_sessions_subscribed_by(&self, client_id: &str) -> Vec<String> {
    self.sessions
        .read()
        .await
        .iter()
        .filter(|(_, session)| {
            session.ownership().subscriber_ids.contains(&client_id.to_string())
        })
        .map(|(id, _)| id.clone())
        .collect()
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core manager::tests::get_sessions --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/manager.rs
git commit -m "feat(session): add ownership query methods to SessionManager"
```

---

### Task 2.4: Export new types from vibes-core

**Files:**
- Modify: `vibes-core/src/lib.rs`

**Step 1: Update exports**

Ensure `vibes-core/src/lib.rs` exports:

```rust
pub use session::{ClientId, Session, SessionManager, SessionOwnership, SessionState};
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-core`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-core/src/lib.rs
git commit -m "feat(core): export ClientId and SessionOwnership"
```

---

## Phase 3: Session Lifecycle Manager

### Task 3.1: Create SessionLifecycleManager

**Files:**
- Create: `vibes-core/src/session/lifecycle.rs`
- Modify: `vibes-core/src/session/mod.rs`

**Step 1: Write the failing test**

Create `vibes-core/src/session/lifecycle.rs`:

```rust
//! Session lifecycle management
//!
//! Handles ownership transfer and cleanup when clients disconnect.

use std::sync::Arc;

use super::manager::SessionManager;
use super::ownership::ClientId;

/// Manages session lifecycle events
pub struct SessionLifecycleManager {
    session_manager: Arc<SessionManager>,
}

/// Result of handling a client disconnect
#[derive(Debug, Default)]
pub struct DisconnectResult {
    /// Sessions where ownership was transferred: (session_id, new_owner_id)
    pub transfers: Vec<(String, ClientId)>,
    /// Sessions that were cleaned up
    pub cleanups: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::traits::{BackendFactory, ClaudeBackend};
    use crate::backend::MockBackend;
    use crate::events::MemoryEventBus;

    struct MockBackendFactory;

    impl BackendFactory for MockBackendFactory {
        fn create(&self, claude_session_id: Option<String>) -> Box<dyn ClaudeBackend> {
            match claude_session_id {
                Some(id) => Box::new(MockBackend::with_session_id(id)),
                None => Box::new(MockBackend::new()),
            }
        }
    }

    fn create_test_lifecycle() -> (SessionLifecycleManager, Arc<SessionManager>) {
        let event_bus = Arc::new(MemoryEventBus::new(100));
        let factory: Arc<dyn BackendFactory> = Arc::new(MockBackendFactory);
        let manager = Arc::new(SessionManager::new(factory, event_bus));
        let lifecycle = SessionLifecycleManager::new(manager.clone());
        (lifecycle, manager)
    }

    #[tokio::test]
    async fn disconnect_owner_transfers_to_subscriber() {
        let (lifecycle, manager) = create_test_lifecycle();

        // Create session owned by client-a
        let session_id = manager.create_session_with_owner(None, "client-a".to_string()).await;

        // Add subscriber
        manager.with_session(&session_id, |s| {
            s.ownership_mut().add_subscriber("client-b".to_string());
        }).await.unwrap();

        // Disconnect owner
        let result = lifecycle.handle_client_disconnect(&"client-a".to_string()).await;

        assert_eq!(result.transfers.len(), 1);
        assert_eq!(result.transfers[0].0, session_id);
        assert_eq!(result.transfers[0].1, "client-b");
        assert!(result.cleanups.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core lifecycle::tests::disconnect_owner_transfers --no-default-features`
Expected: FAIL with `new` and `handle_client_disconnect` not found

**Step 3: Write minimal implementation**

Add to `lifecycle.rs`:

```rust
impl SessionLifecycleManager {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Handle a client disconnecting
    ///
    /// Returns ownership transfers and cleanups that occurred.
    pub async fn handle_client_disconnect(&self, client_id: &ClientId) -> DisconnectResult {
        let mut result = DisconnectResult::default();

        // Get all sessions this client is subscribed to
        let session_ids = self.session_manager.get_sessions_subscribed_by(client_id).await;

        for session_id in session_ids {
            let action = self.session_manager.with_session(&session_id, |session| {
                let was_owner = session.ownership_mut().remove_subscriber(client_id);

                if was_owner {
                    // Try to transfer ownership
                    if let Some(new_owner) = session.ownership().pick_next_owner().cloned() {
                        session.ownership_mut().transfer_to(&new_owner);
                        return Some(LifecycleAction::Transfer(new_owner));
                    }
                }

                // Check if cleanup needed
                if session.ownership().should_cleanup() {
                    return Some(LifecycleAction::Cleanup);
                }

                None
            }).await;

            match action {
                Ok(Some(LifecycleAction::Transfer(new_owner))) => {
                    result.transfers.push((session_id, new_owner));
                }
                Ok(Some(LifecycleAction::Cleanup)) => {
                    self.session_manager.remove_session(&session_id).await.ok();
                    result.cleanups.push(session_id);
                }
                _ => {}
            }
        }

        result
    }
}

enum LifecycleAction {
    Transfer(ClientId),
    Cleanup,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core lifecycle::tests::disconnect_owner_transfers --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/lifecycle.rs
git commit -m "feat(session): add SessionLifecycleManager with disconnect handling"
```

---

### Task 3.2: Add more lifecycle tests

**Files:**
- Modify: `vibes-core/src/session/lifecycle.rs`

**Step 1: Write additional tests**

Add to lifecycle tests:

```rust
#[tokio::test]
async fn disconnect_owner_with_no_subscribers_cleans_up() {
    let (lifecycle, manager) = create_test_lifecycle();

    let session_id = manager.create_session_with_owner(None, "client-a".to_string()).await;

    let result = lifecycle.handle_client_disconnect(&"client-a".to_string()).await;

    assert!(result.transfers.is_empty());
    assert_eq!(result.cleanups.len(), 1);
    assert_eq!(result.cleanups[0], session_id);

    // Verify session was removed
    assert!(manager.get_session_state(&session_id).await.is_err());
}

#[tokio::test]
async fn disconnect_subscriber_does_not_transfer() {
    let (lifecycle, manager) = create_test_lifecycle();

    let session_id = manager.create_session_with_owner(None, "owner".to_string()).await;

    manager.with_session(&session_id, |s| {
        s.ownership_mut().add_subscriber("subscriber".to_string());
    }).await.unwrap();

    let result = lifecycle.handle_client_disconnect(&"subscriber".to_string()).await;

    assert!(result.transfers.is_empty());
    assert!(result.cleanups.is_empty());

    // Owner should still be owner
    let owner = manager.with_session(&session_id, |s| {
        s.ownership().owner_id.clone()
    }).await.unwrap();
    assert_eq!(owner, "owner");
}

#[tokio::test]
async fn disconnect_handles_multiple_sessions() {
    let (lifecycle, manager) = create_test_lifecycle();

    let id1 = manager.create_session_with_owner(None, "client".to_string()).await;
    let id2 = manager.create_session_with_owner(None, "client".to_string()).await;

    manager.with_session(&id1, |s| {
        s.ownership_mut().add_subscriber("other".to_string());
    }).await.unwrap();

    let result = lifecycle.handle_client_disconnect(&"client".to_string()).await;

    // id1 should transfer, id2 should cleanup
    assert_eq!(result.transfers.len(), 1);
    assert_eq!(result.cleanups.len(), 1);
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core lifecycle::tests --no-default-features`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-core/src/session/lifecycle.rs
git commit -m "test(session): add comprehensive lifecycle manager tests"
```

---

### Task 3.3: Add subscribe/unsubscribe helper methods

**Files:**
- Modify: `vibes-core/src/session/lifecycle.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn subscribe_client_adds_to_session() {
    let (lifecycle, manager) = create_test_lifecycle();

    let session_id = manager.create_session_with_owner(None, "owner".to_string()).await;

    lifecycle.subscribe_client(&session_id, &"new-client".to_string()).await.unwrap();

    let is_subscribed = manager.with_session(&session_id, |s| {
        s.ownership().subscriber_ids.contains("new-client")
    }).await.unwrap();

    assert!(is_subscribed);
}

#[tokio::test]
async fn unsubscribe_client_removes_from_session() {
    let (lifecycle, manager) = create_test_lifecycle();

    let session_id = manager.create_session_with_owner(None, "owner".to_string()).await;
    lifecycle.subscribe_client(&session_id, &"client".to_string()).await.unwrap();

    let was_owner = lifecycle.unsubscribe_client(&session_id, &"client".to_string()).await.unwrap();

    assert!(!was_owner);

    let is_subscribed = manager.with_session(&session_id, |s| {
        s.ownership().subscriber_ids.contains("client")
    }).await.unwrap();

    assert!(!is_subscribed);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p vibes-core lifecycle::tests::subscribe --no-default-features`
Expected: FAIL

**Step 3: Write minimal implementation**

Add to `SessionLifecycleManager`:

```rust
/// Subscribe a client to a session
pub async fn subscribe_client(
    &self,
    session_id: &str,
    client_id: &ClientId,
) -> Result<(), crate::error::SessionError> {
    self.session_manager.with_session(session_id, |session| {
        session.ownership_mut().add_subscriber(client_id.clone());
    }).await
}

/// Unsubscribe a client from a session
///
/// Returns true if the client was the owner.
pub async fn unsubscribe_client(
    &self,
    session_id: &str,
    client_id: &ClientId,
) -> Result<bool, crate::error::SessionError> {
    self.session_manager.with_session(session_id, |session| {
        session.ownership_mut().remove_subscriber(client_id)
    }).await
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core lifecycle::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/session/lifecycle.rs
git commit -m "feat(session): add subscribe/unsubscribe to SessionLifecycleManager"
```

---

### Task 3.4: Export lifecycle module

**Files:**
- Modify: `vibes-core/src/session/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Update exports**

In `vibes-core/src/session/mod.rs`:

```rust
mod lifecycle;
mod manager;
mod ownership;
mod state;

pub use lifecycle::{DisconnectResult, SessionLifecycleManager};
pub use manager::SessionManager;
pub use ownership::{ClientId, SessionOwnership};
pub use state::{Session, SessionState};
```

In `vibes-core/src/lib.rs`, ensure exports include:

```rust
pub use session::{
    ClientId, DisconnectResult, Session, SessionLifecycleManager, SessionManager,
    SessionOwnership, SessionState,
};
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-core`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-core/src/session/mod.rs vibes-core/src/lib.rs
git commit -m "feat(core): export SessionLifecycleManager and DisconnectResult"
```

---

## Phase 4: WebSocket Protocol Updates

### Task 4.1: Add SessionInfo type

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Write the failing test**

Add to `protocol.rs`:

```rust
/// Information about an active session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionInfo {
    pub id: String,
    pub name: Option<String>,
    pub state: String,
    pub owner_id: String,
    pub is_owner: bool,
    pub subscriber_count: u32,
    pub created_at: i64,
    pub last_activity_at: i64,
}

#[cfg(test)]
mod tests {
    // Add to existing tests

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: "sess-1".to_string(),
            name: Some("test".to_string()),
            state: "Idle".to_string(),
            owner_id: "client-1".to_string(),
            is_owner: true,
            subscriber_count: 2,
            created_at: 1234567890,
            last_activity_at: 1234567900,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, parsed);
    }
}
```

**Step 2: Run test**

Run: `cargo test -p vibes-server protocol::tests::test_session_info --no-default-features`
Expected: PASS (just adding type)

**Step 3: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(server): add SessionInfo type to protocol"
```

---

### Task 4.2: Add RemovalReason enum

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Add type and test**

```rust
/// Reason a session was removed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RemovalReason {
    OwnerDisconnected,
    Killed,
    SessionFinished,
}

// In tests:
#[test]
fn test_removal_reason_serialization() {
    let reasons = vec![
        RemovalReason::OwnerDisconnected,
        RemovalReason::Killed,
        RemovalReason::SessionFinished,
    ];

    for reason in reasons {
        let json = serde_json::to_string(&reason).unwrap();
        let parsed: RemovalReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, parsed);
    }
}
```

**Step 2: Run test**

Run: `cargo test -p vibes-server protocol::tests::test_removal_reason --no-default-features`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(server): add RemovalReason enum to protocol"
```

---

### Task 4.3: Add new ClientMessage variants

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Add variants and tests**

Add to `ClientMessage` enum:

```rust
/// Request list of all active sessions
ListSessions { request_id: String },

/// Terminate a session
KillSession { session_id: String },
```

Add tests:

```rust
#[test]
fn test_client_message_list_sessions_roundtrip() {
    let msg = ClientMessage::ListSessions {
        request_id: "req-1".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"list_sessions""#));
}

#[test]
fn test_client_message_kill_session_roundtrip() {
    let msg = ClientMessage::KillSession {
        session_id: "sess-1".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"kill_session""#));
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-server protocol::tests::test_client_message --no-default-features`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(server): add ListSessions and KillSession client messages"
```

---

### Task 4.4: Add new ServerMessage variants

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Add variants and tests**

Add to `ServerMessage` enum:

```rust
/// Full session list response
SessionList {
    request_id: String,
    sessions: Vec<SessionInfo>,
},

/// Session was removed
SessionRemoved {
    session_id: String,
    reason: RemovalReason,
},

/// Ownership transferred
OwnershipTransferred {
    session_id: String,
    new_owner_id: String,
    you_are_owner: bool,
},
```

Add tests:

```rust
#[test]
fn test_server_message_session_list_roundtrip() {
    let msg = ServerMessage::SessionList {
        request_id: "req-1".to_string(),
        sessions: vec![SessionInfo {
            id: "sess-1".to_string(),
            name: None,
            state: "Idle".to_string(),
            owner_id: "client-1".to_string(),
            is_owner: true,
            subscriber_count: 1,
            created_at: 0,
            last_activity_at: 0,
        }],
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"session_list""#));
}

#[test]
fn test_server_message_session_removed_roundtrip() {
    let msg = ServerMessage::SessionRemoved {
        session_id: "sess-1".to_string(),
        reason: RemovalReason::OwnerDisconnected,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"session_removed""#));
}

#[test]
fn test_server_message_ownership_transferred_roundtrip() {
    let msg = ServerMessage::OwnershipTransferred {
        session_id: "sess-1".to_string(),
        new_owner_id: "client-2".to_string(),
        you_are_owner: false,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"ownership_transferred""#));
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-server protocol::tests::test_server_message --no-default-features`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(server): add SessionList, SessionRemoved, OwnershipTransferred messages"
```

---

## Phase 5: WebSocket Connection Updates

### Task 5.1: Add client_id to ConnectionState

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Update ConnectionState**

```rust
use uuid::Uuid;

struct ConnectionState {
    /// Unique ID for this connection
    client_id: String,
    /// Session IDs this connection is subscribed to
    subscribed_sessions: HashSet<String>,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
            subscribed_sessions: HashSet::new(),
        }
    }

    fn client_id(&self) -> &str {
        &self.client_id
    }

    // ... existing methods
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): add client_id to WebSocket ConnectionState"
```

---

### Task 5.2: Add SessionLifecycleManager to AppState

**Files:**
- Modify: `vibes-server/src/state.rs`

**Step 1: Update AppState**

```rust
use vibes_core::SessionLifecycleManager;

pub struct AppState {
    // ... existing fields

    /// Session lifecycle manager
    pub lifecycle: Arc<SessionLifecycleManager>,
}

impl AppState {
    pub fn new() -> Self {
        let event_bus = Arc::new(MemoryEventBus::new(10_000));
        let factory: Arc<dyn BackendFactory> =
            Arc::new(PrintModeBackendFactory::new(PrintModeConfig::default()));
        let session_manager = Arc::new(SessionManager::new(factory, event_bus.clone()));
        let lifecycle = Arc::new(SessionLifecycleManager::new(session_manager.clone()));
        // ... rest

        Self {
            session_manager,
            lifecycle,
            // ... rest
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/state.rs
git commit -m "feat(server): add SessionLifecycleManager to AppState"
```

---

### Task 5.3: Handle ListSessions message

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Add handler**

In `handle_text_message`, add case for `ListSessions`:

```rust
ClientMessage::ListSessions { request_id } => {
    let sessions_data = state.session_manager.list_sessions_full().await;

    let sessions: Vec<SessionInfo> = sessions_data
        .into_iter()
        .map(|(id, name, session_state, created_at)| {
            // Get ownership info
            let (owner_id, subscriber_count) = state.session_manager
                .with_session(&id, |s| {
                    (
                        s.ownership().owner_id.clone(),
                        s.ownership().subscriber_ids.len() as u32,
                    )
                })
                .await
                .unwrap_or(("unknown".to_string(), 0));

            let is_owner = owner_id == conn_state.client_id();

            SessionInfo {
                id,
                name,
                state: format!("{:?}", session_state),
                owner_id,
                is_owner,
                subscriber_count,
                created_at: created_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                last_activity_at: 0, // TODO: get from session
            }
        })
        .collect();

    let response = ServerMessage::SessionList {
        request_id,
        sessions,
    };

    let json = serde_json::to_string(&response)?;
    sender.send(Message::Text(json)).await?;
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): handle ListSessions WebSocket message"
```

---

### Task 5.4: Handle KillSession message

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Add handler**

```rust
ClientMessage::KillSession { session_id } => {
    // Check if client is subscribed
    if !conn_state.is_subscribed_to(&session_id) {
        let error_msg = ServerMessage::Error {
            session_id: Some(session_id),
            message: "Not subscribed to session".to_string(),
            code: "NOT_SUBSCRIBED".to_string(),
        };
        let json = serde_json::to_string(&error_msg)?;
        sender.send(Message::Text(json)).await?;
        return Ok(());
    }

    // Remove session
    if let Err(e) = state.session_manager.remove_session(&session_id).await {
        let error_msg = ServerMessage::Error {
            session_id: Some(session_id),
            message: e.to_string(),
            code: "KILL_FAILED".to_string(),
        };
        let json = serde_json::to_string(&error_msg)?;
        sender.send(Message::Text(json)).await?;
    } else {
        // Broadcast removal
        let removed_msg = ServerMessage::SessionRemoved {
            session_id: session_id.clone(),
            reason: RemovalReason::Killed,
        };
        // TODO: Broadcast to all clients
        conn_state.unsubscribe(&[session_id]);
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): handle KillSession WebSocket message"
```

---

### Task 5.5: Handle client disconnect with lifecycle

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Update handle_socket**

At the end of `handle_socket`, before the "WebSocket client disconnected" log:

```rust
// Handle lifecycle cleanup on disconnect
let result = state.lifecycle.handle_client_disconnect(&conn_state.client_id).await;

// Broadcast ownership transfers
for (session_id, new_owner_id) in result.transfers {
    let msg = ServerMessage::OwnershipTransferred {
        session_id: session_id.clone(),
        new_owner_id: new_owner_id.clone(),
        you_are_owner: false, // Will be set correctly per-client
    };
    // Broadcast via event system
    state.broadcast_event(VibesEvent::SessionStateChanged {
        session_id,
        state: format!("ownership_transferred:{}", new_owner_id),
    });
}

// Broadcast session removals
for session_id in result.cleanups {
    state.broadcast_event(VibesEvent::SessionStateChanged {
        session_id,
        state: "removed".to_string(),
    });
}

info!("WebSocket client {} disconnected", conn_state.client_id());
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): handle client disconnect with lifecycle manager"
```

---

## Phase 6: CLI Sessions Commands

### Task 6.1: Create sessions command module

**Files:**
- Create: `vibes-cli/src/commands/sessions.rs`
- Modify: `vibes-cli/src/commands/mod.rs`

**Step 1: Create module structure**

Create `vibes-cli/src/commands/sessions.rs`:

```rust
//! Session management commands

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct SessionsArgs {
    #[command(subcommand)]
    pub command: Option<SessionsCommand>,
}

#[derive(Debug, Subcommand)]
pub enum SessionsCommand {
    /// List active sessions
    List,
    /// Attach to an existing session
    Attach {
        /// Session ID to attach to
        session_id: String,
    },
    /// Terminate a session
    Kill {
        /// Session ID to terminate
        session_id: String,
    },
}

pub async fn run(args: SessionsArgs) -> anyhow::Result<()> {
    match args.command {
        None | Some(SessionsCommand::List) => list_sessions().await,
        Some(SessionsCommand::Attach { session_id }) => attach_session(&session_id).await,
        Some(SessionsCommand::Kill { session_id }) => kill_session(&session_id).await,
    }
}

async fn list_sessions() -> anyhow::Result<()> {
    todo!("Implement list_sessions")
}

async fn attach_session(_session_id: &str) -> anyhow::Result<()> {
    todo!("Implement attach_session")
}

async fn kill_session(_session_id: &str) -> anyhow::Result<()> {
    todo!("Implement kill_session")
}
```

Update `vibes-cli/src/commands/mod.rs`:

```rust
pub mod sessions;
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-cli/src/commands/sessions.rs vibes-cli/src/commands/mod.rs
git commit -m "feat(cli): add sessions command module structure"
```

---

### Task 6.2: Add sessions command to CLI

**Files:**
- Modify: `vibes-cli/src/main.rs`

**Step 1: Add to CLI enum**

Add to the Commands enum:

```rust
/// Manage active sessions
Sessions(commands::sessions::SessionsArgs),
```

Add match arm:

```rust
Commands::Sessions(args) => commands::sessions::run(args).await,
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-cli/src/main.rs
git commit -m "feat(cli): register sessions command in main CLI"
```

---

### Task 6.3: Implement list_sessions

**Files:**
- Modify: `vibes-cli/src/commands/sessions.rs`

**Step 1: Implement list**

```rust
use crate::client::WsClient;
use crate::daemon;
use vibes_server::ws::protocol::{ClientMessage, ServerMessage, SessionInfo};
use colored::Colorize;

async fn list_sessions() -> anyhow::Result<()> {
    // Ensure daemon is running
    daemon::ensure_running().await?;

    // Connect to daemon
    let mut client = WsClient::connect("ws://127.0.0.1:7432/ws").await?;

    // Request session list
    let request_id = uuid::Uuid::new_v4().to_string();
    client.send(&ClientMessage::ListSessions {
        request_id: request_id.clone()
    }).await?;

    // Wait for response
    let sessions = loop {
        match client.recv().await? {
            ServerMessage::SessionList { request_id: rid, sessions } if rid == request_id => {
                break sessions;
            }
            _ => continue,
        }
    };

    if sessions.is_empty() {
        println!("No active sessions.");
        println!();
        println!("Start a session with: {}", "vibes claude \"your prompt\"".cyan());
        return Ok(());
    }

    println!("{}", format!("ACTIVE SESSIONS ({})", sessions.len()).bold());
    println!();
    println!("  {:<12} {:<16} {:<20} {:<8} {}",
        "ID".bold(),
        "NAME".bold(),
        "STATE".bold(),
        "AGE".bold(),
        "OWNER".bold()
    );

    for session in sessions {
        let state_display = format_state(&session.state, session.is_owner);
        let age = format_age(session.created_at);
        let owner = if session.is_owner {
            "you".green().to_string()
        } else {
            "other".dimmed().to_string()
        };

        println!("  {:<12} {:<16} {:<20} {:<8} {}",
            &session.id[..12.min(session.id.len())],
            session.name.as_deref().unwrap_or("-"),
            state_display,
            age,
            owner
        );
    }

    println!();
    println!("Use '{}' to connect to a session.", "vibes sessions attach <id>".cyan());
    println!("Use '{}' to terminate a session.", "vibes sessions kill <id>".cyan());

    Ok(())
}

fn format_state(state: &str, is_owner: bool) -> String {
    match state {
        s if s.contains("Idle") => "🔵 Idle".to_string(),
        s if s.contains("Processing") => "🟢 Processing".green().to_string(),
        s if s.contains("WaitingPermission") => "🟡 Needs approval".yellow().to_string(),
        s if s.contains("Failed") => "🔴 Failed".red().to_string(),
        s if s.contains("Finished") => "⚪ Finished".dimmed().to_string(),
        _ => state.to_string(),
    }
}

fn format_age(created_at: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let diff = now - created_at;

    if diff < 60 {
        format!("{}s", diff)
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else if diff < 86400 {
        format!("{}h", diff / 3600)
    } else {
        format!("{}d", diff / 86400)
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS (may need adjustments for actual client API)

**Step 3: Commit**

```bash
git add vibes-cli/src/commands/sessions.rs
git commit -m "feat(cli): implement sessions list command"
```

---

### Task 6.4: Implement attach_session

**Files:**
- Modify: `vibes-cli/src/commands/sessions.rs`

**Step 1: Implement attach**

```rust
async fn attach_session(session_id: &str) -> anyhow::Result<()> {
    daemon::ensure_running().await?;

    let mut client = WsClient::connect("ws://127.0.0.1:7432/ws").await?;

    // Subscribe to the session
    client.send(&ClientMessage::Subscribe {
        session_ids: vec![session_id.to_string()],
    }).await?;

    println!("Attaching to session {}...", session_id.cyan());
    println!("Connected. You are now a subscriber.");
    println!();

    // Run interactive loop
    run_interactive_session(&mut client, session_id).await
}

async fn run_interactive_session(client: &mut WsClient, session_id: &str) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = client.recv() => {
                match msg? {
                    ServerMessage::Claude { session_id: sid, event } if sid == session_id => {
                        // Print Claude output
                        use vibes_core::ClaudeEvent;
                        match event {
                            ClaudeEvent::TextDelta { text } => print!("{}", text),
                            ClaudeEvent::TurnComplete { .. } => println!(),
                            ClaudeEvent::PermissionRequest { tool, command, .. } => {
                                println!();
                                println!("{}", "⚠️  Permission Request".yellow().bold());
                                println!("    Tool: {}", tool);
                                if let Some(cmd) = command {
                                    println!("    Command: {}", cmd);
                                }
                                println!();
                                println!("    [A]pprove  [D]eny");
                            }
                            _ => {}
                        }
                    }
                    ServerMessage::SessionRemoved { session_id: sid, reason } if sid == session_id => {
                        println!();
                        println!("Session ended: {:?}", reason);
                        break;
                    }
                    _ => {}
                }
            }

            // Handle user input
            line = lines.next_line() => {
                if let Some(input) = line? {
                    let input = input.trim();
                    if input.eq_ignore_ascii_case("a") || input.eq_ignore_ascii_case("approve") {
                        // TODO: Send permission approval
                    } else if input.eq_ignore_ascii_case("d") || input.eq_ignore_ascii_case("deny") {
                        // TODO: Send permission denial
                    } else if !input.is_empty() {
                        client.send(&ClientMessage::Input {
                            session_id: session_id.to_string(),
                            content: input.to_string(),
                        }).await?;
                    }
                } else {
                    // EOF
                    break;
                }
            }
        }
    }

    Ok(())
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-cli/src/commands/sessions.rs
git commit -m "feat(cli): implement sessions attach command"
```

---

### Task 6.5: Implement kill_session

**Files:**
- Modify: `vibes-cli/src/commands/sessions.rs`

**Step 1: Implement kill**

```rust
async fn kill_session(session_id: &str) -> anyhow::Result<()> {
    daemon::ensure_running().await?;

    let mut client = WsClient::connect("ws://127.0.0.1:7432/ws").await?;

    // First subscribe to get permission
    client.send(&ClientMessage::Subscribe {
        session_ids: vec![session_id.to_string()],
    }).await?;

    // Then kill
    client.send(&ClientMessage::KillSession {
        session_id: session_id.to_string(),
    }).await?;

    // Wait for confirmation
    loop {
        match client.recv().await? {
            ServerMessage::SessionRemoved { session_id: sid, reason } if sid == session_id => {
                println!("Session {} terminated ({:?})", session_id, reason);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Failed to kill session: {}", message);
            }
            _ => continue,
        }
    }

    Ok(())
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-cli/src/commands/sessions.rs
git commit -m "feat(cli): implement sessions kill command"
```

---

## Phase 7: Web UI Updates

### Task 7.1: Add SessionInfo TypeScript type

**Files:**
- Modify: `web-ui/src/lib/types.ts`

**Step 1: Add type**

```typescript
export interface SessionInfo {
  id: string;
  name: string | null;
  state: string;
  owner_id: string;
  is_owner: boolean;
  subscriber_count: number;
  created_at: number;
  last_activity_at: number;
}
```

**Step 2: Commit**

```bash
git add web-ui/src/lib/types.ts
git commit -m "feat(web-ui): add SessionInfo type"
```

---

### Task 7.2: Add useSessionList hook

**Files:**
- Create: `web-ui/src/hooks/useSessionList.ts`

**Step 1: Create hook**

```typescript
import { useState, useEffect, useCallback } from 'react';
import { useWebSocket } from './useWebSocket';
import type { SessionInfo } from '../lib/types';

interface SessionListState {
  sessions: SessionInfo[];
  loading: boolean;
  error: string | null;
}

export function useSessionList() {
  const { sendMessage, lastMessage, connected } = useWebSocket();
  const [state, setState] = useState<SessionListState>({
    sessions: [],
    loading: true,
    error: null,
  });

  const refresh = useCallback(() => {
    if (connected) {
      const requestId = crypto.randomUUID();
      sendMessage({
        type: 'list_sessions',
        request_id: requestId,
      });
    }
  }, [connected, sendMessage]);

  useEffect(() => {
    if (connected) {
      refresh();
    }
  }, [connected, refresh]);

  useEffect(() => {
    if (!lastMessage) return;

    switch (lastMessage.type) {
      case 'session_list':
        setState({
          sessions: lastMessage.sessions,
          loading: false,
          error: null,
        });
        break;
      case 'session_notification':
        // New session created, refresh list
        refresh();
        break;
      case 'session_removed':
        setState(prev => ({
          ...prev,
          sessions: prev.sessions.filter(s => s.id !== lastMessage.session_id),
        }));
        break;
      case 'ownership_transferred':
        setState(prev => ({
          ...prev,
          sessions: prev.sessions.map(s =>
            s.id === lastMessage.session_id
              ? { ...s, owner_id: lastMessage.new_owner_id, is_owner: lastMessage.you_are_owner }
              : s
          ),
        }));
        break;
    }
  }, [lastMessage, refresh]);

  return { ...state, refresh };
}
```

**Step 2: Commit**

```bash
git add web-ui/src/hooks/useSessionList.ts
git commit -m "feat(web-ui): add useSessionList hook"
```

---

### Task 7.3: Create SessionList component

**Files:**
- Create: `web-ui/src/components/SessionList.tsx`

**Step 1: Create component**

```typescript
import { SessionInfo } from '../lib/types';
import { SessionCard } from './SessionCard';

interface SessionListProps {
  sessions: SessionInfo[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export function SessionList({ sessions, selectedId, onSelect }: SessionListProps) {
  if (sessions.length === 0) {
    return (
      <div className="session-list-empty">
        <p>No active sessions</p>
        <code>vibes claude "your prompt"</code>
      </div>
    );
  }

  return (
    <div className="session-list">
      {sessions.map(session => (
        <SessionCard
          key={session.id}
          id={session.id}
          name={session.name}
          state={session.state}
          isOwner={session.is_owner}
          subscriberCount={session.subscriber_count}
          createdAt={session.created_at}
          selected={session.id === selectedId}
          onClick={() => onSelect(session.id)}
        />
      ))}
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add web-ui/src/components/SessionList.tsx
git commit -m "feat(web-ui): add SessionList component"
```

---

### Task 7.4: Update SessionCard with status indicators

**Files:**
- Modify: `web-ui/src/components/SessionCard.tsx`

**Step 1: Update component**

```typescript
interface SessionCardProps {
  id: string;
  name: string | null;
  state: string;
  isOwner: boolean;
  subscriberCount: number;
  createdAt: number;
  selected?: boolean;
  onClick?: () => void;
}

function getStateIndicator(state: string) {
  if (state.includes('Idle')) return { icon: '🔵', text: 'Idle', className: 'state-idle' };
  if (state.includes('Processing')) return { icon: '🟢', text: 'Processing', className: 'state-processing' };
  if (state.includes('WaitingPermission')) return { icon: '🟡', text: 'Needs approval', className: 'state-waiting' };
  if (state.includes('Failed')) return { icon: '🔴', text: 'Failed', className: 'state-failed' };
  if (state.includes('Finished')) return { icon: '⚪', text: 'Finished', className: 'state-finished' };
  return { icon: '⚪', text: state, className: '' };
}

function formatAge(timestamp: number): string {
  const now = Date.now() / 1000;
  const diff = now - timestamp;

  if (diff < 60) return `${Math.floor(diff)}s`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
  return `${Math.floor(diff / 86400)}d`;
}

export function SessionCard({
  id,
  name,
  state,
  isOwner,
  subscriberCount,
  createdAt,
  selected,
  onClick,
}: SessionCardProps) {
  const indicator = getStateIndicator(state);
  const needsAttention = state.includes('WaitingPermission');

  return (
    <div
      className={`session-card ${selected ? 'selected' : ''} ${needsAttention ? 'needs-attention' : ''}`}
      onClick={onClick}
    >
      <div className="session-card-header">
        <span className="session-name">{name || id.slice(0, 8)}</span>
        {isOwner && <span className="owner-badge">owner</span>}
      </div>
      <div className={`session-state ${indicator.className}`}>
        <span className="state-icon">{indicator.icon}</span>
        <span className="state-text">{indicator.text}</span>
        {needsAttention && <span className="attention-icon">⚠️</span>}
      </div>
      <div className="session-meta">
        <span className="session-age">{formatAge(createdAt)}</span>
        {subscriberCount > 1 && (
          <span className="subscriber-count">{subscriberCount} viewers</span>
        )}
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add web-ui/src/components/SessionCard.tsx
git commit -m "feat(web-ui): enhance SessionCard with status indicators"
```

---

### Task 7.5: Update ClaudeSessions with responsive layout

**Files:**
- Modify: `web-ui/src/pages/ClaudeSessions.tsx`

**Step 1: Implement responsive layout**

```typescript
import { useState, useEffect } from 'react';
import { useSessionList } from '../hooks/useSessionList';
import { SessionList } from '../components/SessionList';
import { ClaudeSession } from './ClaudeSession';

function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(
    typeof window !== 'undefined' ? window.matchMedia(query).matches : false
  );

  useEffect(() => {
    const mediaQuery = window.matchMedia(query);
    const handler = (e: MediaQueryListEvent) => setMatches(e.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, [query]);

  return matches;
}

export function ClaudeSessions() {
  const { sessions, loading, error, refresh } = useSessionList();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const isDesktop = useMediaQuery('(min-width: 1024px)');

  // Auto-select first session on desktop
  useEffect(() => {
    if (isDesktop && sessions.length > 0 && !selectedId) {
      setSelectedId(sessions[0].id);
    }
  }, [isDesktop, sessions, selectedId]);

  if (loading) {
    return <div className="page"><p>Loading sessions...</p></div>;
  }

  if (error) {
    return <div className="page"><p className="error">{error}</p></div>;
  }

  // Mobile: Show either list or detail
  if (!isDesktop) {
    if (selectedId) {
      return (
        <div className="page">
          <button className="back-button" onClick={() => setSelectedId(null)}>
            ← Sessions
          </button>
          <ClaudeSession sessionId={selectedId} />
        </div>
      );
    }

    return (
      <div className="page">
        <h1>Sessions</h1>
        <SessionList
          sessions={sessions}
          selectedId={null}
          onSelect={setSelectedId}
        />
      </div>
    );
  }

  // Desktop: Split view
  return (
    <div className="page sessions-split-view">
      <aside className="sessions-sidebar">
        <h2>Sessions</h2>
        <SessionList
          sessions={sessions}
          selectedId={selectedId}
          onSelect={setSelectedId}
        />
      </aside>
      <main className="sessions-main">
        {selectedId ? (
          <ClaudeSession sessionId={selectedId} />
        ) : (
          <div className="no-session-selected">
            <p>Select a session to view details</p>
          </div>
        )}
      </main>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add web-ui/src/pages/ClaudeSessions.tsx
git commit -m "feat(web-ui): implement responsive split/single layout"
```

---

### Task 7.6: Add CSS for responsive layout

**Files:**
- Modify: `web-ui/src/styles/main.css` (or appropriate CSS file)

**Step 1: Add styles**

```css
/* Split view layout */
.sessions-split-view {
  display: flex;
  height: 100vh;
}

.sessions-sidebar {
  width: 300px;
  border-right: 1px solid var(--border-color);
  overflow-y: auto;
  padding: 1rem;
}

.sessions-main {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

/* Session list */
.session-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.session-list-empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}

/* Session card */
.session-card {
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.session-card:hover {
  background-color: var(--hover-bg);
}

.session-card.selected {
  border-color: var(--primary-color);
  background-color: var(--selected-bg);
}

.session-card.needs-attention {
  border-color: var(--warning-color);
}

.session-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.session-name {
  font-weight: 600;
}

.owner-badge {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  background-color: var(--primary-color);
  color: white;
  border-radius: 1rem;
}

.session-state {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.state-processing .state-icon {
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.session-meta {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

/* Mobile back button */
.back-button {
  margin-bottom: 1rem;
  padding: 0.5rem 1rem;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 0.25rem;
  cursor: pointer;
}

/* No session selected */
.no-session-selected {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
}
```

**Step 2: Commit**

```bash
git add web-ui/src/styles/main.css
git commit -m "feat(web-ui): add responsive layout styles"
```

---

## Phase 8: Integration Testing

### Task 8.1: Run all tests

**Step 1: Run full test suite**

```bash
just test
```

Expected: All tests pass

**Step 2: Run pre-commit checks**

```bash
just pre-commit
```

Expected: All checks pass

**Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: address test failures and lint issues"
```

---

### Task 8.2: Manual testing checklist

Test the following scenarios:

- [ ] Start session from CLI, verify it appears in Web UI list
- [ ] Start session from Web UI, verify it appears in CLI list
- [ ] Subscribe to session from second client, verify subscriber count updates
- [ ] Disconnect owner, verify ownership transfers to subscriber
- [ ] Disconnect all subscribers, verify session is removed
- [ ] Test split view on desktop (≥1024px)
- [ ] Test single view on mobile (<1024px)
- [ ] Verify status indicators update in real-time
- [ ] Test `vibes sessions list` output format
- [ ] Test `vibes sessions attach <id>` interaction
- [ ] Test `vibes sessions kill <id>` confirmation

---

## Phase 9: Documentation & Cleanup

### Task 9.1: Update PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

Mark milestone 3.2 as complete and add changelog entry.

**Step 1: Update file**

Change the milestone status and add changelog.

**Step 2: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: mark milestone 3.2 as complete"
```

---

### Task 9.2: Final commit and PR

**Step 1: Verify everything passes**

```bash
just pre-commit
```

**Step 2: Push and create PR**

```bash
git push -u origin multi-session-support
gh pr create --title "feat: add multi-session support (milestone 3.2)" --body "$(cat <<'EOF'
## Summary
- Add SessionOwnership with client-owned sessions and ownership transfer
- Add SessionLifecycleManager for disconnect handling and cleanup
- Add WebSocket protocol messages for session list and lifecycle events
- Add CLI `sessions` commands: list, attach, kill
- Update Web UI with responsive split/single layout
- Add real-time status indicators for session states

## Test Plan
- [x] All unit tests passing (`just test`)
- [x] Pre-commit checks pass (`just pre-commit`)
- [ ] Manual testing of ownership transfer scenarios
- [ ] Responsive layout testing on desktop and mobile
EOF
)"
```

---

## Summary

This implementation plan covers:

1. **Phase 1-2:** Core ownership types and Session updates (vibes-core)
2. **Phase 3:** SessionLifecycleManager for disconnect handling
3. **Phase 4-5:** WebSocket protocol updates and connection handling (vibes-server)
4. **Phase 6:** CLI sessions commands (vibes-cli)
5. **Phase 7:** Web UI responsive layout and components
6. **Phase 8-9:** Integration testing and documentation

Each task follows TDD: write failing test → verify failure → implement → verify pass → commit.

Estimated tasks: ~35 bite-sized steps across 9 phases.
