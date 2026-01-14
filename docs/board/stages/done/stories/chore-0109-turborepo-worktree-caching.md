# chore-0109: Turborepo Worktree Caching

## Status
done

## Problem

New git worktrees fail to build because `web-ui/dist/` doesn't exist. The error "missing web-ui/dist/" blocks Rust builds since rust-embed compiles the dist directory into the server binary.

Previously, Rust artifacts were shared via `$CARGO_TARGET_DIR=~/.cargo-target/vibes/` and binaries copied to worktree-local `./target/debug/`. Web-ui had no equivalent caching mechanism.

## Solution

Added Turborepo to the npm workspace for content-aware build caching.

### How It Works

1. Turbo hashes source files to determine if rebuild is needed
2. Build outputs cached in `$HOME/.cache/turbo/vibes` (shared across worktrees)
3. Post-checkout hook runs `turbo run build --filter=vibes-web-ui`
4. Cache hit: instant restoration (~80ms)
5. Cache miss: prints warning, user runs `just build`

### Cache Locations (XDG-compliant)

| Cache | Location |
|-------|----------|
| Rust artifacts | `$HOME/.cache/cargo-target/vibes` |
| Turbo cache | `$HOME/.cache/turbo/vibes` |

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

### 1. turbo.json

```json
{
  "$schema": "https://turbo.build/schema.json",
  "globalDependencies": [],
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "inputs": ["$TURBO_DEFAULT$", "!.turbo/**"],
      "outputs": ["dist/**"]
    },
    "@vibes/design-system#build": {
      "inputs": ["$TURBO_DEFAULT$", "!.turbo/**"],
      "outputs": ["build/**"]
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

Key details:
- `!.turbo/**` excludes log files from input hash (prevents cache invalidation)
- `@vibes/design-system#build` uses package-specific syntax (no leading `#`)
- design-system outputs to `build/` (Ladle), web-ui outputs to `dist/` (Vite)

### 2. package.json

```json
{
  "packageManager": "npm@10.9.4",
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

Note: `packageManager` field is required by Turbo v2.

### 3. flake.nix (shared cache directories)

```bash
export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/vibes"
export TURBO_CACHE_DIR="$HOME/.cache/turbo/vibes"
```

### 4. .githooks/post-checkout

```bash
# Restore web-ui/dist from turbo cache if missing
if [[ ! -d "web-ui/dist" ]] && [[ -f "turbo.json" ]]; then
    echo "Restoring web-ui from turbo cache..."
    if command -v npx &> /dev/null; then
        if npx turbo run build --filter=vibes-web-ui --output-logs=errors-only 2>/dev/null; then
            if [[ -d "web-ui/dist" ]]; then
                echo "web-ui/dist restored from cache"
            else
                echo "web-ui/dist not cached, run 'just build' to build"
            fi
        else
            echo "web-ui/dist not cached, run 'just build' to build"
        fi
    fi
fi
```

### 5. .gitignore

Added `.turbo/` to gitignore (local logs and run summaries).

## Benefits

- **Speed**: Cache hits restore dist in ~80ms vs ~15-20s rebuild
- **Consistency**: Mirrors Rust shared cache pattern
- **Correctness**: Content-aware hashing knows when rebuild is actually needed
- **XDG-compliant**: Both caches now in `$HOME/.cache/`

## Acceptance Criteria

- [x] `turbo.json` configured with correct pipelines
- [x] Root `package.json` scripts use turbo
- [x] Post-checkout hook restores web-ui/dist from cache
- [x] New worktree can build Rust without manual web-ui build
- [x] `just web build` works as before (transparent change)
- [x] Cache shared across all worktrees via `$TURBO_CACHE_DIR`
