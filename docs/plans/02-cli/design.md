# Milestone 1.2: CLI - Design Document

> vibes-cli binary with `vibes claude` pass-through, configuration system, and server stub.

## Overview

This milestone builds the user-facing CLI on top of vibes-core. Users can run `vibes claude` as a drop-in replacement for `claude` with vibes enhancements (session naming, server background, config-based defaults).

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| CLI framework | clap (derive) | Best-in-class for Rust CLIs, derive macros reduce boilerplate |
| Flag handling | Hybrid | Common Claude flags explicit (UX), passthrough for flexibility |
| Config format | TOML | Human-readable, Rust ecosystem standard |
| Config paths | directories crate | Cross-platform XDG/AppData support |
| Server in 1.2 | Stub only | Port binding, logging, no endpoints until 1.4 |

---

## Crate Structure

```
vibes/
├── Cargo.toml                    # Workspace root
├── vibes-core/                   # (existing) Core library
├── vibes-cli/                    # (new) CLI binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # Entry point, command dispatch
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── claude.rs         # vibes claude subcommand
│       │   └── config.rs         # vibes config subcommand
│       ├── config/
│       │   ├── mod.rs
│       │   ├── types.rs          # Config structs
│       │   └── loader.rs         # Load/save config files
│       └── server/
│           └── mod.rs            # Stub: port binding placeholder
```

### Dependencies

```toml
# vibes-cli/Cargo.toml
[dependencies]
vibes-core = { path = "../vibes-core" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
directories = "5"
toml = "0.8"
serde = { version = "1", features = ["derive"] }
anyhow = "1"
```

---

## CLI Structure

### Top-Level Commands

```
vibes [OPTIONS] <COMMAND>

Commands:
  claude    Proxy Claude Code with vibes enhancements
  config    Manage configuration

Global Options:
  -v, --verbose    Verbose output
  -h, --help       Print help
  -V, --version    Print version
```

### Command Definitions

```rust
// main.rs
use clap::{Parser, Subcommand};

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
    Claude(ClaudeArgs),
    /// Manage configuration
    Config(ConfigArgs),
}
```

### vibes claude (Hybrid Flag Approach)

```rust
// commands/claude.rs
use clap::Args;

#[derive(Args)]
pub struct ClaudeArgs {
    // === Vibes-specific flags ===

    /// Human-friendly session name
    #[arg(long)]
    pub session_name: Option<String>,

    /// Disable background server for this session
    #[arg(long)]
    pub no_serve: bool,

    // === Common Claude flags (explicit for UX) ===

    /// Continue most recent session
    #[arg(short = 'c', long)]
    pub continue_session: bool,

    /// Resume specific session by ID
    #[arg(short = 'r', long)]
    pub resume: Option<String>,

    /// Model to use (e.g., claude-sonnet-4-20250514)
    #[arg(long)]
    pub model: Option<String>,

    /// Tools to allow without prompting (comma-separated)
    #[arg(long = "allowedTools")]
    pub allowed_tools: Option<String>,

    /// System prompt to use
    #[arg(long = "system-prompt")]
    pub system_prompt: Option<String>,

    // === Passthrough ===

    /// The prompt to send to Claude
    #[arg(value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// Additional arguments passed directly to claude
    #[arg(last = true)]
    pub passthrough: Vec<String>,
}
```

**Flag precedence:** CLI flags > project config > user config > defaults

### vibes config

```rust
// commands/config.rs
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration (merged)
    Show,
    /// Set a configuration value
    Set {
        /// Config key (e.g., server.port, session.default_model)
        key: String,
        /// Value to set
        value: String,
    },
    /// Show configuration file paths
    Path,
}
```

---

## Configuration System

### Config File Locations

Using `directories` crate for cross-platform paths:

| Platform | User Config | Project Config |
|----------|-------------|----------------|
| Linux | `~/.config/vibes/config.toml` | `./.vibes/config.toml` |
| macOS | `~/Library/Application Support/vibes/config.toml` | `./.vibes/config.toml` |
| Windows | `%APPDATA%\vibes\config.toml` | `.\.vibes\config.toml` |

