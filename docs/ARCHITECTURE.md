# Architecture

vibes uses a **daemon-first architecture** with a PTY-based backend and persistent event streaming. The server owns Claude sessions as persistent PTY processes, and both CLI and Web UI connect as terminal clients.

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                      vibes daemon (server)                          │
│                        localhost:7432                               │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────────────┐ │
│  │ PTY Manager  │◄───│ vibes event  │    │   WebSocket Server    │ │
│  │              │    │  send (CLI)  │    │                       │ │
│  │ ┌──────────┐ │    └──────┬───────┘    │  ┌────────┐ ┌───────┐ │ │
│  │ │ claude   │ │           │            │  │  CLI   │ │  Web  │ │ │
│  │ │  (PTY)   │ │    structured          │  │terminal│ │xterm  │ │ │
│  │ └──────────┘ │    ClaudeEvents        │  └────────┘ └───────┘ │ │
│  └──────────────┘           │            └───────────────────────┘ │
│                             ▼                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    EventLog (Iggy)                            │  │
│  │  Persistent event stream with consumer-based processing       │  │
│  └───────────┬─────────────────┬─────────────────┬──────────────┘  │
│              │                 │                 │                  │
│        ┌─────▼─────┐    ┌──────▼──────┐   ┌─────▼─────┐            │
│        │ WebSocket │    │ Notification│   │ Assessment│            │
│        │ Consumer  │    │  Consumer   │   │ Consumer  │            │
│        │(broadcast)│    │   (push)    │   │ (groove)  │            │
│        └───────────┘    └─────────────┘   └───────────┘            │
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

## Key Components

### Daemon Server

The background process that owns PTY sessions. Sessions survive CLI disconnects—you can close your terminal and reconnect from your phone.

### PTY Manager

Spawns Claude Code in persistent pseudo-terminals. Each session is a real PTY, supporting full ANSI escape sequences, colors, and interactive editing.

### Event CLI (`vibes event send`)

Hook scripts use `vibes event send` to write structured events directly to Iggy. This replaced the earlier HookReceiver Unix socket architecture for simplicity.

```bash
# Hook scripts call this to send events
vibes event send --type hook --session "$VIBES_SESSION_ID" --data "$1"
```

### EventLog (Iggy)

Persistent event stream backed by [Apache Iggy](https://iggy.rs/) message streaming. The EventLog is the single source of truth—all events flow through it, and consumers read from it independently.

### Consumers

Independent event processors that read from the EventLog:

| Consumer | Purpose |
|----------|---------|
| **WebSocket** | Broadcasts events to connected clients in real-time |
| **Notification** | Sends web push notifications for session events |
| **Assessment** | groove's outcome measurement for continual learning |

### CLI Client

Connects to the daemon via WebSocket and proxies PTY I/O to your local terminal. All Claude Code features work normally—it's a transparent proxy.

### Web UI

An xterm.js terminal emulator showing the exact CLI experience. Connect from any device on your network via `http://localhost:7432`.

---

## Event Flow

Events flow through the system in a single direction:

```
Event Producers ──► EventLog ──► Consumers ──► Effects
       │                                          │
       │                                          ├── WebSocket broadcast
       │                                          ├── Push notifications
       │                                          └── Learning assessment
       │
       ├── WebSocket handlers (session create/destroy)
       ├── PTY handlers (session state changes)
       └── Hook scripts (Claude Code events)
```

**Why this design?**

1. **Single source of truth** — All events are persisted before processing
2. **Independent consumers** — Each consumer has its own read position
3. **Replay capability** — New consumers can replay historical events
4. **Crash recovery** — Events are durable; consumers resume from last position

---

## Plugin Architecture

Plugins are dynamically loaded native Rust libraries (`.so`/`.dylib`/`.dll`).

```
┌──────────────────────────────────────────────────┐
│                 vibes daemon                      │
│                                                  │
│  ┌──────────────────┐    ┌───────────────────┐  │
│  │  Plugin Manager  │───▶│  Plugin Instance  │  │
│  │                  │    │  (groove, etc.)   │  │
│  │  • Load/unload   │    │                   │  │
│  │  • Lifecycle     │    │  • on_load()      │  │
│  │  • Events        │    │  • on_event()     │  │
│  └──────────────────┘    │  • on_unload()    │  │
│                          └───────────────────┘  │
│                                    │            │
│                          ┌─────────┴──────────┐ │
│                          │ vibes-plugin-api   │ │
│                          │ (published crate)  │ │
│                          └────────────────────┘ │
└──────────────────────────────────────────────────┘
```

Plugins can:
- Register CLI subcommands (`vibes <plugin-name> <command>`)
- Register HTTP routes (`/api/plugins/<plugin-name>/`)
- React to events from the EventLog
- Access plugin-specific configuration

---

## Session Lifecycle

```
  vibes claude "prompt"
         │
         ▼
┌─────────────────────────────────────────┐
│ 1. CLI connects to daemon WebSocket     │
│ 2. Daemon creates or resumes session    │
│ 3. PTY spawns claude process            │
│ 4. CLI ←→ Daemon ←→ PTY (bidirectional) │
│ 5. Events captured via hooks            │
│ 6. CLI disconnects → Session persists   │
└─────────────────────────────────────────┘
         │
         ▼
  Session survives CLI disconnect
  Reconnect from any client
```

---

## Authentication

vibes supports multiple authentication modes:

| Mode | Use Case |
|------|----------|
| **Localhost bypass** | No auth required for local connections |
| **Cloudflare Access JWT** | Remote access via Cloudflare Tunnel |

When using Cloudflare Tunnel, all requests are authenticated via Cloudflare Access JWTs before reaching vibes.

---

## Testing Infrastructure

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

### Test Utilities

- **MockBackend** — Scripted Claude responses for unit tests
- **SlowMockBackend** — Delayed responses for concurrency tests
- **TestClient** — WebSocket client for protocol testing
- **Playwright** — Browser automation for e2e tests

---

## Related Documentation

- [Product Requirements Document](PRD.md) — Full design and specifications
- [Plugins](PLUGINS.md) — Plugin system and in-tree plugins
- [Planning Board](board/README.md) — Implementation status and roadmap
