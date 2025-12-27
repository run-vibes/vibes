# Progress Tracker

This document tracks the implementation progress of vibes against the roadmap defined in [PRD.md](./PRD.md).

## Quick Links

| Milestone | Status | Design | Implementation |
|-----------|--------|--------|----------------|
| 1.1 Core Proxy | Complete | [design](plans/01-core-proxy/design.md) | [implementation](plans/01-core-proxy/implementation.md) |
| 1.2 CLI | Complete | [design](plans/02-cli/design.md) | [implementation](plans/02-cli/implementation.md) |
| 1.3 Plugin Foundation | Complete | [design](plans/03-plugin-foundation/design.md) | [implementation](plans/03-plugin-foundation/implementation.md) |
| 1.4 Server + Web UI | Complete | [design](plans/04-server-web-ui/design.md) | [implementation](plans/04-server-web-ui/implementation.md) |
| 2.1 Cloudflare Tunnel | Complete | [design](plans/05-cloudflare-tunnel/design.md) | [implementation](plans/05-cloudflare-tunnel/implementation.md) |
| 2.2 Cloudflare Access | Planned | [design](plans/06-cloudflare-access/design.md) | - |
| 2.3 Push Notifications | Planned | - | - |

---

## Legend

- [ ] Not started
- [~] In progress
- [x] Complete

---

## Phase 1: Foundation (MVP)

**Goal:** `vibes claude` works, plugin system functional, local web UI accessible

### Milestone 1.1: Core proxy
- [x] vibes-core crate with Session, EventBus
- [x] Claude Code subprocess management
- [x] Stream-json parsing and event broadcasting
- [x] Basic error handling and recovery

### Milestone 1.2: CLI
- [x] vibes claude pass-through (all claude flags work)
- [x] --session-name support
- [x] vibes config basics (show, path commands)
- [x] Server auto-start (stub)

### Milestone 1.3: Plugin foundation
- [x] Plugin trait and API crate (vibes-plugin-api)
- [x] Dynamic library loading
- [x] Plugin lifecycle (load, unload, enable, disable)
- [x] vibes plugin CLI commands
- [x] Event subscription system

### Milestone 1.4: Server + Web UI
- [x] axum HTTP/WebSocket server
- [x] TanStack web UI with session view
- [x] Permission approve/deny flow
- [x] Simple mode + full mode toggle
- [x] rust-embed for bundling UI

**Phase 1 Deliverable:** Single binary that proxies Claude Code with web UI on localhost

---

## Phase 2: Remote Access

**Goal:** Access vibes from anywhere securely

### Milestone 2.1: Cloudflare Tunnel integration
- [x] vibes tunnel setup wizard (stub)
- [x] cloudflared process management
- [x] Tunnel status in UI
- [x] Auto-reconnect handling

### Milestone 2.2: Cloudflare Access auth
- [ ] AccessConfig type with TOML parsing
- [ ] JwtValidator with JWKS caching
- [ ] AuthMiddleware layer for axum
- [ ] Localhost bypass logic
- [ ] vibes auth CLI commands (status, setup)
- [ ] Auto-detect team/aud from tunnel config
- [ ] Web UI identity display
- [ ] WebSocket auth_context message

### Milestone 2.3: Push notifications
- [ ] --notify flag
- [ ] Web push subscription
- [ ] Notification on completion/error/permission-needed
- [ ] Mobile-friendly notification actions

**Phase 2 Deliverable:** Access vibes from phone anywhere with Cloudflare auth

---

## Phase 3: Polish & Ecosystem

**Goal:** Production-ready with default plugins

### Milestone 3.1: Default plugins
- [ ] analytics (session stats, token usage)
- [ ] history (searchable session history)
- [ ] templates (prompt templates/snippets)
- [ ] export (session export to markdown/JSON)

### Milestone 3.2: Multiple sessions
- [ ] Run multiple Claude sessions concurrently
- [ ] Session switcher in UI
- [ ] Per-session notification settings

### Milestone 3.3: CLI enhancements
- [ ] vibes sessions list/switch/kill
- [ ] Tab completion
- [ ] Interactive session picker

**Phase 3 Deliverable:** Feature-rich vibes with useful default plugins

---

## Future Phases

These phases are planned but not yet scheduled.

### Phase F1: Alternative Claude Code Interaction
- [ ] Investigate PTY wrapper for interactive mode
- [ ] Investigate hook-based permission routing
- [ ] Investigate stream-json bidirectional
- [ ] Benchmark and decide on migration

### Phase F2: Mobile Apps
- [ ] iOS app (Swift)
- [ ] Android app (Kotlin)
- [ ] Push notification integration
- [ ] App Store/Play Store distribution

### Phase F3: Native GUIs
- [ ] macOS: Cocoa/AppKit via objc2
- [ ] Windows: Win32/WinUI via windows-rs
- [ ] Linux: GTK via gtk-rs
- [ ] Menu bar/system tray integration

### Phase F4: Licensing System
- [ ] License validation adapter interface
- [ ] Plugin license checking
- [ ] Grace periods and offline validation
- [ ] License server integration

---

## Changelog

| Date | Change |
|------|--------|
| 2025-12-26 | Initial progress tracker created |
| 2025-12-26 | Milestone 1.1 (Core proxy) complete - vibes-core crate with Session, EventBus, PrintModeBackend, stream-json parser |
| 2025-12-26 | Milestone 1.2 (CLI) complete - vibes-cli crate with claude command, config system, server stub |
| 2025-12-26 | Milestone 1.3 (Plugin foundation) complete - vibes-plugin-api crate with Plugin trait, PluginHost, CLI commands, event dispatch |
| 2025-12-26 | Milestone 1.4 (Server + Web UI) complete - vibes-server crate with axum HTTP/WebSocket, TanStack web UI, daemon auto-start, CLI as WS client |
| 2025-12-26 | Milestone 2.1 (Cloudflare Tunnel) complete - TunnelManager, cloudflared CLI wrapper, tunnel CLI commands, UI status badge |
| 2025-12-27 | Milestone 2.2 (Cloudflare Access) design complete - AuthMiddleware, JwtValidator, localhost bypass, identity display |
