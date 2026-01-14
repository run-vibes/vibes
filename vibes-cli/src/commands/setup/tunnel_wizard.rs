//! Tunnel setup wizard.
//!
//! Interactive wizard for configuring Cloudflare tunnel settings.

use anyhow::{Result, bail};
use dialoguer::{Select, theme::ColorfulTheme};

use super::cloudflared::CloudflaredState;
use super::{print_error, print_header, print_step, print_success};
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
        print_error("cloudflared is not installed");
        println!();
        println!("Install cloudflared to use tunnels:");
        println!(
            "  https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/"
        );
        println!();
        bail!("cloudflared not installed");
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
        println!();
        println!("Named tunnel setup requires additional configuration.");
        println!("Run 'vibes tunnel setup' again when you're ready to set up named tunnels.");
        println!();
        bail!("Named tunnel setup not yet implemented (see feat-0074)");
    }
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
}
