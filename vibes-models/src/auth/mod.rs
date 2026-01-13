//! Credential management for API keys.
//!
//! Provides secure storage of API keys using the system keyring with
//! environment variable fallback for CI/deployment scenarios.
//!
//! # Example
//!
//! ```ignore
//! use vibes_models::auth::CredentialStore;
//!
//! let store = CredentialStore::new("vibes").with_env_fallback();
//!
//! // Store a key in the system keyring
//! store.set("anthropic", "sk-ant-...")?;
//!
//! // Retrieve it (checks keyring first, then env vars)
//! let key = store.get("anthropic")?;
//! ```

use std::collections::HashSet;
use std::env;

use secrecy::{ExposeSecret, SecretString};
use tracing::debug;

use crate::{Error, Result};

/// A secure API key that prevents accidental logging.
///
/// The key is wrapped in `SecretString` which:
/// - Implements `Debug` as `"[REDACTED]"`
/// - Zeroizes memory on drop
/// - Requires explicit `.expose_secret()` to access the value
#[derive(Clone)]
pub struct ApiKey(SecretString);

impl ApiKey {
    /// Create a new API key from a string.
    pub fn new(key: impl Into<String>) -> Self {
        Self(SecretString::from(key.into()))
    }

    /// Expose the secret key value.
    ///
    /// Use sparingly - only when actually sending to an API.
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey([REDACTED])")
    }
}

impl From<String> for ApiKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for ApiKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Environment variable names for each provider.
const ENV_VARS: &[(&str, &str)] = &[
    ("anthropic", "ANTHROPIC_API_KEY"),
    ("openai", "OPENAI_API_KEY"),
    ("google", "GOOGLE_API_KEY"),
    ("groq", "GROQ_API_KEY"),
    ("mistral", "MISTRAL_API_KEY"),
    ("cohere", "COHERE_API_KEY"),
];

/// Get the environment variable name for a provider.
fn env_var_for_provider(provider: &str) -> Option<&'static str> {
    ENV_VARS
        .iter()
        .find(|(p, _)| *p == provider)
        .map(|(_, v)| *v)
}

/// Secure credential storage with system keyring and environment fallback.
///
/// # Storage Priority
///
/// When retrieving credentials:
/// 1. System keyring (if available)
/// 2. Environment variables (if `env_fallback` is enabled)
///
/// When storing credentials:
/// - Always uses system keyring
/// - Environment variables are read-only
///
/// # Thread Safety
///
/// The keyring operations are thread-safe. Multiple instances can
/// access the same credentials.
pub struct CredentialStore {
    service_name: String,
    env_fallback: bool,
}

impl CredentialStore {
    /// Create a new credential store.
    ///
    /// # Arguments
    ///
    /// * `service_name` - Service identifier for keyring (e.g., "vibes")
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            env_fallback: false,
        }
    }

    /// Enable environment variable fallback.
    ///
    /// When enabled, if a credential is not found in the keyring,
    /// the store will check for provider-specific environment variables.
    pub fn with_env_fallback(mut self) -> Self {
        self.env_fallback = true;
        self
    }

    /// Get an API key for a provider.
    ///
    /// Checks the system keyring first, then environment variables
    /// if fallback is enabled.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider name (e.g., "anthropic", "openai")
    ///
    /// # Errors
    ///
    /// Returns `Error::CredentialsNotFound` if no credentials are found.
    pub fn get(&self, provider: &str) -> Result<ApiKey> {
        // Try keyring first
        if let Some(key) = self.get_from_keyring(provider) {
            debug!(provider, "retrieved API key from keyring");
            return Ok(key);
        }

        // Try environment variable fallback
        if self.env_fallback
            && let Some(key) = self.get_from_env(provider)
        {
            debug!(provider, "retrieved API key from environment");
            return Ok(key);
        }

        Err(Error::CredentialsNotFound(provider.to_string()))
    }

    /// Store an API key for a provider in the system keyring.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider name
    /// * `key` - The API key to store
    ///
    /// # Errors
    ///
    /// Returns `Error::Keyring` if the keyring operation fails.
    pub fn set(&self, provider: &str, key: &str) -> Result<()> {
        let entry = self.keyring_entry(provider)?;
        entry
            .set_password(key)
            .map_err(|e| Error::Keyring(e.to_string()))?;
        debug!(provider, "stored API key in keyring");
        Ok(())
    }

    /// Delete an API key from the system keyring.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider name
    ///
    /// # Errors
    ///
    /// Returns `Error::Keyring` if the keyring operation fails.
    /// Returns `Error::CredentialsNotFound` if no credentials exist.
    pub fn delete(&self, provider: &str) -> Result<()> {
        let entry = self.keyring_entry(provider)?;
        entry.delete_credential().map_err(|e| match e {
            keyring::Error::NoEntry => Error::CredentialsNotFound(provider.to_string()),
            _ => Error::Keyring(e.to_string()),
        })?;
        debug!(provider, "deleted API key from keyring");
        Ok(())
    }

    /// Check if credentials exist for a provider.
    ///
    /// Checks both keyring and environment variables (if fallback enabled).
    pub fn has(&self, provider: &str) -> bool {
        self.get(provider).is_ok()
    }

    /// List all providers with stored credentials.
    ///
    /// Returns providers that have credentials in either the keyring
    /// or environment variables (if fallback enabled).
    pub fn list_providers(&self) -> Vec<String> {
        let mut providers = HashSet::new();

        // Check known providers in keyring
        for (provider, _) in ENV_VARS {
            if self.has_in_keyring(provider) {
                providers.insert(provider.to_string());
            }
        }

        // Check environment variables if fallback enabled
        if self.env_fallback {
            for (provider, env_var) in ENV_VARS {
                if env::var(env_var).is_ok() {
                    providers.insert(provider.to_string());
                }
            }
        }

        let mut result: Vec<_> = providers.into_iter().collect();
        result.sort();
        result
    }

    /// Check if a credential exists in the keyring (not env).
    pub fn has_in_keyring(&self, provider: &str) -> bool {
        self.get_from_keyring(provider).is_some()
    }

    /// Check if a credential exists in environment variables.
    pub fn has_in_env(&self, provider: &str) -> bool {
        self.get_from_env(provider).is_some()
    }

    /// Get the source of a credential (keyring or env).
    pub fn credential_source(&self, provider: &str) -> Option<CredentialSource> {
        if self.has_in_keyring(provider) {
            Some(CredentialSource::Keyring)
        } else if self.env_fallback && self.has_in_env(provider) {
            Some(CredentialSource::Environment)
        } else {
            None
        }
    }

    fn keyring_entry(&self, provider: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(&self.service_name, provider).map_err(|e| Error::Keyring(e.to_string()))
    }

    fn get_from_keyring(&self, provider: &str) -> Option<ApiKey> {
        let entry = self.keyring_entry(provider).ok()?;
        entry.get_password().ok().map(ApiKey::new)
    }

    fn get_from_env(&self, provider: &str) -> Option<ApiKey> {
        let env_var = env_var_for_provider(provider)?;
        env::var(env_var).ok().map(ApiKey::new)
    }
}

