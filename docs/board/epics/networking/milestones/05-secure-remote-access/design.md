# Milestone 2.1: Cloudflare Tunnel Integration - Design Document

> Enable remote access to vibes from anywhere via Cloudflare Tunnel.

## Overview

This milestone adds Cloudflare Tunnel integration to vibes, enabling the core value proposition: start a session from your terminal, control it from your phone anywhere in the world. The tunnel creates a secure outbound connection to Cloudflare's network, exposing the vibes server without opening firewall ports.

### Key Decisions

| Decision | Choice | Notes |
|----------|--------|-------|
| cloudflared installation | User pre-installs | Respects system package managers (brew, apt) |
| Tunnel modes | Quick + Named | Quick for testing, named for production |
| Process management | Embedded in daemon | `vibes serve` manages cloudflared lifecycle |
| CLI interface | Hybrid | Flags on serve + dedicated tunnel subcommands |
| Setup wizard | Hybrid detect+guide | Auto-detect existing config, guide through gaps |
| UI status display | Badge + status page | Header badge for glance, page for details |
| Auto-reconnect | Active supervision | Trust cloudflared for network, supervise process |
| Configuration | Main vibes config | `[tunnel]` section in config.toml |

---

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      vibes serve --tunnel                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    ProcessSupervisor                         ││
│  │  ┌─────────────────────┐    ┌─────────────────────────────┐ ││
│  │  │  VibesServer        │    │  TunnelManager              │ ││
│  │  │  (HTTP/WS :7432)    │    │  (spawns cloudflared)       │ ││
│  │  └─────────────────────┘    └─────────────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│         Events: TunnelConnected, TunnelDisconnected, etc.       │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                        EventBus                              ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                               │
            ┌──────────────────┼──────────────────┐
            ▼                  ▼                  ▼
     ┌────────────┐     ┌────────────┐     ┌────────────┐
     │ CLI client │     │  Web UI    │     │ Cloudflare │
     │            │     │            │     │  Network   │
     └────────────┘     └────────────┘     └────────────┘
```

### Key Components

| Component | Location | Responsibility |
|-----------|----------|----------------|
| TunnelManager | vibes-core/src/tunnel/ | Spawn/supervise cloudflared, emit events |
| TunnelConfig | vibes-core/src/config/ | Parse `[tunnel]` section from config |
| TunnelState | vibes-core/src/tunnel/ | Track connection status, URL, errors |
| TunnelWizard | vibes-cli/src/commands/ | Interactive setup for named tunnels |

### Crate Changes

```
vibes/
├── vibes-core/
│   └── src/
│       ├── tunnel/              # NEW MODULE
│       │   ├── mod.rs           # TunnelManager, TunnelConfig
│       │   ├── manager.rs       # Process supervision, stdout parsing
│       │   ├── config.rs        # TunnelConfig, TunnelMode
│       │   ├── state.rs         # TunnelState, TunnelEvent
│       │   └── cloudflared.rs   # cloudflared CLI wrapper
│       ├── events/
│       │   └── mod.rs           # Add TunnelEvent variants
│       └── lib.rs               # Export tunnel module
│
├── vibes-cli/
│   └── src/
│       └── commands/
│           ├── serve.rs         # Add --tunnel, --quick-tunnel flags
│           └── tunnel.rs        # NEW: tunnel subcommands
│
├── vibes-server/
│   └── src/
│       ├── http/
│       │   └── api.rs           # Add GET /api/tunnel/status
│       └── ws/
│           └── protocol.rs      # Add tunnel_state message type
│
└── web-ui/
    └── src/
        ├── components/
        │   ├── TunnelBadge.tsx  # NEW: header status badge
        │   └── TunnelStatus.tsx # NEW: detailed status component
        └── routes/
            └── status.tsx       # NEW: /status page with tunnel info
```

---

## CLI Interface

### Serve Command Extensions

```bash
# Start server with tunnel
vibes serve --tunnel              # Use configured named tunnel
vibes serve --quick-tunnel        # Start quick tunnel (temp URL)
vibes serve --tunnel --port 8080  # Custom local port