**Merge order:** defaults < user config < project config < CLI flags

### Config Types

```rust
// config/types.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VibesConfig {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Port for the vibes server
    #[serde(default = "default_port")]
    pub port: u16,

    /// Auto-start server with vibes claude
    #[serde(default = "default_true")]
    pub auto_start: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            auto_start: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionConfig {
    /// Default model for new sessions
    pub default_model: Option<String>,

    /// Default allowed tools (comma-separated or array)
    pub default_allowed_tools: Option<Vec<String>>,

    /// Default working directory
    pub working_dir: Option<PathBuf>,
}

fn default_port() -> u16 { 7432 }
fn default_true() -> bool { true }
```

### Example Config File

```toml
# ~/.config/vibes/config.toml

[server]
port = 7432
auto_start = true

[session]
default_model = "claude-sonnet-4-20250514"
default_allowed_tools = ["Read", "Glob", "Grep", "Bash"]
```

### Config Loader

```rust
// config/loader.rs
use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

pub struct ConfigLoader;

impl ConfigLoader {
    /// Load merged configuration (user + project)
    pub fn load() -> Result<VibesConfig> {
        let mut config = VibesConfig::default();

        // Layer 1: User config
        if let Some(user_path) = Self::user_config_path() {
            if user_path.exists() {
                let user_config: VibesConfig =
                    toml::from_str(&std::fs::read_to_string(&user_path)?)?;
                config = Self::merge(config, user_config);
            }
        }

        // Layer 2: Project config
        let project_path = PathBuf::from(".vibes/config.toml");
        if project_path.exists() {
            let project_config: VibesConfig =
                toml::from_str(&std::fs::read_to_string(&project_path)?)?;
            config = Self::merge(config, project_config);
        }

        Ok(config)
    }

    /// Get user config path
    pub fn user_config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "vibes")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Get project config path
    pub fn project_config_path() -> PathBuf {
        PathBuf::from(".vibes/config.toml")
    }

    /// Merge two configs (later values override earlier)
    fn merge(base: VibesConfig, overlay: VibesConfig) -> VibesConfig {
        // Implement field-by-field merge with Option handling
        // ...
    }
}
```

---

## Command Execution Flow

### vibes claude Execution

```
┌─────────────────────────────────────────────────────────────────┐
│                   vibes claude "fix the bug"                     │
├─────────────────────────────────────────────────────────────────┤
│  1. Parse CLI arguments (ClaudeArgs)                            │
│  2. Load config (user + project merged)                         │
│  3. Merge CLI flags with config defaults                        │
│  4. Start server stub (if auto_start && !no_serve)              │
│  5. Create EventBus + SessionManager                            │
│  6. Create Session with name (if provided)                      │
│  7. Send prompt to session                                      │
│  8. Stream ClaudeEvents to terminal                             │
│  9. Wait for completion or Ctrl+C                               │
└─────────────────────────────────────────────────────────────────┘
```

### Backend Command Construction

```
┌─────────────────────────────────────────────────────────────────┐
│                    PrintModeBackend                              │
├─────────────────────────────────────────────────────────────────┤
│  Constructs: claude -p "fix the bug"                            │
│              --output-format stream-json                        │
│              --session-id <uuid>                                │
│              --model claude-sonnet-4-20250514    ← from config  │
│              --allowedTools Read,Glob,Grep       ← from config  │
│              --system-prompt "..."               ← if provided  │
│              <passthrough args>                  ← verbatim     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Claude Code                                │
│              (spawned as subprocess, stdout captured)           │
└─────────────────────────────────────────────────────────────────┘
```

### Event Streaming to Terminal

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ PrintMode    │     │  EventBus    │     │   Terminal   │
│ Backend      │────▶│ (broadcast)  │────▶│   Output     │
└──────────────┘     └──────────────┘     └──────────────┘
       │                    │                     │
       │  ClaudeEvent::     │  VibesEvent::       │
       │  TextDelta         │  Claude { event }   │  Print text
       │  ToolUseStart      │                     │  Show tool use
       │  TurnComplete      │                     │  Show stats
