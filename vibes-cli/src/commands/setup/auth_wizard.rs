//! Auth setup wizard.
//!
//! Interactive wizard for configuring Cloudflare Access authentication.

use anyhow::{Result, bail};
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use vibes_core::{AccessConfig, JwtValidator};

use super::{print_header, print_step, print_success};
use crate::config::ConfigLoader;

/// Validate team name (alphanumeric and hyphens only).
pub fn validate_team_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Team name cannot be empty");
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        bail!("Team name must contain only alphanumeric characters and hyphens");
    }
    Ok(())
}

/// Validate AUD tag (32+ characters, alphanumeric only).
pub fn validate_aud(aud: &str) -> Result<()> {
    if aud.is_empty() {
        bail!("AUD tag cannot be empty");
    }
    if aud.len() < 32 {
        bail!("AUD tag must be at least 32 characters");
    }
    if !aud.chars().all(|c| c.is_ascii_alphanumeric()) {
        bail!("AUD tag must contain only alphanumeric characters");
    }
    Ok(())
}

/// Save auth configuration to the project config file.
pub fn save_auth_config(team: &str, aud: &str) -> Result<()> {
    let config_path = ConfigLoader::project_config_path();

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let existing_content = std::fs::read_to_string(&config_path).unwrap_or_default();

    if existing_content.trim().is_empty() {
        let config = format!(
            "[auth]\nenabled = true\nteam = \"{}\"\naud = \"{}\"\nbypass_localhost = true\nclock_skew_seconds = 60\n",
            team, aud
        );
        std::fs::write(&config_path, config)?;
    } else {
        let mut doc = existing_content
            .parse::<toml_edit::DocumentMut>()
            .unwrap_or_default();

        doc["auth"] = toml_edit::Item::Table(toml_edit::Table::new());
        doc["auth"]["enabled"] = toml_edit::value(true);
        doc["auth"]["team"] = toml_edit::value(team);
        doc["auth"]["aud"] = toml_edit::value(aud);
        doc["auth"]["bypass_localhost"] = toml_edit::value(true);
        doc["auth"]["clock_skew_seconds"] = toml_edit::value(60i64);

        std::fs::write(&config_path, doc.to_string())?;
    }

    Ok(())
}

/// Run the auth setup wizard.
pub async fn run() -> Result<()> {
    print_header("VIBES AUTH SETUP");

    // Show prerequisites
    println!("This wizard configures Cloudflare Access authentication for vibes.");
    println!();
    println!("Prerequisites:");
    println!("  1. A Cloudflare account with Zero Trust enabled");
    println!("  2. A Cloudflare Access application configured for vibes");
    println!();

    // Ask if they have an Access application
    let has_app = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Have you already created a Cloudflare Access application?")
        .default(false)
        .interact()?;

    if !has_app {
        show_access_setup_instructions();

        let continue_setup = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue with setup after creating the application?")
            .default(true)
            .interact()?;

        if !continue_setup {
            println!();
            println!("Run 'vibes auth setup' again when ready.");
            return Ok(());
        }
        println!();
    }

    // Prompt for team name
    let team: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your Cloudflare Access team name (e.g., 'mycompany' for mycompany.cloudflareaccess.com)")
        .validate_with(|input: &String| validate_team_name(input).map(|_| ()))
        .interact_text()?;

    // Prompt for AUD
    println!();
    println!("The Application Audience (AUD) tag can be found in:");
    println!("  Zero Trust Dashboard > Access > Applications > [Your App] > Overview");
    println!();

    let aud: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the Application Audience (AUD) tag")
        .validate_with(|input: &String| validate_aud(input).map(|_| ()))
        .interact_text()?;

    // Test configuration by fetching JWKS
    println!();
    let config = AccessConfig::new(&team, &aud);
    print_step(&format!(
        "Testing configuration (fetching JWKS from {})...",
        config.jwks_url()
    ));

    let validator = JwtValidator::new(config);
    match validator.refresh_jwks().await {
        Ok(()) => println!("OK"),
        Err(e) => {
            println!("FAILED");
            println!();
            bail!("Failed to fetch JWKS: {}. Please verify your team name.", e);
        }
    }

    // Save configuration
    println!();
    print_step("Saving configuration...");
    save_auth_config(&team, &aud)?;
    println!("OK");

    // Success message
    print_success("Auth configured successfully!");
    println!();
    println!("Configuration saved:");
    println!("  Team: {}", team);
    println!("  AUD: {}...{}", &aud[..8], &aud[aud.len() - 8..]);
    println!("  Bypass localhost: true");
    println!("  Clock skew: 60s");
    println!();
    println!("Next steps:");
    println!("  Run 'vibes auth status' to verify configuration");
    println!("  Run 'vibes auth test' to test JWT validation");
    println!();

    Ok(())
}

