# vibes

[![CI](https://github.com/run-vibes/vibes/actions/workflows/ci.yml/badge.svg)](https://github.com/run-vibes/vibes/actions/workflows/ci.yml)
[![Progress](https://img.shields.io/badge/progress-7%2F11%20milestones-blue)](docs/PROGRESS.md)
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

## Status

**Phase 2 in progress!** vibes now provides a fully functional Claude Code proxy with web UI, plugin support, and Cloudflare Tunnel integration. See [PROGRESS.md](docs/PROGRESS.md) for detailed tracking.

### Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| **1. Foundation** | Claude Code proxy, plugin system, local web UI | ✅ Complete |
| **2. Remote Access** | Cloudflare Tunnel integration, authentication | 🔄 In Progress |
| **3. Ecosystem** | Default plugins, multi-session support | ⏳ Planned |
| **4. Future** | Mobile apps, native GUIs, licensing | 📋 Future |

### Current Milestones

| Milestone | Description | Status |
|-----------|-------------|--------|
| 1.1 Core Proxy | Session management, event bus, Claude subprocess | ✅ Complete |
| 1.2 CLI | `vibes claude` pass-through, config, server auto-start | ✅ Complete |
| 1.3 Plugin Foundation | Plugin trait, dynamic loading, CLI commands | ✅ Complete |
| 1.4 Server + Web UI | axum server, TanStack UI, permission flows, daemon architecture | ✅ Complete |
| 2.1 Cloudflare Tunnel | Tunnel management, quick/named modes, status UI | ✅ Complete |
| 2.2 Cloudflare Access | JWT validation, auth middleware, identity display | ✅ Complete |

## Architecture

vibes uses a **daemon-first architecture** where a background server owns all session state. The CLI and Web UI connect as WebSocket clients.

```
                         ┌─────────────────────────────────────────────────────┐
                         │              vibes daemon (server)                   │
                         │                localhost:7432                        │
                         ├─────────────────────────────────────────────────────┤
                         │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
                         │  │   Session   │  │  EventBus   │  │ PluginHost  │  │
                         │  │   Manager   │  │  (memory)   │  │             │  │
                         │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  │
                         │         │                │                │         │
                         │         └────────────────┼────────────────┘         │
                         │                          │                          │
                         │         ┌────────────────┴────────────────┐         │
                         │         │  axum HTTP/WebSocket Server     │         │
                         │         │  ┌───────────┐  ┌────────────┐  │         │
                         │         │  │  REST API │  │ WebSocket  │  │         │
                         │         │  └───────────┘  └────────────┘  │         │
                         │         └─────────────────────────────────┘         │
                         │                          │                          │
                         │         ┌────────────────┴────────────────┐         │
                         │         │    Embedded TanStack Web UI     │         │
                         │         │    (rust-embed static files)    │         │
                         │         └─────────────────────────────────┘         │
                         └─────────────────────────────────────────────────────┘
                                    ▲               ▲                ▲
                                    │ WebSocket     │ HTTP           │ spawns
┌─────────────────┐                 │               │                │
│   vibes claude  │◄────────────────┘               │    ┌───────────┴───────┐
│   (CLI client)  │                                 │    │    Claude Code    │
└─────────────────┘                                 │    │   (subprocess)    │
                                                    │    └───────────────────┘
┌─────────────────┐                                 │
│   Web Browser   │◄────────────────────────────────┘
│ (phone/tablet)  │  http://localhost:7432
└─────────────────┘
```

**Key components:**

- **Daemon Server** - Background process (auto-started by CLI) that owns all state
- **SessionManager** - Orchestrates Claude Code sessions
- **EventBus** - Real-time pub/sub for events with late-joiner replay
- **PluginHost** - Loads and manages native Rust plugins
- **CLI Client** - Connects to daemon via WebSocket, streams I/O to terminal
- **Web UI** - TanStack React SPA embedded in binary, served on localhost:7432

## Documentation

- [Product Requirements Document](docs/PRD.md) - Full design, architecture, and roadmap
- [Progress Tracker](docs/PROGRESS.md) - Implementation status and changelog
- [CLAUDE.md](CLAUDE.md) - Development guidance for contributors

### Implementation Plans

| Milestone | Design | Implementation |
|-----------|--------|----------------|
| 1.1 Core Proxy | [design.md](docs/plans/01-core-proxy/design.md) | [implementation.md](docs/plans/01-core-proxy/implementation.md) |
| 1.2 CLI | [design.md](docs/plans/02-cli/design.md) | [implementation.md](docs/plans/02-cli/implementation.md) |
| 1.3 Plugin Foundation | [design.md](docs/plans/03-plugin-foundation/design.md) | [implementation.md](docs/plans/03-plugin-foundation/implementation.md) |
| 1.4 Server + Web UI | [design.md](docs/plans/04-server-web-ui/design.md) | [implementation.md](docs/plans/04-server-web-ui/implementation.md) |
| 2.1 Cloudflare Tunnel | [design.md](docs/plans/05-cloudflare-tunnel/design.md) | [implementation.md](docs/plans/05-cloudflare-tunnel/implementation.md) |
| 2.2 Cloudflare Access | [design.md](docs/plans/06-cloudflare-access/design.md) | [implementation.md](docs/plans/06-cloudflare-access/implementation.md) |

## License

MIT License - see [LICENSE](LICENSE) for details.