```

```rust
// Simplified terminal streaming
async fn stream_to_terminal(
    event_bus: Arc<dyn EventBus>,
    session_id: &str,
) -> Result<()> {
    let mut rx = event_bus.subscribe();

    while let Ok((_, event)) = rx.recv().await {
        match event {
            VibesEvent::Claude { event: ClaudeEvent::TextDelta { text }, .. } => {
                print!("{}", text);
                std::io::stdout().flush()?;
            }
            VibesEvent::Claude { event: ClaudeEvent::ToolUseStart { name, .. }, .. } => {
                eprintln!("\n[Using tool: {}]", name);
            }
            VibesEvent::Claude { event: ClaudeEvent::TurnComplete { usage }, .. } => {
                eprintln!("\n[Done: {} in, {} out tokens]",
                    usage.input_tokens, usage.output_tokens);
                break;
            }
            VibesEvent::Claude { event: ClaudeEvent::Error { message, .. }, .. } => {
                eprintln!("\n[Error: {}]", message);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
```

---

## Server Stub

For milestone 1.2, the server is a placeholder that:
- Binds to the configured port
- Logs that it's listening
- Accepts no connections (TcpListener but no accept loop)

This establishes the infrastructure for milestone 1.4.

```rust
// server/mod.rs
use tokio::net::TcpListener;
use tracing::info;

pub async fn start_stub(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Server stub listening on http://{}", addr);
    info!("(Full server implementation in milestone 1.4)");

    // Keep the listener alive but don't accept connections
    // This reserves the port and signals to users that the server "started"
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
```

---

## Error Handling

The CLI uses `anyhow` for error handling (vs `thiserror` in vibes-core):

```rust
// main.rs
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    match cli.command {
        Commands::Claude(args) => {
            commands::claude::run(args)
                .await
                .context("Failed to run claude command")?;
        }
        Commands::Config(args) => {
            commands::config::run(args)
                .context("Failed to run config command")?;
        }
    }

    Ok(())
}
```

---

## Testing Strategy

### Unit Tests

- Config parsing and merging
- CLI argument parsing
- Flag-to-command translation

### Integration Tests (requires claude CLI)

- End-to-end `vibes claude "hello"` execution
- Config file loading from actual paths
- Server stub port binding

### Test Organization

```
vibes-cli/
├── src/
└── tests/
    ├── cli_parsing.rs      # Argument parsing tests
    ├── config_loading.rs   # Config merge tests
    └── integration/
        └── e2e.rs          # Full flow tests (gated)
```

---

## Design Clarifications

### Terminal Output
Output from `vibes claude` should be **identical to `claude`**. No vibes branding, colors, or prefixes. The proxy should be invisible to the user experience — vibes enhancements happen behind the scenes (server, event bus, config defaults).

### Ctrl+C Handling
Signal handling mirrors Claude Code behavior. Ctrl+C propagates to the Claude subprocess naturally. No special interception or graceful shutdown logic — vibes gets out of the way.

### Session ID Strategy
Vibes uses Claude's `--session-id` directly with no separate mapping layer:
- Generate a UUID for new sessions
- Pass it to Claude via `--session-id`
- The `--session-name` flag is purely for vibes UI/logging, not stored persistently in 1.2

This keeps the implementation simple and avoids state synchronization issues.

---

## Public Interface Summary

After milestone 1.2, users can:

```bash
# Basic usage (like claude)
vibes claude "query"

# With vibes enhancements
vibes claude --session-name "feature-work" "add tests"
vibes claude --no-serve "quick question"

# Common claude flags work
vibes claude -c                            # Continue
vibes claude -r abc123                     # Resume
vibes claude --model claude-opus-4-5       # Model
vibes claude --allowedTools "Bash,Read"    # Tools

# Passthrough anything else
vibes claude "query" -- --some-future-flag

# Configuration
vibes config show
vibes config set server.port 8080
vibes config set session.default_model claude-opus-4-5
vibes config path
```
