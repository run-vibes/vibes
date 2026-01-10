---
id: feat-0072
title: Cloudflared State Detection
type: feat
status: backlog
milestone: 35-setup-wizards
---

# Cloudflared State Detection

Detect cloudflared installation, login status, and list existing tunnels.

## Context

The tunnel setup wizard needs to understand the current state of cloudflared on the system: Is it installed? Is the user logged in? What tunnels exist? This story builds the detection layer that the wizard will use.

## Acceptance Criteria

- [ ] Create `vibes-cli/src/commands/setup/cloudflared.rs`
- [ ] `CloudflaredState` struct with:
  - `installed: bool`
  - `version: Option<String>`
  - `logged_in: bool`
  - `cert_path: Option<PathBuf>`
- [ ] `CloudflaredState::detect()` async function that:
  - Checks if cloudflared is installed (via `cloudflared --version`)
  - Checks for `~/.cloudflared/cert.pem` to determine login status
- [ ] `ExistingTunnel` struct with id, name, created_at, connections
- [ ] `list_tunnels()` parses JSON from `cloudflared tunnel list --output json`
- [ ] `create_tunnel(name)` runs `cloudflared tunnel create`
- [ ] `run_login()` runs `cloudflared login` interactively
- [ ] `route_dns(tunnel, hostname)` runs `cloudflared tunnel route dns`
- [ ] Unit tests with mocked command output

## Technical Notes

Use `tokio::process::Command` for async subprocess handling. Parse JSON output:

```rust
const TUNNEL_LIST_JSON: &str = r#"[
    {"id": "abc123", "name": "vibes-home", "created_at": "2026-01-05T10:00:00Z", "connections": 2}
]"#;

#[test]
fn parse_tunnel_list() {
    let tunnels: Vec<ExistingTunnel> = serde_json::from_str(TUNNEL_LIST_JSON).unwrap();
    assert_eq!(tunnels[0].name, "vibes-home");
}
```

## Size

M - Medium (multiple functions, JSON parsing, async)
