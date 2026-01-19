---
id: FEAT0173
title: CLI command and WebSocket integration
type: feat
status: done
priority: high
scope: tui/41-terminal-ui-framework
depends: [m41-feat-04]
estimate: 4h
---

# CLI Command and WebSocket Integration

## Summary

Wire the TUI into the CLI and connect to the vibes server. This is the integration story that makes the TUI usable as the primary interface.

## Features

### CLI Command

Add `vibes tui` subcommand:

```rust
#[derive(clap::Args)]
pub struct TuiArgs {
    /// Use specific theme
    #[arg(long)]
    theme: Option<String>,

    /// Start in session view
    #[arg(long)]
    session: Option<String>,

    /// Start in agent view
    #[arg(long)]
    agent: Option<String>,
}
```

### Default Command Behavior

Change CLI so `vibes` with no arguments launches TUI:

```rust
// In main.rs
if is_no_args() {
    // Launch TUI instead of showing help
    return commands::tui::run(TuiArgs::default()).await;
}
```

Update `is_top_level_help()` to only trigger on explicit `--help` or `-h`.

### Auto-Start Daemon

Reuse daemon auto-start logic from `vibes claude`:

```rust
pub async fn run(args: TuiArgs) -> Result<()> {
    // Auto-start daemon if needed
    let daemon = daemon::ensure_running().await?;

    // Connect to server
    let client = VibesClient::connect(&daemon.address).await?;

    // Create app with initial view based on args
    let initial_view = match (&args.session, &args.agent) {
        (Some(id), _) => View::Session(id.parse()?),
        (_, Some(id)) => View::Agent(id.parse()?),
        _ => View::Dashboard,
    };

    let mut app = App::new(client, initial_view);
    app.run().await
}
```

### Connection Error Handling

```rust
impl App {
    async fn handle_connection_error(&mut self, error: &Error) {
        self.state.mode = Mode::Normal;
        self.state.error_message = Some(format!(
            "Connection lost: {}. Press 'r' to retry or 'q' to quit.",
            error
        ));
    }
}
```

### WebSocket Client Integration

Adapt `VibesClient` from `vibes-cli/src/client/`:

```rust
pub struct TuiClient {
    inner: VibesClient,
    rx: mpsc::Receiver<ServerMessage>,
}

impl TuiClient {
    pub async fn connect(addr: &str) -> Result<Self>;
    pub async fn send(&self, msg: ClientMessage) -> Result<()>;
    pub fn try_recv(&mut self) -> Option<ServerMessage>;
}
```

## Implementation

1. Create `vibes-cli/src/commands/tui.rs` with TuiArgs
2. Add `Tui` variant to Commands enum in main.rs
3. Update default behavior (no args → TUI)
4. Add vibes-tui dependency to vibes-cli
5. Implement daemon auto-start (reuse from claude.rs)
6. Create TuiClient wrapper for async message handling
7. Wire client into App tick() for processing server messages
8. Handle connection errors gracefully
9. Test `--session` and `--agent` flags

## Acceptance Criteria

- [x] `vibes tui` launches the TUI
- [x] `vibes` (no args) launches the TUI
- [x] `vibes --help` still shows help
- [x] `vibes tui --help` shows TUI options
- [x] Auto-starts daemon if not running
- [x] Waits for daemon to be ready before connecting
- [x] Connection errors display clearly (not a crash)
- [x] `--session <id>` opens Session view
- [x] `--agent <id>` opens Agent view
- [x] Press 'r' to retry connection on error
