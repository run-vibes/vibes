---
id: FEAT0165
title: Instrument key functions with spans
type: feat
status: done
priority: high
epics: [observability]
depends: [m40-feat-03]
estimate: 4h
milestone: 40
---

# Instrument key functions with spans

## Summary

Add `#[instrument]` attributes to key functions in vibes-core and vibes-server. This enables automatic span creation for request tracing.

## Features

### Target Functions

Instrument these key areas:

**vibes-server (HTTP/WebSocket):**
```rust
#[instrument(skip(state), fields(session_id))]
async fn handle_ws_message(state: &AppState, msg: WsMessage) -> Result<()> {
    // ...
}

#[instrument(skip(state))]
async fn create_session(state: &AppState, req: CreateSessionRequest) -> Result<Session> {
    // ...
}
```

**vibes-core (Sessions):**
```rust
#[instrument(skip(self), fields(session_id = %self.id))]
pub async fn process_event(&mut self, event: Event) -> Result<()> {
    // ...
}

#[instrument(skip(self))]
pub async fn save(&self) -> Result<()> {
    // ...
}
```

**vibes-core (Agents):**
```rust
#[instrument(skip(self, task), fields(agent_id = %self.id, task_id = %task.id))]
async fn run(&mut self, task: Task) -> Result<TaskResult> {
    // ...
}
```

### Span Naming Convention

- Format: `<module>::<operation>`
- Examples:
  - `server::handle_ws_message`
  - `session::process_event`
  - `agent::run_task`
  - `model::inference`

### Error Recording

```rust
#[instrument(err)]
async fn fallible_operation() -> Result<()> {
    // Errors automatically recorded on span
}
```

## Implementation

1. Add `tracing` dependency where needed
2. Instrument vibes-server handlers
3. Instrument vibes-core session methods
4. Instrument vibes-core agent methods
5. Follow naming conventions
6. Verify spans appear in output

## Acceptance Criteria

- [x] Server handlers instrumented
- [x] Session methods instrumented
- [x] Agent lifecycle methods instrumented
- [x] Span names follow conventions
- [x] Errors recorded on spans
- [x] No performance regression (benchmark)
