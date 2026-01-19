---
id: FEAT0078
title: Connectivity Validation
type: feat
status: done
priority: medium
scope: networking
---

# Connectivity Validation

Add end-to-end validation after tunnel setup to verify configuration works.

## Context

After saving config, users want confidence that everything works. This story adds optional validation that runs the actual vibes server with tunnel, makes an HTTP request to the public URL, and verifies the web UI is accessible.

## Acceptance Criteria

- [x] After tunnel setup, offer to test connectivity
- [x] Start actual `vibes serve --tunnel` binary as subprocess
- [x] For quick tunnel: parse URL from output, HTTP GET to verify web UI
- [x] For named tunnel: wait for connection, HTTP GET to configured hostname, verify DNS resolves
- [x] Verify response contains vibes web UI (not Cloudflare error page)
- [x] Show clear success/failure message with URL
- [x] Clear error messages for common failures with troubleshooting suggestions

## Design

### E2E Validation Approach

**Quick tunnel:**
1. Spawn `vibes serve --tunnel` subprocess
2. Parse stdout for quick tunnel URL (`https://xxx.trycloudflare.com`)
3. HTTP GET to URL, verify response contains vibes UI
4. Kill subprocess, report result

**Named tunnel:**
1. Spawn `vibes serve --tunnel` subprocess
2. Wait for "connection registered" in output
3. HTTP GET to configured hostname
4. Verify response contains vibes UI (not Cloudflare error)
5. Kill subprocess, report result

### Binary Discovery

Follow existing pattern from `vibes-iggy/src/config.rs`:

```rust
fn find_vibes_binary() -> Option<PathBuf> {
    // 1. Worktree-local target (compile-time VIBES_WORKSPACE_ROOT)
    let workspace_root = env!("VIBES_WORKSPACE_ROOT");
    for profile in ["debug", "release"] {
        let binary = PathBuf::from(workspace_root)
            .join("target").join(profile).join("vibes");
        if binary.exists() { return Some(binary); }
    }

    // 2. CARGO_TARGET_DIR (shared cache)
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        for profile in ["debug", "release"] {
            let binary = PathBuf::from(&target_dir)
                .join(profile).join("vibes");
            if binary.exists() { return Some(binary); }
        }
    }

    // 3. PATH lookup
    which::which("vibes").ok()
}
```

### Error Handling

| Failure | Detection | Troubleshooting |
|---------|-----------|-----------------|
| Server fails to start | Process exits early | Check if port in use: `lsof -i :8080` |
| Tunnel doesn't connect | No URL/connection in 30s | Check login: `cloudflared tunnel list` |
| HTTP request fails | Connection refused/timeout | Check firewall settings |
| Cloudflare error page | Response contains error markers | Verify DNS routing in dashboard |
| Wrong content | 200 but not vibes UI | Another service on hostname |

### Files to Change

| File | Change |
|------|--------|
| `vibes-cli/src/commands/setup/connectivity.rs` | New - E2E test logic |
| `vibes-cli/src/commands/setup/mod.rs` | Add `mod connectivity` |
| `vibes-cli/src/commands/setup/tunnel_wizard.rs` | Call test after save |
| `vibes-cli/Cargo.toml` | Add `reqwest` if needed |

### User Flow

```
✓ Tunnel configured for quick mode!

? Test tunnel connectivity now? (y/n) y

Testing tunnel connectivity...
  Starting vibes server...
  Tunnel URL: https://xxx.trycloudflare.com
  Verifying web UI...

✓ Tunnel connected successfully!
  Your vibes server is accessible at: https://xxx.trycloudflare.com
```

## Size

M - Medium (subprocess management, HTTP client, timeout handling, error diagnostics)
