---
created: 2026-01-01
updated: 2026-01-09
---

# Milestone 35: Setup Wizards - Design

> Interactive setup wizards for tunnel and authentication configuration.

## Problem Statement

Setting up vibes for remote access requires:

1. **Tunnel setup** - Installing cloudflared, logging in, creating tunnels, routing DNS
2. **Auth setup** - Finding team name, AUD tag from Cloudflare Access dashboard
3. **Validation** - Ensuring everything works together

Currently users must:
- Manually run cloudflared commands
- Edit TOML config files by hand
- Hunt for AUD tags in Cloudflare dashboard
- Debug issues without clear feedback

The current commands just print "coming soon":

```rust
fn run_setup() -> Result<()> {
    println!("Tunnel setup wizard - coming soon");
    Ok(())
}
```

## Goals

1. **Zero-friction quick tunnel** - One command to get a working remote URL
2. **Guided named tunnel setup** - Step-by-step wizard for persistent tunnels
3. **Auth configuration** - Simple prompts with validation
4. **Auto-detection** - Discover existing cloudflared state, suggest configs
5. **Connectivity testing** - Verify setup works before saving

## Non-Goals

- Cloudflare API integration (requires API token complexity)
- Creating Cloudflare Access applications (complex, dashboard is better)
- Managing multiple tunnels simultaneously
- GUI/web-based setup

## User Journeys

### Journey 1: Quick Tunnel (New User, Minimal Setup)

```
$ vibes tunnel setup

┌─────────────────────────────────────────────────────────────┐
│ VIBES TUNNEL SETUP                                          │
└─────────────────────────────────────────────────────────────┘

Checking cloudflared installation... Found v2024.12.0

? Which tunnel mode do you want to use?
  > Quick tunnel (temporary URL, no account needed)
    Named tunnel (persistent URL, requires Cloudflare account)

Quick tunnel selected.

Configuration saved to ~/.config/vibes/config.toml

✓ Tunnel setup complete!

Run 'vibes serve --tunnel' to start your server with the tunnel.
```

### Journey 2: Named Tunnel (Persistent Access)

```
$ vibes tunnel setup

┌─────────────────────────────────────────────────────────────┐
│ VIBES TUNNEL SETUP                                          │
└─────────────────────────────────────────────────────────────┘

Checking cloudflared installation... Found v2024.12.0
Checking login status... Not logged in

? Which tunnel mode do you want to use?
    Quick tunnel (temporary URL, no account needed)
  > Named tunnel (persistent URL, requires Cloudflare account)

Named tunnel requires Cloudflare login.
Opening browser for authentication...

[cloudflared login runs interactively]

✓ Logged in successfully

? Select a tunnel:
  > Create new tunnel
    vibes-home (created 2026-01-05)
    work-tunnel (created 2025-11-20)

? Enter name for new tunnel: vibes-laptop
Creating tunnel vibes-laptop... Done

? Enter the hostname for this tunnel (e.g., vibes.yourdomain.com): vibes.alex.dev

? Route DNS to this tunnel? (Recommended)
  > Yes
    No, I'll configure DNS manually

Routing vibes.alex.dev to tunnel vibes-laptop... Done

Configuration saved:
  tunnel.enabled = true
  tunnel.mode = "named"
  tunnel.name = "vibes-laptop"
  tunnel.hostname = "vibes.alex.dev"

✓ Tunnel setup complete!

Run 'vibes serve --tunnel' to start your server with the tunnel.
```

### Journey 3: No cloudflared Installed

```
$ vibes tunnel setup

┌─────────────────────────────────────────────────────────────┐
│ VIBES TUNNEL SETUP                                          │
└─────────────────────────────────────────────────────────────┘

Checking cloudflared installation... Not found

cloudflared is required for tunnel functionality.

Install it using one of these methods:

  macOS:    brew install cloudflared
  Linux:    See https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation
  Windows:  winget install Cloudflare.cloudflared

After installing, run 'vibes tunnel setup' again.
```

### Journey 4: Auth Setup