# Tunnel is disabled by default
vibes serve                       # No tunnel, local only
```

### Tunnel Subcommands

```bash
vibes tunnel setup               # Interactive wizard for named tunnel
vibes tunnel start               # Start tunnel (if not using --tunnel flag)
vibes tunnel stop                # Stop tunnel, keep server running
vibes tunnel status              # Show tunnel state, URL, metrics
vibes tunnel quick               # Start quick tunnel standalone
```

### Example Flows

**Quick start (zero config):**
```bash
$ vibes serve --quick-tunnel
Starting vibes server on http://localhost:7432
Starting quick tunnel...
✓ Tunnel ready: https://random-words-here.trycloudflare.com
```

**Production setup:**
```bash
$ vibes tunnel setup
# ... interactive wizard ...
$ vibes serve --tunnel
Starting vibes server on http://localhost:7432
Starting tunnel 'vibes-home'...
✓ Tunnel ready: https://vibes.example.com
```

---

## Setup Wizard

The `vibes tunnel setup` command guides users through named tunnel configuration.

### Flow

```
$ vibes tunnel setup

╭─ Cloudflare Tunnel Setup ─────────────────────────────────────╮
│                                                                │
│  This wizard will configure a named Cloudflare Tunnel for     │
│  persistent remote access to vibes.                           │
│                                                                │
╰────────────────────────────────────────────────────────────────╯

Checking cloudflared... ✓ v2024.12.0 installed

Checking existing tunnels...
  Found 2 tunnels in ~/.cloudflared/

Select a tunnel:
  › vibes-home (created 2024-11-15)
    vibes-work (created 2024-12-01)
    [Create new tunnel]

Using tunnel: vibes-home

Checking DNS routes...
  ✗ No route configured for this tunnel

What hostname should vibes use?
  Domain must be in your Cloudflare account
  › vibes.example.com

To create the DNS route, run:
┌────────────────────────────────────────────────────────────────┐
│ cloudflared tunnel route dns vibes-home vibes.example.com     │
└────────────────────────────────────────────────────────────────┘

Press Enter when done, or Ctrl+C to cancel...

Verifying DNS route... ✓ Route configured

Configuration saved!

  Tunnel:   vibes-home
  Hostname: vibes.example.com
  Config:   ~/.config/vibes/config.toml

To start vibes with tunnel:
  vibes serve --tunnel
```

### Wizard Steps

1. **Check cloudflared**: Verify installation, show version
2. **List tunnels**: Parse `cloudflared tunnel list` output
3. **Select/create tunnel**: User picks existing or creates new
4. **Check DNS route**: Verify hostname is routed to tunnel
5. **Guide DNS setup**: If missing, show command to run
6. **Verify route**: Confirm DNS is configured
7. **Save config**: Write to vibes config.toml

### Error Handling

| Situation | Wizard Response |
|-----------|-----------------|
| cloudflared not installed | Show install instructions for OS |
| No tunnels exist | Guide through `cloudflared tunnel create` |
| No Cloudflare auth | Guide through `cloudflared tunnel login` |
| DNS route exists for different tunnel | Warn and confirm override |

---

## TunnelManager

Manages the cloudflared subprocess lifecycle.

### Interface

```rust
pub struct TunnelManager {
    config: TunnelConfig,
    process: Option<Child>,
    state: Arc<RwLock<TunnelState>>,
    event_tx: broadcast::Sender<TunnelEvent>,
}

impl TunnelManager {
    pub fn new(config: TunnelConfig) -> Self;
    pub async fn start(&mut self) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
    pub fn state(&self) -> TunnelState;
    pub fn subscribe(&self) -> broadcast::Receiver<TunnelEvent>;
}
```

### TunnelConfig

```rust
pub struct TunnelConfig {
    pub enabled: bool,
    pub mode: TunnelMode,
}

