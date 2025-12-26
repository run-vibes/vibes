# vibes

Vibe coding swiss army knife of enhancements.

**vibes** supercharges your vibe coding workflow by wrapping Claude Code with remote access, session management, and a plugin ecosystem.

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

# Vibes additions
vibes claude --session-name "auth-work"  # Human-friendly session names

# Access from any device on your network
# Web UI available at http://localhost:7432
```

## Status

vibes is under active development. See the [Product Requirements Document](docs/PRD.md) for the full design and roadmap.

### Planned Phases

1. **Foundation** - Claude Code proxy, plugin system, local web UI
2. **Remote Access** - Cloudflare Tunnel integration, authentication
3. **Ecosystem** - Default plugins, multi-session support
4. **Future** - Native mobile apps (iOS/Android), native desktop GUIs, licensing

## Documentation

- [Product Requirements Document](docs/PRD.md) - Full design, architecture, and roadmap
- [CLAUDE.md](CLAUDE.md) - Development guidance for contributors

## License

TBD
