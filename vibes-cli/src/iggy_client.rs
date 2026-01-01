//! HTTP client for Iggy message streaming server.
//!
//! Provides a simple client for sending messages to Iggy topics via HTTP API.
//! Used by the `vibes event send` command.

use std::path::PathBuf;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::IggyClientConfig;

/// Path for cached token relative to cache directory
const TOKEN_CACHE_FILE: &str = "vibes/iggy-token";

/// HTTP client for communicating with Iggy server.
pub struct IggyHttpClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

/// Login request payload
#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Login response containing access token
#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: AccessToken,
}

#[derive(Debug, Deserialize)]
struct AccessToken {
    token: String,
}

/// Message payload for sending to Iggy
#[derive(Debug, Serialize)]
struct SendMessagesRequest {
    partitioning: Partitioning,
    messages: Vec<Message>,
}

/// Partitioning strategy for message distribution
#[derive(Debug, Serialize)]
struct Partitioning {
    kind: &'static str,
    value: String,
}

impl Partitioning {
    /// Create balanced partitioning (round-robin across partitions)
    fn balanced() -> Self {
        Self {
            kind: "balanced",
            value: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Message {
    payload: String,
}

impl IggyHttpClient {
    /// Create a new HTTP client for Iggy.
    #[must_use]
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("http://{}:{}", host, port),
            token: None,
        }
    }

    /// Create a client from configuration.
    #[must_use]
    pub fn from_config(config: &IggyClientConfig) -> Self {
        Self::new(&config.host, config.http_port)
    }

    /// Login to Iggy and store the JWT token.
    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let url = format!("{}/users/login", self.base_url);
        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Iggy server")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Login failed: {} - {}", status, body);
        }

        let login_response: LoginResponse = response
            .json()
            .await
            .context("Failed to parse login response")?;

        self.token = Some(login_response.access_token.token);
        Ok(())
    }

    /// Check if client has a valid token.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Send a message to an Iggy topic.
    ///
    /// # Arguments
    /// * `stream` - Name of the stream
    /// * `topic` - Name of the topic within the stream
    /// * `payload` - Message payload as bytes
    pub async fn send_message(&self, stream: &str, topic: &str, payload: &[u8]) -> Result<()> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated - call login() first"))?;

        // Encode payload as base64 for JSON transport
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, payload);

        let url = format!(
            "{}/streams/{}/topics/{}/messages",
            self.base_url, stream, topic
        );

        let request = SendMessagesRequest {
            partitioning: Partitioning::balanced(),
            messages: vec![Message { payload: encoded }],
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&request)
            .send()
            .await
            .context("Failed to send message to Iggy")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Send message failed: {} - {}", status, body);
        }

        Ok(())
    }

    /// Get the base URL (for testing/debugging).
    #[must_use]
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Authenticate with Iggy, using cached token if available.
    ///
    /// This is the main entry point for authentication. It will:
    /// 1. Try to load a cached token
    /// 2. If no cached token, login and cache the new token
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        // Try to load cached token first
        if let Some(token) = self.load_cached_token() {
            self.token = Some(token);
            return Ok(());
        }

        // No cached token, login fresh
        self.login(username, password).await?;
        self.cache_token()?;
        Ok(())
    }

    /// Get the path to the token cache file.
    fn token_cache_path() -> Option<PathBuf> {
        dirs::cache_dir().map(|d| d.join(TOKEN_CACHE_FILE))
    }

    /// Load a cached token from disk.
    fn load_cached_token(&self) -> Option<String> {
        let path = Self::token_cache_path()?;
        std::fs::read_to_string(path)
            .ok()
            .map(|s| s.trim().to_string())
    }

    /// Cache the current token to disk.
    fn cache_token(&self) -> Result<()> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No token to cache"))?;

        let path = Self::token_cache_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?;

        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create cache directory")?;
        }

        std::fs::write(&path, token).context("Failed to write token cache")?;
        Ok(())
    }

    /// Clear the cached token (useful for testing or forced re-authentication).
    #[allow(dead_code)]
    pub fn clear_cached_token() -> Result<()> {
        if let Some(path) = Self::token_cache_path()
            && path.exists()
        {
            std::fs::remove_file(&path).context("Failed to remove token cache")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs_correct_url() {
        let client = IggyHttpClient::new("localhost", 3001);
        assert_eq!(client.base_url(), "http://localhost:3001");
    }

    #[test]
    fn client_from_config() {
        let config = IggyClientConfig {
            host: "iggy.example.com".to_string(),
            http_port: 8080,
            username: "admin".to_string(),
            password: "secret".to_string(),
        };
        let client = IggyHttpClient::from_config(&config);
        assert_eq!(client.base_url(), "http://iggy.example.com:8080");
    }

    #[test]
    fn client_starts_unauthenticated() {
        let client = IggyHttpClient::new("localhost", 3001);
        assert!(!client.is_authenticated());
    }

    #[tokio::test]
    async fn send_message_requires_auth() {
        let client = IggyHttpClient::new("localhost", 3001);
        let result = client.send_message("vibes", "events", b"test").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Not authenticated")
        );
    }

    #[test]
    fn token_cache_path_returns_some() {
        // Should return a path on most systems
        let path = IggyHttpClient::token_cache_path();
        // This could be None in some CI environments without a home dir
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("vibes"));
            assert!(p.to_string_lossy().contains("iggy-token"));
        }
    }

    #[test]
    fn load_cached_token_returns_none_when_no_file() {
        // Clear any existing token first
        let _ = IggyHttpClient::clear_cached_token();

        let client = IggyHttpClient::new("localhost", 3001);
        let token = client.load_cached_token();
        assert!(token.is_none());
    }

    #[test]
    fn cache_token_requires_token() {
        let client = IggyHttpClient::new("localhost", 3001);
        let result = client.cache_token();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No token to cache")
        );
    }

    #[test]
    fn cache_and_load_token_roundtrip() {
        // Clean up first
        let _ = IggyHttpClient::clear_cached_token();

        // Create client with a token
        let mut client = IggyHttpClient::new("localhost", 3001);
        client.token = Some("test-token-12345".to_string());

        // Cache the token
        client.cache_token().expect("Failed to cache token");

        // Load it back with a new client
        let client2 = IggyHttpClient::new("localhost", 3001);
        let loaded = client2.load_cached_token();

        assert_eq!(loaded, Some("test-token-12345".to_string()));

        // Clean up
        let _ = IggyHttpClient::clear_cached_token();
    }

    #[test]
    fn clear_cached_token_removes_file() {
        // Create a token file
        let mut client = IggyHttpClient::new("localhost", 3001);
        client.token = Some("temp-token".to_string());
        let _ = client.cache_token();

        // Verify it exists
        assert!(client.load_cached_token().is_some());

        // Clear it
        IggyHttpClient::clear_cached_token().expect("Failed to clear token");

        // Verify it's gone
        assert!(client.load_cached_token().is_none());
    }
}
