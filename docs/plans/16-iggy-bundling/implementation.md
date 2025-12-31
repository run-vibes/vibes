# Milestone 4.5: Iggy Bundling - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bundle Apache Iggy so persistent event storage works out of the box with zero user configuration.

**Architecture:** Git submodule at `vendor/iggy`, built separately, auto-started by vibes-server.

**Tech Stack:** Git submodules, Cargo workspaces (separate), IggyManager subprocess management.

---

## Task 1: Add Iggy Submodule

**Files:**
- Create: `.gitmodules`
- Create: `vendor/iggy/` (submodule)
- Modify: `.gitignore`

**Step 1: Add the submodule**

```bash
git submodule add https://github.com/apache/iggy vendor/iggy
```

**Step 2: Pin to stable release**

```bash
cd vendor/iggy
git fetch --tags
git checkout v0.4.300
cd ../..
```

**Step 3: Update .gitignore**

Add to `.gitignore`:
```
# Iggy submodule build artifacts
vendor/iggy/target/

# Iggy local data (dev)
.vibes/
```

**Step 4: Verify structure**

```bash
ls vendor/iggy/Cargo.toml  # Should exist
ls vendor/iggy/server/     # Should exist
```

**Step 5: Commit**

```bash
git add .gitmodules vendor/iggy .gitignore
git commit -m "chore: add iggy as git submodule at v0.4.300"
```

---

## Task 2: Add Build Commands to Justfile

**Files:**
- Modify: `justfile`

**Step 1: Add submodule check helper**

```just
# Check that submodules are initialized
_check-submodules:
    #!/usr/bin/env bash
    if [[ ! -f vendor/iggy/Cargo.toml ]]; then
        echo "Error: Git submodules not initialized."
        echo "Run: git submodule update --init --recursive"
        exit 1
    fi
```

**Step 2: Add iggy build command**

```just
# Build iggy-server from submodule
build-iggy: _check-submodules
    cargo build --release --manifest-path vendor/iggy/Cargo.toml -p iggy-server
    mkdir -p target/release
    cp vendor/iggy/target/release/iggy-server target/release/
    @echo "✓ iggy-server built and copied to target/release/"
```

**Step 3: Add unified build-all command**

```just
# Build everything (vibes + iggy)
build-all: build-web _check-submodules
    cargo build --release
    cargo build --release --manifest-path vendor/iggy/Cargo.toml -p iggy-server
    cp vendor/iggy/target/release/iggy-server target/release/
    @echo "✓ Built: vibes, vibes-server, iggy-server"
```

**Step 4: Add setup command**

```just
# Full setup for new developers
setup: setup-hooks
    #!/usr/bin/env bash
    set -euo pipefail

    # Initialize submodules if needed
    if [[ ! -f vendor/iggy/Cargo.toml ]]; then
        echo "Initializing git submodules..."
        git submodule update --init --recursive
    fi

    # Install npm deps
    npm ci

    echo "✓ Setup complete. Run 'just build-all' to build."
```

**Step 5: Test the commands**

```bash
just build-iggy
ls target/release/iggy-server  # Should exist
```

**Step 6: Commit**

```bash
git add justfile
git commit -m "feat: add just commands for iggy build and setup"
```

---

## Task 3: Update IggyManager to Find Bundled Binary

**Files:**
- Modify: `vibes-iggy/src/manager.rs`
- Modify: `vibes-iggy/src/config.rs`

**Step 1: Update IggyConfig with binary path resolution**

The manager needs to find `iggy-server` in:
1. Same directory as vibes binary (release/installed)
2. `target/release/iggy-server` (development)
3. PATH (fallback)

```rust
impl IggyConfig {
    /// Find the iggy-server binary.
    pub fn find_binary(&self) -> Option<PathBuf> {
        // 1. Explicit path in config
        if let Some(ref path) = self.binary_path {
            if path.exists() {
                return Some(path.clone());
            }
        }

        // 2. Same directory as current executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let sibling = dir.join("iggy-server");
                if sibling.exists() {
                    return Some(sibling);
                }
            }
        }

        // 3. Check PATH
        if let Ok(path) = which::which("iggy-server") {
            return Some(path);
        }

        None
    }
}
```

**Step 2: Update IggyManager to use find_binary**

Update `ensure_running()` to use the resolved binary path.

**Step 3: Add which dependency**

```toml
# vibes-iggy/Cargo.toml
[dependencies]
which = "6"
```

**Step 4: Write tests**

```rust
#[test]
fn test_find_binary_in_same_dir() {
    // Create temp dir with mock iggy-server
    // Verify find_binary returns it
}
```

**Step 5: Run tests**

```bash
cargo test -p vibes-iggy find_binary
```

**Step 6: Commit**

```bash
git add vibes-iggy/
git commit -m "feat(iggy): add binary path resolution for bundled iggy-server"
```

---

## Task 4: Update vibes-server to Auto-Start Iggy

**Files:**
- Modify: `vibes-server/src/state.rs`
- Modify: `vibes-server/src/lib.rs`

**Step 1: Add Iggy to AppState creation**

Update `AppState::new()` to:
1. Try to start IggyManager
2. If successful, use IggyEventLog
3. If failed, fall back to InMemoryEventLog with warning

