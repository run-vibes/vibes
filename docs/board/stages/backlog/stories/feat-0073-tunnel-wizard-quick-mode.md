---
id: feat-0073
title: Tunnel Wizard - Quick Mode
type: feat
status: pending
priority: medium
epics: [cli, networking]
milestone: 35-setup-wizards
---

# Tunnel Wizard - Quick Mode

Implement quick tunnel setup path in the wizard.

## Context

Quick tunnel is the simplest path - it requires no Cloudflare account. Users just run `vibes tunnel setup`, select "Quick tunnel", and config is saved. This is the recommended path for trying vibes.

## Acceptance Criteria

- [ ] Create `vibes-cli/src/commands/setup/tunnel_wizard.rs`
- [ ] `vibes tunnel setup` runs the wizard (update tunnel.rs)
- [ ] Wizard shows header: "VIBES TUNNEL SETUP"
- [ ] Shows cloudflared installation check with version
- [ ] Presents mode selection via dialoguer Select:
  - "Quick tunnel (temporary URL, no account needed)"
  - "Named tunnel (persistent URL, requires Cloudflare account)"
- [ ] Selecting quick saves config:
  ```toml
  [tunnel]
  enabled = true
  mode = "quick"
  ```
- [ ] Shows success message with next steps

## Technical Notes

```rust
use dialoguer::{Select, theme::ColorfulTheme};

let mode = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Which tunnel mode do you want to use?")
    .items(&[
        "Quick tunnel (temporary URL, no account needed)",
        "Named tunnel (persistent URL, requires Cloudflare account)",
    ])
    .default(0)
    .interact()?;
```

## Size

S - Small (single flow, straightforward prompts)
