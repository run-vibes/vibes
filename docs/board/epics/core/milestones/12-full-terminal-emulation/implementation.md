# PTY Backend Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace PrintModeBackend with a PTY wrapper that provides full CLI parity, xterm.js web UI, and structured data capture via Claude hooks.

**Architecture:** Server-owned PTY sessions using `portable-pty` crate. CLI and Web UI connect as terminal clients via WebSocket. Claude hooks provide structured events for analytics/history without ANSI parsing.

**Tech Stack:** Rust (portable-pty, tokio, axum), TypeScript (xterm.js, React), Claude Code hooks

**Design Document:** [design.md](./design.md)

---

## Phase 1: Core PTY Infrastructure

### Task 1.1: Add portable-pty dependency

**Files:**
- Modify: `vibes-core/Cargo.toml`

**Step 1: Add dependency**

```toml
# Add to [dependencies] section
portable-pty = "0.8"
```

**Step 2: Verify it compiles**

Run: `cargo check -p vibes-core`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add vibes-core/Cargo.toml
git commit -m "chore: add portable-pty dependency"
```

---

### Task 1.2: Create PTY module structure

**Files:**
- Create: `vibes-core/src/pty/mod.rs`
- Create: `vibes-core/src/pty/config.rs`
- Create: `vibes-core/src/pty/session.rs`
- Create: `vibes-core/src/pty/manager.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Create module files with structure**

```rust
// vibes-core/src/pty/mod.rs
//! PTY-based Claude backend
//!
//! Spawns Claude Code in a pseudo-terminal for full interactive support.

mod config;
mod manager;
mod session;

pub use config::PtyConfig;
pub use manager::PtyManager;
pub use session::{PtySession, PtySessionHandle};
```

```rust
// vibes-core/src/pty/config.rs
//! PTY configuration

use std::path::PathBuf;

/// Configuration for PTY sessions
#[derive(Debug, Clone)]
pub struct PtyConfig {
    /// Path to claude binary (defaults to "claude")
    pub claude_path: PathBuf,
    /// Initial terminal size
    pub initial_cols: u16,
    pub initial_rows: u16,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            claude_path: PathBuf::from("claude"),
            initial_cols: 120,
            initial_rows: 40,
        }
    }
}
```

```rust
// vibes-core/src/pty/session.rs
//! PTY session management

use std::sync::Arc;
use tokio::sync::Mutex;

/// Handle to interact with a PTY session
#[derive(Clone)]
pub struct PtySessionHandle {
    inner: Arc<Mutex<PtySessionInner>>,
}

struct PtySessionInner {
    // Will be filled in next task
}

/// State of a PTY session
#[derive(Debug, Clone, PartialEq)]
pub enum PtyState {
    Running,
    Exited(i32),
}

/// A PTY session wrapping Claude
pub struct PtySession {
    pub id: String,
    pub name: Option<String>,
    pub state: PtyState,
}
```

```rust
// vibes-core/src/pty/manager.rs
//! PTY session manager

use std::collections::HashMap;
use super::{PtyConfig, PtySessionHandle};

/// Manages multiple PTY sessions
pub struct PtyManager {
    sessions: HashMap<String, PtySessionHandle>,
    config: PtyConfig,
}

impl PtyManager {
    /// Create a new PTY manager
    pub fn new(config: PtyConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
        }
    }
}
```

**Step 2: Export from lib.rs**

Add to `vibes-core/src/lib.rs`:

```rust
pub mod pty;
```

**Step 3: Verify it compiles**

