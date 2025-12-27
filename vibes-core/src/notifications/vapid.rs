//! VAPID key management for Web Push

use std::path::{Path, PathBuf};

use base64ct::{Base64UrlUnpadded, Encoding};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::NotificationError;

/// VAPID keys file name
const VAPID_KEYS_FILE: &str = "vapid_keys.json";

/// VAPID key manager for Web Push authentication
pub struct VapidKeyManager {
    signing_key: SigningKey,
    public_key_base64: String,
    config_path: PathBuf,
}

/// VAPID keypair for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapidKeys {
    /// Private key (PEM-encoded PKCS#8)
    pub private_key_pem: String,
    /// Public key (base64url-encoded, uncompressed point)
    pub public_key: String,
}

impl VapidKeyManager {
    /// Load existing keys or generate new ones
    pub async fn load_or_generate(config_dir: &Path) -> Result<Self, NotificationError> {
        let keys_path = config_dir.join(VAPID_KEYS_FILE);

        let (signing_key, public_key_base64) = if keys_path.exists() {
            // Load existing keys
            let content = fs::read_to_string(&keys_path).await.map_err(|e| {
                NotificationError::Config(format!("failed to read VAPID keys: {}", e))
            })?;
            let keys: VapidKeys = serde_json::from_str(&content).map_err(|e| {
                NotificationError::Config(format!("invalid VAPID keys file: {}", e))
            })?;

            let signing_key = SigningKey::from_pkcs8_pem(&keys.private_key_pem).map_err(|e| {
                NotificationError::Config(format!("invalid VAPID private key: {}", e))
            })?;

            (signing_key, keys.public_key)
        } else {
            // Generate new keys
            let signing_key = SigningKey::random(&mut OsRng);
            let public_key = signing_key.verifying_key();

            // Encode public key as uncompressed point (65 bytes: 0x04 || x || y)
            let public_key_bytes = public_key.to_encoded_point(false);
            let public_key_base64 = Base64UrlUnpadded::encode_string(public_key_bytes.as_bytes());

            // Encode private key as PEM
            let private_key_pem = signing_key.to_pkcs8_pem(Default::default()).map_err(|e| {
                NotificationError::Config(format!("failed to encode private key: {}", e))
            })?;

            let keys = VapidKeys {
                private_key_pem: private_key_pem.to_string(),
                public_key: public_key_base64.clone(),
            };

            // Ensure config directory exists
            if let Some(parent) = keys_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    NotificationError::Config(format!("failed to create config dir: {}", e))
                })?;
            }

            // Save keys
            let content = serde_json::to_string_pretty(&keys).map_err(|e| {
                NotificationError::Config(format!("failed to serialize VAPID keys: {}", e))
            })?;
            fs::write(&keys_path, content).await.map_err(|e| {
                NotificationError::Config(format!("failed to write VAPID keys: {}", e))
            })?;

            (signing_key, public_key_base64)
        };

        Ok(Self {
            signing_key,
            public_key_base64,
            config_path: keys_path,
        })
    }

    /// Get the public key for browser subscription (base64url-encoded)
    pub fn public_key(&self) -> &str {
        &self.public_key_base64
    }

    /// Get the signing key for VAPID signatures
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Get the path where keys are stored
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_generate_new_keys() {
        let temp_dir = tempdir().unwrap();
        let manager = VapidKeyManager::load_or_generate(temp_dir.path())
            .await
            .unwrap();

        assert!(!manager.public_key().is_empty());
        assert!(manager.config_path().exists());
        // Public key should be base64url encoded (65 bytes = ~87 chars)
        assert!(manager.public_key().len() > 80);
    }

    #[tokio::test]
    async fn test_load_existing_keys() {
        let temp_dir = tempdir().unwrap();

        // Generate keys first time
        let manager1 = VapidKeyManager::load_or_generate(temp_dir.path())
            .await
            .unwrap();
        let public_key = manager1.public_key().to_string();

        // Load keys second time
        let manager2 = VapidKeyManager::load_or_generate(temp_dir.path())
            .await
            .unwrap();

        // Should be the same keys
        assert_eq!(manager2.public_key(), public_key);
    }

    #[tokio::test]
    async fn test_public_key_format() {
        let temp_dir = tempdir().unwrap();
        let manager = VapidKeyManager::load_or_generate(temp_dir.path())
            .await
            .unwrap();

        // Decode and verify it's a valid uncompressed point (starts with 0x04)
        let decoded = Base64UrlUnpadded::decode_vec(manager.public_key()).unwrap();
        assert_eq!(decoded.len(), 65);
        assert_eq!(decoded[0], 0x04);
    }
}
