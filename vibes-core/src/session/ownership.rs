//! Session ownership and subscription tracking

use std::collections::HashSet;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// Unique identifier for a connected client
pub type ClientId = String;

/// Session ownership and subscription tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOwnership {
    /// The client that created/owns this session
    pub owner_id: ClientId,
    /// All clients currently subscribed to this session
    pub subscriber_ids: HashSet<ClientId>,
    /// When ownership was last transferred
    pub owned_since: SystemTime,
}

impl SessionOwnership {
    /// Create new ownership with the given owner
    ///
    /// The owner is automatically added as a subscriber.
    pub fn new(owner_id: ClientId) -> Self {
        Self {
            owner_id: owner_id.clone(),
            subscriber_ids: HashSet::from([owner_id]),
            owned_since: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ownership_includes_owner_as_subscriber() {
        let ownership = SessionOwnership::new("client-1".to_string());

        assert_eq!(ownership.owner_id, "client-1");
        assert!(ownership.subscriber_ids.contains("client-1"));
        assert_eq!(ownership.subscriber_ids.len(), 1);
    }
}
