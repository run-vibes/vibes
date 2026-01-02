# Milestone 19: Event CLI - Design Document

> Add `vibes event send` CLI command for writing events directly to Iggy, replacing the Unix socket + HookReceiver architecture.

## Overview

### The Problem

The current hook event architecture is overly complex:

```
Claude Code → Hook scripts → vibes-hook-send → Unix socket → HookReceiver → EventLog
```

- `HookReceiver` listens on a Unix socket
- `vibes-hook-send` is a separate binary that sends to the socket
- Multiple moving parts, none of which are currently wired up
- Hook events never reach the EventLog

### The Solution

Simplify to a single CLI command that writes directly to Iggy:

```
Claude Code → Hook scripts → vibes event send → Iggy HTTP API
```

Benefits:
- No Unix socket
- No HookReceiver process
- No separate `vibes-hook-send` binary
- All Iggy complexity encapsulated in the CLI
- Hook scripts remain simple shell scripts

---

## Scope

### In Scope

1. New CLI subcommand: `vibes event send`
2. Support `--data` option and stdin for payload
3. Iggy authentication handling (config file + env vars)
4. Remove unused `HookReceiver` code
5. Update hook scripts to use new CLI

### Out of Scope

1. Parsing PTY output for ClaudeEvents (separate milestone)
2. New event types beyond existing `VibesEvent` variants
3. Changes to Iggy configuration or setup

---

## CLI Design

### Command Structure

```
vibes event send [OPTIONS]

Options:
  -t, --type <TYPE>      Event type (required)
                         Values: hook, session-state, claude

  -s, --session <ID>     Session ID for event attribution

  -d, --data <JSON>      Event payload as JSON string
                         If omitted, reads from stdin

  --topic <NAME>         Iggy topic name (default: "events")

  -h, --help             Print help
```

### Examples

```bash
# Send hook event with --data
vibes event send --type hook --session abc123 \
  --data '{"type":"pre_tool_use","tool_name":"Bash","input":"{}"}'

# Send from stdin (useful in pipes)
echo '{"type":"stop","reason":"user"}' | vibes event send --type hook --session abc123

# Send session state change
vibes event send --type session-state --session abc123 \
  --data '{"state":"Processing"}'
```

### Event Type Mapping

| CLI Type | VibesEvent Variant |
|----------|-------------------|
| `hook` | `VibesEvent::Hook { session_id, event: HookEvent }` |
| `session-state` | `VibesEvent::SessionStateChanged { session_id, state }` |
| `claude` | `VibesEvent::Claude { session_id, event: ClaudeEvent }` |

---

## Configuration

### Iggy Connection

The CLI reads Iggy configuration from (in priority order):

1. Environment variables:
   - `VIBES_IGGY_HOST` (default: `127.0.0.1`)
   - `VIBES_IGGY_HTTP_PORT` (default: `3000`)
   - `VIBES_IGGY_USERNAME` (default: `iggy`)
   - `VIBES_IGGY_PASSWORD` (default: `iggy`)

2. Config file (`~/.config/vibes/config.toml`):
   ```toml
   [iggy]
   host = "127.0.0.1"
   http_port = 3000
   username = "iggy"
   password = "iggy"
   ```

### Token Caching

To avoid login on every invocation:
1. First call logs in and caches JWT token to `~/.cache/vibes/iggy-token`
2. Subsequent calls reuse cached token
3. If token expired, re-authenticate automatically

---

## Implementation

### File Changes

| File | Change |
|------|--------|
| `vibes-cli/src/commands/mod.rs` | Add `event` subcommand |
| `vibes-cli/src/commands/event.rs` | New file: event send implementation |
| `vibes-core/src/hooks/receiver.rs` | Delete |
| `vibes-core/src/hooks/mod.rs` | Remove HookReceiver exports |
| `vibes-core/src/lib.rs` | Remove HookReceiver re-exports |
| `vibes-core/src/hooks/scripts/` | Update hook scripts to use CLI |

### Dependencies

The CLI needs Iggy HTTP client. Options:

1. **Use `iggy` crate's HttpClient** - Already in workspace
2. **Use `reqwest` directly** - More control, simpler

Recommend option 2 (`reqwest`) for simplicity - just need POST with auth.

### Error Handling

```
vibes event send --type hook --data '...'

# Success: exits 0, no output

# Failures:
# - Exit 1: Invalid JSON payload
# - Exit 2: Iggy connection failed
# - Exit 3: Authentication failed
# - Exit 4: Send failed
```

Hook scripts can check exit code and log errors.

---

## Hook Script Updates

### Current (unused)

```bash
#!/bin/bash
# Sends to Unix socket via vibes-hook-send
echo "$1" | vibes-hook-send
```

### New

```bash
#!/bin/bash
# Sends directly to Iggy via vibes CLI
vibes event send --type hook --session "$VIBES_SESSION_ID" --data "$1"
```

The `VIBES_SESSION_ID` environment variable is already set by vibes when spawning Claude.

---

## Code Removal

### Files to Delete

- `vibes-core/src/hooks/receiver.rs` - HookReceiver implementation
- `vibes-cli/src/bin/vibes-hook-send.rs` - If exists, separate binary

### Code to Update

```rust
// vibes-core/src/hooks/mod.rs
// Remove:
mod receiver;
pub use receiver::{HookReceiver, HookReceiverConfig};

// vibes-core/src/lib.rs
// Remove from re-exports:
HookReceiver, HookReceiverConfig,
```

---

## Testing

### Unit Tests

1. `test_event_send_parses_hook_json` - Valid hook event parsing
2. `test_event_send_parses_session_state` - Valid state event parsing
3. `test_event_send_rejects_invalid_json` - Error on malformed input
4. `test_event_send_reads_stdin` - Reads payload from stdin when --data omitted

### Integration Tests

1. `test_event_send_writes_to_iggy` - End-to-end with running Iggy
2. `test_event_send_authenticates` - Token caching works
3. `test_hook_script_sends_event` - Hook script integration

---

## Checklist

### Phase 1: CLI Implementation
- [ ] Add `event` subcommand to vibes-cli
- [ ] Implement `vibes event send` with --data and stdin support
- [ ] Add Iggy HTTP client (reqwest-based)
- [ ] Implement authentication with token caching
- [ ] Add unit tests

### Phase 2: Cleanup
- [ ] Delete `vibes-core/src/hooks/receiver.rs`
- [ ] Remove HookReceiver from module exports
- [ ] Update hook scripts to use new CLI
- [ ] Remove vibes-hook-send binary if exists

### Phase 3: Verification
- [ ] Integration test with Iggy
- [ ] Manual test: run hook script, verify event in Iggy
- [ ] `just pre-commit` passes

---

## Exit Criteria

- [ ] `vibes event send` command works with --data and stdin
- [ ] Hook scripts updated to use new CLI
- [ ] HookReceiver code removed
- [ ] Events appear in Iggy topic when sent via CLI
- [ ] All tests pass
