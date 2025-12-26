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
│       ├── session/        # Claude Code subprocess management
│       ├── events/         # Pub/sub event system
│       ├── plugins/        # Plugin loading & lifecycle
│       ├── server/         # HTTP/WebSocket server
│       └── config/         # Configuration management
├── vibes-plugin-api/       # Published crate for plugin authors
├── vibes-cli/              # CLI binary
├── web-ui/                 # TanStack frontend (embedded)
└── plugins/                # Default plugins
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

### Phase 3: Polish & Ecosystem

**Goal:** Production-ready with default plugins

#### Milestone 3.1: Default plugins
- analytics (session stats, token usage)
- history (searchable session history)
- templates (prompt templates/snippets)
- export (session export to markdown/JSON)

#### Milestone 3.2: Multiple sessions
- Run multiple Claude sessions concurrently
- Session switcher in UI
- Per-session notification settings

#### Milestone 3.3: CLI enhancements
- vibes sessions list/switch/kill
- Tab completion
- Interactive session picker

**Deliverable:** Feature-rich vibes with useful default plugins

---

### Future Phases

#### Phase F1: Alternative Claude Code Interaction

Revisit the Claude Code interaction model for better performance or capabilities.

- Option B: PTY wrapper for interactive mode
- Option C: Hook-based permission routing
- Option D: stream-json bidirectional (if supported)
- Benchmark and migrate if beneficial

#### Phase F2: Mobile Apps

Native mobile applications for iOS and Android.

- iOS app (Swift, native)
- Android app (Kotlin, native)
- Push notification integration
- App Store/Play Store distribution
- Subscription management

#### Phase F3: Native GUIs

True native desktop applications.

- macOS: Cocoa/AppKit via objc2
- Windows: Win32/WinUI via windows-rs
- Linux: GTK via gtk-rs
- Shared core, platform-specific UI layer
- Menu bar/system tray integration

#### Phase F4: Licensing System

Paid plugin support and license management.

- License validation adapter interface
- Plugin license checking
- Grace periods and offline validation
- License server integration
- Paid plugin distribution mechanism

---

## Architectural Decision Records

### ADR-001: Claude Code Interaction Model

**Status:** Decided (Option A), with alternatives documented for Phase F1

**Context:** Claude Code's `-p` flag exits after one response. We need multi-turn sessions with structured output.

**Decision:** Use multiple `-p` invocations with `--session-id` for session continuity.

**Rationale:** Provides structured JSON output immediately. Per-turn process spawn is acceptable overhead. Permission handling uses `--allowedTools` for pre-approved tools.

**Alternatives preserved:**

| Option | Approach | Pros | Cons |
|--------|----------|------|------|
| A ✓ | Multiple `-p` + session-id | Structured JSON, clean lifecycle | Process spawn per turn, pre-configured permissions |
| B | PTY wrapper (interactive) | True interactive, real-time permissions | Unstructured output, parse terminal codes |
| C | Interactive + hooks | Permission routing works | Still unstructured output |
| D | `--input-format stream-json` | Potential best of both | Needs testing, may not exist |

**Revisit trigger:** If process spawn overhead becomes noticeable, or if permission handling becomes painful.

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

**Status:** Decided

**Context:** ADR-001 chose `-p` mode for MVP but preserved alternatives (PTY, hooks, stream-json bidirectional). We need to swap interaction backends without rewriting Session/EventBus logic.

**Decision:** Abstract Claude interaction behind a `ClaudeBackend` trait. Each backend handles its own process lifecycle, output parsing, and input mechanism, emitting normalized events.

**Architecture:**
```rust
#[async_trait]
pub trait ClaudeBackend: Send + Sync {
    async fn send(&mut self, input: &str) -> Result<()>;
    fn events(&self) -> broadcast::Receiver<ClaudeEvent>;
    async fn respond_permission(&mut self, request_id: &str, approved: bool) -> Result<()>;
    fn claude_session_id(&self) -> &str;
    fn state(&self) -> BackendState;
    async fn shutdown(&mut self) -> Result<()>;
}

pub enum BackendState {
    Idle,                              // Ready for input
    Processing,                        // Claude working
    WaitingPermission(PermissionRequest),
    Finished,
    Failed(String),
}
```

**Implementations:**
| Backend | Process Model | Output Format | MVP? |
|---------|---------------|---------------|------|
| PrintModeBackend | Per-turn spawn | stream-json | ✓ |
| PtyBackend | Long-running | Terminal codes | Future |
| StreamJsonBackend | Long-running | stream-json bidirectional | Investigate |

**Key design points:**
- Backend parses its native format → normalized `ClaudeEvent`
- Session never sees raw stream-json or terminal codes
- "Turns" are internal to PrintModeBackend, not exposed in trait
- Consistent adapter pattern with EventBus (ADR-007)

**Rationale:**
- Enables backend swapping without API changes
- Mock backend for testing
- Future API-direct backend possible
- A/B testing backends for performance

**Revisit trigger:** When investigating PTY mode (Phase F1) or if stream-json bidirectional proves viable.

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
