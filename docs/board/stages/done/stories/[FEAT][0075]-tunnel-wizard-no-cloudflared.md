---
id: FEAT0075
title: Tunnel Wizard - No Cloudflared
type: feat
status: done
priority: medium
scope: networking
---

# Tunnel Wizard - No Cloudflared

Handle the case when cloudflared is not installed.

## Context

Users who don't have cloudflared installed shouldn't see a cryptic error. The wizard should detect this, explain the situation, and provide platform-specific installation instructions.

## Acceptance Criteria

- [x] Detect when cloudflared is not installed
- [x] Show clear message explaining cloudflared is required
- [x] Show platform-specific install instructions:
  - macOS: `brew install cloudflared`
  - Linux: Link to Cloudflare docs
  - Windows: `winget install Cloudflare.cloudflared`
- [x] Exit gracefully (exit code 0, not an error)
- [x] Tell user to run `vibes tunnel setup` again after installing

## Technical Notes

```rust
fn show_install_instructions() -> Result<()> {
    println!("cloudflared is required for tunnel functionality.\n");
    println!("Install it using one of these methods:\n");
    println!("  macOS:    brew install cloudflared");
    println!("  Linux:    See https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation");
    println!("  Windows:  winget install Cloudflare.cloudflared");
    println!("\nAfter installing, run 'vibes tunnel setup' again.");
    Ok(())
}
```

Detect platform using `std::env::consts::OS` to highlight the relevant command.

## Size

S - Small (detection + message display)
