//! Tunnel setup wizard.
//!
//! Interactive wizard for configuring Cloudflare tunnel settings.

use anyhow::{Result, bail};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

use super::cloudflared::{CloudflaredState, create_tunnel, list_tunnels, route_dns, run_login};
use super::{print_header, print_step, print_success};
use crate::config::ConfigLoader;

/// Mode selection options for the tunnel wizard.
const MODE_QUICK: &str = "Quick tunnel (temporary URL, no account needed)";
const MODE_NAMED: &str = "Named tunnel (persistent URL, requires Cloudflare account)";

/// Run the tunnel setup wizard.
pub async fn run() -> Result<()> {
    print_header("VIBES TUNNEL SETUP");

    // Check cloudflared installation
    print_step("Checking cloudflared...");
    let state = CloudflaredState::detect().await;

    if !state.installed {
        println!();
        show_install_instructions();
        return Ok(());
    }

    let version = state.version.as_deref().unwrap_or("unknown");
    println!("OK (v{})", version);
    println!();

    // Present mode selection
    let options = [MODE_QUICK, MODE_NAMED];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which tunnel mode do you want to use?")
        .items(&options)
        .default(0)
        .interact()?;

    // Handle based on selection
    if selection == 0 {
        // Quick tunnel mode
        save_quick_config()?;
        print_success("Tunnel configured for quick mode!");
        println!();
        println!("Next steps:");
        println!("  Run 'vibes serve --tunnel' to start with tunnel");
        println!();
        Ok(())
    } else {
        // Named tunnel mode
        setup_named_tunnel(state).await
    }
}

/// Run the named tunnel setup wizard.
async fn setup_named_tunnel(state: CloudflaredState) -> Result<()> {
    // Step 1: Check login status
    if !state.logged_in {
        println!();
        println!("Named tunnel requires Cloudflare login.");
        println!("Opening browser for authentication...\n");
        run_login().await?;
        println!();
    }

    // Step 2: List existing tunnels or create new one
    print_step("Fetching existing tunnels...");
    let tunnels = list_tunnels().await?;

    let tunnel_name = if tunnels.is_empty() {
        println!("No existing tunnels found.\n");
        prompt_new_tunnel_name()?
    } else {
        println!("Found {} existing tunnel(s).\n", tunnels.len());
        select_or_create_tunnel(&tunnels)?
    };

    // Step 3: Create tunnel if new
    let tunnel_id = if !tunnels.iter().any(|t| t.name == tunnel_name) {
        print_step(&format!("Creating tunnel '{}'...", tunnel_name));
        let id = create_tunnel(&tunnel_name).await?;
        println!("Tunnel created with ID: {}\n", id);
        id
    } else {
        tunnels
            .iter()
            .find(|t| t.name == tunnel_name)
            .map(|t| t.id.clone())
            .unwrap()
    };

    // Step 4: Prompt for hostname
    let hostname: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the hostname for this tunnel (e.g., vibes.yourdomain.com)")
        .validate_with(|input: &String| validate_hostname(input).map(|_| ()))
        .interact_text()?;

    // Step 5: Prompt for DNS routing
    println!();
    let route_dns_enabled = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Route DNS to this tunnel? (Recommended)")
        .default(true)
        .interact()?;

    if route_dns_enabled {
        print_step(&format!("Routing DNS {} -> {}...", hostname, tunnel_name));
        route_dns(&tunnel_id, &hostname).await?;
        println!("DNS routing configured.\n");
    }

    // Step 6: Save configuration
    save_named_config(&tunnel_name, &hostname)?;

    // Step 7: Success message
    print_success("Named tunnel configured!");
    println!();
    println!("Configuration saved:");
    println!("  Tunnel: {}", tunnel_name);
    println!("  Hostname: {}", hostname);
    if route_dns_enabled {
        println!("  DNS routing: enabled");
    }
    println!();
    println!("Next steps:");
    println!("  Run 'vibes serve --tunnel' to start with your named tunnel");
    println!();

    Ok(())
}

/// Prompt for a new tunnel name with validation.
fn prompt_new_tunnel_name() -> Result<String> {
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a name for the new tunnel (alphanumeric and hyphens only)")
        .validate_with(|input: &String| validate_tunnel_name(input).map(|_| ()))
        .interact_text()?;
    Ok(name)
}

/// Show selection of existing tunnels with option to create new.
fn select_or_create_tunnel(tunnels: &[super::cloudflared::ExistingTunnel]) -> Result<String> {
    const CREATE_NEW: &str = "➕ Create new tunnel";

    let mut options: Vec<String> = tunnels
        .iter()
        .map(|t| format!("{} ({})", t.name, &t.id[..8]))
        .collect();
    options.push(CREATE_NEW.to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a tunnel or create a new one")
        .items(&options)
        .default(0)
        .interact()?;

    if selection == tunnels.len() {
        // Create new selected
        prompt_new_tunnel_name()
    } else {
        Ok(tunnels[selection].name.clone())
    }
}

/// Validate tunnel name (alphanumeric and hyphens only).
fn validate_tunnel_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Tunnel name cannot be empty");
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        bail!("Tunnel name must contain only alphanumeric characters and hyphens");
    }
    Ok(())
}

