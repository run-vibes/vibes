# vibes - Product Requirements Document

> Vibe coding swiss army knife of enhancements

## Overview

vibes is a utility application that supercharges vibe coding workflows. It provides a shared Rust core used by CLI, native GUI, and web interfaces across Linux, macOS, and Windows. The initial focus is proxying Claude Code on the CLI with remote access capabilities, followed by a plugin ecosystem with free and paid extensions.

## Goals

1. **Enhance Claude Code** - Proxy all Claude Code functionality while adding remote access, session management, and extensibility
2. **Universal access** - Control coding sessions from any device (phone, tablet, laptop) via web UI
3. **Extensible architecture** - Plugin system that allows extending vibes with new commands, UI, and workflows
4. **Cross-platform** - Single codebase supporting Linux, macOS, Windows with native experiences

## Non-Goals (Current Phase)

- Licensing/payment system (Phase F4)
- Native GUI applications (Phase F3)
- Native mobile apps (Phase F2)
- Alternative Claude Code interaction models (Phase F1)

---

## Architecture

### High-Level Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                    vibes (single binary)                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     vibes-core                               ││
│  │  (always loaded - sessions, plugins, event bus, server)     ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│         ┌────────────────────┼────────────────────┐             │
│         ▼                    ▼                    ▼             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐       │
│  │  CLI Mode   │     │  GUI Mode   │     │ Server Mode │       │
│  │  (default)  │     │  (native)   │     │ (HTTP/WS)   │       │
│  └─────────────┘     └─────────────┘     └─────────────┘       │
│                                                                  │
│  Cargo features:                                                 │
│  --features cli        (default, always included)               │
│  --features gui        (adds native GUI, increases binary)      │
│  --features server     (adds web UI serving)                    │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
              ┌─────────────────────────┐
              │      Claude Code        │
              │   (subprocess, stdio)   │
              └─────────────────────────┘
```

### Key Principles

- **vibes-core** is a Rust library containing all business logic
- CLI, GUI, and web UI are thin shells that consume vibes-core
- Claude Code runs as a subprocess; vibes captures and broadcasts its I/O
- Plugins extend vibes-core, not the UI layers
- Event bus enables real-time mirroring to multiple connected clients

### Crate Structure

```
vibes/
├── Cargo.toml              # Workspace root
├── vibes-core/             # Core library
│   └── src/
│       ├── lib.rs
│       ├── pty/            # PTY management (Claude as persistent terminal)
│       ├── hooks/          # Claude Code hooks (structured event capture)
│       ├── events/         # Pub/sub event system
│       ├── plugins/        # Plugin loading & lifecycle
│       ├── auth/           # Cloudflare Access JWT validation
│       ├── tunnel/         # Cloudflare Tunnel integration
│       ├── notifications/  # Web Push support
│       └── error.rs        # Error types
├── vibes-server/           # HTTP/WebSocket server (axum)
│   └── src/
│       ├── lib.rs
│       ├── http/           # REST API routes
│       ├── ws/             # WebSocket protocol
│       ├── middleware/     # Auth middleware
│       └── state.rs        # Shared AppState
├── vibes-cli/              # CLI binary
│   └── src/
│       ├── main.rs
│       ├── commands/       # CLI commands (auth, claude, config, plugin, serve, tunnel)
│       ├── client/         # WebSocket client to daemon
│       ├── daemon/         # Auto-start logic
│       └── config/         # Configuration loading
├── vibes-plugin-api/       # Published crate for plugin authors
├── vibes-introspection/    # Harness detection and capability discovery
├── vibes-groove/           # Continual learning system (Phase 4)
├── web-ui/                 # TanStack frontend (embedded via rust-embed)
└── examples/plugins/       # Example plugins
```

---

## Core Components

### Session Manager

Wraps Claude Code subprocess and manages lifecycle.

```rust
pub struct Session {
    id: String,                              // Claude's session UUID
    name: Option<String>,                    // Human-friendly name
    process: Child,
    state: SessionState,
    event_tx: broadcast::Sender<VibesEvent>,
}