```
$ vibes auth setup

┌─────────────────────────────────────────────────────────────┐
│ VIBES AUTH SETUP                                            │
└─────────────────────────────────────────────────────────────┘

Cloudflare Access protects your vibes server with SSO authentication.

Prerequisites:
  1. A Cloudflare Access application for your tunnel hostname
  2. Your team name and Application AUD tag

? Have you created a Cloudflare Access application?
  > Yes, I have the details ready
    No, show me how to create one

─────────────────────────────────────────────────────────────
To create a Cloudflare Access application:

  1. Go to: https://one.dash.cloudflare.com
  2. Navigate to: Access > Applications
  3. Click "Add an application" > Self-hosted
  4. Configure:
     - Application name: vibes
     - Session duration: 24 hours (or your preference)
     - Application domain: your tunnel hostname
  5. Add access policies (who can access)
  6. Copy the "Application Audience (AUD) Tag"

Press Enter when ready to continue...
─────────────────────────────────────────────────────────────

? Enter your Cloudflare Access team name
  (The 'xxx' in xxx.cloudflareaccess.com): mycompany

? Enter the Application AUD tag: 32f6...abc1

Testing configuration...
  Fetching JWKS from: https://mycompany.cloudflareaccess.com/cdn-cgi/access/certs
  ✓ JWKS fetched successfully (3 keys)

Configuration saved:
  auth.enabled = true
  auth.team = "mycompany"
  auth.aud = "32f6...abc1"

✓ Auth setup complete!
```

## Technical Design

### Dependencies

Add to `vibes-cli/Cargo.toml`:

```toml
dialoguer = "0.11"  # Interactive prompts (Select, Input, Confirm)
console = "0.15"    # Terminal colors/formatting (transitive dep)
```

### Module Structure

```
vibes-cli/src/
├── commands/
│   ├── tunnel.rs           # Update run_setup() to call wizard
│   ├── auth.rs              # Add AuthCommand::Setup
│   └── setup/               # NEW: Setup wizard module
│       ├── mod.rs
│       ├── prompts.rs       # Reusable prompt helpers
│       ├── cloudflared.rs   # Cloudflared detection/commands
│       ├── tunnel_wizard.rs # Tunnel setup wizard
│       └── auth_wizard.rs   # Auth setup wizard
```

### Cloudflared Detection

```rust
// vibes-cli/src/commands/setup/cloudflared.rs

/// State of cloudflared on this system
pub struct CloudflaredState {
    pub installed: bool,
    pub version: Option<String>,
    pub logged_in: bool,
    pub cert_path: Option<PathBuf>,
}

/// Existing tunnel from cloudflared tunnel list
pub struct ExistingTunnel {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub connections: u32,
}

impl CloudflaredState {
    /// Detect cloudflared installation and login state
    pub async fn detect() -> Self {
        let installed = check_installation().await;
        let (version, logged_in, cert_path) = if installed.is_some() {
            let info = installed.unwrap();
            let cert = Self::find_cert();
            (Some(info.version), cert.is_some(), cert)
        } else {
            (None, false, None)
        };

        Self { installed: version.is_some(), version, logged_in, cert_path }
    }

    fn find_cert() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let cert = home.join(".cloudflared/cert.pem");
        cert.exists().then_some(cert)
    }
}

/// List existing tunnels (requires login)
pub async fn list_tunnels() -> Result<Vec<ExistingTunnel>> {
    let output = Command::new("cloudflared")
        .args(["tunnel", "list", "--output", "json"])
        .output()
        .await?;

    // Parse JSON output
    let tunnels: Vec<ExistingTunnel> = serde_json::from_slice(&output.stdout)?;
    Ok(tunnels)
}

/// Create a new tunnel
pub async fn create_tunnel(name: &str) -> Result<ExistingTunnel> {
    let output = Command::new("cloudflared")
        .args(["tunnel", "create", "--output", "json", name])
        .output()
        .await?;

    let tunnel: ExistingTunnel = serde_json::from_slice(&output.stdout)?;
    Ok(tunnel)
}

/// Run cloudflared login interactively
pub fn run_login() -> Result<()> {
    // This opens browser, must be interactive
    std::process::Command::new("cloudflared")
        .arg("login")
        .status()?;
    Ok(())
}

/// Route DNS for a tunnel
pub async fn route_dns(tunnel_name: &str, hostname: &str) -> Result<()> {
    Command::new("cloudflared")
        .args(["tunnel", "route", "dns", tunnel_name, hostname])
        .status()
        .await?;
    Ok(())
}
```

### Tunnel Wizard

