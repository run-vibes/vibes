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

    /// Add a subscriber to this session
    pub fn add_subscriber(&mut self, client_id: ClientId) {
        self.subscriber_ids.insert(client_id);
    }

    /// Remove a subscriber from this session
    ///
    /// Returns true if the removed client was the owner.
    pub fn remove_subscriber(&mut self, client_id: &ClientId) -> bool {
        self.subscriber_ids.remove(client_id);
        &self.owner_id == client_id
    }

    /// Check if client is a subscriber
    pub fn is_subscriber(&self, client_id: &ClientId) -> bool {
        self.subscriber_ids.contains(client_id)
    }

    /// Check if client is the owner
    pub fn is_owner(&self, client_id: &ClientId) -> bool {
        &self.owner_id == client_id
    }

    /// Returns true if session should be cleaned up (no subscribers)
    pub fn should_cleanup(&self) -> bool {
        self.subscriber_ids.is_empty()
    }

    /// Transfer ownership to another subscriber
    ///
    /// Returns true if transfer succeeded (new_owner was a subscriber).
    /// Updates owned_since timestamp on success.
    pub fn transfer_to(&mut self, new_owner: &ClientId) -> bool {
        if self.subscriber_ids.contains(new_owner) {
            self.owner_id = new_owner.clone();
            self.owned_since = SystemTime::now();
            true
        } else {
            false
        }
    }

    /// Pick next owner from remaining subscribers
    ///
    /// Returns None if no subscribers remain.
    pub fn pick_next_owner(&self) -> Option<&ClientId> {
        self.subscriber_ids.iter().next()
    }

    /// Get the number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.subscriber_ids.len()
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

    // ==================== Subscriber Management Tests ====================

    #[test]
    fn add_subscriber_adds_to_set() {
        let mut ownership = SessionOwnership::new("client-1".to_string());

        ownership.add_subscriber("client-2".to_string());

        assert!(ownership.is_subscriber(&"client-2".to_string()));
        assert_eq!(ownership.subscriber_ids.len(), 2);
    }

    #[test]
    fn add_subscriber_is_idempotent() {
        let mut ownership = SessionOwnership::new("client-1".to_string());

        ownership.add_subscriber("client-2".to_string());
        ownership.add_subscriber("client-2".to_string());

        assert_eq!(ownership.subscriber_ids.len(), 2);
    }

    #[test]
    fn remove_subscriber_removes_from_set() {
        let mut ownership = SessionOwnership::new("client-1".to_string());
        ownership.add_subscriber("client-2".to_string());

        let was_owner = ownership.remove_subscriber(&"client-2".to_string());

        assert!(!was_owner);
        assert!(!ownership.is_subscriber(&"client-2".to_string()));
        assert_eq!(ownership.subscriber_ids.len(), 1);
    }

    #[test]
    fn remove_subscriber_returns_true_for_owner() {
        let mut ownership = SessionOwnership::new("client-1".to_string());
        ownership.add_subscriber("client-2".to_string());

        let was_owner = ownership.remove_subscriber(&"client-1".to_string());

        assert!(was_owner);
    }

    #[test]
    fn is_owner_returns_correct_value() {
        let ownership = SessionOwnership::new("client-1".to_string());

        assert!(ownership.is_owner(&"client-1".to_string()));
        assert!(!ownership.is_owner(&"client-2".to_string()));
    }

    #[test]
    fn should_cleanup_when_no_subscribers() {
        let mut ownership = SessionOwnership::new("client-1".to_string());
        ownership.remove_subscriber(&"client-1".to_string());

        assert!(ownership.should_cleanup());
    }

    #[test]
    fn should_not_cleanup_when_subscribers_exist() {
        let ownership = SessionOwnership::new("client-1".to_string());

        assert!(!ownership.should_cleanup());
    }

    // ==================== Ownership Transfer Tests ====================

    #[test]
    fn transfer_to_subscriber_succeeds() {
        let mut ownership = SessionOwnership::new("client-1".to_string());
        ownership.add_subscriber("client-2".to_string());

        let success = ownership.transfer_to(&"client-2".to_string());

        assert!(success);
        assert!(ownership.is_owner(&"client-2".to_string()));
        assert!(!ownership.is_owner(&"client-1".to_string()));
    }

    #[test]
    fn transfer_to_non_subscriber_fails() {
        let mut ownership = SessionOwnership::new("client-1".to_string());

        let success = ownership.transfer_to(&"client-2".to_string());

        assert!(!success);
        assert!(ownership.is_owner(&"client-1".to_string()));
    }

    #[test]
    fn pick_next_owner_returns_subscriber() {
        let mut ownership = SessionOwnership::new("client-1".to_string());
        ownership.add_subscriber("client-2".to_string());

        let next = ownership.pick_next_owner();

        assert!(next.is_some());
        assert!(ownership.is_subscriber(next.unwrap()));
    }

    #[test]
    fn pick_next_owner_returns_none_when_empty() {
        let mut ownership = SessionOwnership::new("client-1".to_string());
        ownership.remove_subscriber(&"client-1".to_string());

        let next = ownership.pick_next_owner();

        assert!(next.is_none());
    }

    #[test]
    fn subscriber_count_tracks_subscribers() {
        let mut ownership = SessionOwnership::new("client-1".to_string());
        assert_eq!(ownership.subscriber_count(), 1);

        ownership.add_subscriber("client-2".to_string());
        assert_eq!(ownership.subscriber_count(), 2);

        ownership.remove_subscriber(&"client-1".to_string());
        assert_eq!(ownership.subscriber_count(), 1);
    }
}