/// Show instructions for creating a Cloudflare Access application.
fn show_access_setup_instructions() {
    println!();
    println!("To create a Cloudflare Access application:");
    println!();
    println!("  1. Go to https://one.dash.cloudflare.com");
    println!("  2. Navigate to: Access > Applications > Add an Application");
    println!("  3. Choose 'Self-hosted' application type");
    println!("  4. Configure:");
    println!("     - Application name: e.g., 'vibes'");
    println!("     - Session duration: 24 hours (recommended)");
    println!("     - Application domain: your vibes server URL");
    println!("  5. Add an access policy (e.g., allow your email domain)");
    println!("  6. Save the application");
    println!("  7. Copy the 'Application Audience (AUD) Tag' from the Overview page");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Team name validation tests ===

    #[test]
    fn validate_team_name_accepts_alphanumeric_with_hyphens() {
        assert!(validate_team_name("my-team").is_ok());
        assert!(validate_team_name("mycompany").is_ok());
        assert!(validate_team_name("my-cool-team").is_ok());
        assert!(validate_team_name("team123").is_ok());
    }

    #[test]
    fn validate_team_name_rejects_empty() {
        let result = validate_team_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn validate_team_name_rejects_invalid_chars() {
        assert!(validate_team_name("my_team").is_err()); // underscore
        assert!(validate_team_name("my team").is_err()); // space
        assert!(validate_team_name("my.team").is_err()); // dot
        assert!(validate_team_name("team@work").is_err()); // special char
    }

    // === AUD validation tests ===

    #[test]
    fn validate_aud_accepts_valid_aud() {
        // 64-char hex string (typical AUD format)
        let valid_aud = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        assert!(validate_aud(valid_aud).is_ok());

        // 32-char minimum
        let min_aud = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4";
        assert!(validate_aud(min_aud).is_ok());
    }

    #[test]
    fn validate_aud_rejects_too_short() {
        let result = validate_aud("abc123"); // Only 6 chars
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32"));
    }

    #[test]
    fn validate_aud_rejects_empty() {
        let result = validate_aud("");
        assert!(result.is_err());
    }

    #[test]
    fn validate_aud_rejects_non_alphanumeric() {
        let with_special = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4-x";
        assert!(validate_aud(with_special).is_err());

        let with_space = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4 x";
        assert!(validate_aud(with_space).is_err());
    }

    // === Save auth config tests ===

    use tempfile::TempDir;

    #[test]
    fn save_auth_config_creates_new_file() {
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

        let result = save_auth_config("my-team", "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4");

        unsafe {
            std::env::remove_var("VIBES_PROJECT_CONFIG_DIR");
        }

        assert!(result.is_ok());
        assert!(config_path.exists());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[auth]"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains(r#"team = "my-team""#));
        assert!(content.contains(r#"aud = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4""#));
        assert!(content.contains("bypass_localhost = true"));
        assert!(content.contains("clock_skew_seconds = 60"));
    }

    #[test]
    fn save_auth_config_preserves_existing_sections() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".vibes");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");

        // Write existing config with server section
        std::fs::write(
            &config_path,
            "[server]\nport = 8080\n\n[tunnel]\nmode = \"quick\"\n",
        )
        .unwrap();

        // SAFETY: tests run with --test-threads=1
        unsafe {
            std::env::set_var(
                "VIBES_PROJECT_CONFIG_DIR",
                config_dir.to_string_lossy().as_ref(),
            );
        }

        let result = save_auth_config("prod-team", "x1y2z3a4b5c6d7e8f9g0x1y2z3a4b5c6");

        unsafe {
            std::env::remove_var("VIBES_PROJECT_CONFIG_DIR");
        }

        assert!(result.is_ok());

        let content = std::fs::read_to_string(&config_path).unwrap();
        // Should preserve existing sections
        assert!(content.contains("[server]"));
        assert!(content.contains("port = 8080"));
        assert!(content.contains("[tunnel]"));
        // Should add auth section
        assert!(content.contains("[auth]"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains(r#"team = "prod-team""#));
    }
}