```rust
// vibes-cli/src/commands/setup/tunnel_wizard.rs

use dialoguer::{Select, Input, Confirm, theme::ColorfulTheme};

pub async fn run() -> Result<()> {
    print_header("VIBES TUNNEL SETUP");

    // Step 1: Check cloudflared
    print_step("Checking cloudflared installation...");
    let state = CloudflaredState::detect().await;

    if !state.installed {
        return show_install_instructions();
    }
    println!("Found v{}", state.version.as_ref().unwrap());

    // Step 2: Choose mode
    let mode = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which tunnel mode do you want to use?")
        .items(&[
            "Quick tunnel (temporary URL, no account needed)",
            "Named tunnel (persistent URL, requires Cloudflare account)",
        ])
        .default(0)
        .interact()?;

    match mode {
        0 => setup_quick_tunnel().await,
        1 => setup_named_tunnel(state).await,
        _ => unreachable!(),
    }
}

async fn setup_quick_tunnel() -> Result<()> {
    let config = TunnelConfigSection {
        enabled: true,
        mode: Some("quick".to_string()),
        name: None,
        hostname: None,
    };

    save_tunnel_config(&config)?;
    print_success("Tunnel setup complete!");
    println!("\nRun 'vibes serve --tunnel' to start your server with the tunnel.");
    Ok(())
}

async fn setup_named_tunnel(state: CloudflaredState) -> Result<()> {
    // Step 3: Ensure logged in
    if !state.logged_in {
        println!("\nNamed tunnel requires Cloudflare login.");
        println!("Opening browser for authentication...\n");
        run_login()?;
        println!("\n✓ Logged in successfully\n");
    }

    // Step 4: Select or create tunnel
    let tunnels = list_tunnels().await?;
    let tunnel = select_or_create_tunnel(&tunnels).await?;

    // Step 5: Configure hostname
    let hostname: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the hostname for this tunnel")
        .validate_with(|input: &String| {
            if input.contains('.') && !input.starts_with('.') {
                Ok(())
            } else {
                Err("Please enter a valid hostname (e.g., vibes.yourdomain.com)")
            }
        })
        .interact_text()?;

    // Step 6: Optionally route DNS
    let route = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Route DNS to this tunnel? (Recommended)")
        .default(true)
        .interact()?;

    if route {
        print_step(&format!("Routing {} to tunnel {}...", hostname, tunnel.name));
        route_dns(&tunnel.name, &hostname).await?;
        println!("Done");
    }

    // Step 7: Save config
    let config = TunnelConfigSection {
        enabled: true,
        mode: Some("named".to_string()),
        name: Some(tunnel.name.clone()),
        hostname: Some(hostname),
    };

    save_tunnel_config(&config)?;

    print_success("Tunnel setup complete!");
    println!("\nRun 'vibes serve --tunnel' to start your server with the tunnel.");
    Ok(())
}

async fn select_or_create_tunnel(existing: &[ExistingTunnel]) -> Result<ExistingTunnel> {
    let mut items: Vec<String> = vec!["Create new tunnel".to_string()];
    for t in existing {
        items.push(format!("{} (created {})", t.name, t.created_at));
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a tunnel")
        .items(&items)
        .default(0)
        .interact()?;

    if selection == 0 {
        // Create new
        let name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter name for new tunnel")
            .validate_with(|input: &String| {
                if input.chars().all(|c| c.is_alphanumeric() || c == '-') {
                    Ok(())
                } else {
                    Err("Tunnel name must be alphanumeric with hyphens only")
                }
            })
            .interact_text()?;

        print_step(&format!("Creating tunnel {}...", name));
        let tunnel = create_tunnel(&name).await?;
        println!("Done");
        Ok(tunnel)
    } else {
        Ok(existing[selection - 1].clone())
    }
}
```

### Auth Wizard

