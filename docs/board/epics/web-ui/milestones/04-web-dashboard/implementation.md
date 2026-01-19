# Milestone 1.4: Server + Web UI - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform vibes into a daemon-based architecture with HTTP/WebSocket server and TanStack web UI for remote session control.

**Architecture:** The server owns SessionManager/EventBus/PluginHost. Both CLI and Web UI connect as WebSocket clients using identical protocol. Daemon auto-starts on demand when `vibes claude` runs.

**Tech Stack:** Rust (axum, tokio-tungstenite, rust-embed), TypeScript (TanStack Router, TanStack Query, Vite, React)

---

## Phase Overview

| Phase | Description | Tasks |
|-------|-------------|-------|
| 1 | vibes-server crate foundation | 5 |
| 2 | WebSocket protocol | 3 |
| 3 | Daemon lifecycle | 4 |
| 4 | CLI as WebSocket client | 2 |
| 5 | Web UI foundation | 4 |
| 6 | Web UI views | 2 |
| 7 | Documentation updates | 2 |

**Total: 22 tasks**

---

## Phase 1: vibes-server Crate Foundation

### Task 1.1: Create vibes-server crate

**Files:**
- Create: `vibes-server/Cargo.toml`
- Create: `vibes-server/src/lib.rs`
- Create: `vibes-server/src/error.rs`
- Modify: `Cargo.toml` (workspace)

**Steps:**
1. Create directory: `mkdir -p vibes-server/src`
2. Create Cargo.toml with axum, tokio, tower-http, serde, tracing, uuid, thiserror dependencies
3. Create minimal lib.rs with ServerConfig struct
4. Create error.rs with ServerError enum
5. Add to workspace members in root Cargo.toml
6. Run: `cargo check -p vibes-server`
7. Commit: `feat(server): create vibes-server crate skeleton`

---

### Task 1.2: Add shared AppState

**Files:**
- Create: `vibes-server/src/state.rs`
- Modify: `vibes-server/src/lib.rs`

**Steps:**
1. Create state.rs with AppState struct holding SessionManager, PluginHost, started_at
2. Add test for AppState creation
3. Export from lib.rs
4. Run: `cargo test -p vibes-server`
5. Commit: `feat(server): add AppState with SessionManager and PluginHost`

---

### Task 1.3: Create HTTP router skeleton

**Files:**
- Create: `vibes-server/src/http/mod.rs`
- Create: `vibes-server/src/http/api.rs`
- Modify: `vibes-server/src/lib.rs`

**Steps:**
1. Create http/mod.rs with create_router function
2. Create http/api.rs with health endpoint returning status, version, uptime
3. Add axum-test dev dependency
4. Write test for health endpoint
5. Run tests
6. Commit: `feat(server): add HTTP router with health endpoint`

---

### Task 1.4: Add sessions API endpoints

**Files:**
- Modify: `vibes-server/src/http/api.rs`
- Modify: `vibes-server/src/http/mod.rs`

**Steps:**
1. Add chrono dependency for timestamps
2. Create SessionSummary and SessionListResponse types
3. Add list_sessions handler at `/api/claude/sessions`
4. Write test for empty session list
5. Run tests
6. Commit: `feat(server): add sessions list endpoint`

---

### Task 1.5: Add VibesServer struct with run method

**Files:**
- Modify: `vibes-server/src/lib.rs`

**Steps:**
1. Create VibesServer struct with config and state
2. Add run() method that binds TcpListener and serves router
3. Add with_state() constructor for testing
4. Write tests for config defaults and addr format
5. Run tests
6. Commit: `feat(server): add VibesServer with run method`

---

## Phase 2: WebSocket Protocol

### Task 2.1: Define WebSocket message types

**Files:**
- Create: `vibes-server/src/ws/mod.rs`
- Create: `vibes-server/src/ws/protocol.rs`
- Modify: `vibes-server/src/lib.rs`

**Steps:**
1. Create protocol.rs with ClientMessage enum (Subscribe, Unsubscribe, CreateSession, Input, PermissionResponse)
2. Create ServerMessage enum (SessionCreated, Claude, SessionState, Error)
3. Write serialization tests
4. Export from ws/mod.rs and lib.rs
5. Run tests
6. Commit: `feat(server): add WebSocket protocol message types`

---

### Task 2.2: Add WebSocket connection handler

