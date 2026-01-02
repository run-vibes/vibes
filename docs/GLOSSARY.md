# Glossary

Key terms used throughout the vibes documentation and codebase.

| Term | Definition |
|------|------------|
| **vibes** | The mech suit—wraps Claude Code with remote access, persistence, and plugins |
| **daemon** | Background server that owns PTY sessions and persists across CLI disconnects |
| **session** | A Claude Code conversation running in a persistent PTY |
| **groove** | Continual learning plugin that remembers what works and injects it into future sessions |
| **EventLog** | Persistent event stream backed by Apache Iggy |
| **consumer** | Independent processor that reads from the EventLog (WebSocket, notifications, assessment) |
| **PTY** | Pseudo-terminal—real terminal emulation for full escape sequence support |
| **hook** | Claude Code extension point; vibes uses hooks to capture structured events |
| **learning** | A captured pattern or preference that groove injects into future contexts |
| **harness** | Any AI coding assistant (Claude Code, Cursor, etc.)—groove is harness-agnostic |
| **Iggy** | [Apache Iggy](https://iggy.rs/)—the message streaming platform backing the EventLog |
| **plugin** | Native Rust library (`.so`/`.dylib`/`.dll`) that extends vibes functionality |
| **mirroring** | Real-time sync of terminal I/O between CLI and Web UI clients |
| **scope** | Learning context level: global (all projects), user (all your projects), or project (single repo) |

## See Also

- [Architecture](ARCHITECTURE.md) — How components fit together
- [Plugins](PLUGINS.md) — Plugin system and groove details
- [groove Branding](groove/BRANDING.md) — Personality and voice of the learning system