pub enum TunnelMode {
    Quick,
    Named {
        name: String,
        hostname: String,
        credentials_path: Option<PathBuf>,
    },
}
```

### TunnelState

```rust
pub enum TunnelState {
    Disabled,
    Starting,
    Connected { url: String, connected_at: DateTime<Utc> },
    Reconnecting { attempt: u32, last_error: String },
    Failed { error: String, can_retry: bool },
    Stopped,
}
```

### TunnelEvent

```rust
pub enum TunnelEvent {
    Starting,
    Connected { url: String },
    Disconnected { reason: String },
    Reconnecting { attempt: u32 },
    Failed { error: String },
    Stopped,
    Log { level: LogLevel, message: String },
}
```

---

## Process Supervision

### Spawn Strategy

**Quick tunnel:**
```bash
cloudflared tunnel --url http://localhost:7432
```

**Named tunnel:**
```bash
cloudflared tunnel --config ~/.cloudflared/config.yml run <tunnel-name>
# Or without config file:
cloudflared tunnel run --url http://localhost:7432 <tunnel-name>
```

### Output Parsing

cloudflared logs to stderr with structured output. Key patterns:

```
# Quick tunnel URL (parse this!)
INF +--------------------------------------------------------+
INF | Your quick Tunnel has been created! Visit it at:       |
INF | https://random-words-here.trycloudflare.com            |
INF +--------------------------------------------------------+

# Connection established
INF Connection registered connIndex=0 connection=<uuid>

# Connection lost
ERR Unregistered tunnel connection connIndex=0

# Retrying
INF Retrying connection in 1s connIndex=0
```

### Restart Backoff

```rust
struct RestartPolicy {
    attempts: Vec<Instant>,
    max_attempts_per_window: u32,  // 5
    window_duration: Duration,      // 60 seconds
    backoff: ExponentialBackoff,
}

impl RestartPolicy {
    fn should_restart(&mut self) -> Option<Duration> {
        // Clean old attempts outside window
        self.attempts.retain(|t| t.elapsed() < self.window_duration);

        if self.attempts.len() >= self.max_attempts_per_window {
            return None; // Give up
        }

        let delay = self.backoff.next_delay();
        self.attempts.push(Instant::now());
        Some(delay)
    }
}
```

Backoff schedule:
- Attempt 1: immediate
- Attempt 2: 1 second
- Attempt 3: 5 seconds
- Attempt 4: 15 seconds
- Attempt 5: 30 seconds
- After 5 failures in 60 seconds: stop retrying, emit Failed event

---

## Configuration

### Config Schema

```toml
# ~/.config/vibes/config.toml

[server]
port = 7432
host = "0.0.0.0"

[tunnel]
enabled = false                    # Auto-start with serve
mode = "named"                     # "quick" | "named"

# Named tunnel settings (ignored if mode = "quick")
name = "vibes-home"
hostname = "vibes.example.com"
# credentials_path = "~/.cloudflared/abc123.json"  # Auto-detected
```

### Config Types

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct TunnelConfigFile {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_mode")]
    pub mode: String,  // "quick" | "named"

    pub name: Option<String>,
    pub hostname: Option<String>,
    pub credentials_path: Option<PathBuf>,
}
```

---

## HTTP API

### New Endpoints

```
GET /api/tunnel/status    # Tunnel state and info
```

### Response Schema

```typescript
// GET /api/tunnel/status
{
  "state": "connected",  // "disabled" | "starting" | "connected" | "reconnecting" | "failed" | "stopped"
  "mode": "named",       // "quick" | "named" | null
  "url": "https://vibes.example.com",
  "tunnel_name": "vibes-home",
  "connected_at": "2025-12-26T10:00:00Z",
  "uptime_seconds": 3600,
  "error": null          // Error message if failed/reconnecting
}
```

### WebSocket Messages

```typescript
// Server → Client
{
  "type": "tunnel_state",
  "state": "connected",
  "url": "https://vibes.example.com"
}
```

---

## Web UI

### Header Badge Component

```tsx
// TunnelBadge.tsx
function TunnelBadge() {
  const { state, url } = useTunnelStatus();

  const badge = {
    disabled: { color: 'gray', icon: '○', tooltip: 'No tunnel' },
    starting: { color: 'yellow', icon: '◐', tooltip: 'Connecting...' },
    connected: { color: 'green', icon: '●', tooltip: url },
    reconnecting: { color: 'yellow', icon: '◐', tooltip: 'Reconnecting...' },
    failed: { color: 'red', icon: '●', tooltip: 'Connection failed' },
    stopped: { color: 'gray', icon: '○', tooltip: 'Tunnel stopped' },
  }[state];

  return (
    <Link to="/status">
      <span style={{ color: badge.color }} title={badge.tooltip}>
        {badge.icon}
      </span>
    </Link>
  );
}
```