```rust
// vibes-cli/src/commands/setup/auth_wizard.rs

pub async fn run() -> Result<()> {
    print_header("VIBES AUTH SETUP");

    println!("Cloudflare Access protects your vibes server with SSO authentication.\n");
    println!("Prerequisites:");
    println!("  1. A Cloudflare Access application for your tunnel hostname");
    println!("  2. Your team name and Application AUD tag\n");

    // Check if they have Access app ready
    let ready = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Have you created a Cloudflare Access application?")
        .default(true)
        .interact()?;

    if !ready {
        show_access_instructions();
        println!("\nPress Enter when ready to continue...");
        std::io::stdin().read_line(&mut String::new())?;
    }

    // Get team name
    let team: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your Cloudflare Access team name\n(The 'xxx' in xxx.cloudflareaccess.com)")
        .validate_with(|input: &String| {
            if input.chars().all(|c| c.is_alphanumeric() || c == '-') {
                Ok(())
            } else {
                Err("Team name must be alphanumeric with hyphens only")
            }
        })
        .interact_text()?;

    // Get AUD
    let aud: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the Application AUD tag")
        .validate_with(|input: &String| {
            if input.len() >= 32 && input.chars().all(|c| c.is_alphanumeric()) {
                Ok(())
            } else {
                Err("AUD should be a 64-character hex string")
            }
        })
        .interact_text()?;

    // Test configuration
    println!("\nTesting configuration...");
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
    println!("✓ JWKS fetched successfully\n");

    // Save config
    save_auth_config(&config)?;

    print_success("Auth setup complete!");
    Ok(())
}

fn show_access_instructions() {
    println!("─────────────────────────────────────────────────────────────");
    println!("To create a Cloudflare Access application:\n");
    println!("  1. Go to: https://one.dash.cloudflare.com");
    println!("  2. Navigate to: Access > Applications");
    println!("  3. Click \"Add an application\" > Self-hosted");
    println!("  4. Configure:");
    println!("     - Application name: vibes");
    println!("     - Session duration: 24 hours (or your preference)");
    println!("     - Application domain: your tunnel hostname");
    println!("  5. Add access policies (who can access)");
    println!("  6. Copy the \"Application Audience (AUD) Tag\"");
    println!("─────────────────────────────────────────────────────────────");
}
```

### Prompt Helpers

```rust
// vibes-cli/src/commands/setup/prompts.rs

use console::style;

pub fn print_header(title: &str) {
    let width = 60;
    let border = "─".repeat(width);
    println!("┌{}┐", border);
    println!("│ {:<width$} │", title, width = width - 2);
    println!("└{}┘\n", border);
}

pub fn print_step(message: &str) {
    print!("{} ", message);
    std::io::stdout().flush().ok();
}

pub fn print_success(message: &str) {
    println!("\n{} {}", style("✓").green().bold(), style(message).green());
}

pub fn print_error(message: &str) {
    println!("\n{} {}", style("✗").red().bold(), style(message).red());
}
```

### Config Saving

```rust
// vibes-cli/src/commands/setup/mod.rs

fn save_tunnel_config(tunnel: &TunnelConfigSection) -> Result<()> {
    let mut config = ConfigLoader::load()?;
    config.tunnel = tunnel.clone();
    ConfigLoader::save(&config)?;

    println!("\nConfiguration saved:");
    println!("  tunnel.enabled = {}", tunnel.enabled);
    if let Some(mode) = &tunnel.mode {
        println!("  tunnel.mode = \"{}\"", mode);
    }
    if let Some(name) = &tunnel.name {
        println!("  tunnel.name = \"{}\"", name);
    }
    if let Some(hostname) = &tunnel.hostname {
        println!("  tunnel.hostname = \"{}\"", hostname);
    }

    Ok(())
}

fn save_auth_config(auth: &AccessConfig) -> Result<()> {
    let mut config = ConfigLoader::load()?;
    config.auth = auth.clone();
    ConfigLoader::save(&config)?;

    println!("Configuration saved:");
    println!("  auth.enabled = true");
    println!("  auth.team = \"{}\"", auth.team);
    println!("  auth.aud = \"{}...\"", &auth.aud[..8]);

    Ok(())
}
```

## Stories

### Story 1: Setup Wizard Infrastructure
**Type:** feature | **Size:** S

Add dialoguer dependency and create setup module with prompt helpers.

**Acceptance Criteria:**
- [ ] dialoguer added to vibes-cli/Cargo.toml
- [ ] `setup/mod.rs` with module structure
- [ ] `setup/prompts.rs` with print_header, print_step, print_success, print_error
- [ ] Tests for prompt formatting

### Story 2: Cloudflared State Detection
**Type:** feature | **Size:** M

Detect cloudflared installation, login status, and list existing tunnels.

**Acceptance Criteria:**
- [ ] CloudflaredState struct with installed, version, logged_in, cert_path
- [ ] ExistingTunnel struct with id, name, created_at, connections
- [ ] CloudflaredState::detect() async function
- [ ] list_tunnels() returns Vec<ExistingTunnel>
- [ ] create_tunnel(name) creates and returns new tunnel
- [ ] run_login() runs cloudflared login interactively
- [ ] route_dns(tunnel, hostname) routes DNS
- [ ] Tests with mocked command output

