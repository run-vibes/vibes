# PTY Backend Design

## Overview

Replace PrintModeBackend with a PTY wrapper to achieve full CLI parity. The CLI experience should feel exactly like running `claude` directly, with web UI mirroring via xterm.js and structured data capture via Claude's hooks system.

## Goals

1. **Full CLI parity** - `vibes claude` feels identical to running `claude`
2. **Web UI mirroring** - xterm.js shows exactly what CLI sees
3. **Structured data capture** - Analytics, history, iOS GUI via hooks
4. **Session persistence** - Sessions survive CLI disconnect
5. **Multi-session support** - Multiple concurrent sessions per server

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         vibes server                                │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────────────┐ │
│  │ PTY Manager  │◄───│ Hook Receiver│    │   WebSocket Server    │ │
│  │              │    │   (events)   │    │                       │ │
│  │ ┌──────────┐ │    └──────┬───────┘    │  ┌────────┐ ┌───────┐ │ │
│  │ │ Session 1│ │           │            │  │ CLI 1  │ │ Web 1 │ │ │
│  │ │ (claude) │ │    structured          │  │        │ │xterm  │ │ │
│  │ └──────────┘ │    ClaudeEvents        │  └────────┘ └───────┘ │ │
│  │ ┌──────────┐ │           │            │  ┌────────┐ ┌───────┐ │ │
│  │ │ Session 2│ │           ▼            │  │ CLI 2  │ │ Web 2 │ │ │
│  │ │ (claude) │ │    ┌──────────────┐    │  │        │ │       │ │ │
│  │ └──────────┘ │    │  Event Bus   │    │  └────────┘ └───────┘ │ │
│  └──────────────┘    └──────────────┘    └───────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Web UI approach | xterm.js terminal | True parity, Claude provides chat UX |
| Session ownership | Server-owned PTY | Sessions survive CLI disconnect |
| CLI UX | Create new by default, `--attach` flag | Simple mental model |
| PTY library | `portable-pty` crate | Cross-platform, battle-tested |
| Structured data | Claude hooks system | Reliable, no ANSI parsing needed |
| Hook transport | Unix socket (Unix), TCP localhost (Windows) | Fast on Unix, works everywhere |
| Hook installation | Auto-configure on daemon start | Zero friction |

## PTY Manager

```rust
// vibes-core/src/pty/manager.rs

pub struct PtyManager {
    sessions: HashMap<SessionId, PtySession>,
    config: PtyConfig,
}

pub struct PtySession {
    id: SessionId,
    name: Option<String>,
    pty: Box<dyn portable_pty::MasterPty>,
    child: Box<dyn portable_pty::Child>,
    state: SessionState,
    created_at: Instant,
    subscribers: HashSet<ClientId>,
}

pub struct PtyConfig {
    initial_size: PtySize,        // Default 120x40
    claude_path: PathBuf,         // Path to claude binary
    working_dir: Option<PathBuf>, // Inherited from session creator
}
```

### Operations

| Method | Description |
|--------|-------------|
| `create_session()` | Spawn new `claude` in PTY, return session ID |
| `attach(session_id, client_id)` | Subscribe client to PTY I/O |
| `detach(session_id, client_id)` | Unsubscribe client |
| `write(session_id, bytes)` | Send input to PTY |
| `resize(session_id, cols, rows)` | Resize PTY (propagates SIGWINCH) |
| `kill(session_id)` | Terminate session |

## WebSocket Protocol

### New Message Types

```typescript
// Client → Server
| { type: 'create_session'; name?: string; request_id: string }
| { type: 'attach'; session_id: string }
| { type: 'detach'; session_id: string }
| { type: 'pty_input'; session_id: string; data: string }  // base64
| { type: 'pty_resize'; session_id: string; cols: number; rows: number }
| { type: 'kill_session'; session_id: string }
| { type: 'list_sessions'; request_id: string }

// Server → Client
| { type: 'session_created'; request_id: string; session_id: string }
| { type: 'attach_ack'; session_id: string; cols: number; rows: number }
| { type: 'pty_output'; session_id: string; data: string }  // base64
| { type: 'pty_exit'; session_id: string; exit_code: number | null }
| { type: 'session_state'; session_id: string; state: SessionState }
| { type: 'session_list'; request_id: string; sessions: SessionInfo[] }
| { type: 'session_removed'; session_id: string; reason: string }
| { type: 'error'; message: string; code: string }
```

### Removed Message Types (PrintMode-specific)