pub enum SessionState {
    Starting,
    Running,
    WaitingForInput,
    WaitingForPermission(ToolRequest),
    Completed,
    Failed(String),
}
```

### Event Bus

Central pub/sub for all system events. Enables session mirroring across clients.

```rust
pub enum VibesEvent {
    // From Claude Code
    ClaudeOutput(StreamMessage),
    ClaudeStateChange(SessionState),
    PermissionRequest(ToolRequest),

    // From any client (CLI, GUI, remote)
    UserInput(String),
    PermissionResponse(bool),

    // System
    PluginLoaded(PluginInfo),
    ServerClientConnected(ClientId),
}

pub struct EventBus {
    tx: broadcast::Sender<VibesEvent>,
}
```

### Plugin Host

Loads and manages native Rust plugins.

```rust
// Plugin loading paths (in order of precedence)
// ~/.vibes/plugins/          # User-level (shared across projects)
// ./.vibes/plugins/          # Project-level (per-repo)

pub fn load_plugin(path: &Path) -> Result<Box<dyn Plugin>>;
```

### Server

HTTP + WebSocket server for remote access.

```rust
pub struct VibesServer {
    event_bus: EventBus,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    plugins: Arc<RwLock<PluginRegistry>>,
    auth: Box<dyn AuthAdapter>,
}
```

---

## CLI Interface

### Commands

```
vibes [OPTIONS] <COMMAND>

Commands:
  claude    Proxy Claude Code with vibes enhancements
  serve     Run server only (headless mode)
  gui       Launch native GUI (when available)
  plugin    Manage plugins
  config    Manage configuration
  update    Update vibes to the latest version
  auth      Authentication setup (Phase 2)
  tunnel    Cloudflare Tunnel management (Phase 2)

Global Options:
  --no-serve          Disable background server
  --port <PORT>       Server port [default: 7432]
  --config <PATH>     Config file path
  -v, --verbose       Verbose output
```

### vibes claude

The core command - proxies Claude Code with all its flags plus vibes additions.

```bash
# All claude flags pass through identically
vibes claude "query"
vibes claude -c                          # Continue last session
vibes claude -r <session>                # Resume specific session
vibes claude --model claude-opus-4-5
vibes claude --allowedTools "Bash,Read"

# Vibes-specific additions
vibes claude --no-serve                  # Disable server for this session
vibes claude --session-name "refactor"   # Human-friendly session name
vibes claude --notify                    # Push notification on completion (Phase 2)
```

**Key behavior:** `vibes claude` always starts the server by default. If you don't want the server, use `claude` directly.

### vibes plugin

```bash
vibes plugin list                    # List installed plugins
vibes plugin install <path|url>      # Install a plugin
vibes plugin remove <name>           # Remove a plugin
vibes plugin enable <name>           # Enable a plugin
vibes plugin disable <name>          # Disable without removing
```

### vibes update

```bash
vibes update                         # Update to latest stable version
vibes update --check                 # Check for updates without installing
vibes update --version <version>     # Update to specific version
```

---

## Plugin System

### Plugin Trait

```rust
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
    fn subscriptions(&self) -> Vec<EventFilter> { vec![] }
    fn on_event(&mut self, event: &VibesEvent, ctx: &mut PluginContext) -> Result<()> { Ok(()) }
}

pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: PluginLicense,
    pub commands: Vec<CommandSpec>,
    pub settings: Vec<SettingSpec>,
}

