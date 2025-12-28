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
| 3.3 CLI ↔ Web Mirroring | Complete | [design](plans/10-cli-web-mirroring/design.md) | [implementation](plans/10-cli-web-mirroring/implementation.md) |
| 3.4 PTY Backend | Complete | [design](plans/12-pty-backend/design.md) | [implementation](plans/12-pty-backend/implementation.md) |
| **◉ groove** | | [branding](groove/BRANDING.md) | |
| 4.1 Harness Introspection | Not started | [design](plans/15-harness-introspection/design.md) | [implementation](plans/15-harness-introspection/implementation.md) |
| 4.2 Storage Foundation | Not started | [design](plans/14-continual-learning/design.md#42-storage-foundation) | — |
| 4.2.5 Security Foundation | Not started | [design](plans/14-continual-learning/design.md#425-security-foundation--new) | — |
| 4.3 Capture & Inject | Not started | [design](plans/14-continual-learning/design.md#43-capture--inject-mvp) | — |
| 4.4 Assessment Framework | Not started | [design](plans/14-continual-learning/design.md#44-assessment-framework--new) | — |
| 4.5 Learning Extraction | Not started | [design](plans/14-continual-learning/design.md#45-learning-extraction) | — |
| 4.6 Attribution Engine | Not started | [design](plans/14-continual-learning/design.md#46-attribution-engine--new) | — |
| 4.7 Adaptive Strategies | Not started | [design](plans/14-continual-learning/design.md#47-adaptive-strategies) | — |
| 4.8 groove Dashboard | Not started | [design](plans/14-continual-learning/design.md#48-observability-dashboard--new) | — |
| 4.9 Open-World Adaptation | Not started | [design](plans/14-continual-learning/design.md#49-open-world-adaptation) | — |
| 5.1 Setup Wizards | Not started | — | — |
| 5.2 Default Plugins | Not started | — | — |
| 5.3 CLI Enhancements | Not started | — | — |
| 5.4 iOS App | Not started | — | — |

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

## Phase 3: Multi-Client Experience ✓

**Goal:** Full multi-client support with real-time sessions

**Status:** Complete

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
- [x] Add `InputSource` enum and update `UserInput` event
- [x] Add `source` column to messages table
- [x] Implement `SubscribeAck` with paginated catch-up
- [x] CLI displays remote input with `[Web UI]:` prefix
- [x] CLI input history with up/down navigation
- [x] Web UI shows source attribution on messages
- [x] Web UI catch-up on session join

### Milestone 3.4: PTY Backend
- [x] Replace PrintModeBackend with PTY wrapper for full CLI parity
- [x] Add `portable-pty` crate for cross-platform PTY management
- [x] Implement PtyManager with spawn/read/write/resize/kill
- [x] Refactor `vibes claude` to PTY client mode
- [x] Web UI terminal via xterm.js (replace chat-based UI)
- [x] Claude hooks integration for structured data capture
- [x] Auto-configure hooks on daemon start
- [x] Remove PrintModeBackend and stream-json parser

**Phase 3 Deliverable:** Full CLI parity with real-time multi-client sessions ✓

---

## Phase 4: vibes groove ◉

> **groove** - The continual learning system that finds your coding rhythm.

**Goal:** Progressive improvement through accumulated experience - zero friction, fully adaptive.

**Design:** [vibes groove Design](plans/14-continual-learning/design.md) | [Branding Guide](groove/BRANDING.md)

### Milestone 4.1: Harness Introspection
- [ ] `Harness` trait and `HarnessCapabilities` struct
- [ ] `ClaudeCodeHarness` implementation
- [ ] `GenericHarnessDiscovery` for unknown harnesses
- [ ] Capability caching in storage

### Milestone 4.2: Storage Foundation
- [ ] CozoDB setup with schema and migrations
- [ ] `Learning` model with UUIDv7 identifiers
- [ ] `LearningStorage` trait and CozoDB implementation
- [ ] `AdaptiveParam` with Bayesian update mechanics
- [ ] `AdaptiveConfig` for system-wide parameters

### Milestone 4.2.5: Security Foundation
- [ ] `TrustLevel` enum (Local → Quarantined hierarchy)
- [ ] `TrustContext`, `TrustSource`, `Permissions` types
- [ ] `Provenance` with `ContentHash` and `CustodyChain`
- [ ] `ContentSecurityScanner` for injection detection
- [ ] `SecureInjector` with trust-aware wrapping
- [ ] `AuditLog` trait and CozoDB audit schema
- [ ] `OrgRole` RBAC (Admin, Curator, Member, Viewer)

### Milestone 4.3: Capture & Inject (MVP)
- [ ] `CaptureAdapter` trait for abstract capture
- [ ] `ClaudeCodeHooksCapture` using stop hooks
- [ ] `ClaudeJsonlParser` with version detection
- [ ] `InjectionAdapter` trait for abstract injection
- [ ] `ClaudeCodeInjector` via CLAUDE.md
- [ ] Session context preparation with scope hierarchy

### Milestone 4.4: Assessment Framework
- [ ] Lightweight assessment (every message, <10ms)
- [ ] Medium assessment (checkpoints, async LLM)
- [ ] Heavy assessment (session end, sampled 20%)
- [ ] `CircuitBreaker` for real-time intervention
- [ ] Outcome signals: token metrics, linguistic patterns, behavioral
- [ ] `SamplingConfig` with burn-in and boost conditions

### Milestone 4.5: Learning Extraction
- [ ] Transcript parser for Claude JSONL format
- [ ] Error recovery pattern extraction
- [ ] User correction detection
- [ ] `Embedder` trait with local/API implementations (hybrid)
- [ ] Semantic search via CozoDB HNSW index

### Milestone 4.6: Attribution Engine
- [ ] Layer 1: Activation detection (embedding similarity)
- [ ] Layer 2: Temporal correlation (signal proximity)
- [ ] Layer 3: Ablation testing (causal impact)
- [ ] Layer 4: Value aggregation (multi-source estimation)
- [ ] Negative learning detection and deprecation
- [ ] `Attribution` storage schema

### Milestone 4.7: Adaptive Strategies
- [ ] `InjectionStrategy` enum (MainContext, Subagent, BackgroundSubagent, Deferred)
- [ ] `StrategyLearner` with Thompson sampling
- [ ] Subagent injection support
- [ ] Outcome-based parameter updates

### Milestone 4.8: groove Dashboard
- [ ] `LearningOverview`, `SessionTrends`, `AttributionInsights` data models
- [ ] API endpoints for `vibes groove` CLI commands
- [ ] Session quality trend visualization
- [ ] Learning list with filtering and attribution
- [ ] Real-time `◉ groove: learning...` indicator
- [ ] System health metrics

### Milestone 4.9: Open-World Adaptation
- [ ] `NoveltyDetector` for unknown patterns
- [ ] `PatternFingerprint` for known/unknown classification
- [ ] `AnomalyCluster` for grouping similar unknowns
- [ ] `CapabilityGapDetector` for surfacing limitations
- [ ] Emergent pattern discovery and notification
- [ ] Meta-learning metrics

**Phase 4 Deliverable:** groove - a self-improving system that finds your coding rhythm

```
◉ groove: You're in the groove. 47 learnings applied this session.
```

---

## Phase 5: Polish & Ecosystem

**Goal:** Production-ready with setup wizards, default plugins, and mobile apps

### Milestone 5.1: Setup Wizards
- [ ] Interactive `vibes tunnel setup` wizard
- [ ] Interactive `vibes auth setup` wizard
- [ ] Guide through cloudflared installation
- [ ] Auto-detect team/AUD from tunnel config
- [ ] Test connectivity and validation

### Milestone 5.2: Default Plugins
- [ ] analytics (session stats, token usage)
- [ ] templates (prompt templates/snippets)
- [ ] export (session export to markdown/JSON)

### Milestone 5.3: CLI Enhancements
- [ ] Tab completion
- [ ] Interactive session picker

### Milestone 5.4: iOS App
- [ ] Swift native app with xterm.js WebView
- [ ] Push notification integration
- [ ] Session list and attach
- [ ] Structured data display (from hooks)

**Phase 5 Deliverable:** Feature-rich vibes with mobile access

---

## Future Phases

These phases are planned but not yet scheduled.

### Phase F1: Android App
- [ ] Kotlin native app with terminal WebView
- [ ] Push notification integration
- [ ] Play Store distribution

### Phase F2: Native GUIs
- [ ] macOS: Cocoa/AppKit via objc2
- [ ] Windows: Win32/WinUI via windows-rs
- [ ] Linux: GTK via gtk-rs
- [ ] Menu bar/system tray integration

### Phase F3: Licensing System
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
| 2025-12-27 | Milestone 3.3 (CLI ↔ Web Mirroring) implementation complete - InputSource enum, source column in messages table, SubscribeAck with history catch-up, CLI remote input display with [Web UI]: prefix, InputHistory struct for arrow key navigation, Web UI source attribution badges |
| 2025-12-27 | Test infrastructure added - Integration tests (in-process WebSocket, concurrency, history catch-up), E2E tests (Playwright smoke tests), CI workflow updated for E2E |
| 2025-12-27 | Milestone 3.4 (PTY Backend) design complete - Replace PrintModeBackend with PTY wrapper, xterm.js web UI, Claude hooks for structured data, auto-configure hooks on daemon start |
| 2025-12-27 | Roadmap updated: PTY Backend promoted to Milestone 3.4, iOS App moved to Phase 4, setup wizards consolidated to Milestone 3.5 |
| 2025-12-27 | Milestone 3.4 (PTY Backend) implementation complete - portable-pty for PTY sessions, xterm.js web UI, Claude hooks receiver with auto-install, raw terminal mode CLI, deprecated legacy protocol messages |
| 2025-12-28 | Phase 3 marked complete (deliverable achieved with PTY Backend) |
| 2025-12-28 | Continual Learning design complete - comprehensive design for vibes-learning plugin with harness introspection, adaptive parameters, open-world adaptation |
| 2025-12-28 | Roadmap reorganized: New Phase 4 (Continual Learning) with 6 milestones (L0-L3), Setup Wizards moved to Phase 5, old Phase 4 becomes Phase 5 |
| 2025-12-28 | Continual Learning design expanded: 10 milestones (4.1-4.9 + 4.2.5), added Assessment Framework, Attribution Engine, Security Architecture, Observability Dashboard |