Run: `cargo check -p vibes-core`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add vibes-core/src/pty/ vibes-core/src/lib.rs
git commit -m "feat(pty): add module structure"
```

---

### Task 1.3: Implement PtySession spawn

**Files:**
- Modify: `vibes-core/src/pty/session.rs`
- Create: `vibes-core/src/pty/error.rs`
- Modify: `vibes-core/src/pty/mod.rs`

**Step 1: Write the failing test**

Add to `vibes-core/src/pty/session.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty::PtyConfig;

    #[test]
    fn spawn_creates_running_session() {
        let config = PtyConfig::default();
        // Use 'echo' for testing instead of claude
        let mut test_config = config;
        test_config.claude_path = "echo".into();

        let session = PtySession::spawn("test-id".to_string(), None, &test_config);
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
        assert_eq!(session.state, PtyState::Running);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core pty::session::tests::spawn_creates_running_session`
Expected: FAIL - "spawn" method doesn't exist

**Step 3: Add error type**

```rust
// vibes-core/src/pty/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PtyError {
    #[error("Failed to create PTY: {0}")]
    CreateFailed(String),

    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("PTY I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
```

Update `vibes-core/src/pty/mod.rs`:

```rust
mod error;
pub use error::PtyError;
```

**Step 4: Implement spawn**

Replace `vibes-core/src/pty/session.rs`:

```rust
//! PTY session management

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{PtyConfig, PtyError};

/// State of a PTY session
#[derive(Debug, Clone, PartialEq)]
pub enum PtyState {
    Running,
    Exited(i32),
}

/// Handle to interact with a PTY session
#[derive(Clone)]
pub struct PtySessionHandle {
    inner: Arc<Mutex<PtySessionInner>>,
}

struct PtySessionInner {
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    reader: Box<dyn std::io::Read + Send>,
    writer: Box<dyn std::io::Write + Send>,
}

/// A PTY session wrapping Claude
pub struct PtySession {
    pub id: String,
    pub name: Option<String>,
    pub state: PtyState,
    pub handle: PtySessionHandle,
}

impl PtySession {
    /// Spawn a new PTY session
    pub fn spawn(
        id: String,
        name: Option<String>,
        config: &PtyConfig,
    ) -> Result<Self, PtyError> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows: config.initial_rows,
                cols: config.initial_cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::CreateFailed(e.to_string()))?;

        let cmd = CommandBuilder::new(&config.claude_path);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let inner = PtySessionInner {
            master: pair.master,
            child,
            reader,
            writer,
        };

        let handle = PtySessionHandle {
            inner: Arc::new(Mutex::new(inner)),
        };

        Ok(Self {
            id,
            name,
            state: PtyState::Running,
            handle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty::PtyConfig;

    #[test]
    fn spawn_creates_running_session() {
        let mut config = PtyConfig::default();
        // Use 'cat' for testing - it will wait for input
        config.claude_path = "cat".into();

        let session = PtySession::spawn("test-id".to_string(), None, &config);
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
        assert_eq!(session.state, PtyState::Running);
    }

    #[test]
    fn spawn_with_name() {
        let mut config = PtyConfig::default();
        config.claude_path = "cat".into();

        let session = PtySession::spawn(
            "test-id".to_string(),
            Some("my-session".to_string()),
            &config,
        ).unwrap();

        assert_eq!(session.name, Some("my-session".to_string()));
    }

    #[test]
    fn spawn_invalid_command_fails() {
        let mut config = PtyConfig::default();
        config.claude_path = "/nonexistent/binary".into();

        let result = PtySession::spawn("test-id".to_string(), None, &config);
        assert!(result.is_err());
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p vibes-core pty::session::tests`
Expected: All 3 tests PASS

**Step 6: Commit**

```bash
git add vibes-core/src/pty/
git commit -m "feat(pty): implement PtySession spawn"
```

---

### Task 1.4: Implement PTY read/write

**Files:**
- Modify: `vibes-core/src/pty/session.rs`

**Step 1: Write the failing test**

Add to tests in `vibes-core/src/pty/session.rs`:

```rust
    #[tokio::test]
    async fn write_and_read_data() {
        let mut config = PtyConfig::default();
        // Use 'cat' - it echoes input back
        config.claude_path = "cat".into();

        let session = PtySession::spawn("test-id".to_string(), None, &config).unwrap();

        // Write some data
        session.handle.write(b"hello\n").await.unwrap();

        // Give cat time to echo
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Read it back
        let data = session.handle.read().await.unwrap();
        assert!(!data.is_empty());
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core pty::session::tests::write_and_read_data`
Expected: FAIL - "write" and "read" methods don't exist

**Step 3: Implement read/write on PtySessionHandle**

Add to `PtySessionHandle` impl in `vibes-core/src/pty/session.rs`:

```rust
impl PtySessionHandle {
    /// Write data to the PTY
    pub async fn write(&self, data: &[u8]) -> Result<(), PtyError> {
        let mut inner = self.inner.lock().await;
        use std::io::Write;
        inner.writer.write_all(data)?;
        inner.writer.flush()?;
        Ok(())
    }

    /// Read available data from the PTY (non-blocking)
    pub async fn read(&self) -> Result<Vec<u8>, PtyError> {
        let mut inner = self.inner.lock().await;
        let mut buf = vec![0u8; 4096];

        // Set non-blocking temporarily
        use std::io::Read;
        match inner.reader.read(&mut buf) {
            Ok(n) if n > 0 => {
                buf.truncate(n);
                Ok(buf)
            }
            Ok(_) => Ok(vec![]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(vec![]),
            Err(e) => Err(PtyError::IoError(e)),
        }
    }

    /// Resize the PTY
    pub async fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        let inner = self.inner.lock().await;
        inner
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core pty::session::tests`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add vibes-core/src/pty/session.rs
git commit -m "feat(pty): implement read/write/resize"
```

---

### Task 1.5: Implement PtyManager

**Files:**
- Modify: `vibes-core/src/pty/manager.rs`

**Step 1: Write the failing test**

```rust
// vibes-core/src/pty/manager.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty::PtyConfig;

    #[test]
    fn create_session_adds_to_manager() {
        let mut config = PtyConfig::default();
        config.claude_path = "cat".into();

        let mut manager = PtyManager::new(config);

        let id = manager.create_session(None).unwrap();
        assert!(!id.is_empty());
        assert!(manager.get_session(&id).is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core pty::manager::tests::create_session_adds_to_manager`
Expected: FAIL - methods don't exist

**Step 3: Implement PtyManager methods**

Replace `vibes-core/src/pty/manager.rs`:

```rust
//! PTY session manager

use std::collections::HashMap;
use uuid::Uuid;

use super::{PtyConfig, PtyError, PtySession, PtySessionHandle, session::PtyState};

/// Info about a session (without the handle)
#[derive(Debug, Clone)]
pub struct PtySessionInfo {
    pub id: String,
    pub name: Option<String>,
    pub state: PtyState,
}

/// Manages multiple PTY sessions
pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
    config: PtyConfig,
}

impl PtyManager {
    /// Create a new PTY manager
    pub fn new(config: PtyConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
        }
    }

    /// Create a new PTY session
    pub fn create_session(&mut self, name: Option<String>) -> Result<String, PtyError> {
        let id = Uuid::new_v4().to_string();
        let session = PtySession::spawn(id.clone(), name, &self.config)?;
        self.sessions.insert(id.clone(), session);
        Ok(id)
    }

    /// Get a session handle by ID
    pub fn get_session(&self, id: &str) -> Option<&PtySession> {
        self.sessions.get(id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut PtySession> {
        self.sessions.get_mut(id)
    }

    /// Get session handle for I/O
    pub fn get_handle(&self, id: &str) -> Option<PtySessionHandle> {
        self.sessions.get(id).map(|s| s.handle.clone())
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<PtySessionInfo> {
        self.sessions
            .values()
            .map(|s| PtySessionInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                state: s.state.clone(),
            })
            .collect()
    }

    /// Remove a session
    pub fn remove_session(&mut self, id: &str) -> Option<PtySession> {
        self.sessions.remove(id)
    }

    /// Kill a session (send SIGTERM and remove)
    pub async fn kill_session(&mut self, id: &str) -> Result<(), PtyError> {
        if let Some(session) = self.sessions.remove(id) {
            let mut inner = session.handle.inner.lock().await;
            let _ = inner.child.kill();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> PtyConfig {
        let mut config = PtyConfig::default();
        config.claude_path = "cat".into();
        config
    }

    #[test]
    fn create_session_adds_to_manager() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(None).unwrap();
        assert!(!id.is_empty());
        assert!(manager.get_session(&id).is_some());
    }

    #[test]
    fn create_session_with_name() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(Some("my-session".to_string())).unwrap();
        let session = manager.get_session(&id).unwrap();
        assert_eq!(session.name, Some("my-session".to_string()));
    }

    #[test]
    fn list_sessions_returns_all() {
        let mut manager = PtyManager::new(test_config());

        manager.create_session(Some("session1".to_string())).unwrap();
        manager.create_session(Some("session2".to_string())).unwrap();

        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn remove_session_removes_from_manager() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(None).unwrap();
        assert!(manager.get_session(&id).is_some());

        manager.remove_session(&id);
        assert!(manager.get_session(&id).is_none());
    }

    #[test]
    fn get_handle_returns_cloneable_handle() {
        let mut manager = PtyManager::new(test_config());

        let id = manager.create_session(None).unwrap();
        let handle1 = manager.get_handle(&id);
        let handle2 = manager.get_handle(&id);

        assert!(handle1.is_some());
        assert!(handle2.is_some());
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core pty::manager::tests`
Expected: All 5 tests PASS

**Step 5: Commit**

```bash
git add vibes-core/src/pty/manager.rs
git commit -m "feat(pty): implement PtyManager"
```

---

## Phase 2: WebSocket Protocol

### Task 2.1: Add PTY message types

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`
- Modify: `web-ui/src/lib/types.ts`

**Step 1: Add Rust message types**

Add to `vibes-server/src/ws/protocol.rs` ClientMessage enum:

```rust
    /// Attach to a session (receive PTY output)
    #[serde(rename = "attach")]
    Attach { session_id: String },

    /// Detach from a session
    #[serde(rename = "detach")]
    Detach { session_id: String },

    /// Send input to PTY
    #[serde(rename = "pty_input")]
    PtyInput { session_id: String, data: String }, // base64 encoded

    /// Resize PTY
    #[serde(rename = "pty_resize")]
    PtyResize { session_id: String, cols: u16, rows: u16 },
```

Add to ServerMessage enum:

```rust
    /// PTY output data
    #[serde(rename = "pty_output")]
    PtyOutput { session_id: String, data: String }, // base64 encoded

    /// PTY exited
    #[serde(rename = "pty_exit")]
    PtyExit { session_id: String, exit_code: Option<i32> },

    /// Attach acknowledged
    #[serde(rename = "attach_ack")]
    AttachAck { session_id: String, cols: u16, rows: u16 },
```

**Step 2: Add TypeScript types**

Add to `web-ui/src/lib/types.ts` ClientMessage:

```typescript
  | { type: 'attach'; session_id: string }
  | { type: 'detach'; session_id: string }
  | { type: 'pty_input'; session_id: string; data: string }
  | { type: 'pty_resize'; session_id: string; cols: number; rows: number }
```

Add to ServerMessage:

```typescript
  | { type: 'pty_output'; session_id: string; data: string }
  | { type: 'pty_exit'; session_id: string; exit_code: number | null }
  | { type: 'attach_ack'; session_id: string; cols: number; rows: number }
```

**Step 3: Verify it compiles**

Run: `cargo check -p vibes-server && cd web-ui && npm run typecheck`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add vibes-server/src/ws/protocol.rs web-ui/src/lib/types.ts
git commit -m "feat(protocol): add PTY message types"
```

---

### Task 2.2: Wire PtyManager into server

**Files:**
- Modify: `vibes-server/src/lib.rs`
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Add PtyManager to server state**

This task integrates the PtyManager with the WebSocket connection handler.
The implementation will handle attach/detach, pty_input, pty_resize messages
and broadcast pty_output to attached clients.

*Detailed implementation steps to be written based on existing connection.rs structure*

**Step 2: Verify with integration test**

Run: `cargo test -p vibes-server`
Expected: All tests pass

**Step 3: Commit**

```bash
git add vibes-server/
git commit -m "feat(server): wire PtyManager into WebSocket handler"
```

---

## Phase 3: CLI Client

### Task 3.1: Add raw terminal mode

**Files:**
- Create: `vibes-cli/src/terminal/mod.rs`
- Create: `vibes-cli/src/terminal/raw.rs`
- Modify: `vibes-cli/src/lib.rs`

**Step 1: Create terminal module**

```rust
// vibes-cli/src/terminal/mod.rs
mod raw;
pub use raw::RawTerminal;
```

```rust
// vibes-cli/src/terminal/raw.rs
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, Write};

pub struct RawTerminal {
    was_raw: bool,
}

impl RawTerminal {
    pub fn new() -> io::Result<Self> {
        let was_raw = terminal::is_raw_mode_enabled()?;
        if !was_raw {
            terminal::enable_raw_mode()?;
        }
        Ok(Self { was_raw })
    }

    pub fn read_event(&self) -> io::Result<Option<Event>> {
        if event::poll(std::time::Duration::from_millis(10))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    pub fn write(&self, data: &[u8]) -> io::Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(data)?;
        stdout.flush()?;
        Ok(())
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        if !self.was_raw {
            let _ = terminal::disable_raw_mode();
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p vibes-cli`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add vibes-cli/src/terminal/
git commit -m "feat(cli): add raw terminal mode support"
```

---

### Task 3.2: Refactor vibes claude to PTY client

**Files:**
- Modify: `vibes-cli/src/commands/claude.rs`

**Step 1: Update command to use PTY protocol**

The command will:
1. Connect to server
2. Create or attach to session
3. Enter raw mode
4. Proxy I/O bidirectionally
5. Handle resize events

*Detailed implementation based on existing claude.rs structure*

**Step 2: Test manually**

Run: `cargo run -p vibes-cli -- claude`
Expected: Interactive session works

**Step 3: Commit**

```bash
git add vibes-cli/src/commands/claude.rs
git commit -m "feat(cli): refactor to PTY client mode"
```

---

## Phase 4: Web UI

### Task 4.1: Add xterm.js dependencies

**Files:**
- Modify: `web-ui/package.json`

**Step 1: Install dependencies**

Run:
```bash
cd web-ui
npm install xterm xterm-addon-fit xterm-addon-web-links
npm install -D @types/xterm
```

**Step 2: Verify it builds**

Run: `npm run build`
Expected: Builds successfully

**Step 3: Commit**

```bash
git add web-ui/package.json web-ui/package-lock.json
git commit -m "chore(web-ui): add xterm.js dependencies"
```

---

### Task 4.2: Create Terminal component

**Files:**
- Create: `web-ui/src/components/Terminal.tsx`
- Create: `web-ui/src/components/Terminal.css`

**Step 1: Create Terminal component**

```tsx
// web-ui/src/components/Terminal.tsx
import { useEffect, useRef } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { WebLinksAddon } from 'xterm-addon-web-links';
import 'xterm/css/xterm.css';
import './Terminal.css';

interface TerminalProps {
  sessionId: string;
  onInput: (data: string) => void;
  onResize: (cols: number, rows: number) => void;
}

export function SessionTerminal({ sessionId, onInput, onResize }: TerminalProps) {
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'JetBrains Mono, Menlo, monospace',
      theme: {
        background: '#1a1a2e',
        foreground: '#eaeaea',
      },
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());

    term.open(termRef.current);
    fitAddon.fit();

    // Handle input
    term.onData((data) => {
      onInput(btoa(data)); // base64 encode
    });

    // Handle resize
    term.onResize(({ cols, rows }) => {
      onResize(cols, rows);
    });

    // Store refs
    terminalRef.current = term;
    fitAddonRef.current = fitAddon;

    // Handle window resize
    const handleResize = () => {
      fitAddon.fit();
    };
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      term.dispose();
    };
  }, [sessionId, onInput, onResize]);

  // Expose write method via ref or context
  useEffect(() => {
    // Will be connected to WebSocket in parent
  }, []);

  return <div ref={termRef} className="terminal-container" />;
}

// Export a way to write to terminal
export function useTerminalWriter() {
  const terminalRef = useRef<Terminal | null>(null);

  const write = (data: string) => {
    if (terminalRef.current) {
      terminalRef.current.write(atob(data)); // base64 decode
    }
  };

  return { terminalRef, write };
}
```

```css
/* web-ui/src/components/Terminal.css */
.terminal-container {
  width: 100%;
  height: 100%;
  background: #1a1a2e;
}

.terminal-container .xterm {
  height: 100%;
  padding: 8px;
}
```

**Step 2: Verify it compiles**

Run: `cd web-ui && npm run typecheck`
Expected: No type errors

**Step 3: Commit**

```bash
git add web-ui/src/components/Terminal.tsx web-ui/src/components/Terminal.css
git commit -m "feat(web-ui): add xterm.js Terminal component"
```

---

### Task 4.3: Replace ClaudeSession with Terminal

**Files:**
- Modify: `web-ui/src/pages/ClaudeSession.tsx`
- Delete: `web-ui/src/components/MessageList.tsx` (after migration)
- Delete: `web-ui/src/components/ChatInput.tsx` (after migration)

**Step 1: Update ClaudeSession page**

Replace chat-based UI with Terminal component, connecting WebSocket events to terminal I/O.

*Detailed implementation based on existing page structure*

**Step 2: Verify it works**

Run: `npm run dev` and navigate to a session
Expected: Terminal UI displays instead of chat bubbles

**Step 3: Commit**

```bash
git add web-ui/src/pages/ClaudeSession.tsx
git commit -m "feat(web-ui): replace chat UI with terminal"
```

---

## Phase 5: Hooks Integration

### Task 5.1: Create hook receiver

**Files:**
- Create: `vibes-core/src/hooks/mod.rs`
- Create: `vibes-core/src/hooks/receiver.rs`
- Create: `vibes-core/src/hooks/transport.rs`

*Implementation details for Unix socket / TCP receiver*

---

### Task 5.2: Create hook scripts

**Files:**
- Create: `vibes-core/src/hooks/scripts/pre-tool-use.sh`
- Create: `vibes-core/src/hooks/scripts/post-tool-use.sh`
- Create: `vibes-core/src/hooks/scripts/stop.sh`
- Create: `vibes-core/src/hooks/scripts/vibes-hook-send.sh`

*Shell scripts that pipe hook data to vibes server*

---

### Task 5.3: Auto-install hooks on daemon start

**Files:**
- Modify: `vibes-server/src/lib.rs`
- Create: `vibes-core/src/hooks/installer.rs`

*Implementation that patches ~/.claude/settings.json on startup*

---

## Phase 6: Cleanup

### Task 6.1: Remove PrintModeBackend

**Files:**
- Delete: `vibes-core/src/backend/print_mode.rs`
- Delete: `vibes-core/src/backend/slow_mock.rs`
- Delete: `vibes-core/src/parser/stream_json.rs`
- Modify: `vibes-core/src/backend/mod.rs`
- Modify: `vibes-core/src/lib.rs`

*Remove deprecated code after PTY is stable*

---

### Task 6.2: Simplify session state

**Files:**
- Modify: `vibes-core/src/session/state.rs`

*Simplify SessionState to Running/Exited*

---

### Task 6.3: Remove deprecated protocol messages

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`
- Modify: `web-ui/src/lib/types.ts`

*Remove input, claude, user_input, permission_response messages*

---

### Task 6.4: Update documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/VISION.md`
- Modify: `CLAUDE.md`

*Update docs to reflect PTY architecture*

---

## Verification Checklist

Before marking complete:

- [ ] `just test` - All unit tests pass
- [ ] `just clippy` - No warnings
- [ ] `just fmt-check` - Formatting correct
- [ ] Manual test: `vibes claude` feels like native `claude`
- [ ] Manual test: Web UI terminal works
- [ ] Manual test: Multiple clients can attach to same session
- [ ] Manual test: Session survives CLI disconnect