- `input` → replaced by `pty_input`
- `claude` (ClaudeEvent wrapper) → hooks provide structured data
- `user_input` → `pty_input` handles
- `permission_response` → PTY stdin handles naturally
- `subscribe`/`unsubscribe` → renamed to `attach`/`detach`

## Hooks Integration

### Installed Hooks

```json
{
  "hooks": {
    "PreToolUse": [{
      "hooks": [{ "type": "command", "command": "~/.vibes/hooks/pre-tool-use.sh" }]
    }],
    "PostToolUse": [{
      "hooks": [{ "type": "command", "command": "~/.vibes/hooks/post-tool-use.sh" }]
    }],
    "Stop": [{
      "hooks": [{ "type": "command", "command": "~/.vibes/hooks/stop.sh" }]
    }]
  }
}
```

### Data Captured

| Hook | Data Available |
|------|----------------|
| PreToolUse | tool name, input params, can block |
| PostToolUse | tool name, output, duration, success |
| Stop | transcript_path (full conversation) |

### Hook Transport

| Platform | Transport | Path/Address |
|----------|-----------|--------------|
| macOS | Unix socket | `/tmp/vibes-hooks.sock` |
| Linux | Unix socket | `/tmp/vibes-hooks.sock` |
| Windows | TCP localhost | `127.0.0.1:7744` |

Hook scripts use `~/.vibes/bin/vibes-hook-send` helper which abstracts the transport.

## CLI Changes

```
vibes claude [OPTIONS] [PROMPT]

Options:
  --attach <ID>       Attach to existing session
  --list              List active sessions
  --session-name <N>  Human-friendly name for new session
  --no-serve          Don't auto-start daemon
```

The CLI becomes a thin PTY client:
1. Ensures daemon is running
2. Creates or attaches to session
3. Enters raw terminal mode
4. Proxies I/O bidirectionally
5. Propagates resize events

## Web UI

Replace chat-based UI with xterm.js terminal:

```tsx
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';

function SessionTerminal({ sessionId }) {
  // Terminal input → pty_input message
  // pty_output message → terminal.write()
  // Resize → pty_resize message
}
```

### Dependencies

```json
{
  "xterm": "^5.3.0",
  "xterm-addon-fit": "^0.8.0",
  "xterm-addon-web-links": "^0.9.0"
}
```

### Removed Components

- `ClaudeSession.tsx`
- `MessageList.tsx`
- `ChatInput.tsx`
- `PermissionDialog.tsx`

## Code Cleanup

### Files to Remove

```
vibes-core/src/backend/print_mode.rs
vibes-core/src/backend/slow_mock.rs
vibes-core/src/parser/stream_json.rs
```

### Simplified Backend Trait

```rust
pub trait ClaudeBackend: Send + Sync {
    async fn spawn(&mut self, working_dir: &Path) -> Result<()>;
    async fn write(&mut self, data: &[u8]) -> Result<()>;
    async fn read(&mut self) -> Result<Vec<u8>>;
    async fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    fn exit_status(&self) -> Option<i32>;
    async fn kill(&mut self) -> Result<()>;
}
```

### Simplified Session State

```rust
pub enum SessionState {
    Running,
    Exited(i32),
}
```

## Implementation Phases

### Phase 1: Core PTY Infrastructure
- Add `portable-pty` to vibes-core
- Implement `PtyManager` with spawn/read/write/resize/kill
- Unit tests with mock PTY

### Phase 2: Server Integration
- Add PTY message types to WebSocket protocol
- Wire `PtyManager` into vibes-server
- Implement attach/detach for multiple subscribers

### Phase 3: CLI Client
- Refactor `vibes claude` to PTY client mode
- Raw terminal handling with crossterm
- Resize propagation

### Phase 4: Web UI
- Add xterm.js dependencies
- Create `SessionTerminal` component
- Remove old chat components

### Phase 5: Hooks Integration
- Hook receiver (Unix socket / TCP)
- Auto-install hooks on daemon start
- Emit ClaudeEvents from hook data
- Parse transcripts for history

### Phase 6: Cleanup
- Remove PrintModeBackend
- Remove stream-json parser
- Simplify session state machine
- Update documentation

## Migration Strategy

1. Build PTY alongside PrintMode (feature flag)
2. Test thoroughly before switching
3. One PR per phase for reviewability
4. PrintMode removed only after PTY is stable

## References

- [portable-pty crate](https://crates.io/crates/portable-pty)
- [xterm.js](https://xtermjs.org/)
- [Claude Code Hooks](https://code.claude.com/docs/en/hooks)
