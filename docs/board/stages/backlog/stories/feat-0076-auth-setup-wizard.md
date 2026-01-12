---
id: feat-0076
title: Auth Setup Wizard
type: feat
status: pending
priority: medium
epics: [cli]
milestone: 35-setup-wizards
---

# Auth Setup Wizard

Implement auth setup wizard with team/AUD prompts and JWKS validation.

## Context

Cloudflare Access protects vibes with SSO authentication. Users need to configure their team name and Application AUD tag. This wizard guides them through the process, provides instructions for creating an Access application, and validates the configuration by fetching JWKS.

## Acceptance Criteria

- [ ] Add `AuthCommand::Setup` variant to auth.rs
- [ ] Create `vibes-cli/src/commands/setup/auth_wizard.rs`
- [ ] `vibes auth setup` runs the wizard
- [ ] Show header and prerequisites explanation
- [ ] Ask if user has created Cloudflare Access application
- [ ] If no, show step-by-step instructions for creating one
- [ ] Prompt for team name (validate: alphanumeric + hyphens)
- [ ] Prompt for AUD tag (validate: 32+ chars, alphanumeric)
- [ ] Test configuration by fetching JWKS from `{team}.cloudflareaccess.com`
- [ ] Save config:
  ```toml
  [auth]
  enabled = true
  team = "<team>"
  aud = "<aud>"
  bypass_localhost = true
  clock_skew_seconds = 60
  ```
- [ ] Show success message

## Technical Notes

```rust
// Test JWKS before saving
let config = AccessConfig {
    enabled: true,
    team: team.clone(),
    aud: aud.clone(),
    bypass_localhost: true,
    clock_skew_seconds: 60,
};

print_step(&format!("Fetching JWKS from: {}", config.jwks_url()));
let validator = JwtValidator::new(config.clone());
validator.refresh_jwks().await?;
println!("✓ JWKS fetched successfully");
```

## Size

M - Medium (prompts, validation, JWKS test, instructions)