**Files:**
- Create: `vibes-server/src/ws/connection.rs`
- Modify: `vibes-server/src/ws/mod.rs`
- Modify: `vibes-server/src/http/mod.rs`

**Steps:**
1. Add futures dependency
2. Create ws_handler for WebSocket upgrade
3. Create handle_socket for message loop
4. Create handle_message to dispatch client messages
5. Add `/ws` route to router
6. Run: `cargo check -p vibes-server`
7. Commit: `feat(server): add WebSocket connection handler`

---

### Task 2.3: Add event broadcasting to WebSocket clients

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`
- Modify: `vibes-server/src/state.rs`

**Steps:**
1. Add broadcast channel to AppState for (session_id, VibesEvent)
2. Add subscribe_events() method to AppState
3. Update handle_socket to use tokio::select! for bidirectional handling
4. Add vibes_event_to_server_message conversion
5. Run tests
6. Commit: `feat(server): add event broadcasting to WebSocket clients`

---

## Phase 3: Daemon Lifecycle

### Task 3.1: Add `vibes serve` command

**Files:**
- Create: `vibes-cli/src/commands/serve.rs`
- Modify: `vibes-cli/src/commands/mod.rs`
- Modify: `vibes-cli/src/main.rs`
- Modify: `vibes-cli/Cargo.toml`

**Steps:**
1. Add vibes-server dependency
2. Create ServeArgs with port, host, daemon flags
3. Create ServeCommand subcommand enum (Stop, Status)
4. Implement run_foreground, stub start_daemon/stop_daemon/show_status
5. Add to CLI enum and main match
6. Run: `cargo run -p vibes-cli -- serve --help`
7. Commit: `feat(cli): add vibes serve command`

---

### Task 3.2: Implement daemon state file

**Files:**
- Create: `vibes-cli/src/daemon/mod.rs`
- Create: `vibes-cli/src/daemon/state.rs`
- Modify: `vibes-cli/src/main.rs`

**Steps:**
1. Create DaemonState struct with pid, port, started_at
2. Create state_file_path() returning ~/.config/vibes/daemon.json
3. Implement read_daemon_state, write_daemon_state, clear_daemon_state
4. Add is_process_alive helper (Unix: kill -0)
5. Write serialization test
6. Run tests
7. Commit: `feat(cli): add daemon state file handling`

---

### Task 3.3: Implement daemon stop and status

**Files:**
- Modify: `vibes-cli/src/commands/serve.rs`

**Steps:**
1. Add nix dependency for Unix signals
2. Implement stop_daemon using SIGTERM
3. Implement show_status displaying PID, port, uptime
4. Update run_foreground to write/clear state file
5. Run: `cargo build -p vibes-cli`
6. Commit: `feat(cli): implement daemon stop and status commands`

---

### Task 3.4: Implement daemon auto-start

**Files:**
- Create: `vibes-cli/src/daemon/autostart.rs`
- Modify: `vibes-cli/src/daemon/mod.rs`

**Steps:**
1. Add reqwest and libc dependencies
2. Create ensure_daemon_running function
3. Implement start_daemon_process using detached child process
4. Implement wait_for_daemon_ready polling /api/health
5. Export from daemon/mod.rs
6. Run: `cargo build -p vibes-cli`
7. Commit: `feat(cli): implement daemon auto-start`

---

## Phase 4: CLI as WebSocket Client

### Task 4.1: Create WebSocket client module

**Files:**
- Create: `vibes-cli/src/client/mod.rs`
- Create: `vibes-cli/src/client/connection.rs`
- Modify: `vibes-cli/src/main.rs`

**Steps:**
1. Add tokio-tungstenite and futures-util dependencies
2. Create VibesClient struct with tx/rx channels
3. Implement connect() establishing WebSocket
4. Implement send(), recv(), create_session(), send_input()
5. Add client module to main.rs
6. Run: `cargo build -p vibes-cli`
7. Commit: `feat(cli): add WebSocket client module`

---

### Task 4.2: Rewrite claude command as WebSocket client

**Files:**
- Modify: `vibes-cli/src/commands/claude.rs`

**Steps:**
1. Rewrite handle_claude to call ensure_daemon_running
2. Connect VibesClient to daemon
3. Create or resume session via WebSocket
4. Send prompt if provided
5. Implement stream_output rendering ClaudeEvents to terminal
6. Run: `cargo build -p vibes-cli`
7. Commit: `feat(cli): rewrite claude command as WebSocket client`

---

## Phase 5: Web UI Foundation

### Task 5.1: Initialize web-ui project

**Files:**
- Create: `web-ui/package.json`
- Create: `web-ui/tsconfig.json`
- Create: `web-ui/vite.config.ts`
- Create: `web-ui/index.html`

**Steps:**
1. Create web-ui directory
2. Create package.json with TanStack dependencies
3. Create tsconfig.json for React/TypeScript
4. Create vite.config.ts with proxy for /api and /ws
5. Create index.html entry point
6. Run: `cd web-ui && npm install`
7. Commit: `feat(web): initialize web-ui project with Vite and TanStack`

---

### Task 5.2: Create React app entry point

**Files:**
- Create: `web-ui/src/main.tsx`
- Create: `web-ui/src/App.tsx`
- Create: `web-ui/src/index.css`

**Steps:**
1. Create main.tsx with QueryClientProvider and ReactDOM.render
2. Create App.tsx with TanStack Router setup (/, /claude routes)
3. Create index.css with base styles
4. Run: `cd web-ui && npm run build`
5. Commit: `feat(web): create React app entry point with TanStack Router`

---

### Task 5.3: Add WebSocket hook

**Files:**
- Create: `web-ui/src/hooks/useWebSocket.ts`
- Create: `web-ui/src/lib/types.ts`

**Steps:**
1. Create types.ts with ClientMessage and ServerMessage types
2. Create useWebSocket hook with connected state, send(), subscribe()
3. Handle connection/reconnection in useEffect
4. Create hooks/index.ts export
5. Commit: `feat(web): add WebSocket hook and message types`

---

### Task 5.4: Embed web-ui in Rust server

**Files:**
- Modify: `vibes-server/Cargo.toml`
- Create: `vibes-server/src/http/static_files.rs`
- Modify: `vibes-server/src/http/mod.rs`
- Modify: `justfile`

**Steps:**
1. Add rust-embed and mime_guess dependencies
2. Create static_files.rs with RustEmbed struct and handler
3. Add fallback route to router for SPA routing
4. Add build-web target to justfile
5. Update build target to depend on build-web
6. Run: `just build`
7. Commit: `feat(server): embed web-ui static files with rust-embed`

---

## Phase 6: Web UI Views

### Task 6.1: Create session list page

**Files:**
- Create: `web-ui/src/pages/ClaudeSessions.tsx`
- Create: `web-ui/src/components/SessionCard.tsx`
- Modify: `web-ui/src/App.tsx`
- Modify: `web-ui/src/index.css`

**Steps:**
1. Create SessionCard component with name, state badge, ID
2. Create ClaudeSessions page fetching /api/claude/sessions
3. Use TanStack Query with 5s refetch interval
4. Add route to App.tsx
5. Add grid and card styles
6. Run: `just build`
7. Commit: `feat(web): add session list page with SessionCard component`

---

### Task 6.2: Create session detail page with simple/full mode

**Files:**
- Create: `web-ui/src/pages/ClaudeSession.tsx`
- Create: `web-ui/src/components/SimpleMode.tsx`
- Create: `web-ui/src/components/FullMode.tsx`
- Create: `web-ui/src/hooks/useViewMode.ts`
- Modify: `web-ui/src/App.tsx`
- Modify: `web-ui/src/index.css`

**Steps:**
1. Create useViewMode hook with responsive detection and override
2. Create SimpleMode component (status, permission card, quick input)
3. Create FullMode component (conversation, streaming text, input area)
4. Create ClaudeSession page connecting WebSocket and rendering mode
5. Add /claude/$id route
6. Add styles for both modes
7. Run: `just build`
8. Commit: `feat(web): add session detail page with simple/full mode toggle`

---

## Phase 7: Documentation Updates

### Task 7.1: Update PRD with ADR-010

**Files:**
- Modify: `docs/VISION.md`

**Steps:**
1. Add ADR-010: Daemon Architecture section
2. Update architecture diagram to show daemon model
3. Commit: `docs: add ADR-010 daemon architecture to PRD`

---

### Task 7.2: Update PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

**Steps:**
1. Mark all Milestone 1.4 items as complete
2. Add changelog entry with date
3. Commit: `docs: mark milestone 1.4 complete`

---

## Execution

Each task follows TDD:
1. Write failing test (where applicable)
2. Verify it fails
3. Write minimal implementation
4. Verify tests pass
5. Commit

Run `just pre-commit` before each commit to ensure code quality.
