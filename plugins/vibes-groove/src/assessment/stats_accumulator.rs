//! Pre-computed statistics accumulator for assessment events.
//!
//! This module provides incremental statistics computation that updates
//! on each assessment event, enabling fast dashboard queries without
//! re-scanning all events.
//!
//! ## Pattern: Precomputed Aggregations
//!
//! This follows the standard pattern for aggregated statistics:
//! 1. Maintain in-memory accumulator that updates incrementally
//! 2. Emit snapshot to Iggy topic after each update
//! 3. On recovery, load latest snapshot and replay from that offset
//!
//! ## Usage
//!
//! ```ignore
//! let mut accumulator = StatsAccumulator::new();
//! accumulator.update(&assessment_result);
//! let snapshot = accumulator.snapshot();
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Tier distribution counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TierCounts {
    pub lightweight: usize,
    pub medium: usize,
    pub heavy: usize,
    pub checkpoint: usize,
}

impl TierCounts {
    /// Increment the count for a given tier.
    pub fn increment(&mut self, tier: &str) {
        match tier {
            "lightweight" => self.lightweight += 1,
            "medium" => self.medium += 1,
            "heavy" => self.heavy += 1,
            "checkpoint" => self.checkpoint += 1,
            _ => {}
        }
    }

    /// Total count across all tiers.
    pub fn total(&self) -> usize {
        self.lightweight + self.medium + self.heavy + self.checkpoint
    }
}

/// A snapshot of the current statistics state.
///
/// This is serialized to Iggy for persistence and recovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatsSnapshot {
    /// Global tier distribution across all sessions.
    pub global: TierCounts,
    /// Per-session tier distribution.
    pub sessions: HashMap<String, TierCounts>,
    /// Total number of assessments processed.
    pub total_assessments: usize,
    /// Offset of the last processed assessment event (for recovery).
    pub last_offset: u64,
    /// Timestamp when this snapshot was created (Unix millis).
    pub timestamp_ms: u64,
}

/// Accumulator for pre-computing assessment statistics.
///
/// Updates incrementally as assessment events are processed.
pub struct StatsAccumulator {
    global: TierCounts,
    sessions: HashMap<String, TierCounts>,
    total_assessments: usize,
    last_offset: u64,
}

impl StatsAccumulator {
    /// Create a new empty accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            global: TierCounts::default(),
            sessions: HashMap::new(),
            total_assessments: 0,
            last_offset: 0,
        }
    }

    /// Restore accumulator state from a snapshot.
    #[must_use]
    pub fn from_snapshot(snapshot: StatsSnapshot) -> Self {
        Self {
            global: snapshot.global,
            sessions: snapshot.sessions,
            total_assessments: snapshot.total_assessments,
            last_offset: snapshot.last_offset,
        }
    }

    /// Update statistics with a new assessment result.
    pub fn update(&mut self, session_id: &str, tier: &str, offset: u64) {
        // Update global counts
        self.global.increment(tier);

        // Update per-session counts
        self.sessions
            .entry(session_id.to_string())
            .or_default()
            .increment(tier);

        self.total_assessments += 1;
        self.last_offset = offset;
    }

    /// Get the current global tier distribution.
    #[must_use]
    pub fn global_counts(&self) -> &TierCounts {
        &self.global
    }

    /// Get tier distribution for a specific session.
    #[must_use]
    pub fn session_counts(&self, session_id: &str) -> Option<&TierCounts> {
        self.sessions.get(session_id)
    }

    /// Get all session statistics.
    #[must_use]
    pub fn all_session_counts(&self) -> &HashMap<String, TierCounts> {
        &self.sessions
    }

    /// Get total number of assessments.
    #[must_use]
    pub fn total_assessments(&self) -> usize {
        self.total_assessments
    }

    /// Get the last processed offset.
    #[must_use]
    pub fn last_offset(&self) -> u64 {
        self.last_offset
    }

    /// Create a snapshot of the current state for persistence.
    #[must_use]
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            global: self.global.clone(),
            sessions: self.sessions.clone(),
            total_assessments: self.total_assessments,
            last_offset: self.last_offset,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Get top N sessions by total assessment count.
    #[must_use]
    pub fn top_sessions(&self, n: usize) -> Vec<(String, usize)> {
        let mut sessions: Vec<_> = self
            .sessions
            .iter()
            .map(|(id, counts)| (id.clone(), counts.total()))
            .collect();
        sessions.sort_by(|a, b| b.1.cmp(&a.1));
        sessions.truncate(n);
        sessions
    }
}

impl Default for StatsAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accumulator_has_zero_counts() {
        let acc = StatsAccumulator::new();