```rust
impl AppState {
    pub async fn new_with_iggy() -> Self {
        let event_log: Arc<dyn EventLog<VibesEvent>> = match Self::try_start_iggy().await {
            Ok(log) => {
                tracing::info!("Using Iggy for persistent event storage");
                Arc::new(log)
            }
            Err(e) => {
                tracing::warn!("Failed to start Iggy, using in-memory storage: {}", e);
                Arc::new(InMemoryEventLog::new())
            }
        };

        Self {
            event_log,
            // ... other fields
        }
    }

    async fn try_start_iggy() -> Result<IggyEventLog, Error> {
        let config = IggyConfig::default();
        let manager = IggyManager::new(config);
        manager.ensure_running().await?;
        manager.create_event_log().await
    }
}
```

**Step 2: Update server startup**

In `VibesServer::run()`, use the async constructor:

```rust
let state = Arc::new(AppState::new_with_iggy().await);
```

**Step 3: Configure default data directory**

Use XDG data directory (`~/.local/share/vibes/iggy`):

```rust
impl Default for IggyConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .map(|d| d.join("vibes/iggy"))
            .unwrap_or_else(|| PathBuf::from(".vibes/iggy"));

        Self {
            data_dir,
            tcp_port: 8090,
            // ...
        }
    }
}
```

**Step 4: Run server and verify**

```bash
just build-all
./target/release/vibes serve &
sleep 2
pgrep -a iggy-server  # Should show iggy running
pkill vibes
```

**Step 5: Commit**

```bash
git add vibes-server/ vibes-iggy/
git commit -m "feat(server): auto-start iggy for persistent event storage"
```

---

## Task 5: Update GitHub Actions

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml` (if exists)

**Step 1: Add submodule checkout**

```yaml
- uses: actions/checkout@v4
  with:
    submodules: recursive
```

**Step 2: Cache iggy build**

```yaml
- name: Cache Iggy build
  uses: actions/cache@v4
  with:
    path: vendor/iggy/target
    key: iggy-${{ runner.os }}-${{ hashFiles('vendor/iggy/Cargo.lock') }}
```

**Step 3: Update build step**

```yaml
- name: Build
  run: just build-all
```

**Step 4: Update release packaging (if applicable)**

```yaml
- name: Package release
  run: |
    tar -czvf vibes-${{ matrix.target }}.tar.gz \
      -C target/release vibes vibes-server iggy-server
```

**Step 5: Test CI locally (optional)**

```bash
act -j build  # If using act for local CI testing
```

**Step 6: Commit**

```bash
git add .github/
git commit -m "ci: add submodule checkout and iggy build"
```

---

## Task 6: Update Documentation

**Files:**
- Modify: `README.md`
- Modify: `CLAUDE.md`

**Step 1: Update README.md Quick Start**

Add/update the Quick Start section:

```markdown
## Quick Start

```bash
# Clone with submodules
git clone --recursive https://github.com/run-vibes/vibes
cd vibes

# Enter dev environment (Nix)
direnv allow

# Build everything
just build-all

# Run
./target/release/vibes claude
```

Persistent event storage is automatic - no additional setup required.
```

**Step 2: Add submodule section to CLAUDE.md**

```markdown
## Git Submodules

This project uses a git submodule for Apache Iggy (message streaming):

```
vendor/iggy/  → github.com/apache/iggy
```

### First-Time Setup

```bash
# If you cloned without --recursive:
git submodule update --init --recursive
```

### Building

```bash
just build-all   # Builds vibes + iggy-server
```

The `iggy-server` binary is copied to `target/release/` alongside vibes.

### Updating Iggy

```bash
cd vendor/iggy
git fetch --tags
git checkout v0.5.0  # New version
cd ../..
git add vendor/iggy
git commit -m "chore: update iggy to v0.5.0"
```

### How It Works

When `vibes serve` starts, it automatically:
1. Looks for `iggy-server` next to the vibes binary
2. Spawns it as a subprocess if not running
3. Connects using the Iggy client SDK
4. Falls back to in-memory storage if Iggy unavailable
```

**Step 3: Commit**

```bash
git add README.md CLAUDE.md
git commit -m "docs: add submodule and iggy documentation"
```

---

## Task 7: Add Integration Test

**Files:**
- Create: `vibes-server/tests/iggy_integration.rs` or add to existing

**Step 1: Write integration test**

```rust
#[tokio::test]
#[ignore]  // Requires iggy-server binary
async fn test_server_auto_starts_iggy() {
    // Start server
    // Verify iggy-server process exists
    // Append event
    // Stop server
    // Restart server
    // Verify event persisted
}
```

**Step 2: Run test**

```bash
just build-all
cargo test -p vibes-server iggy_integration -- --ignored
```

**Step 3: Commit**

```bash
git add vibes-server/tests/
git commit -m "test(server): add iggy auto-start integration test"
```

---

## Task 8: Update PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Add milestone entry**

Add to the Phase 4 section:
```markdown
### Milestone 4.5: Iggy Bundling
- [x] Add iggy as git submodule
- [x] Add build commands to justfile
- [x] Update IggyManager binary resolution
- [x] Auto-start iggy in vibes-server
- [x] Update CI for submodules
- [x] Update documentation
```

**Step 2: Add changelog entry**

Add to changelog table.

**Step 3: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: mark milestone 4.5 complete"
```

---

## Summary

This implementation:

1. **Bundles Iggy** via git submodule at `vendor/iggy`
2. **Builds automatically** with `just build-all`
3. **Starts automatically** when `vibes serve` runs
4. **Falls back gracefully** to in-memory if Iggy unavailable
5. **Requires zero configuration** from users

Total tasks: 8
Expected commits: 8-10

After completion, developers and users get persistent event storage by default with no additional steps.
