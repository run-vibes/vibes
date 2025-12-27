//! Cloudflared CLI wrapper and output parsing

use tokio::process::{Child, Command};

use super::config::TunnelMode;
use std::process::Stdio;

/// Result of checking cloudflared installation
#[derive(Debug, Clone)]
pub struct CloudflaredInfo {
    pub version: String,
    pub path: String,
}

/// Check if cloudflared is installed and get version
pub async fn check_installation() -> Option<CloudflaredInfo> {
    let output = Command::new("cloudflared")
        .arg("--version")
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    // Parse "cloudflared version 2024.12.0 (built 2024-12-01)"
    let version = version_str
        .split_whitespace()
        .nth(2)
        .unwrap_or("unknown")
        .to_string();

    let path = which::which("cloudflared")
        .ok()?
        .to_string_lossy()
        .to_string();

    Some(CloudflaredInfo { version, path })
}

/// Spawn cloudflared process for the given mode
pub fn spawn_tunnel(mode: &TunnelMode, local_port: u16) -> std::io::Result<Child> {
    let mut cmd = Command::new("cloudflared");

    match mode {
        TunnelMode::Quick => {
            cmd.arg("tunnel")
                .arg("--url")
                .arg(format!("http://localhost:{}", local_port));
        }
        TunnelMode::Named {
            name,
            credentials_path,
            ..
        } => {
            cmd.arg("tunnel");
            if let Some(creds) = credentials_path {
                cmd.arg("--credentials-file").arg(creds);
            }
            cmd.arg("run")
                .arg("--url")
                .arg(format!("http://localhost:{}", local_port))
                .arg(name);
        }
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}

/// Parse a quick tunnel URL from cloudflared output
///
/// Looks for pattern: "https://xxx.trycloudflare.com"
pub fn parse_quick_tunnel_url(line: &str) -> Option<String> {
    // cloudflared prints URL in a box like:
    // | https://random-words.trycloudflare.com |
    if line.contains("trycloudflare.com") {
        // Extract URL using regex-like matching
        let start = line.find("https://")?;
        let url_part = &line[start..];
        // Find the end of the URL (whitespace, |, or end of string)
        let end = url_part
            .find(|c: char| c.is_whitespace() || c == '|')
            .unwrap_or(url_part.len());
        return Some(url_part[..end].to_string());
    }
    None
}

/// Parse log level from cloudflared output
pub fn parse_log_level(line: &str) -> Option<(&str, &str)> {
    // Format: "INF message here" or "ERR message here"
    let level = if line.starts_with("INF") {
        "info"
    } else if line.starts_with("WRN") {
        "warn"
    } else if line.starts_with("ERR") {
        "error"
    } else if line.starts_with("DBG") {
        "debug"
    } else {
        return None;
    };

    let message = line.get(4..)?.trim();
    Some((level, message))
}

/// Check if line indicates successful connection
pub fn is_connection_registered(line: &str) -> bool {
    line.contains("Connection") && line.contains("registered")
}

/// Check if line indicates connection lost
pub fn is_connection_lost(line: &str) -> bool {
    line.contains("Unregistered") || line.contains("connection lost")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_quick_tunnel_url_from_box() {
        let line = "INF | https://random-words-here.trycloudflare.com            |";
        let url = parse_quick_tunnel_url(line);
        assert_eq!(
            url,
            Some("https://random-words-here.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn parse_quick_tunnel_url_plain() {
        let line = "Your quick Tunnel is https://test-tunnel.trycloudflare.com";
        let url = parse_quick_tunnel_url(line);
        assert_eq!(
            url,
            Some("https://test-tunnel.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn parse_quick_tunnel_url_no_match() {
        let line = "Starting tunnel connector";
        assert!(parse_quick_tunnel_url(line).is_none());
    }

    #[test]
    fn parse_log_level_info() {
        let (level, msg) = parse_log_level("INF Starting tunnel").unwrap();
        assert_eq!(level, "info");
        assert_eq!(msg, "Starting tunnel");
    }

    #[test]
    fn parse_log_level_error() {
        let (level, msg) = parse_log_level("ERR Connection failed").unwrap();
        assert_eq!(level, "error");
        assert_eq!(msg, "Connection failed");
    }

    #[test]
    fn parse_log_level_unknown() {
        assert!(parse_log_level("Some random text").is_none());
    }

    #[test]
    fn is_connection_registered_true() {
        let line = "INF Connection 0 registered connIndex=0";
        assert!(is_connection_registered(line));
    }

    #[test]
    fn is_connection_registered_false() {
        let line = "INF Starting tunnel";
        assert!(!is_connection_registered(line));
    }

    #[test]
    fn is_connection_lost_unregistered() {
        let line = "ERR Unregistered tunnel connection";
        assert!(is_connection_lost(line));
    }
}
