//! Connectivity validation for tunnel setup.
//!
//! E2E tests that spawn the vibes binary with tunnel enabled and verify
//! the web UI is accessible via the tunnel URL.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Result, anyhow};
use dialoguer::console::style;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;
use vibes_core::tunnel::cloudflared::{is_connection_registered, parse_quick_tunnel_url};

use super::prompts::{print_error, print_step, print_success};

/// Default timeout for connectivity tests.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(45);

/// HTTP request timeout.
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// Result of a connectivity test.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectivityResult {
    /// Tunnel connected and web UI verified accessible.
    Success { url: String },
    /// vibes binary not found.
    BinaryNotFound,
    /// Server process failed to start.
    ServerStartFailed { reason: String },
    /// Tunnel didn't connect within timeout.
    TunnelTimeout,
    /// HTTP request to tunnel URL failed.
    HttpFailed { url: String, error: String },
    /// Got response but it's a Cloudflare error page.
    CloudflareError { url: String },
    /// Got 200 but content doesn't look like vibes UI.
    UnexpectedContent { url: String },
}

impl ConnectivityResult {
    /// Returns true if the test succeeded.
    #[cfg(test)]
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Print the result with appropriate formatting and troubleshooting.
    pub fn print(&self) {
        match self {
            Self::Success { url } => {
                print_success("Tunnel connected successfully!");
                println!(
                    "  Your vibes server is accessible at: {}",
                    style(url).cyan()
                );
            }
            Self::BinaryNotFound => {
                print_error("vibes binary not found");
                println!();
                println!("Troubleshooting:");
                println!("  - Run 'just build' to compile vibes");
                println!("  - Ensure you're in the vibes project directory");
            }
            Self::ServerStartFailed { reason } => {
                print_error("Server failed to start");
                println!();
                println!("Error: {}", reason);
                println!();
                println!("Troubleshooting:");
                println!("  - Check if port 8080 is in use: lsof -i :8080");
                println!("  - Check server logs for errors");
            }
            Self::TunnelTimeout => {
                print_error("Tunnel failed to connect within timeout");
                println!();
                println!("Troubleshooting:");
                println!("  - Check your internet connection");
                println!("  - Verify cloudflared login: cloudflared tunnel list");
                println!("  - For named tunnels, verify tunnel exists");
            }
            Self::HttpFailed { url, error } => {
                print_error(&format!("Could not reach web UI at {}", url));
                println!();
                println!("Error: {}", error);
                println!();
                println!("Troubleshooting:");
                println!("  - The tunnel connected but HTTP request failed");
                println!("  - Check firewall settings");
                println!("  - Try accessing the URL manually in a browser");
            }
            Self::CloudflareError { url } => {
                print_error(&format!("Cloudflare error page at {}", url));
                println!();
                println!("Troubleshooting:");
                println!("  - For named tunnels: verify DNS is routed correctly");
                println!("  - Run: cloudflared tunnel route dns <tunnel> <hostname>");
                println!("  - Check Cloudflare dashboard for tunnel status");
            }
            Self::UnexpectedContent { url } => {
                print_error(&format!("Unexpected content at {}", url));
                println!();
                println!("Troubleshooting:");
                println!("  - Another service may be running on this hostname");
                println!("  - Verify tunnel configuration points to vibes");
            }
        }
    }
}

/// Find the vibes binary for spawning.
///
/// Resolution order:
/// 1. Worktree-local target (set at compile time by build.rs)
/// 2. CARGO_TARGET_DIR environment variable (shared target directory)
/// 3. PATH lookup
#[must_use]
pub fn find_vibes_binary() -> Option<PathBuf> {
    // 1. Worktree-local target (set at compile time)
    let workspace_root = env!("VIBES_WORKSPACE_ROOT");
    for profile in ["debug", "release"] {
        let binary = PathBuf::from(workspace_root)
            .join("target")
            .join(profile)
            .join("vibes");
        if binary.exists() {
            return Some(binary);
        }
    }

    // 2. CARGO_TARGET_DIR (shared cache)
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        for profile in ["debug", "release"] {
            let binary = PathBuf::from(&target_dir).join(profile).join("vibes");
            if binary.exists() {
                return Some(binary);
            }
        }
    }

    // 3. PATH lookup
    which::which("vibes").ok()
}

/// Test quick tunnel connectivity.
///
/// Spawns `vibes serve --tunnel`, waits for the quick tunnel URL,
/// then verifies the web UI is accessible.
pub async fn test_quick_tunnel() -> ConnectivityResult {
    let binary = match find_vibes_binary() {
        Some(b) => b,
        None => return ConnectivityResult::BinaryNotFound,
    };

    print_step("Starting vibes server with tunnel...");

    let mut child = match spawn_vibes_server(&binary).await {
        Ok(c) => c,
        Err(e) => {
            println!();
            return ConnectivityResult::ServerStartFailed {
                reason: e.to_string(),
            };
        }
    };

    // Wait for tunnel URL
    let url = match wait_for_quick_tunnel_url(&mut child, DEFAULT_TIMEOUT).await {
        Ok(url) => {
            println!("OK");
            println!("  Tunnel URL: {}", style(&url).cyan());
            url
        }
        Err(_) => {
            println!();
            let _ = child.kill().await;
            return ConnectivityResult::TunnelTimeout;
        }
    };

    // Verify web UI
    print_step("Verifying web UI...");
    let result = verify_web_ui(&url).await;
    println!("Done");

    // Cleanup
    let _ = child.kill().await;

    result
}