pub enum PluginLicense {
    Free,
    Paid { product_id: String },
    Trial { days: u32 },
}
```

### Plugin Capabilities

```rust
pub struct PluginContext<'a> {
    // Register new CLI commands
    pub fn register_command(&mut self, spec: CommandSpec, handler: CommandHandler);

    // Access event bus (read/write)
    pub fn event_bus(&self) -> &EventBus;

    // Access current session
    pub fn session(&self) -> Option<&Session>;

    // Plugin configuration
    pub fn config(&self) -> &PluginConfig;
    pub fn set_config(&mut self, key: &str, value: Value);

    // HTTP route registration (for web UI extensions)
    pub fn register_route(&mut self, route: Route);

    // WebSocket message handlers
    pub fn register_ws_handler(&mut self, msg_type: &str, handler: WsHandler);
}
```

### Plugin Loading

Plugins are native Rust dynamic libraries (.so/.dylib/.dll) loaded from:
1. `~/.vibes/plugins/` - User-level (shared across projects)
2. `./.vibes/plugins/` - Project-level (can override user-level)

```rust
// Export macro for plugin authors
vibes_plugin::export_plugin!(MyPlugin);
```

---

## Server & Web UI

### HTTP Endpoints

```
GET  /api/health                    # Health check
GET  /api/sessions                  # List active sessions
GET  /api/sessions/:id              # Session details
POST /api/sessions/:id/input        # Send input to session
POST /api/sessions/:id/permission   # Approve/deny permission request
GET  /api/plugins                   # List plugins
POST /api/plugins/:name/enable      # Enable plugin
POST /api/plugins/:name/disable     # Disable plugin
```

### WebSocket Protocol

```typescript
// Client -> Server
{ type: "subscribe", sessions: ["abc123"] }
{ type: "input", session_id: "abc123", content: "user prompt" }
{ type: "permission", session_id: "abc123", approved: true }

// Server -> Client
{ type: "claude_output", session_id: "abc123", data: { /* stream-json */ } }
{ type: "permission_request", session_id: "abc123", tool: "Bash", command: "npm test" }
{ type: "session_state", session_id: "abc123", state: "thinking" }
```

### Web UI Stack

- **Framework:** TanStack (Router, Query)
- **Bundling:** Embedded in binary via rust-embed
- **Modes:**
  - **Simple mode:** Approve/deny prompts, quick input, recent activity
  - **Full mode:** Complete streaming output like a terminal

### Auth Adapter Interface

```rust
#[async_trait]
pub trait AuthAdapter: Send + Sync {
    async fn validate_request(&self, req: &Request) -> Result<AuthResult>;
    async fn get_login_url(&self) -> Option<String>;
}