/// Source of a stored credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    /// Stored in system keyring.
    Keyring,
    /// From environment variable.
    Environment,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_debug_is_redacted() {
        let key = ApiKey::new("sk-secret-key-12345");
        let debug = format!("{:?}", key);
        assert_eq!(debug, "ApiKey([REDACTED])");
        assert!(!debug.contains("sk-secret"));
    }

    #[test]
    fn api_key_expose_secret_returns_value() {
        let key = ApiKey::new("sk-secret-key-12345");
        assert_eq!(key.expose_secret(), "sk-secret-key-12345");
    }

    #[test]
    fn api_key_from_string() {
        let key: ApiKey = "my-key".into();
        assert_eq!(key.expose_secret(), "my-key");

        let key: ApiKey = String::from("my-key").into();
        assert_eq!(key.expose_secret(), "my-key");
    }

    #[test]
    fn env_var_for_known_providers() {
        assert_eq!(env_var_for_provider("anthropic"), Some("ANTHROPIC_API_KEY"));
        assert_eq!(env_var_for_provider("openai"), Some("OPENAI_API_KEY"));
        assert_eq!(env_var_for_provider("google"), Some("GOOGLE_API_KEY"));
        assert_eq!(env_var_for_provider("unknown"), None);
    }

    #[test]
    fn credential_store_new() {
        let store = CredentialStore::new("test-service");
        assert_eq!(store.service_name, "test-service");
        assert!(!store.env_fallback);
    }

    #[test]
    fn credential_store_with_env_fallback() {
        let store = CredentialStore::new("test").with_env_fallback();
        assert!(store.env_fallback);
    }

    #[test]
    fn credential_store_env_fallback_works() {
        // Set a test env var
        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::set_var("ANTHROPIC_API_KEY", "test-key-from-env") };

        let store = CredentialStore::new("test-vibes-nonexistent").with_env_fallback();

        // Should find it via env fallback (keyring won't have it)
        let result = store.get("anthropic");

        // Clean up
        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };

        // The key should have been found from env
        assert!(result.is_ok());
        assert_eq!(result.unwrap().expose_secret(), "test-key-from-env");
    }

    #[test]
    fn credential_store_without_fallback_fails() {
        let store = CredentialStore::new("test-vibes-nonexistent");

        // Without env fallback, should fail if not in keyring
        let result = store.get("unknown-provider");
        assert!(result.is_err());
    }

    #[test]
    fn credential_source_from_env() {
        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::set_var("OPENAI_API_KEY", "test-key") };

        let store = CredentialStore::new("test-vibes-nonexistent").with_env_fallback();
        let source = store.credential_source("openai");

        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::remove_var("OPENAI_API_KEY") };

        assert_eq!(source, Some(CredentialSource::Environment));
    }

    #[test]
    fn list_providers_includes_env_vars() {
        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::set_var("GROQ_API_KEY", "test-key") };

        let store = CredentialStore::new("test-vibes-nonexistent").with_env_fallback();
        let providers = store.list_providers();

        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::remove_var("GROQ_API_KEY") };

        assert!(providers.contains(&"groq".to_string()));
    }

    #[test]
    fn has_in_env_detects_env_var() {
        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::set_var("MISTRAL_API_KEY", "test-key") };

        let store = CredentialStore::new("test");
        assert!(store.has_in_env("mistral"));
        assert!(!store.has_in_env("unknown"));

        // SAFETY: Tests run single-threaded via cargo test default
        unsafe { env::remove_var("MISTRAL_API_KEY") };
    }
}
