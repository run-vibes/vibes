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
| 2.2 Cloudflare Access | Complete | [design](plans/06-cloudflare-access/design.md) | [implementation](plans/06-cloudflare-access/implementation.md) |
| 2.3 Push Notifications | Complete | [design](plans/07-push-notifications/design.md) | [implementation](plans/07-push-notifications/implementation.md) |
| 3.1 Chat History | Complete | [design](plans/08-chat-history/design.md) | [implementation](plans/08-chat-history/implementation.md) |
| 3.2 Multi-Session Support | Complete | [design](plans/09-multi-session/design.md) | [implementation](plans/09-multi-session/implementation.md) |
| 3.3 CLI ↔ Web Mirroring | Design complete | [design](plans/10-cli-web-mirroring/design.md) | — |
| 3.4 Cloudflare Tunnel Wizard | Not started | — | — |
| 3.5 Cloudflare Auth Wizard | Not started | — | — |

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
- [x] AccessConfig type with TOML parsing
- [x] JwtValidator with JWKS caching
- [x] AuthMiddleware layer for axum
- [x] Localhost bypass logic
- [x] vibes auth CLI commands (status, test)
- [x] Web UI identity display
- [x] WebSocket auth_context message

Note: Auto-detect team/aud moved to Milestone 3.5 (Cloudflare Auth Wizard)

### Milestone 2.3: Push notifications
- [x] --notify flag
- [x] Web push subscription
- [x] Notification on completion/error/permission-needed
- [x] Mobile-friendly notification actions

**Phase 2 Deliverable:** Access vibes from phone anywhere with Cloudflare auth

---

## Phase 3: Multi-Client Experience

**Goal:** Full multi-client support with setup wizards for remote access

### Milestone 3.1: Chat History
- [x] Persistent session history storage (SQLite with FTS5)
- [x] Session search and filtering
- [x] Replay previous sessions from any client
- [x] History pagination for large session counts

### Milestone 3.2: Multi-Session Support
- [x] Multiple concurrent Claude sessions on same server
- [x] Session list view in Web UI with status indicators
- [x] Session isolation (events/input per session)
- [x] Graceful session cleanup on disconnect

### Milestone 3.3: CLI ↔ Web Mirroring
- [x] Design complete (input attribution, catch-up protocol)
- [ ] Add `InputSource` enum and update `UserInput` event
- [ ] Add `source` column to messages table
- [ ] Implement `SubscribeAck` with paginated catch-up
- [ ] CLI displays remote input with `[Web UI]:` prefix
- [ ] CLI input history with up/down navigation
- [ ] Web UI shows source attribution on messages
- [ ] Web UI catch-up on session join

### Milestone 3.4: Cloudflare Tunnel Wizard
- [ ] Interactive `vibes tunnel setup` wizard
- [ ] Guide user through cloudflared installation check
- [ ] Tunnel mode selection (quick vs named)
- [ ] DNS configuration assistance for named tunnels
- [ ] Test connectivity and display public URL

### Milestone 3.5: Cloudflare Auth Wizard
- [ ] Interactive `vibes auth setup` wizard
- [ ] Auto-detect team/AUD from existing tunnel config
- [ ] Manual configuration fallback
- [ ] Test JWT validation with sample request
- [ ] Display identity information on success

**Phase 3 Deliverable:** Multiple clients share sessions in real-time with guided setup

---

## Phase 4: Polish & Ecosystem

**Goal:** Production-ready with default plugins

### Milestone 4.1: Default plugins
- [ ] analytics (session stats, token usage)
- [ ] history (searchable session history)
- [ ] templates (prompt templates/snippets)
- [ ] export (session export to markdown/JSON)

### Milestone 4.2: CLI enhancements
- [ ] vibes sessions list/switch/kill
- [ ] Tab completion
- [ ] Interactive session picker

### Milestone 4.3: Advanced permissions
- [ ] Per-session notification settings
- [ ] First-responder policy for permission requests
- [ ] Permission request timeout handling

**Phase 4 Deliverable:** Feature-rich vibes with useful default plugins

---

## Future Phases

These phases are planned but not yet scheduled.

### Phase F1: Alternative Claude Code Backends
- [ ] Investigate PTY wrapper for interactive mode
- [ ] Investigate hook-based permission routing
- [ ] Investigate stream-json bidirectional
- [ ] Benchmark PrintModeBackend vs alternatives
- [ ] Migrate if significant benefits proven

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
| 2025-12-27 | Milestone 2.2 (Cloudflare Access) implementation complete - JWT validation, auth middleware, CLI commands, WebSocket auth_context, Web UI identity |
| 2025-12-27 | Milestone 2.3 (Push Notifications) complete - VAPID keys, subscription store, NotificationService, web push endpoints, service worker, usePushSubscription hook, NotificationSettings UI |
| 2025-12-27 | Roadmap re-planned: New Phase 3 (Multi-Client Experience) with chat history, multi-session, CLI↔Web mirroring, setup wizards. Old Phase 3 becomes Phase 4. |
| 2025-12-27 | Milestone 3.1 (Chat History) complete - SQLite storage with FTS5 full-text search, HistoryService, REST API endpoints, Web UI history page with search/filter/pagination |
| 2025-12-27 | Milestone 3.2 (Multi-Session) design complete - SessionOwnership, SessionLifecycleManager, ownership transfer, responsive UI, CLI sessions commands |
| 2025-12-27 | Milestone 3.2 (Multi-Session) implementation complete - SessionOwnership with subscriber tracking, SessionLifecycleManager for disconnect handling, WebSocket protocol for session list/lifecycle events, CLI `sessions` command (list/attach/kill), Web UI with responsive layout and real-time updates |
| 2025-12-27 | Milestone 3.3 (CLI ↔ Web Mirroring) design complete - Input source attribution, paginated catch-up protocol, CLI input history with arrow keys |