// Implementations
pub struct NoAuth;                    // Phase 1: Local network only
pub struct CloudflareAccessAuth;      // Phase 2: Cloudflare Access
pub struct DevicePairingAuth;         // Future: QR code pairing
```

---

## Phased Roadmap

### Phase 1: Foundation (MVP)

**Goal:** `vibes claude` works, plugin system functional, local web UI accessible

#### Milestone 1.1: Core proxy
- vibes-core crate with Session, EventBus
- Claude Code subprocess management
- Stream-json parsing and event broadcasting
- Basic error handling and recovery

#### Milestone 1.2: CLI
- vibes claude pass-through (all claude flags work)
- --session-name support
- vibes config basics
- Server auto-start

#### Milestone 1.3: Plugin foundation
- Plugin trait and API crate (vibes-plugin-api)
- Dynamic library loading
- Plugin lifecycle (load, unload, enable, disable)
- vibes plugin CLI commands
- Event subscription system

#### Milestone 1.4: Server + Web UI
- axum HTTP/WebSocket server
- TanStack web UI with session view
- Permission approve/deny flow
- Simple mode + full mode toggle
- rust-embed for bundling UI

**Deliverable:** Single binary that proxies Claude Code with web UI on localhost

---

### Phase 2: Remote Access

**Goal:** Access vibes from anywhere securely

#### Milestone 2.1: Cloudflare Tunnel integration
- vibes tunnel setup wizard
- cloudflared process management
- Tunnel status in UI
- Auto-reconnect handling

#### Milestone 2.2: Cloudflare Access auth
- AuthAdapter implementation
- JWT validation
- Login flow redirect
- Session binding to identity

#### Milestone 2.3: Push notifications
- --notify flag
- Web push subscription
- Notification on completion/error/permission-needed
- Mobile-friendly notification actions

**Deliverable:** Access vibes from phone anywhere with Cloudflare auth

---

### Phase 3: Multi-Client Experience ✓

**Goal:** Full multi-client support with real-time sessions

**Status:** Complete

#### Milestone 3.1: Chat History
- Persistent session history storage (SQLite with FTS5)
- Session search and filtering
- Replay previous sessions from any client
- History pagination for large session counts

#### Milestone 3.2: Multi-Session Support
- Multiple concurrent Claude sessions on same server
- Session list view in Web UI with status indicators
- Session isolation (events/input per session)
- Graceful session cleanup on disconnect

#### Milestone 3.3: CLI ↔ Web Mirroring
- Real-time bidirectional sync between CLI and Web UI
- Any connected client can send input to active session
- Late-joiner catches up with full session history
- Input source attribution (show who typed what)

#### Milestone 3.4: PTY Backend
- Replace stream-json backend with PTY wrapper for full CLI parity
- xterm.js web UI terminal emulator
- Claude hooks integration for structured data capture
- Auto-configure hooks on daemon start

**Deliverable:** Full CLI parity with real-time multi-client sessions ✓

---

### Phase 4: vibes groove ◉

> **groove** - The continual learning system that finds your coding rhythm.

**Goal:** Progressive improvement through accumulated experience - zero friction, fully adaptive.

**Design:** See [vibes groove Design](plans/14-continual-learning/design.md) and [Branding Guide](groove/BRANDING.md)

#### Milestone 4.1: Harness Introspection ✓
- `vibes-introspection` crate with public API
- `Harness` trait and `ClaudeCodeHarness` implementation
- Cross-platform support (Windows, macOS, Linux)
- `HarnessCapabilities` with 3-tier scope hierarchy

#### Milestone 4.2: Storage Foundation (In Progress)
- CozoDB setup with schema and migrations
- `Learning` model with UUIDv7 identifiers
- `LearningStorage` trait and CozoDB implementation
- `AdaptiveParam` with Bayesian update mechanics

#### Milestone 4.2.5: Security Foundation ✓
- `TrustLevel` enum (Local → Quarantined hierarchy)
- `ContentSecurityScanner` for injection detection
- `SecureInjector` with trust-aware wrapping
- RBAC, audit logging, quarantine workflow

#### Milestone 4.3-4.9: (See PROGRESS.md)
- Capture & Inject MVP
- Assessment Framework
- Learning Extraction
- Attribution Engine
- Adaptive Strategies
- groove Dashboard
- Open-World Adaptation

**Deliverable:** groove - a self-improving system that finds your coding rhythm

---

### Phase 5: Polish & Ecosystem

**Goal:** Production-ready with setup wizards, default plugins, and mobile apps

#### Milestone 5.1: Setup Wizards
- Interactive `vibes tunnel setup` wizard
- Interactive `vibes auth setup` wizard
- Guide through cloudflared installation
- Auto-detect team/AUD from tunnel config

#### Milestone 5.2: Default plugins
- analytics (session stats, token usage)
- templates (prompt templates/snippets)
- export (session export to markdown/JSON)

#### Milestone 5.3: CLI Enhancements
- Tab completion
- Interactive session picker

#### Milestone 5.4: iOS App
- Swift native app with xterm.js WebView
- Push notification integration
- Session list and attach

**Deliverable:** Feature-rich vibes with mobile access

---

### Future Phases

#### Phase F1: Android App
- Kotlin native app with terminal WebView
- Push notification integration
- Play Store distribution

#### Phase F2: Native GUIs

True native desktop applications.

- macOS: Cocoa/AppKit via objc2
- Windows: Win32/WinUI via windows-rs
- Linux: GTK via gtk-rs
- Shared core, platform-specific UI layer
- Menu bar/system tray integration

#### Phase F3: Licensing System

Paid plugin support and license management.

- License validation adapter interface
- Plugin license checking
- Grace periods and offline validation
- License server integration
- Paid plugin distribution mechanism

---

## Architectural Decision Records

### ADR-001: Claude Code Interaction Model

**Status:** Revised — Migrated from Option A to Option B+C (PTY + Hooks) in Milestone 3.4

**Context:** Claude Code's `-p` flag exits after one response. We needed multi-turn sessions with structured output.

**Original Decision (Phase 1):** Use multiple `-p` invocations with `--session-id` for session continuity (PrintModeBackend).

**Revised Decision (Phase 3):** PTY wrapper with Claude hooks for structured data capture. Implemented in Milestone 3.4.

**Rationale for migration:** PTY backend provides full CLI parity including colors, interactive prompts, and real-time output. Claude hooks provide structured event data (tool calls, completions) without parsing terminal output.

**Options evaluated:**

| Option | Approach | Status |
|--------|----------|--------|
| A | Multiple `-p` + session-id (PrintModeBackend) | Deprecated - Phase 1-2 |
| B+C ✓ | PTY wrapper + Hooks | **Implemented** - Phase 3 |
| D | `--input-format stream-json` | Not pursued |

**Current architecture:** Claude runs as a PTY process. vibes receives structured events via Claude Code hooks (stop hooks), providing best of both worlds: full terminal experience + structured data.

---

### ADR-002: Plugin Execution Model

**Status:** Decided

**Decision:** Native Rust plugins loaded as dynamic libraries (.so/.dylib/.dll)

**Rationale:** Maximum performance, tight integration with vibes-core. Plugin authors need Rust toolchain but get full power.

**Alternatives considered:**
- WASM: Sandboxed but performance overhead, more accessible
- Subprocess (MCP-style): Most flexible, most overhead
- Embedded scripting (Lua/Rhai): Easy to write, limited capabilities

---

### ADR-003: GUI Strategy

**Status:** Decided

**Decision:** True native GUIs using platform libraries (Cocoa, Win32, GTK) with shared Rust core.

**Rationale:** Best platform feel, though more development effort per platform.

**Alternatives considered:**
- Tauri: Web frontend, smaller binaries, but not truly native feel
- egui/Iced: Pure Rust, but less polished native experience

---

### ADR-004: Authentication Architecture

**Status:** Decided (adapter pattern)

**Decision:** Auth adapter trait with Cloudflare Access as initial implementation.

**Rationale:** Cloudflare Access integrates naturally with Cloudflare Tunnel. Adapter pattern allows future providers.

**Future adapters:**
- Device pairing (QR code)
- Self-hosted token
- OAuth providers

---

### ADR-005: Single Binary Architecture

**Status:** Decided

**Decision:** One binary with cargo features controlling included functionality.

```toml
[features]
default = ["cli", "server"]
cli = []
server = ["axum", "tower", "rust-embed"]
gui = ["platform-specific-deps"]
full = ["cli", "server", "gui"]
```

**Rationale:** Single install, single update path. Users can compile minimal versions if needed.

---

### ADR-006: Session Mirroring

**Status:** Decided

**Decision:** True real-time mirrored sessions where local terminal and remote devices see the same stream and can both inject input.

**Rationale:** Full remote control is the core value proposition. Mirroring provides the most powerful user experience.

**Implementation:** Pub/sub architecture via EventBus. All clients subscribe to session events, any client can send input.

---

### ADR-007: Event Bus Architecture

**Status:** Decided

**Context:** The EventBus needs to support real-time broadcasting, late-joiner replay, and future extensibility (persistence, distribution).

**Decision:** Adapter pattern separating the EventBus trait from implementations. MVP uses in-memory implementation; can swap to persistent or distributed backends later.

**Architecture:**
```
EventBus (trait)
├── publish(event)
├── subscribe() -> Stream<Event>
├── subscribe_from(seq) -> Stream<Event>  // replay from sequence number
└── get_events(session_id) -> Vec<Event>

