# vibes

[![CI](https://github.com/run-vibes/vibes/actions/workflows/ci.yml/badge.svg)](https://github.com/run-vibes/vibes/actions/workflows/ci.yml)

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

## Status

vibes is under active development. See [PROGRESS.md](docs/PROGRESS.md) for detailed tracking.

### Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| **1. Foundation** | Claude Code proxy, plugin system, local web UI | 🔨 In Progress |
| **2. Remote Access** | Cloudflare Tunnel integration, authentication | ⏳ Planned |
| **3. Ecosystem** | Default plugins, multi-session support | ⏳ Planned |
| **4. Future** | Mobile apps, native GUIs, licensing | 📋 Future |

### Phase 1 Progress

| Milestone | Description | Status |
|-----------|-------------|--------|
| 1.1 Core Proxy | Session management, event bus, Claude subprocess | ✅ Complete |
| 1.2 CLI | `vibes claude` pass-through, config, server auto-start | ✅ Complete |
| 1.3 Plugin Foundation | Plugin trait, dynamic loading, CLI commands | ⏳ Next |
| 1.4 Server + Web UI | axum server, TanStack UI, permission flows | ⏳ Planned |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              vibes binary                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                           vibes-core                                     ││
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                   ││
│  │  │   Session    │  │   EventBus   │  │ PluginHost   │◄──────────────┐   ││
│  │  │   Manager    │  │  (memory)    │  │              │               │   ││
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │   ││
│  │         │                 │                 │                        │   ││
│  │         │    ┌────────────┴────────────┐    │                        │   ││
│  │         │    │      Event Flow         │    │                        │   ││
│  │         │    │  ┌─────────────────┐    │    │                        │   ││
│  │         └────┼─►│   VibesEvent    │────┼────┘                        │   ││
│  │              │  └─────────────────┘    │                             │   ││
│  │              └─────────────────────────┘                             │   ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│         ┌──────────────────────────┼──────────────────────────┐             │
│         ▼                          ▼                          ▼             │
│  ┌─────────────┐           ┌─────────────┐           ┌─────────────┐       │
│  │  CLI Mode   │           │ Server Mode │           │  GUI Mode   │       │
│  │  (vibes-cli)│           │   (axum)    │           │  (future)   │       │
│  └─────────────┘           └─────────────┘           └─────────────┘       │
└─────────────────────────────────────────────────────────────────────────────┘
         │                          │
         │ spawns                   │ loads
         ▼                          ▼
┌─────────────────┐    ┌─────────────────────────────────────────────────────┐
│   Claude Code   │    │              ~/.config/vibes/plugins/                │
│  (subprocess)   │    ├─────────────────────────────────────────────────────┤
└─────────────────┘    │  ┌───────────────┐  ┌───────────────┐               │
                       │  │  analytics/   │  │   history/    │  ...          │
                       │  │  .0.1.0.so    │  │  .0.2.0.so    │               │
                       │  │  .so ────────►│  │  .so ────────►│               │
                       │  │  config.toml  │  │  config.toml  │               │
                       │  └───────────────┘  └───────────────┘               │
                       │  registry.toml ─── enabled: [analytics, history]    │
                       └─────────────────────────────────────────────────────┘
```

**Key components:**

- **SessionManager** - Orchestrates Claude Code sessions
- **EventBus** - Real-time pub/sub for events with late-joiner replay
- **ClaudeBackend** - Adapter pattern for Claude interaction (print mode, PTY, etc.)
- **PluginHost** - Loads and manages native Rust plugins with panic/timeout isolation
- **CLI/Server/GUI** - Thin shells consuming vibes-core

## Documentation

- [Product Requirements Document](docs/PRD.md) - Full design, architecture, and roadmap
- [Progress Tracker](docs/PROGRESS.md) - Implementation status and changelog
- [CLAUDE.md](CLAUDE.md) - Development guidance for contributors

### Implementation Plans

| Milestone | Design | Implementation |
|-----------|--------|----------------|
| 1.1 Core Proxy | [design.md](docs/plans/01-core-proxy/design.md) | [implementation.md](docs/plans/01-core-proxy/implementation.md) |
| 1.2 CLI | [design.md](docs/plans/02-cli/design.md) | [implementation.md](docs/plans/02-cli/implementation.md) |
| 1.3 Plugin Foundation | [design.md](docs/plans/03-plugin-foundation/design.md) | TBD |

## License

TBD