        assert_eq!(acc.global_counts().lightweight, 0);
        assert_eq!(acc.global_counts().medium, 0);
        assert_eq!(acc.global_counts().heavy, 0);
        assert_eq!(acc.global_counts().checkpoint, 0);
        assert_eq!(acc.total_assessments(), 0);
        assert_eq!(acc.last_offset(), 0);
    }

    #[test]
    fn update_increments_global_counts() {
        let mut acc = StatsAccumulator::new();

        acc.update("session-1", "lightweight", 1);
        acc.update("session-1", "lightweight", 2);
        acc.update("session-1", "medium", 3);

        assert_eq!(acc.global_counts().lightweight, 2);
        assert_eq!(acc.global_counts().medium, 1);
        assert_eq!(acc.global_counts().heavy, 0);
        assert_eq!(acc.total_assessments(), 3);
        assert_eq!(acc.last_offset(), 3);
    }

    #[test]
    fn update_increments_session_counts() {
        let mut acc = StatsAccumulator::new();

        acc.update("session-a", "lightweight", 1);
        acc.update("session-a", "medium", 2);
        acc.update("session-b", "heavy", 3);

        let session_a = acc.session_counts("session-a").unwrap();
        assert_eq!(session_a.lightweight, 1);
        assert_eq!(session_a.medium, 1);
        assert_eq!(session_a.heavy, 0);

        let session_b = acc.session_counts("session-b").unwrap();
        assert_eq!(session_b.heavy, 1);
        assert_eq!(session_b.lightweight, 0);
    }

    #[test]
    fn snapshot_captures_current_state() {
        let mut acc = StatsAccumulator::new();
        acc.update("sess-1", "lightweight", 10);
        acc.update("sess-2", "heavy", 20);

        let snapshot = acc.snapshot();

        assert_eq!(snapshot.global.lightweight, 1);
        assert_eq!(snapshot.global.heavy, 1);
        assert_eq!(snapshot.total_assessments, 2);
        assert_eq!(snapshot.last_offset, 20);
        assert_eq!(snapshot.sessions.len(), 2);
        assert!(snapshot.timestamp_ms > 0);
    }

    #[test]
    fn from_snapshot_restores_state() {
        let snapshot = StatsSnapshot {
            global: TierCounts {
                lightweight: 100,
                medium: 50,
                heavy: 10,
                checkpoint: 5,
            },
            sessions: {
                let mut m = HashMap::new();
                m.insert(
                    "restored-session".to_string(),
                    TierCounts {
                        lightweight: 50,
                        medium: 25,
                        heavy: 5,
                        checkpoint: 2,
                    },
                );
                m
            },
            total_assessments: 165,
            last_offset: 1000,
            timestamp_ms: 12345,
        };

        let acc = StatsAccumulator::from_snapshot(snapshot);

        assert_eq!(acc.global_counts().lightweight, 100);
        assert_eq!(acc.global_counts().medium, 50);
        assert_eq!(acc.total_assessments(), 165);
        assert_eq!(acc.last_offset(), 1000);
        assert!(acc.session_counts("restored-session").is_some());
    }

    #[test]
    fn top_sessions_returns_sorted_by_count() {
        let mut acc = StatsAccumulator::new();

        // Session B has most assessments
        acc.update("session-b", "lightweight", 1);
        acc.update("session-b", "lightweight", 2);
        acc.update("session-b", "medium", 3);

        // Session A has fewer
        acc.update("session-a", "heavy", 4);

        // Session C has middle count
        acc.update("session-c", "checkpoint", 5);
        acc.update("session-c", "checkpoint", 6);

        let top = acc.top_sessions(10);

        assert_eq!(top.len(), 3);
        assert_eq!(top[0].0, "session-b");
        assert_eq!(top[0].1, 3);
        assert_eq!(top[1].0, "session-c");
        assert_eq!(top[1].1, 2);
        assert_eq!(top[2].0, "session-a");
        assert_eq!(top[2].1, 1);
    }

    #[test]
    fn snapshot_serialization_roundtrip() {
        let mut acc = StatsAccumulator::new();
        acc.update("sess-1", "lightweight", 1);
        acc.update("sess-1", "medium", 2);
        acc.update("sess-2", "heavy", 3);

        let snapshot = acc.snapshot();
        let json = serde_json::to_string(&snapshot).expect("serialize");
        let restored: StatsSnapshot = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.global, snapshot.global);
        assert_eq!(restored.sessions, snapshot.sessions);
        assert_eq!(restored.total_assessments, snapshot.total_assessments);
        assert_eq!(restored.last_offset, snapshot.last_offset);
    }

    #[test]
    fn tier_counts_total() {
        let counts = TierCounts {
            lightweight: 10,
            medium: 5,
            heavy: 3,
            checkpoint: 2,
        };

        assert_eq!(counts.total(), 20);
    }

    #[test]
    fn unknown_tier_is_ignored() {
        let mut acc = StatsAccumulator::new();
        acc.update("session-1", "unknown_tier", 1);

        assert_eq!(acc.global_counts().total(), 0);
        assert_eq!(acc.total_assessments(), 1); // Still counts as an assessment
    }
}