/// Show platform-specific installation instructions for cloudflared.
/// Returns the instructions as a String (for testability).
fn get_install_instructions() -> String {
    let current_os = std::env::consts::OS;

    let mut output = String::new();
    output.push_str("cloudflared is required for tunnel functionality.\n\n");
    output.push_str("Install it using one of these methods:\n\n");

    let instructions = [
        ("macos", "  macOS:    brew install cloudflared"),
        (
            "linux",
            "  Linux:    See https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation",
        ),
        (
            "windows",
            "  Windows:  winget install Cloudflare.cloudflared",
        ),
    ];

    for (os, instruction) in instructions {
        if os == current_os {
            output.push_str(&format!("→ {}\n", instruction));
        } else {
            output.push_str(&format!("{}\n", instruction));
        }
    }

    output.push_str("\nAfter installing, run 'vibes tunnel setup' again.\n");
    output
}

/// Show installation instructions and exit gracefully.
fn show_install_instructions() {
    println!("{}", get_install_instructions());
}

/// Validate hostname (must contain dot, not start with dot).
fn validate_hostname(hostname: &str) -> Result<()> {
    if hostname.is_empty() {
        bail!("Hostname cannot be empty");
    }
    if hostname.starts_with('.') {
        bail!("Hostname cannot start with a dot");
    }
    if !hostname.contains('.') {
        bail!("Hostname must contain at least one dot (e.g., vibes.example.com)");
    }
    Ok(())
}

/// Save named tunnel configuration to the project config file.
fn save_named_config(name: &str, hostname: &str) -> Result<()> {
    let config_path = ConfigLoader::project_config_path();

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let existing_content = std::fs::read_to_string(&config_path).unwrap_or_default();

    if existing_content.trim().is_empty() {
        let config = format!(
            "[tunnel]\nenabled = true\nmode = \"named\"\nname = \"{}\"\nhostname = \"{}\"\n",
            name, hostname
        );
        std::fs::write(&config_path, config)?;
    } else {
        let mut doc = existing_content
            .parse::<toml_edit::DocumentMut>()
            .unwrap_or_default();

        doc["tunnel"] = toml_edit::Item::Table(toml_edit::Table::new());
        doc["tunnel"]["enabled"] = toml_edit::value(true);
        doc["tunnel"]["mode"] = toml_edit::value("named");
        doc["tunnel"]["name"] = toml_edit::value(name);
        doc["tunnel"]["hostname"] = toml_edit::value(hostname);

        std::fs::write(&config_path, doc.to_string())?;
    }

    Ok(())
}