Implementations:
├── MemoryEventBus (MVP)
│   ├── Vec<Event> for replay
│   └── tokio::broadcast for real-time
├── SqliteEventBus (future - persistence)
└── DistributedEventBus (future - Redis Streams, NATS, Iggy)
```

**Rationale:**
- Late joiner support is essential (web UI opens mid-session)
- Adapter pattern allows swapping implementations without API changes
- Opens path to distributed vibes (multiple machines sharing events)
- In-memory MVP is simple; persistence can wait until needed

**Alternatives considered:**
- Pure tokio::broadcast: No replay, late joiners miss events
- Iggy/Kafka from day one: Overkill for single-machine tool, adds operational complexity
- SQLite from day one: Reasonable, but adds complexity before we need persistence

**Revisit trigger:** When crash recovery or cross-machine distribution becomes a requirement.

---

### ADR-008: Claude Backend Abstraction

**Status:** Revised — PtyBackend is now the active implementation (Milestone 3.4)

**Context:** We needed to swap interaction backends without rewriting Session/EventBus logic. The abstraction enabled migration from PrintModeBackend to PtyBackend.

**Decision:** Abstract Claude interaction behind PTY manager in `vibes-core/src/pty/`. Claude runs as a long-running PTY process with structured events captured via Claude Code hooks.

**Current Architecture:**
- `PtyManager` in `vibes-core/src/pty/` manages Claude as a PTY process
- `HookReceiver` in `vibes-core/src/hooks/` captures structured Claude events
- Raw terminal I/O passed to CLI and Web UI (xterm.js)
- Session lifecycle tied to PTY process

**Implementation History:**
| Backend | Process Model | Status |
|---------|---------------|--------|
| PrintModeBackend | Per-turn spawn | Deprecated (Phase 1-2) |
| **PtyBackend + Hooks** | Long-running PTY | **Active** (Phase 3+) |
| MockBackend | Scripted responses | Testing only |

**Key design points:**
- PTY provides full terminal parity (colors, raw mode, interactive prompts)
- Claude hooks provide structured events (tool calls, completions)
- MockBackend enables unit testing without real Claude process
- Consistent adapter pattern with EventBus (ADR-007)

**Outcome:** Successfully migrated in Milestone 3.4. PrintModeBackend and stream-json parser removed.

---

### ADR-009: Plugin API Versioning Strategy

**Status:** Decided

**Context:** Rust plugins are compiled against specific struct layouts and trait definitions. ABI changes between vibes versions can cause plugins to crash or behave incorrectly. We need a versioning strategy that protects users from incompatible plugins.

**Decision:** Start with strict version matching; migrate to semver once API stabilizes.

**Phase 1 (MVP):** Strict matching
- Plugin declares `api_version: u32` in manifest
- vibes refuses to load plugins where `api_version != vibes_plugin_api::API_VERSION`
- Any API change increments the version, requiring plugin rebuild
- Clear error message: "Plugin 'foo' requires API v2, but vibes has v3. Please rebuild."

**Phase 2 (Post-stabilization):** Semver compatibility
- Switch to `api_version: "1.2.0"` (semver string)
- Major version must match
- Minor version additions are backwards-compatible
- Plugins built against 1.2.0 work with vibes 1.3.0, not 2.0.0

**Migration trigger:** When the plugin API has been stable for 3+ releases and we have confidence in backwards compatibility guarantees.

**Rationale:**
- ABI breakage causes hard-to-debug crashes
- Strict matching is simple to implement and reason about
- Plugin authors get clear "rebuild needed" signals
- Semver adds complexity we don't need until API stabilizes

**Alternatives considered:**
- Semver from day one: Harder to maintain ABI stability during rapid iteration
- No versioning: Poor UX when plugins crash mysteriously
- abi_stable crate: Adds dependency, learning curve; overkill for MVP

### ADR-010: Server + Web UI Architecture

**Status:** Decided

**Context:** To enable remote session monitoring (the core value prop: "start from terminal, control from phone"), we need a server architecture that can serve multiple clients. Key decisions: where does session state live, how do clients communicate, and how is the web UI served.

**Decision:** Daemon-based architecture with WebSocket-first communication.

**Architecture choices:**

1. **Daemon owns all state:** The server process owns SessionManager, EventBus, and PluginHost. CLI becomes a client rather than directly spawning Claude processes.

2. **WebSocket for CLI and Web UI:** Both use the same WebSocket protocol. This guarantees feature parity—anything the web UI can do, the CLI can do.

3. **Auto-start daemon:** Running `vibes claude` automatically starts the daemon if not running. Users don't manage server lifecycle manually.

4. **SPA with embedded assets:** TanStack Router/Query frontend built to static assets, embedded in binary via rust-embed. Single binary deployment.

5. **Harness-prefixed URLs:** Routes like `/claude/:id` allow future extension to other AI backends (Codex, Gemini).

**Rationale:**
- Daemon model enables multiple simultaneous clients
- Single WebSocket protocol reduces maintenance burden
- Auto-start preserves simple CLI UX (`vibes claude "prompt"` just works)
- Embedded assets enable single binary distribution
- TanStack provides type-safe routing and smart data fetching

**Alternatives considered:**
- Direct Claude spawning from CLI: Can't share sessions with web UI
- REST API for CLI: Polling less efficient than WebSocket streaming
- Separate web server binary: Complicates deployment
- Server-side rendering: Unnecessary complexity for this use case

---

### ADR-011: Auth Middleware Architecture

**Status:** Decided

**Context:** vibes needs to authenticate requests coming through Cloudflare Tunnel while allowing unauthenticated local access for development convenience.

**Decision:** Implement auth as an axum middleware layer that:
1. Checks if request source is localhost → skip auth
2. Otherwise, validate Cloudflare Access JWT from header/cookie
3. Attach AuthContext (Local, Authenticated, or Anonymous) to request

**Key design points:**
- **Localhost bypass:** Requests from 127.0.0.1/localhost skip authentication entirely
- **Cloudflare handles login:** Unauthenticated tunnel requests are redirected by CF Access before reaching vibes
- **Identity display:** Show authenticated user's email in Web UI header (no persistence)

**Rationale:**
- Middleware pattern cleanly separates auth from business logic
- Localhost bypass enables frictionless local development
- Relying on Cloudflare for login redirect keeps vibes simple

**Full design:** See [Milestone 2.2 Design](plans/06-cloudflare-access/design.md)

---

### ADR-012: JWT Validation Strategy

**Status:** Decided

**Context:** Cloudflare Access sends JWTs that must be validated against Cloudflare's public keys. Keys rotate every 6 weeks.

**Decision:** Implement JWT validation with JWKS caching:
1. Extract JWT from `Cf-Access-Jwt-Assertion` header or `CF_Authorization` cookie
2. Decode header to get `kid` (key ID)
3. Fetch JWKS from `https://<team>.cloudflareaccess.com/cdn-cgi/access/certs` (cached 1 hour)
4. Validate signature, `aud` claim, and expiry (with 60s clock skew leeway)

