# Milestone 1.4: Server + Web UI - Design Document

> Daemon architecture with HTTP/WebSocket server and TanStack web UI for remote session control.

## Overview

This milestone transforms vibes from a CLI wrapper into a full server-based architecture. The server owns all session state, and both CLI and web UI connect as WebSocket clients. This enables the core value proposition: start a session from your terminal, monitor and control it from your phone.

### Key Decisions

| Decision | Choice | Notes |
|----------|--------|-------|
| Architecture | Daemon owns SessionManager | CLI becomes a client, not direct caller |
| CLI-Server protocol | WebSocket (same as Web UI) | Single protocol, guaranteed parity |
| Daemon lifecycle | Auto-start on demand | `vibes claude` starts daemon if needed |
| Frontend framework | TanStack Router + Query (SPA) | Static assets embedded via rust-embed |
| WebSocket model | Single multiplexed connection | Subscribe to sessions of interest |
| View modes | Responsive auto-detect + toggle | Simple mode on mobile, full on desktop |
| Auth | None for MVP | Bind to 0.0.0.0, auth in Phase 2 |
| URL structure | Harness-prefixed (`/claude/:id`) | Extensible for Codex, Gemini, etc. |

---

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         vibes serve (daemon)                             │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                         vibes-core                                 │  │
│  │   SessionManager ◄──► EventBus (Memory) ◄──► PluginHost           │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                   │                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                      vibes-server (new crate)                      │  │
│  │   ┌─────────────────┐     ┌─────────────────────────────────────┐ │  │
│  │   │  HTTP (axum)    │     │  WebSocket                          │ │  │
│  │   │  - GET /        │     │  - Subscribe to sessions            │ │  │
│  │   │  - GET /api/*   │     │  - Send input                       │ │  │
│  │   │  - Static files │     │  - Permission responses             │ │  │
│  │   └─────────────────┘     └─────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
          ▲                              ▲                        ▲
          │ WS                           │ WS                     │ HTTP
          │                              │                        │
   ┌──────────────┐              ┌──────────────┐          ┌────────────┐
   │ vibes claude │              │   Web UI     │          │  Browser   │
   │  (CLI client)│              │   (SPA)      │          │  (static)  │
   └──────────────┘              └──────────────┘          └────────────┘
```

### Key Changes from Previous Milestones

- New `vibes-server` crate containing HTTP/WebSocket server
- Server owns `SessionManager`, `EventBus`, and `PluginHost`
- `vibes claude` becomes a WebSocket client, not a direct Claude caller
- Web UI is embedded and served from the same binary

---

## Crate Structure

```
vibes/
├── vibes-core/                    # Existing - no major changes
│   └── src/
│       ├── session/               # SessionManager, Session
│       ├── events/                # EventBus, MemoryEventBus
│       ├── plugins/               # PluginHost
│       ├── backend/               # ClaudeBackend, PrintModeBackend
│       └── parser/                # Stream-JSON parser
│
├── vibes-server/                  # NEW CRATE
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 # VibesServer, ServerConfig
│       ├── http/
│       │   ├── mod.rs             # axum router setup
│       │   ├── api.rs             # REST API handlers
│       │   └── static_files.rs    # rust-embed static file serving
│       ├── ws/
│       │   ├── mod.rs             # WebSocket upgrade handler
│       │   ├── connection.rs      # Individual WS connection management
│       │   └── protocol.rs        # Message types, serialization
│       └── state.rs               # Shared server state (Arc<AppState>)
│
├── vibes-cli/                     # Existing - significant changes
│   └── src/
│       ├── main.rs
│       ├── commands/
│       │   ├── claude.rs          # REWRITE: WS client, not direct session
│       │   ├── serve.rs           # NEW: daemon management
│       │   ├── config.rs          # Existing
│       │   └── plugin.rs          # Existing
│       ├── client/                # NEW: WebSocket client for CLI
│       │   ├── mod.rs
│       │   └── connection.rs
│       └── daemon/                # NEW: auto-start logic
│           └── mod.rs
│
├── vibes-plugin-api/              # Existing - no changes
│
└── web-ui/                        # NEW: TanStack SPA
    ├── package.json
    ├── vite.config.ts
    ├── index.html
    └── src/
        ├── main.tsx
        ├── routes/                # TanStack Router
        ├── components/
        ├── hooks/                 # useWebSocket, useSession, etc.
        └── lib/                   # API client, types
```

---

## WebSocket Protocol

Both CLI and Web UI use the same protocol. Single connection, multiplexed sessions.

### Client → Server Messages

```typescript
// Subscribe to session events
{ "type": "subscribe", "session_ids": ["sess-abc", "sess-xyz"] }

// Unsubscribe from sessions
{ "type": "unsubscribe", "session_ids": ["sess-abc"] }

// Create a new session
{ "type": "create_session", "name": "my-feature", "request_id": "req-1" }

// Send user input to a session
{ "type": "input", "session_id": "sess-abc", "content": "Help me refactor this" }

// Respond to permission request
{ "type": "permission_response", "session_id": "sess-abc", "request_id": "perm-1", "approved": true }
```

### Server → Client Messages

```typescript
// Session created confirmation
{ "type": "session_created", "request_id": "req-1", "session_id": "sess-abc", "name": "my-feature" }

// Claude events (streamed)
{ "type": "claude", "session_id": "sess-abc", "event": {
    "type": "text_delta", "text": "Here's how..."
}}

// Permission request
{ "type": "claude", "session_id": "sess-abc", "event": {
    "type": "permission_request", "id": "perm-1", "tool": "Bash", "description": "Run: npm test"
}}

// Session state change
{ "type": "session_state", "session_id": "sess-abc", "state": "processing" }

// Error
{ "type": "error", "session_id": "sess-abc", "message": "Session not found", "code": "NOT_FOUND" }
```

### Connection Flow (CLI)

```
vibes claude "query"
    │
    ├─► Connect WebSocket to ws://localhost:7432/ws
    ├─► Send: { "type": "create_session", "request_id": "1" }
    ├─► Recv: { "type": "session_created", "session_id": "sess-abc", ... }
    ├─► Send: { "type": "subscribe", "session_ids": ["sess-abc"] }
    ├─► Send: { "type": "input", "session_id": "sess-abc", "content": "query" }
    ├─► Recv: { "type": "claude", ... } (stream events to terminal)
    ├─► Recv: { "type": "claude", "event": { "type": "turn_complete" }}
    └─► Disconnect (session stays alive on server)
```

### Connection Flow (Web UI)

```
Browser loads /
    │
    ├─► Connect WebSocket to ws://localhost:7432/ws
    ├─► Fetch GET /api/claude/sessions (list existing)
    ├─► Send: { "type": "subscribe", "session_ids": ["sess-abc"] }
    ├─► Recv: streamed events, render in UI
    ├─► User clicks "Approve" on permission
    ├─► Send: { "type": "permission_response", ... }
    └─► Connection stays open while page is open
```

---

## HTTP API

REST endpoints for non-streaming operations. WebSocket handles real-time, HTTP handles queries and static files.

### Endpoints

```
GET  /                              # Serve web UI (index.html)
GET  /assets/*                      # Static files (JS, CSS, images)

GET  /api/health                    # Health check, server version
GET  /api/claude/sessions           # List Claude sessions
GET  /api/claude/sessions/:id       # Session details + recent history
GET  /api/claude/sessions/:id/history   # Full conversation history (paginated)

POST /api/claude/sessions           # Create session (alternative to WS)
DELETE /api/claude/sessions/:id     # Terminate session

GET  /api/plugins                   # List plugins and status
POST /api/plugins/:name/enable      # Enable a plugin
POST /api/plugins/:name/disable     # Disable a plugin
```

### Example Responses

```typescript
// GET /api/health
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "active_sessions": 2,
  "connected_clients": 3
}

// GET /api/claude/sessions
{
  "sessions": [
    {
      "id": "sess-abc",
      "name": "refactoring",
      "state": "waiting_for_input",
      "created_at": "2025-12-26T10:00:00Z",
      "last_activity": "2025-12-26T10:15:00Z"
    }
  ]
}

// GET /api/claude/sessions/:id
{
  "id": "sess-abc",
  "name": "refactoring",
  "state": "processing",
  "created_at": "2025-12-26T10:00:00Z",
  "usage": { "input_tokens": 1500, "output_tokens": 3200 },
  "pending_permission": {
    "id": "perm-1",
    "tool": "Bash",
    "description": "Run: npm test"
  }
}
```

### Static File Serving

```rust
#[derive(RustEmbed)]
#[folder = "web-ui/dist/"]
struct WebAssets;

// Serve index.html for all non-API routes (SPA routing)
// Serve actual files for /assets/*
```

---

## Daemon Lifecycle

Auto-start on demand with graceful management.

### Commands

```bash
vibes serve                 # Run daemon in foreground (for development/debugging)
vibes serve --daemon        # Daemonize (background, detach from terminal)
vibes serve stop            # Stop running daemon
vibes serve status          # Show daemon status

vibes claude "query"        # Auto-starts daemon if not running, then connects
```

### Auto-Start Flow

```
vibes claude "query"
    │
    ├─► Try connect to ws://localhost:7432/ws
    │
    ├─► If connection fails:
    │   ├─► Check if port is in use by non-vibes process → error
    │   ├─► Spawn `vibes serve --daemon` as background process
    │   ├─► Wait up to 5 seconds for server to become ready
    │   │   (poll /api/health until 200)
    │   └─► Retry WebSocket connection
    │
    └─► Proceed with session creation
```

### Daemon State File

```
~/.config/vibes/daemon.json
{
  "pid": 12345,
  "port": 7432,
  "started_at": "2025-12-26T10:00:00Z"
}
```

Used for:
- `vibes serve status` to report daemon info
- `vibes serve stop` to find and signal the process
- Stale detection (check if PID is still alive)

### Shutdown Behavior

- **Graceful:** `vibes serve stop` sends SIGTERM, server finishes active requests, closes WebSocket connections cleanly
- **Sessions preserved:** Session state stays in memory while daemon runs. Lost on daemon stop (persistence is future work)
- **No auto-stop:** Daemon keeps running until explicitly stopped. This enables "start from CLI, continue from phone" workflow.

---

## Web UI Structure

TanStack Router + Query SPA with responsive simple/full mode.

### Routes

```
/                        → Dashboard (all harnesses overview)
/claude                  → ClaudeSessionListPage (Claude sessions)
/claude/:id              → ClaudeSessionPage (session view)
/codex                   → (future) CodexSessionListPage
/codex/:id               → (future) CodexSessionPage
/settings                → SettingsPage
```

### Component Hierarchy

```
App
├── Layout
│   ├── Header
│   │   ├── Logo
│   │   ├── ConnectionStatus (WS connected indicator)
│   │   └── ModeToggle (simple/full override)
│   └── Outlet (router content)
│
├── SessionListPage (/claude)
│   ├── SessionCard (for each session)
│   │   ├── SessionName
│   │   ├── SessionState (badge: thinking, waiting, etc.)
│   │   ├── PermissionAlert (if pending permission)
│   │   └── QuickActions (approve/deny buttons)
│   └── CreateSessionButton
│
└── SessionPage (/claude/:id)
    ├── SimpleMode (phone/tablet default)
    │   ├── SessionStatus (large, clear state indicator)
    │   ├── PermissionCard (prominent approve/deny)
    │   ├── QuickInput (simple text field + send)
    │   └── RecentActivity (last few messages, collapsed)
    │
    └── FullMode (desktop default)
        ├── ConversationView
        │   ├── MessageBubble (user messages)
        │   ├── AssistantMessage (claude responses)
        │   │   ├── TextContent (markdown rendered)
        │   │   ├── ThinkingBlock (collapsible)
        │   │   └── ToolUseBlock (command + result)
        │   └── PermissionRequestCard
        ├── InputArea (multi-line, submit button)
        └── SessionSidebar (usage stats, session info)
```

### Responsive Breakpoints

```typescript
const useViewMode = () => {
  const [override, setOverride] = useState<'simple' | 'full' | null>(null);
  const isSmallScreen = useMediaQuery('(max-width: 768px)');

  const mode = override ?? (isSmallScreen ? 'simple' : 'full');

  return { mode, setOverride };
};
```

### Key Hooks

```typescript
// WebSocket connection management
useWebSocket() → { connected, send, subscribe }

// Session data with real-time updates
useSession(id) → { session, messages, permissions, isLoading }

// Session list with polling fallback
useSessions() → { sessions, refetch }
```

---

## Build Pipeline

### Web UI Build

```bash
# Development
cd web-ui
npm install
npm run dev          # Vite dev server with HMR at :5173

# Production build
npm run build        # Output to web-ui/dist/
```

### Rust Embedding

```rust
// vibes-server/src/http/static_files.rs

#[derive(RustEmbed)]
#[folder = "../web-ui/dist/"]
struct WebAssets;
```

### Build Order

```bash
just build           # Builds web-ui first, then Rust crates
```

```just
# justfile additions
build-web:
    cd web-ui && npm run build

build: build-web
    cargo build --release
```

---

## Testing Strategy

### vibes-server (Rust)

```
src/
├── http/
│   ├── api.rs
│   └── api_test.rs      # axum test client, mock SessionManager
├── ws/
│   ├── connection.rs
│   └── connection_test.rs   # Mock WebSocket, protocol tests
└── tests/
    └── integration.rs   # Full server startup, real HTTP/WS
```

### web-ui (TypeScript)

```
src/
├── components/
│   ├── SessionCard.tsx
│   └── SessionCard.test.tsx   # Vitest + Testing Library
├── hooks/
│   ├── useWebSocket.ts
│   └── useWebSocket.test.ts   # Mock WebSocket
└── e2e/
    └── session.spec.ts        # Playwright (optional, future)
```

### CI Pipeline Updates

```yaml
# .github/workflows/ci.yml additions
jobs:
  build:
    steps:
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Build web-ui
        run: cd web-ui && npm ci && npm run build

      - name: Rust build & test
        run: |
          nix develop --command just test
          nix develop --command just build
```

---

## Dependencies

### vibes-server/Cargo.toml

```toml
[package]
name = "vibes-server"
version = "0.1.0"
edition = "2021"

[dependencies]
vibes-core = { path = "../vibes-core" }
axum = { version = "0.7", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
rust-embed = "8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
```

### web-ui/package.json

```json
{
  "dependencies": {
    "@tanstack/react-query": "^5",
    "@tanstack/react-router": "^1",
    "react": "^18",
    "react-dom": "^18"
  },
  "devDependencies": {
    "@types/react": "^18",
    "@vitejs/plugin-react": "^4",
    "typescript": "^5",
    "vite": "^5",
    "vitest": "^1",
    "@testing-library/react": "^14"
  }
}
```

---

## Deliverables

### New Crates & Directories

| Component | Description |
|-----------|-------------|
| `vibes-server/` | New crate: axum HTTP/WS server, rust-embed static serving |
| `web-ui/` | New directory: TanStack Router + Query SPA |

### Modified Crates

| Crate | Changes |
|-------|---------|
| `vibes-cli` | Rewrite `claude.rs` as WS client, add `serve.rs` command, add `client/` and `daemon/` modules |
| `vibes-core` | Minor: ensure SessionManager/EventBus work with server ownership model |

### Documentation Updates

| Document | Updates Needed |
|----------|----------------|
| `docs/VISION.md` | Update architecture diagram, clarify daemon model vs embedded server, add ADR-010 |
| `docs/PROGRESS.md` | Mark Milestone 1.4 items as complete when done |
| `README.md` | Add web UI usage section, update quick start |
| `CLAUDE.md` | Add `just build-web` command, note about Node.js requirement |

---

## Milestone 1.4 Checklist

- [ ] vibes-server crate with axum HTTP/WebSocket
- [ ] Daemon lifecycle (serve, serve stop, auto-start)
- [ ] CLI rewrite as WebSocket client
- [ ] WebSocket protocol implementation
- [ ] REST API endpoints
- [ ] TanStack web UI with routing
- [ ] Simple mode (mobile) view
- [ ] Full mode (desktop) view
- [ ] Permission approve/deny flow
- [ ] rust-embed static file bundling
- [ ] Build pipeline (web-ui → rust)
- [ ] Update PRD with ADR-010
