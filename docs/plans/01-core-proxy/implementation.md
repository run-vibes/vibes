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
- [ ] `flake.nix`

**Validation:**
```bash
nix flake check
nix develop  # Should enter shell with all tools
```

### Step 1.2: Configure direnv

Create `.envrc` for automatic shell loading.

**Files:**
- [ ] `.envrc`

**Validation:**
```bash
cd .. && cd vibes  # Should auto-load
just --version     # Should work
```

### Step 1.3: Create justfile

Set up task runner with common commands.

**Files:**
- [ ] `justfile`

**Validation:**
```bash
just  # Should list available commands
```

---

## Phase 2: Project Scaffolding

### Step 2.1: Create workspace Cargo.toml

Initialize Cargo workspace with vibes-core member.

**Files:**
- [ ] `Cargo.toml` (workspace root)
- [ ] `vibes-core/Cargo.toml`
- [ ] `vibes-core/src/lib.rs`

**Validation:**
```bash
just check  # cargo check should pass
```

### Step 2.2: Set up module structure

Create empty module files with TODO placeholders.

**Files:**
- [ ] `vibes-core/src/error.rs`
- [ ] `vibes-core/src/events/mod.rs`
- [ ] `vibes-core/src/events/types.rs`
- [ ] `vibes-core/src/events/bus.rs`
- [ ] `vibes-core/src/events/memory.rs`
- [ ] `vibes-core/src/backend/mod.rs`
- [ ] `vibes-core/src/backend/traits.rs`
- [ ] `vibes-core/src/backend/mock.rs`
- [ ] `vibes-core/src/backend/print_mode.rs`
- [ ] `vibes-core/src/parser/mod.rs`
- [ ] `vibes-core/src/parser/stream_json.rs`
- [ ] `vibes-core/src/session/mod.rs`
- [ ] `vibes-core/src/session/session.rs`
- [ ] `vibes-core/src/session/manager.rs`

**Validation:**
```bash
just check  # Should compile (even with empty modules)
```

---

## Phase 3: Error Types

### Step 3.1: Implement error types

Implement all error types using thiserror.

**Files:**
- [ ] `vibes-core/src/error.rs`

**Tests:**
- [ ] Error Display implementations work
- [ ] Error conversions (From) work

**Validation:**
```bash
just test
```

---

## Phase 4: Event Types

### Step 4.1: Implement ClaudeEvent

Define ClaudeEvent enum with all variants.

**Files:**
- [ ] `vibes-core/src/events/types.rs`

**Tests:**
- [ ] Serialization round-trip
- [ ] Clone and Debug work

### Step 4.2: Implement VibesEvent

Define VibesEvent enum wrapping ClaudeEvent and client events.

**Files:**
- [ ] `vibes-core/src/events/types.rs` (extend)

**Tests:**
- [ ] Serialization round-trip
- [ ] Session ID extraction works

### Step 4.3: Implement Usage and other supporting types

**Files:**
- [ ] `vibes-core/src/events/types.rs` (extend)

**Validation:**
```bash
just test
```

---

## Phase 5: Stream-JSON Parser

### Step 5.1: Define StreamMessage types

Implement serde-tagged enum for Claude's stream-json format.

**Files:**
- [ ] `vibes-core/src/parser/stream_json.rs`

**Tests:**
- [ ] Parse real stream-json samples
- [ ] Unknown message types deserialize to Unknown variant

### Step 5.2: Implement parse_line function

Parse single line with resilient error handling.

**Files:**
- [ ] `vibes-core/src/parser/stream_json.rs` (extend)

**Tests:**
- [ ] Empty lines return None
- [ ] Invalid JSON logs warning, returns None
- [ ] Valid JSON parses correctly

### Step 5.3: Implement to_claude_event conversion

Convert StreamMessage to ClaudeEvent.

**Files:**
- [ ] `vibes-core/src/parser/stream_json.rs` (extend)

**Tests:**
- [ ] Each StreamMessage variant converts correctly
- [ ] Unknown returns None

**Validation:**
```bash
just test
```

---

## Phase 6: EventBus

### Step 6.1: Define EventBus trait

Create the trait with publish, subscribe, subscribe_from, get_session_events.

**Files:**
- [ ] `vibes-core/src/events/bus.rs`

### Step 6.2: Implement MemoryEventBus

Implement the trait with Vec storage and broadcast channel.

**Files:**
- [ ] `vibes-core/src/events/memory.rs`

**Tests:**
- [ ] Publish increments sequence number
- [ ] Subscribe receives new events
- [ ] subscribe_from replays historical events
- [ ] get_session_events filters by session_id
- [ ] Concurrent publish/subscribe works

**Validation:**
```bash
just test
```

---

## Phase 7: Backend Abstraction

