//! Cloudflared state detection and tunnel management.
//!
//! Detects cloudflared installation, login status, and manages tunnels.

// This module is infrastructure for the tunnel setup wizard.
// Functions will be used once the wizard is implemented.
#![allow(unused)]

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use tokio::process::Command;

/// State of cloudflared on the system.
#[derive(Debug, Clone, PartialEq)]
pub struct CloudflaredState {
    /// Whether cloudflared is installed.
    pub installed: bool,
    /// Version string if installed.
    pub version: Option<String>,
    /// Whether user is logged in to Cloudflare.
    pub logged_in: bool,
    /// Path to certificate if logged in.
    pub cert_path: Option<PathBuf>,
}

impl CloudflaredState {
    /// Detect cloudflared state on the system.
    pub async fn detect() -> Self {
        Self::detect_with_paths(default_cert_path()).await
    }

    /// Detect cloudflared state with custom cert path (for testing).
    pub async fn detect_with_paths(cert_path: PathBuf) -> Self {
        // Check if cloudflared is installed by running --version
        let (installed, version) = match Command::new("cloudflared").arg("--version").output().await
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let version = parse_version(&stdout);
                (true, version)
            }
            _ => (false, None),
        };

        // Check if logged in by looking for cert.pem
        let logged_in = cert_path.exists();
        let cert_path = if logged_in { Some(cert_path) } else { None };

        Self {
            installed,
            version,
            logged_in,
            cert_path,
        }
    }
}

/// Get the default cert.pem path (~/.cloudflared/cert.pem).
fn default_cert_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".cloudflared")
        .join("cert.pem")
}

/// Parse version from cloudflared --version output.
/// Example: "cloudflared version 2024.1.0 (built 2024-01-15-1234)"
fn parse_version(output: &str) -> Option<String> {
    output
        .lines()
        .next()?
        .split_whitespace()
        .nth(2)
        .map(String::from)
}

/// An existing Cloudflare tunnel.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExistingTunnel {
    /// Tunnel ID.
    pub id: String,
    /// Tunnel name.
    pub name: String,
    /// When the tunnel was created.
    pub created_at: DateTime<Utc>,
    /// Number of active connections.
    pub connections: u32,
}

/// List existing tunnels.
pub async fn list_tunnels() -> anyhow::Result<Vec<ExistingTunnel>> {
    let output = Command::new("cloudflared")
        .args(["tunnel", "list", "--output", "json"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cloudflared tunnel list failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // cloudflared returns `null` when no tunnels exist, not `[]`
    let tunnels: Option<Vec<ExistingTunnel>> = serde_json::from_str(&stdout)?;
    Ok(tunnels.unwrap_or_default())
}

/// Create a new tunnel.
pub async fn create_tunnel(name: &str) -> anyhow::Result<String> {
    let output = Command::new("cloudflared")
        .args(["tunnel", "create", name])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cloudflared tunnel create failed: {}", stderr);
    }

    // Parse tunnel ID from output like:
    // "Tunnel credentials written to /home/user/.cloudflared/abc123.json.
    //  Created tunnel my-tunnel with id abc123-..."
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_tunnel_id(&stdout).ok_or_else(|| anyhow::anyhow!("Failed to parse tunnel ID from output"))
}

/// Parse tunnel ID from create tunnel output.
fn parse_tunnel_id(output: &str) -> Option<String> {
    // Look for "with id <uuid>" pattern
    for line in output.lines() {
        if let Some(idx) = line.find("with id ") {
            let id_start = idx + "with id ".len();
            let id = line[id_start..].split_whitespace().next()?;
            return Some(id.to_string());
        }
    }
    None
}

/// Run cloudflared login interactively.
pub async fn run_login() -> anyhow::Result<()> {
    let status = Command::new("cloudflared").arg("login").status().await?;

    if !status.success() {
        anyhow::bail!("cloudflared login failed");
    }
    Ok(())
}

/// Route DNS for a tunnel to a hostname.
pub async fn route_dns(tunnel_id: &str, hostname: &str) -> anyhow::Result<()> {
    let output = Command::new("cloudflared")
        .args(["tunnel", "route", "dns", tunnel_id, hostname])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cloudflared tunnel route dns failed: {}", stderr);
    }
    Ok(())
}

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
        assert_eq!(tunnels[0].id, "abc123");
        assert_eq!(tunnels[0].name, "vibes-home");
        assert_eq!(tunnels[0].connections, 2);
    }

    #[test]
    fn parse_tunnel_list_handles_null() {
        // cloudflared returns `null` when no tunnels exist
        let tunnels: Option<Vec<ExistingTunnel>> = serde_json::from_str("null").unwrap();
        assert_eq!(tunnels.unwrap_or_default(), vec![]);
    }

    #[test]
    fn parse_tunnel_list_handles_empty_array() {
        // cloudflared could also return an empty array
        let tunnels: Option<Vec<ExistingTunnel>> = serde_json::from_str("[]").unwrap();
        assert_eq!(tunnels.unwrap_or_default(), vec![]);
    }

    #[tokio::test]
    async fn detect_with_nonexistent_cert_path() {
        // Use a path that definitely doesn't exist
        let fake_cert = PathBuf::from("/nonexistent/path/cert.pem");
        let state = CloudflaredState::detect_with_paths(fake_cert).await;

        // logged_in and cert_path depend only on cert file existence
        assert!(!state.logged_in);
        assert!(state.cert_path.is_none());
        // installed/version depend on whether cloudflared is on the system
        // We can't control that in this test, so we just verify the struct is populated
    }

    #[test]
    fn parse_version_extracts_version_number() {
        let output = "cloudflared version 2024.1.0 (built 2024-01-15-1234 lzNLoIi6)\n";
        assert_eq!(super::parse_version(output), Some("2024.1.0".to_string()));
    }

    #[test]
    fn parse_version_handles_empty_input() {
        assert_eq!(super::parse_version(""), None);
    }

    #[test]
    fn parse_tunnel_id_extracts_id() {
        let output = r#"Tunnel credentials written to /home/user/.cloudflared/abc123-def456.json. cloudflared chose this file based on where your origin certificate was found. Keep this file secret. To revoke these credentials, delete the tunnel.

Created tunnel my-tunnel with id abc123-def456-ghi789"#;
        assert_eq!(
            super::parse_tunnel_id(output),
            Some("abc123-def456-ghi789".to_string())
        );
    }

    #[test]
    fn parse_tunnel_id_handles_no_match() {
        assert_eq!(super::parse_tunnel_id("some other output"), None);
    }
}