**Configuration:**
- **Auto-detect:** Team name and AUD derived from tunnel config when possible
- **Fallback:** Manual configuration via `vibes auth setup` wizard

**Rationale:**
- JWKS caching reduces latency and Cloudflare API load
- Automatic refresh on unknown `kid` handles key rotation seamlessly
- Clock skew leeway prevents false rejections

**Full design:** See [Milestone 2.2 Design](plans/06-cloudflare-access/design.md)

---

### ADR-013: Push Notification Architecture

**Status:** Decided

**Context:** Users need to be notified when Claude sessions complete, fail, or require permission approval - especially when accessing vibes remotely from a phone or when multitasking on desktop.

**Decision:** Use Web Push API with auto-generated VAPID keys and deep links for notification actions.

**Key choices:**

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Delivery mechanism | Web Push API only | Single implementation covers mobile + desktop browsers |
| VAPID keys | Auto-generate on first run | Zero setup friction, stored in vibes config |
| Notification actions | Deep link to Web UI | Universal browser support, leverages existing permission UI |
| Default behavior | All events notify | Opt-out model for remote monitoring use case |

**Architecture:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Push Notification Flow                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────────────────┐ │
│  │   Browser    │     │    vibes     │     │   Push Service           │ │
│  │  (Web UI)    │     │   server     │     │  (FCM/Mozilla/Apple)     │ │
│  └──────┬───────┘     └──────┬───────┘     └────────────┬─────────────┘ │
│         │                    │                          │               │
│         │ 1. Enable notifs   │                          │               │
│         │ ──────────────────>│                          │               │
│         │                    │                          │               │
│         │ 2. Subscribe       │                          │               │
│         │    (VAPID pubkey)  │                          │               │
│         │<───────────────────│                          │               │
│         │                    │                          │               │
│         │ 3. PushSubscription│                          │               │
│         │    {endpoint, keys}│                          │               │
│         │ ──────────────────>│                          │               │
│         │                    │ 4. Store subscription    │               │
│         │                    │    (in memory + file)    │               │
│         │                    │                          │               │
│         │    ═══════════════ Later: Event occurs ═══════════════       │
│         │                    │                          │               │
│         │                    │ 5. POST to endpoint      │               │
│         │                    │    (signed with VAPID)   │               │
│         │                    │ ────────────────────────>│               │
│         │                    │                          │               │
│         │                    │                          │ 6. Push       │
│         │<───────────────────────────────────────────────│               │
│         │                    │                          │               │
│         │ 7. Service Worker  │                          │               │
│         │    shows notif     │                          │               │
│         │                    │                          │               │
│         │ 8. User clicks     │                          │               │
│         │    → Open Web UI   │                          │               │
└─────────────────────────────────────────────────────────────────────────┘
```

**Components:**

| Component | Location | Responsibility |
|-----------|----------|----------------|
| VapidKeyManager | vibes-core | Generate/load VAPID keypair |
| SubscriptionStore | vibes-core | Store push subscriptions (file-backed) |
| NotificationService | vibes-core | Send push notifications on events |
| Service Worker | web-ui | Receive pushes, display notifications |
| NotificationSettings | web-ui | UI for enabling/configuring notifications |

**Notification events (all on by default):**
- Permission needed → "Claude needs approval" with deep link to permission UI
- Session completed → "Session finished" with deep link to session view
- Session error → "Session failed" with deep link to error details

**Rationale:**
- Web Push is the standard, works across all modern browsers including mobile
- Auto-generated VAPID keys eliminate setup friction for users
- Deep links are universally supported vs. notification actions which have inconsistent browser support
- All-on-by-default matches the "remote monitoring" use case where users want full visibility

**Alternatives considered:**
- Native OS notifications (notify-rust): Would require separate implementation, doesn't work for remote access
- Third-party services (Pushover, ntfy): Adds external dependencies and configuration burden
- Notification action buttons: Inconsistent mobile browser support, complex service worker logic

**Full design:** See [Milestone 2.3 Design](plans/07-push-notifications/design.md)

---

## Technical Notes

### Claude Code Integration

Key flags for programmatic use:
```bash
claude -p "query"                    # Print mode (exits after response)
--output-format stream-json          # Newline-delimited JSON streaming
--session-id "uuid"                  # Session continuity across invocations
--allowedTools "Bash,Read,Edit"      # Pre-approve tools
--input-format stream-json           # Accept streaming JSON input (investigate)
```

Session files stored in `~/.claude/` with UUIDs.

### Remote Access via Cloudflare

- **Cloudflare Tunnel:** `cloudflared` daemon creates outbound-only connections
- **Cloudflare Access:** Identity layer with SSO (Google, GitHub, etc.)
- Both require Cloudflare account

### Web UI Embedding

TanStack frontend built to `web-ui/dist/`, embedded via rust-embed:
```rust
#[derive(RustEmbed)]
#[folder = "web-ui/dist/"]
struct WebAssets;
```

---

## Open Questions

1. **Stream-json bidirectional:** Does `--input-format stream-json` work without `-p` for long-running interactive sessions? Needs testing.

2. **Permission UX:** How to handle permission requests when multiple clients are connected? First responder wins? Require specific client?

3. **Plugin distribution:** How will users discover and install third-party plugins? Registry? Git URLs?

4. **Offline support:** Should web UI work offline with service workers? Or always require server connection?