### Step 7.1: Define ClaudeBackend trait

Create the trait with send, subscribe, respond_permission, etc.

**Files:**
- [ ] `vibes-core/src/backend/traits.rs`

### Step 7.2: Define BackendState enum

**Files:**
- [ ] `vibes-core/src/backend/traits.rs` (extend)

### Step 7.3: Define BackendFactory trait

**Files:**
- [ ] `vibes-core/src/backend/traits.rs` (extend)

### Step 7.4: Implement MockBackend

Implement backend that emits scripted events.

**Files:**
- [ ] `vibes-core/src/backend/mock.rs`

**Tests:**
- [ ] queue_response works
- [ ] send() emits queued events
- [ ] State transitions correctly
- [ ] Multiple sends work with queue

**Validation:**
```bash
just test
```

---

## Phase 8: Session

### Step 8.1: Define SessionState enum

**Files:**
- [ ] `vibes-core/src/session/session.rs`

### Step 8.2: Implement Session struct

Implement session with backend, event bus, state machine.

**Files:**
- [ ] `vibes-core/src/session/session.rs` (extend)

**Tests:**
- [ ] New session starts in Idle state
- [ ] send() transitions to Processing
- [ ] Events forwarded to EventBus
- [ ] Error transitions to Failed
- [ ] retry() resets Failed to Idle

### Step 8.3: Implement event forwarding

Background task that forwards ClaudeEvents to VibesEvents on bus.

**Files:**
- [ ] `vibes-core/src/session/session.rs` (extend)

**Tests:**
- [ ] ClaudeEvents appear as VibesEvent::Claude on bus
- [ ] State changes published

**Validation:**
```bash
just test
```

---

## Phase 9: SessionManager

### Step 9.1: Implement SessionManager

Manage multiple sessions with create, get, list.

**Files:**
- [ ] `vibes-core/src/session/manager.rs`

**Tests:**
- [ ] create_session returns unique ID
- [ ] get_session retrieves by ID
- [ ] list_sessions returns all
- [ ] Sessions use injected BackendFactory

**Validation:**
```bash
just test
```

---

## Phase 10: PrintModeBackend

### Step 10.1: Implement process spawning

Spawn `claude -p` with correct arguments.

**Files:**
- [ ] `vibes-core/src/backend/print_mode.rs`

**Tests (unit with mock process):**
- [ ] Command built with correct args
- [ ] --session-id passed correctly
- [ ] --allowedTools passed when configured

### Step 10.2: Implement stdout streaming

Read stdout lines, parse stream-json, emit events.

**Files:**
- [ ] `vibes-core/src/backend/print_mode.rs` (extend)

**Tests (unit):**
- [ ] Lines parsed and emitted as events
- [ ] Parse errors logged, not fatal

### Step 10.3: Implement process lifecycle

Handle process exit, errors, state transitions.

**Files:**
- [ ] `vibes-core/src/backend/print_mode.rs` (extend)

**Tests (unit):**
- [ ] Clean exit transitions to Idle
- [ ] Crash transitions to Failed
- [ ] Exit code captured

**Validation:**
```bash
just test
```

---

## Phase 11: Integration Tests

### Step 11.1: Create integration test harness

Set up tests that spawn real Claude process.

**Files:**
- [ ] `vibes-core/tests/integration/mod.rs`
- [ ] `vibes-core/tests/integration/print_mode_test.rs`

### Step 11.2: Test real Claude interaction

**Tests:**
- [ ] Simple prompt returns text
- [ ] Session ID continuity works
- [ ] Tool use events parsed correctly

**Validation:**
```bash
just test-all  # Requires Claude CLI
```

---

## Phase 12: Public API & Documentation

### Step 12.1: Finalize lib.rs exports

Export public API surface.

**Files:**
- [ ] `vibes-core/src/lib.rs`

### Step 12.2: Add rustdoc comments

Document public types and functions.

**Files:**
- [ ] All public items

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
| 1 | Development Environment | [ ] |
| 2 | Project Scaffolding | [ ] |
| 3 | Error Types | [ ] |
| 4 | Event Types | [ ] |
| 5 | Stream-JSON Parser | [ ] |
| 6 | EventBus | [ ] |
| 7 | Backend Abstraction | [ ] |
| 8 | Session | [ ] |
| 9 | SessionManager | [ ] |
| 10 | PrintModeBackend | [ ] |
| 11 | Integration Tests | [ ] |
| 12 | Public API & Docs | [ ] |
| 13 | Final Validation | [ ] |

---

## Notes

- Each phase should be a separate commit (or PR if working in branches)
- Run `just pre-commit` before each commit
- Update PROGRESS.md as phases complete
- Integration tests require Claude Code CLI installed and configured
