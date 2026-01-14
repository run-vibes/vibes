# chore-0109: Turborepo Worktree Caching

## Status
backlog

## Problem

New git worktrees fail to build because `web-ui/dist/` doesn't exist. The error "missing web-ui/dist/" blocks Rust builds since rust-embed compiles the dist directory into the server binary.

Currently, Rust artifacts are shared via `$CARGO_TARGET_DIR=~/.cargo-target/vibes/` and binaries are copied to worktree-local `./target/debug/`. Web-ui has no equivalent caching mechanism.

## Solution

Add Turborepo to the npm workspace for content-aware build caching.

### How It Works

1. Turbo hashes source files to determine if rebuild is needed
2. Build outputs cached in `~/.turbo` (shared across worktrees automatically)
3. Post-checkout hook runs `turbo run build --filter=web-ui`
4. Cache hit: instant restoration (~100ms)
5. Cache miss: prints warning, user runs `just build`

### Dependency Graph

```
design-system (standalone, source-only export)
       |
       v
    web-ui (depends on @vibes/design-system)
       |
       v
   e2e-tests (tests web-ui)
```

## Implementation

### 1. turbo.json (new file)

```json
{
  "$schema": "https://turbo.build/schema.json",
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**"]
    },
    "#@vibes/design-system#build": {
      "outputs": [".ladle/**"]
    },
    "typecheck": {
      "dependsOn": ["^build"]
    },
    "test": {
      "dependsOn": ["^build"]
    },
    "dev": {
      "cache": false,
      "persistent": true
    }
  }
}
```

### 2. package.json changes

```json
{
  "scripts": {
    "dev": "turbo run dev --filter=web-ui",
    "build": "turbo run build",
    "build:web": "turbo run build --filter=web-ui",
    "typecheck": "turbo run typecheck",
    "test": "turbo run test",
    "test:e2e": "turbo run test --filter=e2e-tests",
    "test:e2e:headed": "npm run test:headed --workspace=e2e-tests"
  },
  "devDependencies": {
    "turbo": "^2"
  }
}
```

### 3. .githooks/post-checkout addition

```bash
# Web-ui dist restoration from turbo cache
if [ "$3" = "1" ]; then
    if [ ! -d "web-ui/dist" ]; then
        echo "Restoring web-ui from turbo cache..."
        if command -v npx &> /dev/null && [ -f "turbo.json" ]; then
            npx turbo run build --filter=web-ui --output-logs=errors-only 2>/dev/null
            if [ -d "web-ui/dist" ]; then
                echo "✓ web-ui/dist restored from cache"
            else
                echo "⚠ web-ui/dist not cached, run 'just build' to build"
            fi
        fi
    fi
fi
```

### 4. .justfiles/web.just updates

Commands remain the same from user perspective (`just web build`), internally use turbo.

## Benefits

- **Speed**: Cache hits restore dist in ~100ms vs ~15-20s rebuild
- **Consistency**: Mirrors Rust shared cache + local copy pattern
- **Correctness**: Content-aware hashing knows when rebuild is actually needed
- **Foundation**: Enables future npm workspace task orchestration

## Acceptance Criteria

- [ ] `turbo.json` configured with correct pipelines
- [ ] Root `package.json` scripts use turbo
- [ ] Post-checkout hook restores web-ui/dist from cache
- [ ] New worktree can build Rust without manual web-ui build
- [ ] `just web build` works as before (transparent change)
- [ ] Cache shared across all worktrees via `~/.turbo`
