# vibes

[![CI](https://github.com/run-vibes/vibes/actions/workflows/ci.yml/badge.svg)](https://github.com/run-vibes/vibes/actions/workflows/ci.yml)
[![Progress](https://img.shields.io/badge/progress-11%2F25%20milestones-blue)](docs/PROGRESS.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Remote control for your Claude Code sessions.

**vibes** wraps Claude Code with remote access, session management, and a plugin ecosystem — control your AI coding sessions from anywhere.

## Features

- **Remote Access** - Control Claude Code sessions from your phone, tablet, or any device via web UI
- **Session Mirroring** - Real-time sync between your terminal and remote devices
- **Plugin System** - Extend vibes with native Rust plugins for custom commands and workflows
- **Cross-Platform** - Single binary for Linux, macOS, and Windows

## Usage

```bash
# Use like claude, but with superpowers
vibes claude "refactor the auth module"

# All claude flags work
vibes claude -c                          # Continue last session
vibes claude --model claude-opus-4-5     # Model override
vibes claude --system-prompt "Be terse"  # Custom system prompt

# Vibes additions
vibes claude --session-name "auth-work"  # Human-friendly session names
vibes claude --no-serve                  # Disable background server

# Configuration
vibes config show                        # Display merged configuration
vibes config path                        # Show config file locations

# Access from any device on your network
# Web UI available at http://localhost:7432
```

## Plugins

vibes supports native Rust plugins for extending functionality. Plugins can react to session events, track token usage, log conversations, and more.

```bash
# List installed plugins
vibes plugin list

# Enable/disable plugins
vibes plugin enable analytics
vibes plugin disable history

# Show plugin details
vibes plugin info my-plugin
```

### Plugin Directory

Plugins are installed to `~/.config/vibes/plugins/`:

```
~/.config/vibes/plugins/
├── registry.toml           # Tracks enabled plugins
└── my-plugin/
    ├── my-plugin.0.1.0.so  # Versioned binary
    ├── my-plugin.so        # Symlink to current version
    └── config.toml         # Plugin configuration
```

### Writing Plugins

See the [example plugin](examples/plugins/hello-plugin/) for a complete working example.

```rust
use vibes_plugin_api::{export_plugin, Plugin, PluginContext, PluginError, PluginManifest};

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "My custom plugin".to_string(),
            ..Default::default()
        }
    }

    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        ctx.log_info("Plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

export_plugin!(MyPlugin);
```

## Architecture

vibes uses a **daemon-first architecture** with a PTY-based backend. The server owns Claude sessions as persistent PTY processes, and both CLI and Web UI connect as terminal clients.

```
┌─────────────────────────────────────────────────────────────────────┐
│                      vibes daemon (server)                          │
│                        localhost:7432                               │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────────────┐ │
│  │ PTY Manager  │◄───│ Hook Receiver│    │   WebSocket Server    │ │
│  │              │    │   (events)   │    │                       │ │
│  │ ┌──────────┐ │    └──────┬───────┘    │  ┌────────┐ ┌───────┐ │ │
│  │ │ claude   │ │           │            │  │  CLI   │ │  Web  │ │ │
│  │ │  (PTY)   │ │    structured          │  │terminal│ │xterm  │ │ │
│  │ └──────────┘ │    ClaudeEvents        │  └────────┘ └───────┘ │ │
│  └──────────────┘           │            └───────────────────────┘ │
│                             ▼                                      │
│                    ┌──────────────┐                                │
│                    │  Event Bus   │──► Analytics, History, iOS     │
│                    └──────────────┘                                │
└─────────────────────────────────────────────────────────────────────┘
         ▲                           ▲
         │ PTY I/O via WebSocket     │ PTY I/O via WebSocket
         │                           │
┌────────┴────────┐         ┌────────┴────────┐
│  vibes claude   │         │   Web Browser   │
│  (CLI client)   │         │   (xterm.js)    │
│  Raw terminal   │         │   Terminal UI   │
└─────────────────┘         └─────────────────┘
```

**Key components:**

- **Daemon Server** - Background process that owns PTY sessions (survives CLI disconnect)
- **PTY Manager** - Spawns Claude in persistent pseudo-terminals
- **Hook Receiver** - Captures structured events via Claude Code hooks
- **CLI Client** - Connects to daemon, proxies PTY I/O to local terminal
- **Web UI** - xterm.js terminal emulator showing exact CLI experience
- **Event Bus** - Real-time pub/sub fed by hooks for analytics/history

## Testing

vibes uses a three-layer testing pyramid:

```
┌─────────────────────────────────────────────────────────┐
│           E2E Tests (e2e-tests/)                        │
│  Playwright: Browser + CLI + Server integration         │
│  Critical user journeys, runs on PR/main                │
├─────────────────────────────────────────────────────────┤
│        Integration Tests (crate/tests/)                 │
│  WebSocket protocol, server config, concurrency         │
│  In-process, runs in CI                                 │
├─────────────────────────────────────────────────────────┤
│           Unit Tests (#[cfg(test)])                     │
│  Logic correctness, edge cases, MockBackend-based       │
│  Fast, isolated, no I/O                                 │
└─────────────────────────────────────────────────────────┘
```

### Running Tests

```bash
# Unit + integration tests (recommended for development)
just test

# All tests including those requiring Claude CLI
just test-all

# E2E browser tests (requires Playwright installed)
just test-e2e

# Pre-commit checks (fmt, clippy, test)
just pre-commit
```

### Test Infrastructure

- **MockBackend** - Scripted Claude responses for unit tests
- **SlowMockBackend** - Delayed responses for concurrency tests
- **TestClient** - WebSocket client for protocol testing
- **Playwright** - Browser automation for e2e tests

## Documentation

- [Product Requirements Document](docs/PRD.md) - Full design, architecture, and roadmap
- [Progress Tracker](docs/PROGRESS.md) - Implementation status and changelog
- [Planning Conventions](docs/PLAN.md) - How to create design and implementation plans
- [CLAUDE.md](CLAUDE.md) - Development guidance for contributors

## Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| [**1. Foundation**](docs/PROGRESS.md#phase-1-foundation-mvp) | Claude Code proxy, plugin system, local web UI | ✅ Complete |
| [**2. Remote Access**](docs/PROGRESS.md#phase-2-remote-access) | Cloudflare Tunnel, authentication, push notifications | ✅ Complete |
| [**3. Multi-Client**](docs/PROGRESS.md#phase-3-multi-client-experience-) | PTY backend, xterm.js UI, multi-session, mirroring | ✅ Complete |
| [**4. Continual Learning**](docs/PROGRESS.md#phase-4-continual-learning) | Self-improving assistant that learns from every session | ⏳ Planned |
| [**5. Polish**](docs/PROGRESS.md#phase-5-polish--ecosystem) | Setup wizards, default plugins, iOS app | ⏳ Planned |

See [PROGRESS.md](docs/PROGRESS.md) for detailed milestone tracking and changelog.

## License

MIT License - see [LICENSE](LICENSE) for details.
