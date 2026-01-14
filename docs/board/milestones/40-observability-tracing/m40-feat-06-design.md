# vibes observe CLI Design

## Overview

Add CLI commands for viewing and managing traces. Users can tail traces in real-time, filter by session/agent/level, and configure export targets.

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                    vibes-server                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │  tracing-subscriber Registry                     │    │
│  │  └─ TraceBroadcaster Layer                       │    │
│  │     (captures spans, broadcasts to subscribers)  │    │
│  └─────────────────────────────────────────────────┘    │
│                         │                                │
│                         ▼                                │
│  ┌─────────────────────────────────────────────────┐    │
│  │  broadcast::Sender<TraceEvent>                   │    │
│  │  (stored in AppState)                            │    │
│  └─────────────────────────────────────────────────┘    │
│                         │                                │
│            ┌────────────┴────────────┐                  │
│            ▼                         ▼                  │
│  ┌──────────────────┐     ┌──────────────────┐         │
│  │ CLI Client 1     │     │ CLI Client 2     │         │
│  │ (session filter) │     │ (agent filter)   │         │
│  └──────────────────┘     └──────────────────┘         │
└─────────────────────────────────────────────────────────┘
                    │                    │
                    ▼                    ▼
              ┌──────────┐        ┌──────────┐
              │ vibes    │        │ vibes    │
              │ observe  │        │ observe  │
              │ traces   │        │ traces   │
              └──────────┘        └──────────┘
```

## Message Types

### Client → Server

```rust
// Add to ClientMessage enum
SubscribeTraces {
    session_id: Option<String>,
    agent_id: Option<String>,
    level: Option<String>,  // trace, debug, info, warn, error
},
UnsubscribeTraces,
```

### Server → Client

```rust
// Add to ServerMessage enum
TraceEvent {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    name: String,
    level: String,
    timestamp: DateTime<Utc>,
    duration_ms: Option<f64>,
    session_id: Option<String>,
    agent_id: Option<String>,
    attributes: HashMap<String, String>,
    status: SpanStatus,
},
TraceSubscribed,
```

## Server-Side Implementation

### TraceBroadcaster

New component in vibes-observe that implements `tracing_subscriber::Layer`:

```rust
// vibes-observe/src/subscriber.rs
pub struct TraceBroadcaster {
    tx: broadcast::Sender<TraceEvent>,
}

impl TraceBroadcaster {
    pub fn new(capacity: usize) -> Self;
    pub fn subscribe(&self) -> broadcast::Receiver<TraceEvent>;
}

impl<S: Subscriber> Layer<S> for TraceBroadcaster {
    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        // Convert span to TraceEvent
        // Broadcast to all subscribers
    }
}
```

### Integration

1. Create `TraceBroadcaster` during server startup
2. Add to tracing subscriber registry as a layer
3. Store `Arc<TraceBroadcaster>` in `AppState`
4. On `SubscribeTraces`, spawn task that filters and forwards events

## CLI Implementation

### Command Structure

```rust
// vibes-cli/src/commands/observe.rs
#[derive(Args)]
pub struct ObserveArgs {
    #[command(subcommand)]
    pub command: ObserveCommands,
}

#[derive(Subcommand)]
pub enum ObserveCommands {
    Traces {
        #[arg(value_name = "SESSION")]
        session: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long, default_value = "info")]
        level: String,
    },
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Set { key: String, value: String },
}
```

### Output Formatting

Tree structure with color coding:

```
Trace: 019abc12... | Session: 019def34...
└─ server::handle_ws_message (2.3ms)
   └─ session::process_event (1.8ms)
      ├─ model::inference (1.2ms) tokens=450
      └─ tool::execute (0.4ms) tool=read_file
```

Colors:
- Green: successful spans
- Red: error spans
- Yellow: warning level
- Dim: duration and attributes

## Config Command

Shows and updates observe configuration in `~/.config/vibes/config.toml`:

```toml
[observe]
enabled = true
sample_rate = 1.0

[[observe.exporters]]
type = "console"
format = "pretty"
```

Changes require server restart (no hot-reload complexity).

## Files to Create/Modify

### New Files
- `vibes-observe/src/subscriber.rs` - TraceBroadcaster layer
- `vibes-cli/src/commands/observe.rs` - CLI command
- `vibes-cli/src/commands/observe/format.rs` - Output formatting

### Modified Files
- `vibes-observe/src/lib.rs` - Export TraceBroadcaster
- `vibes-server/src/messages.rs` - Add message types
- `vibes-server/src/websocket.rs` - Handle trace subscription
- `vibes-server/src/state.rs` - Add TraceBroadcaster to AppState
- `vibes-cli/src/main.rs` - Add observe command
- `vibes-cli/src/commands/mod.rs` - Export observe module
- `vibes-core/src/config.rs` - Add observe config section

## Acceptance Criteria

- [ ] `vibes observe traces` shows live traces
- [ ] Session filter works correctly
- [ ] Agent filter works correctly
- [ ] Level filter works correctly
- [ ] `vibes observe config` shows settings
- [ ] Config updates persist