### Story 3: Tunnel Wizard - Quick Mode
**Type:** feature | **Size:** S

Implement quick tunnel setup path in the wizard.

**Acceptance Criteria:**
- [ ] `vibes tunnel setup` runs the wizard
- [ ] Shows cloudflared installation check
- [ ] Presents mode selection (quick/named)
- [ ] Selecting quick saves config with mode=quick
- [ ] Shows success message with next steps

### Story 4: Tunnel Wizard - Named Mode
**Type:** feature | **Size:** M

Implement named tunnel setup with login, tunnel selection, and DNS.

**Acceptance Criteria:**
- [ ] Detects login status, prompts login if needed
- [ ] Lists existing tunnels with create option
- [ ] Prompts for tunnel name when creating
- [ ] Prompts for hostname
- [ ] Optionally routes DNS
- [ ] Saves config with mode=named, name, hostname
- [ ] Shows success with next steps

### Story 5: Tunnel Wizard - No cloudflared
**Type:** feature | **Size:** S

Handle case when cloudflared is not installed.

**Acceptance Criteria:**
- [ ] Detects missing cloudflared
- [ ] Shows platform-specific install instructions (macOS, Linux, Windows)
- [ ] Exits gracefully without error
- [ ] Tells user to retry after installing

### Story 6: Auth Setup Wizard
**Type:** feature | **Size:** M

Implement auth setup wizard with team/AUD prompts and JWKS validation.

**Acceptance Criteria:**
- [ ] `vibes auth setup` runs the wizard
- [ ] Shows Access application instructions if user not ready
- [ ] Prompts for team name with validation
- [ ] Prompts for AUD with validation
- [ ] Tests JWKS fetch before saving
- [ ] Saves config with enabled=true, team, aud
- [ ] Shows success message

### Story 7: Config Save/Load
**Type:** feature | **Size:** S

Ensure wizards can save config properly.

**Acceptance Criteria:**
- [ ] ConfigLoader::save() writes to config.toml
- [ ] Preserves existing config sections when saving
- [ ] Creates config file if it doesn't exist
- [ ] Shows what was saved to user

### Story 8: Connectivity Validation (Optional)
**Type:** feature | **Size:** M

Add end-to-end validation after setup.

**Acceptance Criteria:**
- [ ] After tunnel setup, optionally test by starting tunnel briefly
- [ ] After auth setup, fetch JWKS to validate
- [ ] Clear error messages if validation fails
- [ ] Suggest fixes for common problems

## Testing Strategy

### Unit Tests
- Prompt helpers formatting
- Cloudflared output parsing (tunnel list JSON, version string)
- Config serialization/deserialization

### Integration Tests (Manual)
- Full wizard flow with actual cloudflared
- Login flow (requires browser)
- Tunnel creation/selection
- Auth JWKS validation

### Mocking Strategy
For unit tests, mock Command output:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    const TUNNEL_LIST_JSON: &str = r#"[
        {"id": "abc123", "name": "vibes-home", "created_at": "2026-01-05T10:00:00Z", "connections": 2}
    ]"#;

    #[test]
    fn parse_tunnel_list() {
        let tunnels: Vec<ExistingTunnel> = serde_json::from_str(TUNNEL_LIST_JSON).unwrap();
        assert_eq!(tunnels.len(), 1);
        assert_eq!(tunnels[0].name, "vibes-home");
    }
}
```

## Open Questions

1. **Should we support non-interactive mode?** (e.g., `vibes tunnel setup --quick` for scripts)
   - Recommendation: Add in follow-up milestone

2. **Should auth wizard auto-detect team from tunnel hostname?**
   - Could parse hostname to suggest team name
   - Recommendation: Nice-to-have, not required

3. **What about existing config?**
   - Should wizard warn if overwriting existing config?
   - Recommendation: Yes, show current values and confirm overwrite

## Deliverables

- [ ] Interactive `vibes tunnel setup` wizard
- [ ] Interactive `vibes auth setup` wizard
- [ ] Cloudflared state detection
- [ ] Config saving with visual feedback
- [ ] Platform-specific install instructions
- [ ] JWKS validation for auth setup