/// Save quick tunnel configuration to the project config file.
fn save_quick_config() -> Result<()> {
    let config_path = ConfigLoader::project_config_path();

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Load existing config or create new one
    let existing_content = std::fs::read_to_string(&config_path).unwrap_or_default();

    // If file exists and has content, we need to merge with existing config
    if existing_content.trim().is_empty() {
        // Write fresh config with just tunnel section
        std::fs::write(&config_path, "[tunnel]\nenabled = true\nmode = \"quick\"\n")?;
    } else {
        // Parse existing and update tunnel section
        let mut doc = existing_content
            .parse::<toml_edit::DocumentMut>()
            .unwrap_or_default();

        // Create or update tunnel table
        doc["tunnel"] = toml_edit::Item::Table(toml_edit::Table::new());
        doc["tunnel"]["enabled"] = toml_edit::value(true);
        doc["tunnel"]["mode"] = toml_edit::value("quick");

        std::fs::write(&config_path, doc.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn mode_constants_have_correct_descriptions() {
        assert!(MODE_QUICK.contains("temporary"));
        assert!(MODE_QUICK.contains("no account"));
        assert!(MODE_NAMED.contains("persistent"));
        assert!(MODE_NAMED.contains("Cloudflare account"));
    }

    #[test]
    fn save_quick_config_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".vibes");
        let config_path = config_dir.join("config.toml");

        // Set env var to use our temp directory
        // SAFETY: tests run with --test-threads=1
        unsafe {
            std::env::set_var(
                "VIBES_PROJECT_CONFIG_DIR",
                config_dir.to_string_lossy().as_ref(),
            );
        }

        let result = save_quick_config();

        unsafe {
            std::env::remove_var("VIBES_PROJECT_CONFIG_DIR");
        }

        assert!(result.is_ok());
        assert!(config_path.exists());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[tunnel]"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains(r#"mode = "quick""#));
    }

    #[test]
    fn save_quick_config_preserves_existing_sections() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".vibes");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");

        // Write existing config with server section
        std::fs::write(
            &config_path,
            "[server]\nport = 8080\n\n[session]\ndefault_model = \"claude-sonnet-4\"\n",
        )
        .unwrap();

        // Set env var to use our temp directory
        // SAFETY: tests run with --test-threads=1
        unsafe {
            std::env::set_var(
                "VIBES_PROJECT_CONFIG_DIR",
                config_dir.to_string_lossy().as_ref(),
            );
        }

        let result = save_quick_config();

        unsafe {
            std::env::remove_var("VIBES_PROJECT_CONFIG_DIR");
        }

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&config_path).unwrap();
        // Should preserve existing sections
        assert!(content.contains("[server]"));
        assert!(content.contains("port = 8080"));
        assert!(content.contains("[session]"));
        // Should add tunnel section
        assert!(content.contains("[tunnel]"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains(r#"mode = "quick""#));
    }

    // === Named tunnel tests ===

    #[test]
    fn validate_tunnel_name_accepts_alphanumeric() {
        assert!(validate_tunnel_name("my-tunnel").is_ok());
        assert!(validate_tunnel_name("vibes123").is_ok());
        assert!(validate_tunnel_name("my-vibes-tunnel").is_ok());
    }

    #[test]
    fn validate_tunnel_name_rejects_invalid_chars() {
        assert!(validate_tunnel_name("my_tunnel").is_err()); // underscore
        assert!(validate_tunnel_name("my tunnel").is_err()); // space
        assert!(validate_tunnel_name("my.tunnel").is_err()); // dot
        assert!(validate_tunnel_name("tunnel@home").is_err()); // special char
    }

    #[test]
    fn validate_tunnel_name_rejects_empty() {
        assert!(validate_tunnel_name("").is_err());
    }

    #[test]
    fn validate_hostname_accepts_valid_domains() {
        assert!(validate_hostname("vibes.example.com").is_ok());
        assert!(validate_hostname("my-tunnel.dev").is_ok());
        assert!(validate_hostname("sub.domain.org").is_ok());
    }

    #[test]
    fn validate_hostname_rejects_no_dot() {
        assert!(validate_hostname("localhost").is_err());
        assert!(validate_hostname("vibes").is_err());
    }

    #[test]
    fn validate_hostname_rejects_leading_dot() {
        assert!(validate_hostname(".example.com").is_err());
    }

    #[test]
    fn validate_hostname_rejects_empty() {
        assert!(validate_hostname("").is_err());
    }

    #[test]
    fn save_named_config_creates_correct_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".vibes");
        let config_path = config_dir.join("config.toml");

        // SAFETY: tests run with --test-threads=1
        unsafe {
            std::env::set_var(
                "VIBES_PROJECT_CONFIG_DIR",
                config_dir.to_string_lossy().as_ref(),
            );
        }

        let result = save_named_config("my-tunnel", "vibes.example.com");

        unsafe {
            std::env::remove_var("VIBES_PROJECT_CONFIG_DIR");
        }

        assert!(result.is_ok());
        assert!(config_path.exists());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[tunnel]"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains(r#"mode = "named""#));
        assert!(content.contains(r#"name = "my-tunnel""#));
        assert!(content.contains(r#"hostname = "vibes.example.com""#));
    }

    #[test]
    fn save_named_config_preserves_existing_sections() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".vibes");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");

        std::fs::write(&config_path, "[server]\nport = 9000\n").unwrap();

        // SAFETY: tests run with --test-threads=1
        unsafe {
            std::env::set_var(
                "VIBES_PROJECT_CONFIG_DIR",
                config_dir.to_string_lossy().as_ref(),
            );
        }

        let result = save_named_config("prod-tunnel", "vibes.mysite.io");

        unsafe {
            std::env::remove_var("VIBES_PROJECT_CONFIG_DIR");
        }

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[server]"));
        assert!(content.contains("port = 9000"));
        assert!(content.contains("[tunnel]"));
        assert!(content.contains(r#"mode = "named""#));
        assert!(content.contains(r#"name = "prod-tunnel""#));
        assert!(content.contains(r#"hostname = "vibes.mysite.io""#));
    }

    // === Install instructions tests ===

    fn get_install_instructions_output() -> String {
        super::get_install_instructions()
    }

    #[test]
    fn install_instructions_contains_all_platforms() {
        let output = get_install_instructions_output();
        assert!(output.contains("macOS"));
        assert!(output.contains("brew install cloudflared"));
        assert!(output.contains("Linux"));
        assert!(output.contains("developers.cloudflare.com"));
        assert!(output.contains("Windows"));
        assert!(output.contains("winget install"));
    }

    #[test]
    fn install_instructions_includes_rerun_message() {
        let output = get_install_instructions_output();
        assert!(output.contains("vibes tunnel setup"));
    }
}
