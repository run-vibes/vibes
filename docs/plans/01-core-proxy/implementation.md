# Milestone 1.1: Core Proxy - Implementation Plan

> Step-by-step implementation guide for vibes-core

## Prerequisites

- Rust toolchain (managed via Nix)
- Claude Code CLI installed (`claude` binary in PATH)
- Nix with flakes enabled
- direnv installed

---

## Phase 1: Development Environment

### Step 1.1: Create Nix flake

Create `flake.nix` with Rust toolchain, cargo-nextest, cargo-mutants, cargo-watch, just, and direnv.

**Files:**
- [x] `flake.nix`

**Validation:**
```bash
nix flake check
nix develop  # Should enter shell with all tools
```

### Step 1.2: Configure direnv

Create `.envrc` for automatic shell loading.

**Files:**
- [x] `.envrc`

**Validation:**
```bash
cd .. && cd vibes  # Should auto-load
just --version     # Should work
```

### Step 1.3: Create justfile

Set up task runner with common commands.

**Files:**
- [x] `justfile`

**Validation:**
```bash
just  # Should list available commands
```

---

## Phase 2: Project Scaffolding

### Step 2.1: Create workspace Cargo.toml

Initialize Cargo workspace with vibes-core member.

**Files:**
- [x] `Cargo.toml` (workspace root)
- [x] `vibes-core/Cargo.toml`
- [x] `vibes-core/src/lib.rs`

**Validation:**
```bash
just check  # cargo check should pass
```

### Step 2.2: Set up module structure

Create empty module files with TODO placeholders.

**Files:**
- [x] `vibes-core/src/error.rs`
- [x] `vibes-core/src/events/mod.rs`
- [x] `vibes-core/src/events/types.rs`
- [x] `vibes-core/src/events/bus.rs`
- [x] `vibes-core/src/events/memory.rs`
- [x] `vibes-core/src/backend/mod.rs`
- [x] `vibes-core/src/backend/traits.rs`
- [x] `vibes-core/src/backend/mock.rs`
- [x] `vibes-core/src/backend/print_mode.rs`
- [x] `vibes-core/src/parser/mod.rs`
- [x] `vibes-core/src/parser/stream_json.rs`
- [x] `vibes-core/src/session/mod.rs`
- [x] `vibes-core/src/session/state.rs`
- [x] `vibes-core/src/session/manager.rs`

**Validation:**
```bash
just check  # Should compile (even with empty modules)
```

---

## Phase 3: Error Types

### Step 3.1: Implement error types

Implement all error types using thiserror.

**Files:**
- [x] `vibes-core/src/error.rs`

**Tests:**
- [x] Error Display implementations work
- [x] Error conversions (From) work

**Validation:**
```bash
just test
```

---

## Phase 4: Event Types

### Step 4.1: Implement ClaudeEvent

Define ClaudeEvent enum with all variants.

**Files:**
- [x] `vibes-core/src/events/types.rs`

**Tests:**
- [x] Serialization round-trip
- [x] Clone and Debug work

### Step 4.2: Implement VibesEvent

Define VibesEvent enum wrapping ClaudeEvent and client events.

**Files:**
- [x] `vibes-core/src/events/types.rs` (extend)

**Tests:**
- [x] Serialization round-trip
- [x] Session ID extraction works

### Step 4.3: Implement Usage and other supporting types

**Files:**
- [x] `vibes-core/src/events/types.rs` (extend)

**Validation:**
```bash
just test
```

---

## Phase 5: Stream-JSON Parser

### Step 5.1: Define StreamMessage types

Implement serde-tagged enum for Claude's stream-json format.

**Files:**
- [x] `vibes-core/src/parser/stream_json.rs`

**Tests:**
- [x] Parse real stream-json samples
- [x] Unknown message types deserialize to Unknown variant

### Step 5.2: Implement parse_line function

Parse single line with resilient error handling.

**Files:**
- [x] `vibes-core/src/parser/stream_json.rs` (extend)

**Tests:**
- [x] Empty lines return None
- [x] Invalid JSON logs warning, returns None
- [x] Valid JSON parses correctly

### Step 5.3: Implement to_claude_event conversion

Convert StreamMessage to ClaudeEvent.

**Files:**
- [x] `vibes-core/src/parser/stream_json.rs` (extend)

**Tests:**
- [x] Each StreamMessage variant converts correctly
- [x] Unknown returns None

**Validation:**
```bash
just test
```

---

## Phase 6: EventBus

### Step 6.1: Define EventBus trait

Create the trait with publish, subscribe, subscribe_from, get_session_events.

**Files:**
- [x] `vibes-core/src/events/bus.rs`

### Step 6.2: Implement MemoryEventBus

Implement the trait with Vec storage and broadcast channel.

**Files:**
- [x] `vibes-core/src/events/memory.rs`

**Tests:**
- [x] Publish increments sequence number
- [x] Subscribe receives new events
- [x] subscribe_from replays historical events
- [x] get_session_events filters by session_id
- [x] Concurrent publish/subscribe works