### Status Page

```tsx
// routes/status.tsx
function StatusPage() {
  const tunnel = useTunnelStatus();
  const server = useServerStatus();

  return (
    <div>
      <h1>Status</h1>

      <section>
        <h2>Server</h2>
        <dl>
          <dt>Port</dt><dd>{server.port}</dd>
          <dt>Uptime</dt><dd>{formatDuration(server.uptime)}</dd>
          <dt>Clients</dt><dd>{server.connected_clients}</dd>
        </dl>
      </section>

      <section>
        <h2>Tunnel</h2>
        <TunnelStatusCard tunnel={tunnel} />
      </section>
    </div>
  );
}
```

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage |
|-----------|---------------|
| TunnelConfig | Parse valid/invalid TOML, defaults |
| TunnelState | State transitions, serialization |
| Output parser | Quick tunnel URL extraction, log parsing |
| RestartPolicy | Backoff timing, give-up threshold |

### Integration Tests

| Test | Description |
|------|-------------|
| Quick tunnel lifecycle | Start, parse URL, stop |
| Named tunnel lifecycle | Start with config, verify connection |
| Supervision | Kill cloudflared, verify restart |
| API endpoint | GET /api/tunnel/status responses |

### Manual Testing

- [ ] Quick tunnel works on fresh machine
- [ ] Named tunnel with existing cloudflared setup
- [ ] Setup wizard completes successfully
- [ ] UI badge updates in real-time
- [ ] Reconnection after network drop
- [ ] Clean shutdown on vibes serve stop

---

## Dependencies

### vibes-core/Cargo.toml additions

```toml
# For process supervision
tokio-stream = "0.1"
```

### web-ui/package.json additions

None required - using existing React Query for state.

---

## Deliverables

### New Files

| File | Description |
|------|-------------|
| vibes-core/src/tunnel/mod.rs | Module exports |
| vibes-core/src/tunnel/manager.rs | TunnelManager implementation |
| vibes-core/src/tunnel/config.rs | TunnelConfig types |
| vibes-core/src/tunnel/state.rs | TunnelState, TunnelEvent |
| vibes-core/src/tunnel/cloudflared.rs | CLI wrapper, output parsing |
| vibes-cli/src/commands/tunnel.rs | Tunnel subcommands |
| web-ui/src/components/TunnelBadge.tsx | Header badge |
| web-ui/src/components/TunnelStatus.tsx | Status card |
| web-ui/src/routes/status.tsx | Status page |

### Modified Files

| File | Changes |
|------|---------|
| vibes-core/src/lib.rs | Export tunnel module |
| vibes-core/src/events/mod.rs | Add TunnelEvent to VibesEvent |
| vibes-core/src/config/mod.rs | Add TunnelConfigFile parsing |
| vibes-cli/src/main.rs | Add tunnel subcommand |
| vibes-cli/src/commands/serve.rs | Add --tunnel, --quick-tunnel flags |
| vibes-server/src/http/api.rs | Add /api/tunnel/status |
| vibes-server/src/ws/protocol.rs | Add tunnel_state message |
| vibes-server/src/state.rs | Include TunnelManager in AppState |
| web-ui/src/components/Header.tsx | Add TunnelBadge |
| web-ui/src/App.tsx | Add /status route |

### Documentation Updates

| Document | Changes |
|----------|---------|
| docs/VISION.md | Mark milestone 2.1 decisions |
| docs/PROGRESS.md | Update milestone 2.1 checkboxes |
| README.md | Add tunnel setup section |

---

## Milestone 2.1 Checklist

- [ ] TunnelManager with process supervision
- [ ] Quick tunnel support (cloudflared tunnel --url)
- [ ] Named tunnel support (cloudflared tunnel run)
- [ ] vibes tunnel setup wizard
- [ ] vibes tunnel start/stop/status commands
- [ ] vibes serve --tunnel and --quick-tunnel flags
- [ ] Auto-reconnect with exponential backoff
- [ ] GET /api/tunnel/status endpoint
- [ ] WebSocket tunnel_state messages
- [ ] TunnelBadge header component
- [ ] /status page with tunnel info
- [ ] Configuration in [tunnel] section
- [ ] Integration tests
