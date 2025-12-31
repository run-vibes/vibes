# Milestone 4.5: Iggy Bundling - Design Document

> Bundle Apache Iggy as the default persistent event storage backend, automatically managed as invisible infrastructure.

## Overview

vibes currently uses `InMemoryEventLog` which loses all events on restart. For the continual learning system (groove) to work effectively, we need persistent event storage that survives restarts and supports multi-process access.

Apache Iggy is a high-performance message streaming platform written in Rust. Rather than treating it as an external dependency users must install, we bundle it directly with vibes so persistent storage "just works" out of the box.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Plugin vs Built-in** | Built-in | Core infrastructure, not optional functionality |
| **Integration method** | Git submodule | Bundles with releases, shares toolchain, version controlled |
| **Runtime behavior** | Auto-start as subprocess | Invisible to user, managed by IggyManager |
| **Manual management** | None required | No `just iggy-start` commands; fully automatic |
| **Fallback** | InMemoryEventLog | For tests and when Iggy unavailable |

---

## Architecture

### Auto-Start Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                        vibes claude                                  │
│                             │                                        │
│                             ▼                                        │
│              ┌──────────────────────────────┐                        │
│              │     Daemon Auto-Start        │                        │
│              │   (existing behavior)        │                        │
│              └──────────────┬───────────────┘                        │
│                             │                                        │
│                             ▼                                        │
│              ┌──────────────────────────────┐                        │
│              │       vibes serve            │                        │
│              │           │                  │                        │
│              │           ▼                  │                        │
│              │   ┌───────────────────┐      │                        │
│              │   │   IggyManager     │      │                        │
│              │   │                   │      │                        │
│              │   │ • Check if running│      │                        │
│              │   │ • Spawn if not    │      │                        │
│              │   │ • Connect client  │      │                        │
│              │   └─────────┬─────────┘      │                        │
│              │             │                │                        │
│              │             ▼                │                        │
│              │   ┌───────────────────┐      │                        │
│              │   │   iggy-server     │◄─────┼── Subprocess           │
│              │   │   (persistent)    │      │                        │
│              │   └───────────────────┘      │                        │
│              └──────────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Overview

| Component | Location | Responsibility |
|-----------|----------|----------------|
| `IggyManager` | vibes-iggy | Subprocess lifecycle, health checks |
| `IggyEventLog` | vibes-iggy | EventLog implementation using Iggy client |
| `iggy-server` | vendor/iggy | The actual message streaming server |
| `AppState` | vibes-server | Holds EventLog, decides backend |

---

## Submodule Structure

```
vibes/
├── vendor/
│   └── iggy/                    # git submodule → github.com/apache/iggy
│       ├── server/              # iggy-server binary crate
│       ├── sdk/                 # iggy client SDK (we already use this)
│       ├── Cargo.toml           # Iggy's workspace root
│       └── target/              # Separate from our target/
├── vibes-iggy/                  # Our wrapper around iggy
│   └── src/
│       ├── manager.rs           # IggyManager (already exists)
│       └── iggy_log.rs          # IggyEventLog (already exists)
├── vibes-server/
│   └── src/
│       └── state.rs             # Switch from InMemory to Iggy
└── Cargo.toml                   # Our workspace (does NOT include vendor/iggy)
```

**Key insight**: Cargo doesn't support nested workspaces. Iggy has its own workspace, so we build it separately and copy the binary.

---

## Build Strategy

### Separate Builds, Unified Output

Since we can't nest workspaces, we build Iggy separately:

```bash
# Build vibes (our workspace)
cargo build --release

# Build iggy-server (their workspace)
cargo build --release --manifest-path vendor/iggy/Cargo.toml -p iggy-server

# Copy to our target/ for unified output
cp vendor/iggy/target/release/iggy-server target/release/
```

This is encapsulated in `just build-all`.

### Developer Workflow

```bash
# Clone with submodules
git clone --recursive https://github.com/run-vibes/vibes
cd vibes
direnv allow

# Build everything (one command)
just build-all

# Run vibes - iggy starts automatically
./target/release/vibes claude
```

If submodules weren't cloned:
```bash
git submodule update --init --recursive
```

---

## Runtime Behavior

### IggyManager Responsibilities

The existing `IggyManager` handles:

1. **Startup**: Check if iggy-server is running; spawn if not
2. **Health**: Periodic health checks, restart if crashed
3. **Shutdown**: Graceful shutdown when vibes-server stops
4. **Configuration**: Data directory, ports, credentials

### Server Integration

```rust
// vibes-server/src/lib.rs (conceptual)

impl VibesServer {
    pub async fn run(self) -> Result<(), ServerError> {
        // Start Iggy if configured (default: true)
        let event_log = if self.config.use_iggy {
            let manager = IggyManager::new(IggyConfig::default());
            manager.ensure_running().await?;
            Arc::new(manager.create_event_log().await?) as Arc<dyn EventLog<VibesEvent>>
        } else {
            Arc::new(InMemoryEventLog::new()) as Arc<dyn EventLog<VibesEvent>>
        };

        // ... rest of startup
    }
}
```

