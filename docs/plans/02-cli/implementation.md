# Milestone 1.2: CLI - Implementation Plan

> Step-by-step implementation guide for vibes-cli.

## Prerequisites

- Milestone 1.1 complete (vibes-core with Session, EventBus, PrintModeBackend)
- Nix dev environment working (`direnv allow`)

---

## Phase 1: Crate Setup

### 1.1 Create vibes-cli crate

```bash
mkdir -p vibes-cli/src
```

Create `vibes-cli/Cargo.toml`:
```toml
[package]
name = "vibes-cli"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "CLI for vibes - Claude Code proxy with remote access"

[[bin]]
name = "vibes"
path = "src/main.rs"

[dependencies]
vibes-core = { path = "../vibes-core" }
clap = { version = "4", features = ["derive"] }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
directories = "5"
toml = "0.8"
serde = { workspace = true }
anyhow = "1"

[dev-dependencies]
tempfile = "3"
```

### 1.2 Add to workspace

Update root `Cargo.toml`:
```toml
[workspace]
members = ["vibes-core", "vibes-cli"]
```

### 1.3 Verify setup

```bash
just check
```

---

## Phase 2: CLI Skeleton

### 2.1 Create main.rs with command structure

```rust
// vibes-cli/src/main.rs
use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod server;

#[derive(Parser)]
#[command(name = "vibes", about = "Remote control for Claude Code")]
#[command(version, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Proxy Claude Code with vibes enhancements
    Claude(commands::claude::ClaudeArgs),
    /// Manage configuration
    Config(commands::config::ConfigArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    match cli.command {
        Commands::Claude(args) => commands::claude::run(args).await,
        Commands::Config(args) => commands::config::run(args),
    }
}
```

### 2.2 Create module structure

```
vibes-cli/src/
├── main.rs
├── commands/
│   ├── mod.rs
│   ├── claude.rs
│   └── config.rs
├── config/
│   ├── mod.rs
│   ├── types.rs
│   └── loader.rs
└── server/
    └── mod.rs
```

### 2.3 Stub out modules

Each module starts with minimal implementation that compiles:

```rust
// commands/mod.rs
pub mod claude;
pub mod config;

// commands/claude.rs
use clap::Args;
use anyhow::Result;

#[derive(Args)]
pub struct ClaudeArgs {
    pub prompt: Option<String>,
}

pub async fn run(_args: ClaudeArgs) -> Result<()> {
    todo!("Implement claude command")
}

// commands/config.rs
use clap::{Args, Subcommand};
use anyhow::Result;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Show,
    Path,
}

pub fn run(_args: ConfigArgs) -> Result<()> {
    todo!("Implement config command")
}
```

### 2.4 Verify skeleton compiles

```bash
just check
cargo run -p vibes-cli -- --help
```

---

## Phase 3: Configuration System

### 3.1 Implement config types

Create `config/types.rs` with `VibesConfig`, `ServerConfig`, `SessionConfig` structs as specified in design.

**Tests:**
- Default values are correct
- TOML serialization round-trips
- Partial configs deserialize with defaults

### 3.2 Implement config loader

Create `config/loader.rs` with:
- `ConfigLoader::load()` - merge user + project configs
- `ConfigLoader::user_config_path()` - platform-specific path
- `ConfigLoader::project_config_path()` - `.vibes/config.toml`
- `ConfigLoader::save_user_config()` - for `vibes config set`

**Tests:**
- Loading from non-existent paths returns defaults
- Project config overrides user config
- Invalid TOML returns clear error

### 3.3 Implement config commands

Implement `commands/config.rs`:
- `show` - load and print merged config as TOML
- `path` - print user and project config paths
- `set` - parse key path, update user config, save

**Tests:**
- `vibes config show` outputs valid TOML
- `vibes config path` shows correct platform paths

---

## Phase 4: Claude Command (Core)

### 4.1 Implement ClaudeArgs

Full argument struct with:
- Vibes flags: `--session-name`, `--no-serve`
- Common Claude flags: `-c`, `-r`, `--model`, `--allowedTools`, `--system-prompt`
- Passthrough: `prompt`, trailing args via `#[arg(last = true)]`

**Tests:**
- Parse various flag combinations
- Passthrough args captured correctly

### 4.2 Implement argument-to-config merging

Create function to merge CLI args with loaded config:

```rust
fn build_backend_config(args: &ClaudeArgs, config: &VibesConfig) -> PrintModeConfig {
    PrintModeConfig {
        model: args.model.clone()
            .or(config.session.default_model.clone()),
        allowed_tools: args.allowed_tools.clone()
            .or(config.session.default_allowed_tools.as_ref()
                .map(|v| v.join(","))),
        // ...
    }
}
```

**Tests:**
- CLI flags override config
- Config used when CLI not specified
- Defaults used when neither specified

### 4.3 Implement run function

