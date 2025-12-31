//! Provenance tracking for learnings
//!
//! Provides content hashing, metadata, and custody chain tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// SHA-256 content hash for integrity verification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(#[serde(with = "hex_bytes")] pub [u8; 32]);

impl ContentHash {
    /// Compute hash from content
    pub fn from_content(content: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        Self(result.into())
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Verify content matches this hash
    pub fn verify(&self, content: &str) -> bool {
        let computed = Self::from_content(content);
        self == &computed
    }
}

// Custom serialization for hex bytes
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("expected 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

/// Full provenance record for a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Hash of the current content
    pub content_hash: ContentHash,
    /// When and by whom this was created
    pub created: CreationEvent,
    /// Chain of modifications
    pub custody_chain: Vec<CustodyEvent>,
}

impl Provenance {
    /// Create new provenance for content
    pub fn new(content: &str, creator_id: impl Into<String>) -> Self {
        Self {
            content_hash: ContentHash::from_content(content),
            created: CreationEvent {
                creator_id: creator_id.into(),
                created_at: Utc::now(),
                device_id: None,
                source_type: "user_created".to_string(),
            },
            custody_chain: Vec::new(),
        }
    }

    /// Record a modification in the custody chain
    pub fn record_modification(
        &mut self,
        new_content: &str,
        actor: impl Into<String>,
        event_type: CustodyEventType,
    ) {
        let previous_hash = self.content_hash.clone();
        let new_hash = ContentHash::from_content(new_content);

        self.custody_chain.push(CustodyEvent {
            event_type,
            timestamp: Utc::now(),
            actor: actor.into(),
            previous_hash,
            new_hash: new_hash.clone(),
        });

        self.content_hash = new_hash;
    }

    /// Verify content matches the current hash
    pub fn verify(&self, content: &str) -> bool {
        self.content_hash.verify(content)
    }
}

/// Record of initial creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationEvent {
    /// ID of the creator
    pub creator_id: String,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Device ID if available
    pub device_id: Option<String>,
    /// Type of source (user_created, transcript, imported, etc.)
    pub source_type: String,
}

/// Record of a custody chain event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEvent {
    /// Type of event
    pub event_type: CustodyEventType,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Who performed the action
    pub actor: String,
    /// Hash before the change
    pub previous_hash: ContentHash,
    /// Hash after the change
    pub new_hash: ContentHash,
}

/// Types of custody chain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyEventType {
    /// Content was modified
    Modified,
    /// Learning was promoted to different tier
    Promoted { from_tier: String, to_tier: String },
    /// Learning was imported from external source
    Imported { source: String },
    /// Learning was transferred between users/orgs
    Transferred { from: String, to: String },
    /// Learning was verified/reviewed
    Verified { verifier: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_from_content() {
        let content = "Use thiserror for library errors";
        let hash = ContentHash::from_content(content);

        // Same content produces same hash
        let hash2 = ContentHash::from_content(content);
        assert_eq!(hash, hash2);

        // Different content produces different hash
        let hash3 = ContentHash::from_content("Different content");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_content_hash_hex_roundtrip() {
        let content = "Test content";
        let hash = ContentHash::from_content(content);
        let hex = hash.to_hex();
        let parsed = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_content_hash_verification() {
        let content = "Original content";
        let hash = ContentHash::from_content(content);

        assert!(hash.verify(content));
        assert!(!hash.verify("Modified content"));
    }

    #[test]
    fn test_content_hash_serialization() {
        let hash = ContentHash::from_content("test");
        let json = serde_json::to_string(&hash).unwrap();
        let parsed: ContentHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_provenance_new() {
        let content = "Test learning content";
        let prov = Provenance::new(content, "alice");

        assert!(prov.verify(content));
        assert_eq!(prov.created.creator_id, "alice");
        assert!(prov.custody_chain.is_empty());
    }

    #[test]
    fn test_provenance_modification_tracking() {
        let content = "Original content";
        let mut prov = Provenance::new(content, "alice");

        let new_content = "Modified content";
        prov.record_modification(new_content, "bob", CustodyEventType::Modified);

        assert!(!prov.verify(content)); // Old content no longer matches
        assert!(prov.verify(new_content)); // New content matches
        assert_eq!(prov.custody_chain.len(), 1);
        assert_eq!(prov.custody_chain[0].actor, "bob");
    }

    #[test]
    fn test_provenance_custody_chain() {
        let mut prov = Provenance::new("v1", "alice");
        prov.record_modification("v2", "bob", CustodyEventType::Modified);
        prov.record_modification(
            "v3",
            "curator",
            CustodyEventType::Verified {
                verifier: "curator".into(),
            },
        );

        assert_eq!(prov.custody_chain.len(), 2);
        assert!(prov.verify("v3"));
    }
}