### Data Storage

Following XDG Base Directory conventions:

```
~/.config/vibes/             # Configuration
├── plugins/                 # Plugin configs
└── config.toml              # Main config (future)

~/.local/share/vibes/        # Application data
└── iggy/                    # Iggy server data
    ├── streams/             # Message streams
    ├── users/               # User database
    └── server.log           # Iggy logs
```

---

## Configuration

### Default Behavior (No Config Required)

| Setting | Default | Notes |
|---------|---------|-------|
| Backend | Iggy | Auto-start subprocess |
| Data directory | `~/.local/share/vibes/iggy` | Per-user persistent storage (XDG) |
| TCP port | 8090 | Iggy default |
| Username/password | vibes/vibes | Local only, not exposed |

### Override via Environment (Optional)

```bash
VIBES_EVENT_BACKEND=memory vibes serve    # Force in-memory
VIBES_IGGY_DATA_DIR=/custom/path vibes serve
```

---

## Failure Modes

| Scenario | Behavior |
|----------|----------|
| Iggy fails to start | Log warning, fall back to InMemoryEventLog |
| Iggy crashes mid-session | IggyManager restarts it, reconnects |
| Port 8090 in use | Try alternative ports or fail with clear error |
| Disk full | Iggy handles gracefully, events may be lost |

---

## CI/Release Integration

### GitHub Actions Changes

```yaml
- uses: actions/checkout@v4
  with:
    submodules: recursive    # Clone submodules

- name: Build all
  run: just build-all        # Builds vibes + iggy-server

- name: Package release
  run: |
    tar -czvf vibes-${{ matrix.target }}.tar.gz \
      -C target/release vibes vibes-server iggy-server
```

### Release Artifacts

Each release includes:
- `vibes` - Main CLI
- `vibes-server` - Daemon (usually auto-started)
- `iggy-server` - Message streaming (auto-started by daemon)

---

## Documentation Updates

### README.md

Add to Quick Start:
```markdown
## Quick Start

```bash
git clone --recursive https://github.com/run-vibes/vibes
cd vibes && direnv allow
just build-all
./target/release/vibes claude
```

Persistent event storage is automatic - no additional setup required.
```

### CLAUDE.md

Add section:
```markdown
## Git Submodules

This project uses a git submodule for bundled dependencies:

- `vendor/iggy` - Apache Iggy message streaming server

### First-time setup

```bash
git clone --recursive https://github.com/run-vibes/vibes
# or if you forgot --recursive:
git submodule update --init --recursive
```

### Building

```bash
just build-all   # Builds vibes + iggy-server
```

### Updating Iggy

```bash
cd vendor/iggy
git fetch --tags
git checkout v0.5.0
cd ../..
git add vendor/iggy
git commit -m "chore: update iggy to v0.5.0"
```
```

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage |
|-----------|---------------|
| IggyManager | Subprocess spawn, health check, shutdown |
| IggyEventLog | Append, consume, offset tracking |
| Fallback logic | Switch to memory when Iggy unavailable |

### Integration Tests

| Test | Description |
|------|-------------|
| `test_iggy_auto_start` | Server starts Iggy subprocess |
| `test_iggy_persistence` | Events survive server restart |
| `test_iggy_fallback` | Falls back to memory gracefully |

---

## Deliverables

### Milestone 4.5 Checklist

**Submodule Setup:**
- [ ] Add iggy as git submodule at `vendor/iggy`
- [ ] Pin to stable release tag
- [ ] Update `.gitignore` for iggy artifacts

**Build Integration:**
- [ ] Add `just build-iggy` command
- [ ] Add `just build-all` command
- [ ] Add submodule check to build commands
- [ ] Update `just build-release` to include iggy

**Runtime Integration:**
- [ ] Update `IggyManager` to find bundled binary
- [ ] Update `vibes-server` to auto-start Iggy
- [ ] Configure default data directory
- [ ] Add fallback to InMemoryEventLog

**CI/Release:**
- [ ] Update GitHub Actions for submodule checkout
- [ ] Update release workflow to bundle iggy-server
- [ ] Test release artifacts include all binaries

**Documentation:**
- [ ] Update README.md with submodule instructions
- [ ] Update CLAUDE.md with dev workflow
- [ ] Update this design doc with completion status
- [ ] Update PROGRESS.md

---

## Version Pinning Strategy

Pin to the latest stable Iggy release. As of writing:
- **Tag**: `v0.4.300`
- **Compatibility**: Works with iggy SDK 0.6.x (which we use)

Update process:
1. Review Iggy changelog for breaking changes
2. Update submodule to new tag
3. Run full test suite
4. Update SDK version if needed
5. Commit and create PR
