---
id: feat-0074
title: Tunnel Wizard - Named Mode
type: feat
status: done
priority: medium
epics: [cli, networking]
milestone: 35-setup-wizards
---

# Tunnel Wizard - Named Mode

Implement named tunnel setup with login, tunnel selection, and DNS routing.

## Context

Named tunnels provide persistent hostnames (e.g., vibes.yourdomain.com). This requires a Cloudflare account, logging in, creating/selecting a tunnel, and optionally routing DNS. The wizard guides users through this multi-step process.

## Acceptance Criteria

- [x] Detect login status, prompt login if not logged in
- [x] After login, list existing tunnels with "Create new tunnel" option
- [x] If creating new tunnel:
  - Prompt for tunnel name (validate: alphanumeric + hyphens)
  - Run `cloudflared tunnel create <name>`
- [x] Prompt for hostname (validate: contains dot, not starting with dot)
- [x] Prompt: "Route DNS to this tunnel? (Recommended)" with Confirm
- [x] If yes, run `cloudflared tunnel route dns <tunnel> <hostname>`
- [x] Save config:
  ```toml
  [tunnel]
  enabled = true
  mode = "named"
  name = "<tunnel-name>"
  hostname = "<hostname>"
  ```
- [x] Show success message with next steps

## Technical Notes

```rust
async fn setup_named_tunnel(state: CloudflaredState) -> Result<()> {
    if !state.logged_in {
        println!("Named tunnel requires Cloudflare login.");
        println!("Opening browser for authentication...\n");
        run_login()?;
    }

    let tunnels = list_tunnels().await?;
    let tunnel = select_or_create_tunnel(&tunnels).await?;

    let hostname: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the hostname for this tunnel")
        .interact_text()?;

    // ... rest of flow
}
```

## Size

M - Medium (multiple prompts, subprocess calls, validation)