**Validation:**
```bash
just test
```

---

## Phase 7: Backend Abstraction

### Step 7.1: Define ClaudeBackend trait

Create the trait with send, subscribe, respond_permission, etc.

**Files:**
- [x] `vibes-core/src/backend/traits.rs`

### Step 7.2: Define BackendState enum

**Files:**
- [x] `vibes-core/src/backend/traits.rs` (extend)

### Step 7.3: Define BackendFactory trait

**Files:**
- [x] `vibes-core/src/backend/traits.rs` (extend)

### Step 7.4: Implement MockBackend

Implement backend that emits scripted events.

**Files:**
- [x] `vibes-core/src/backend/mock.rs`

**Tests:**
- [x] queue_response works
- [x] send() emits queued events
- [x] State transitions correctly
- [x] Multiple sends work with queue

**Validation:**
```bash
just test
```

---

## Phase 8: Session

### Step 8.1: Define SessionState enum

**Files:**
- [x] `vibes-core/src/session/state.rs`

### Step 8.2: Implement Session struct

Implement session with backend, event bus, state machine.

**Files:**
- [x] `vibes-core/src/session/state.rs` (extend)

**Tests:**
- [x] New session starts in Idle state
- [x] send() transitions to Processing
- [x] Events forwarded to EventBus
- [x] Error transitions to Failed
- [x] retry() resets Failed to Idle

### Step 8.3: Implement event forwarding

Background task that forwards ClaudeEvents to VibesEvents on bus.

**Files:**
- [x] `vibes-core/src/session/state.rs` (extend)

**Tests:**
- [x] ClaudeEvents appear as VibesEvent::Claude on bus
- [x] State changes published

**Validation:**
```bash
just test
```

---

## Phase 9: SessionManager

### Step 9.1: Implement SessionManager

Manage multiple sessions with create, get, list.

**Files:**
- [x] `vibes-core/src/session/manager.rs`

**Tests:**
- [x] create_session returns unique ID
- [x] get_session retrieves by ID
- [x] list_sessions returns all
- [x] Sessions use injected BackendFactory

**Validation:**
```bash
just test
```

---

## Phase 10: PrintModeBackend

### Step 10.1: Implement process spawning

Spawn `claude -p` with correct arguments.

**Files:**
- [x] `vibes-core/src/backend/print_mode.rs`

**Tests (unit with mock process):**
- [x] Command built with correct args
- [x] --session-id passed correctly
- [x] --allowedTools passed when configured

### Step 10.2: Implement stdout streaming

Read stdout lines, parse stream-json, emit events.

**Files:**
- [x] `vibes-core/src/backend/print_mode.rs` (extend)

**Tests (unit):**
- [x] Lines parsed and emitted as events
- [x] Parse errors logged, not fatal

### Step 10.3: Implement process lifecycle

Handle process exit, errors, state transitions.

**Files:**
- [x] `vibes-core/src/backend/print_mode.rs` (extend)

**Tests (unit):**
- [x] Clean exit transitions to Idle
- [x] Crash transitions to Failed
- [x] Exit code captured

**Validation:**
```bash
just test
```

---

## Phase 11: Integration Tests

### Step 11.1: Create integration test harness

Set up tests that spawn real Claude process.

**Files:**
- [x] `vibes-core/tests/integration.rs`
- [x] `vibes-core/tests/print_mode_test.rs`

### Step 11.2: Test real Claude interaction

**Tests:**
- [x] Simple prompt returns text
- [x] Session ID continuity works
- [x] Tool use events parsed correctly

**Validation:**
```bash
just test-all  # Requires Claude CLI
```

---

## Phase 12: Public API & Documentation

### Step 12.1: Finalize lib.rs exports

Export public API surface.

**Files:**
- [x] `vibes-core/src/lib.rs`

### Step 12.2: Add rustdoc comments

Document public types and functions.

**Files:**
- [x] All public items

**Validation:**
```bash
cargo doc --open
```

---

## Phase 13: Final Validation

### Step 13.1: Full test suite

```bash
just pre-commit  # fmt-check, clippy, test
```

### Step 13.2: Mutation testing

```bash
just mutants
```

Review surviving mutants and add tests or accept as reasonable.

### Step 13.3: Update PROGRESS.md

Mark Milestone 1.1 items as complete.

---

## Checklist Summary

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Development Environment | [x] |
| 2 | Project Scaffolding | [x] |
| 3 | Error Types | [x] |
| 4 | Event Types | [x] |
| 5 | Stream-JSON Parser | [x] |
| 6 | EventBus | [x] |
| 7 | Backend Abstraction | [x] |
| 8 | Session | [x] |
| 9 | SessionManager | [x] |
| 10 | PrintModeBackend | [x] |
| 11 | Integration Tests | [x] |
| 12 | Public API & Docs | [x] |
| 13 | Final Validation | [x] |

---

## Notes

- Each phase should be a separate commit (or PR if working in branches)
- Run `just pre-commit` before each commit
- Update PROGRESS.md as phases complete
- Integration tests require Claude Code CLI installed and configured