/// Test named tunnel connectivity.
///
/// Spawns `vibes serve --tunnel`, waits for connection registration,
/// then verifies the web UI is accessible at the configured hostname.
pub async fn test_named_tunnel(hostname: &str) -> ConnectivityResult {
    let binary = match find_vibes_binary() {
        Some(b) => b,
        None => return ConnectivityResult::BinaryNotFound,
    };

    print_step("Starting vibes server with tunnel...");

    let mut child = match spawn_vibes_server(&binary).await {
        Ok(c) => c,
        Err(e) => {
            println!();
            return ConnectivityResult::ServerStartFailed {
                reason: e.to_string(),
            };
        }
    };

    // Wait for connection registration
    match wait_for_connection(&mut child, DEFAULT_TIMEOUT).await {
        Ok(()) => println!("Connected"),
        Err(_) => {
            println!();
            let _ = child.kill().await;
            return ConnectivityResult::TunnelTimeout;
        }
    }

    // Verify web UI at hostname
    let url = format!("https://{}", hostname);
    print_step(&format!("Verifying web UI at {}...", hostname));
    let result = verify_web_ui(&url).await;
    println!("Done");

    // Cleanup
    let _ = child.kill().await;

    result
}

/// Spawn vibes server with tunnel enabled.
async fn spawn_vibes_server(binary: &PathBuf) -> Result<Child> {
    Command::new(binary)
        .args(["serve", "--tunnel"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn vibes: {}", e))
}

/// Wait for quick tunnel URL in server output.
async fn wait_for_quick_tunnel_url(
    child: &mut Child,
    timeout_duration: Duration,
) -> Result<String> {
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("No stderr"))?;
    let mut reader = BufReader::new(stderr).lines();

    timeout(timeout_duration, async {
        while let Some(line) = reader.next_line().await? {
            if let Some(url) = parse_quick_tunnel_url(&line) {
                return Ok(url);
            }
        }
        Err(anyhow!("Server exited without providing URL"))
    })
    .await
    .map_err(|_| anyhow!("Timeout waiting for tunnel URL"))?
}

/// Wait for tunnel connection registration.
async fn wait_for_connection(child: &mut Child, timeout_duration: Duration) -> Result<()> {
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("No stderr"))?;
    let mut reader = BufReader::new(stderr).lines();

    timeout(timeout_duration, async {
        while let Some(line) = reader.next_line().await? {
            if is_connection_registered(&line) {
                return Ok(());
            }
        }
        Err(anyhow!("Server exited without registering connection"))
    })
    .await
    .map_err(|_| anyhow!("Timeout waiting for connection"))?
}

/// Verify web UI is accessible and contains expected content.
async fn verify_web_ui(url: &str) -> ConnectivityResult {
    let client = match reqwest::Client::builder().timeout(HTTP_TIMEOUT).build() {
        Ok(c) => c,
        Err(e) => {
            return ConnectivityResult::HttpFailed {
                url: url.to_string(),
                error: e.to_string(),
            };
        }
    };

    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            return ConnectivityResult::HttpFailed {
                url: url.to_string(),
                error: e.to_string(),
            };
        }
    };

    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            return ConnectivityResult::HttpFailed {
                url: url.to_string(),
                error: e.to_string(),
            };
        }
    };

    // Check for Cloudflare error indicators
    if is_cloudflare_error(&body) {
        return ConnectivityResult::CloudflareError {
            url: url.to_string(),
        };
    }

    // Check for vibes UI markers
    if !is_vibes_ui(&body) {
        return ConnectivityResult::UnexpectedContent {
            url: url.to_string(),
        };
    }

    ConnectivityResult::Success {
        url: url.to_string(),
    }
}

/// Check if response body indicates a Cloudflare error page.
fn is_cloudflare_error(body: &str) -> bool {
    // Cloudflare error pages contain these markers
    body.contains("Error 1033")
        || body.contains("Argo Tunnel error")
        || body.contains("cloudflare-error")
        || (body.contains("Cloudflare") && body.contains("Error"))
}

/// Check if response body looks like the vibes web UI.
fn is_vibes_ui(body: &str) -> bool {
    // The vibes web UI should contain these markers
    body.contains("vibes") || body.contains("Vibes") || body.contains("<title>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_vibes_binary_returns_some_when_binary_exists() {
        // The binary should exist after `just build`
        let binary = find_vibes_binary();
        assert!(binary.is_some(), "vibes binary should be found");

        let path = binary.unwrap();
        assert!(path.exists(), "binary path should exist");
        assert!(
            path.to_string_lossy().contains("vibes"),
            "path should contain 'vibes'"
        );
    }

    #[test]
    fn connectivity_result_is_success() {
        let success = ConnectivityResult::Success {
            url: "https://test.trycloudflare.com".to_string(),
        };
        assert!(success.is_success());

        let failure = ConnectivityResult::TunnelTimeout;
        assert!(!failure.is_success());
    }

    #[test]
    fn is_cloudflare_error_detects_error_pages() {
        assert!(is_cloudflare_error("Error 1033: Argo Tunnel error"));
        assert!(is_cloudflare_error("<div class='cloudflare-error'>"));
        assert!(is_cloudflare_error("Cloudflare Error 522"));

        assert!(!is_cloudflare_error("<html><title>vibes</title></html>"));
        assert!(!is_cloudflare_error("Welcome to vibes"));
    }

    #[test]
    fn is_vibes_ui_detects_valid_ui() {
        assert!(is_vibes_ui("<html><title>vibes</title></html>"));
        assert!(is_vibes_ui("Welcome to Vibes dashboard"));
        assert!(is_vibes_ui("<title>My App</title>"));

        // Empty or unrelated content
        assert!(!is_vibes_ui(""));
        assert!(!is_vibes_ui("Just some random text"));
    }
}