```rust
pub async fn run(args: ClaudeArgs) -> Result<()> {
    let config = ConfigLoader::load()?;
    let backend_config = build_backend_config(&args, &config);

    // Start server stub if enabled
    if config.server.auto_start && !args.no_serve {
        tokio::spawn(server::start_stub(config.server.port));
    }

    // Build prompt
    let prompt = args.prompt.ok_or_else(|| anyhow!("No prompt provided"))?;

    // Create session infrastructure
    let event_bus = Arc::new(MemoryEventBus::new(1000));
    let factory = Arc::new(PrintModeBackendFactory::new(backend_config));
    let manager = SessionManager::new(factory, event_bus.clone());

    // Create session and send prompt
    let session_id = manager.create_session(args.session_name).await;
    manager.send_input(&session_id, &prompt).await?;

    // Stream to stdout (pass-through, identical to claude)
    stream_output(event_bus, &session_id).await
}
```

### 4.4 Implement output streaming

Stream events to stdout with identical formatting to Claude:

```rust
async fn stream_output(event_bus: Arc<dyn EventBus>, session_id: &str) -> Result<()> {
    let mut rx = event_bus.subscribe();

    while let Ok((_, event)) = rx.recv().await {
        if let VibesEvent::Claude { session_id: sid, event } = event {
            if sid != session_id { continue; }

            match event {
                ClaudeEvent::TextDelta { text } => {
                    print!("{}", text);
                    std::io::Write::flush(&mut std::io::stdout())?;
                }
                ClaudeEvent::TurnComplete { .. } => break,
                ClaudeEvent::Error { message, .. } => {
                    eprintln!("{}", message);
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
```

---

## Phase 5: Server Stub

### 5.1 Implement port binding stub

```rust
// server/mod.rs
use tokio::net::TcpListener;
use tracing::info;

pub async fn start_stub(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Vibes server listening on http://{}", addr);

    // Hold the port but don't accept connections
    // Full implementation in milestone 1.4
    std::future::pending::<()>().await;
    drop(listener);
    Ok(())
}
```

### 5.2 Handle port-in-use gracefully

```rust
pub async fn start_stub(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            info!("Vibes server listening on http://{}", addr);
            std::future::pending::<()>().await;
            drop(listener);
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            info!("Port {} already in use (another vibes instance?)", port);
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
```

---

## Phase 6: PrintModeBackend Updates

### 6.1 Update PrintModeConfig

The existing `PrintModeConfig` may need updates to support all the flags we're passing through. Verify it includes:
- `model: Option<String>`
- `allowed_tools: Option<String>`
- `system_prompt: Option<String>`
- `continue_session: bool`
- `resume_session: Option<String>`

### 6.2 Update command construction

Ensure `PrintModeBackend::spawn_turn()` constructs the full command:
```rust
let mut cmd = Command::new("claude");
cmd.args(["-p", input, "--output-format", "stream-json"]);

if let Some(id) = &self.claude_session_id {
    cmd.args(["--session-id", id]);
}
if let Some(model) = &self.config.model {
    cmd.args(["--model", model]);
}
// ... etc
```

---

## Phase 7: Integration Testing

### 7.1 Manual testing checklist

- [ ] `vibes claude "hello"` produces identical output to `claude -p "hello"`
- [ ] `vibes claude --model claude-opus-4-5 "test"` uses correct model
- [ ] `vibes claude --no-serve "test"` skips server
- [ ] `vibes config show` displays merged config
- [ ] `vibes config set server.port 8080` persists to user config
- [ ] Config file in `.vibes/config.toml` overrides user config

### 7.2 Automated integration tests

Create `tests/integration/e2e.rs` (gated behind `integration` feature):

```rust
#[cfg(feature = "integration")]
#[tokio::test]
async fn test_vibes_claude_hello() {
    let output = Command::new("cargo")
        .args(["run", "-p", "vibes-cli", "--", "claude", "say hello"])
        .output()
        .await?;

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("hello"));
}
```

---

## Phase 8: Documentation & Polish

### 8.1 Update PROGRESS.md

Mark milestone 1.2 items as complete:
- [x] vibes claude pass-through
- [x] --session-name support
- [x] vibes config basics
- [x] Server auto-start (stub)

### 8.2 Update README usage examples

Ensure examples work as documented.

### 8.3 Add --help improvements

Review clap help text for clarity:
```rust
#[arg(long, help = "Human-friendly name for this session (shown in UI)")]
session_name: Option<String>,
```

---

## Verification Checklist

Before marking 1.2 complete:

- [ ] `just pre-commit` passes (fmt, clippy, test)
- [ ] `vibes --help` shows all commands
- [ ] `vibes claude --help` shows all flags
- [ ] `vibes config show` works with no config file
- [ ] `vibes config set` creates user config if missing
- [ ] Manual test: output identical to raw `claude` command
- [ ] Integration tests pass (if Claude CLI available)
