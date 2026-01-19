---
id: FEAT0076
title: Auth Setup Wizard
type: feat
status: done
priority: medium
scope: networking
---

# Auth Setup Wizard

Implement auth setup wizard with team/AUD prompts and JWKS validation.

## Context

Cloudflare Access protects vibes with SSO authentication. Users need to configure their team name and Application AUD tag. This wizard guides them through the process, provides instructions for creating an Access application, and validates the configuration by fetching JWKS.

## Acceptance Criteria

- [x] Add `AuthCommand::Setup` variant to auth.rs
- [x] Create `vibes-cli/src/commands/setup/auth_wizard.rs`
- [x] `vibes auth setup` runs the wizard
- [x] Show header and prerequisites explanation
- [x] Ask if user has created Cloudflare Access application
- [x] If no, show step-by-step instructions for creating one
- [x] Prompt for team name (validate: alphanumeric + hyphens)
- [x] Prompt for AUD tag (validate: 32+ chars, alphanumeric)
- [x] Test configuration by fetching JWKS from `{team}.cloudflareaccess.com`
- [x] Save config:
  ```toml
  [auth]
  enabled = true
  team = "<team>"
  aud = "<aud>"
  bypass_localhost = true
  clock_skew_seconds = 60
  ```
- [x] Show success message

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
