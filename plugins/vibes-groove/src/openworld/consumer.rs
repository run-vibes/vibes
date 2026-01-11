//! OpenWorld event streaming via Iggy
//!
//! Provides event production for open-world adaptation events, enabling
//! audit trails and downstream processing of novelty detection, gap lifecycle,
//! and strategy feedback events.
//!
//! # Architecture
//!
//! Events are routed to topics by type:
//! - `novelty` - Novelty detection and cluster updates
//! - `gaps` - Gap creation, status changes, solution generation
//! - `feedback` - Strategy feedback adjustments
//!
//! # Reconnect Buffer
//!
//! When disconnected from Iggy, events are buffered in memory (up to 10,000).
//! On reconnection, buffered events are flushed.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use iggy::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::Result;
use crate::error::GrooveError;
use crate::types::LearningId;
use vibes_iggy::IggyManager;

use super::types::{
    AnomalyCluster, CapabilityGap, ClusterId, GapId, GapStatus, OpenWorldEvent, PatternFingerprint,
    SuggestedSolution,
};

/// Stream name for open-world events.
pub const OPENWORLD_STREAM: &str = "groove.openworld";

/// Topic names for open-world events in Iggy.
pub mod topics {
    /// Topic for novelty detection events (fingerprints, clusters).
    pub const NOVELTY: &str = "novelty";
    /// Topic for gap lifecycle events (creation, status, solutions).
    pub const GAPS: &str = "gaps";
    /// Topic for strategy feedback events.
    pub const FEEDBACK: &str = "feedback";
    /// Number of partitions (1 for simple offset tracking).
    pub const PARTITION_COUNT: u32 = 1;
}

/// Check if an Iggy error indicates a resource already exists.
fn is_already_exists_error(e: &IggyError) -> bool {
    let err_str = e.to_string();
    err_str.contains("already exists")
        || err_str.contains("already_exists")
        || err_str.contains("AlreadyExists")
}

/// Maximum events to buffer during disconnect before dropping oldest.
const MAX_RECONNECT_BUFFER: usize = 10_000;

/// Statistics for the OpenWorld producer.
#[derive(Debug, Default, Clone)]
pub struct ProducerStats {
    /// Total events produced.
    pub events_produced: u64,
    /// Novelty events produced.
    pub novelty_events: u64,
    /// Gap events produced.
    pub gap_events: u64,
    /// Feedback events produced.
    pub feedback_events: u64,
    /// Events dropped due to buffer full.
    pub events_dropped: u64,
}

/// Configuration for the OpenWorld producer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWorldProducerConfig {
    /// Whether the producer is enabled.
    pub enabled: bool,
}

impl Default for OpenWorldProducerConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Producer for open-world adaptation events via Iggy.
///
/// Emits events for novelty detection, gap lifecycle, and strategy feedback
/// to the `groove.openworld` stream with topic-based routing.
pub struct OpenWorldProducer {
    /// Configuration.
    config: OpenWorldProducerConfig,

    /// The Iggy manager for server lifecycle.
    manager: Arc<IggyManager>,

    /// The Iggy client for sending messages.
    client: IggyClient,

    /// Buffer for events while disconnected.
    reconnect_buffer: RwLock<Vec<(String, OpenWorldEvent)>>,

    /// Whether connected to Iggy.
    connected: RwLock<bool>,

    /// Statistics counters.
    events_produced: AtomicU64,
    novelty_events: AtomicU64,
    gap_events: AtomicU64,
    feedback_events: AtomicU64,
    events_dropped: AtomicU64,
}

impl OpenWorldProducer {
    /// Create a new OpenWorldProducer.
    ///
    /// The manager should be started before calling this.
    /// Call `connect()` to establish the connection.
    pub fn new(manager: Arc<IggyManager>, config: OpenWorldProducerConfig) -> Self {
        let client = IggyClient::builder()
            .with_tcp()
            .with_server_address(manager.connection_address())
            .build()
            .expect("Failed to build Iggy client");

        Self {
            config,
            manager,
            client,
            reconnect_buffer: RwLock::new(Vec::new()),
            connected: RwLock::new(false),
            events_produced: AtomicU64::new(0),
            novelty_events: AtomicU64::new(0),
            gap_events: AtomicU64::new(0),
            feedback_events: AtomicU64::new(0),
            events_dropped: AtomicU64::new(0),
        }
    }

    /// Connect to Iggy and create stream/topics if needed.
    pub async fn connect(&self) -> Result<()> {
        // Connect to server
        self.client
            .connect()
            .await
            .map_err(|e| GrooveError::Database(format!("Failed to connect to Iggy: {}", e)))?;
        info!(
            "OpenWorldProducer connected to Iggy at {}",
            self.manager.connection_address()
        );

        // Login with default credentials
        self.client
            .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
            .await
            .map_err(|e| GrooveError::Database(format!("Failed to login to Iggy: {}", e)))?;
        debug!("Logged in to Iggy as root user");

        // Create stream if needed
        self.ensure_stream().await?;

        // Create topics if needed
        self.ensure_topics().await?;

        *self.connected.write().await = true;
        info!("OpenWorldProducer fully connected and ready");

        // Flush any buffered events
        self.flush_buffer().await?;

        Ok(())
    }

    /// Ensure the stream exists.
    async fn ensure_stream(&self) -> Result<()> {
        let streams = self
            .client
            .get_streams()
            .await
            .map_err(|e| GrooveError::Database(format!("Failed to get streams: {}", e)))?;

        let stream_exists = streams.iter().any(|s| s.name == OPENWORLD_STREAM);

        if stream_exists {
            debug!("Stream '{}' already exists", OPENWORLD_STREAM);
        } else {
            match self.client.create_stream(OPENWORLD_STREAM).await {
                Ok(_) => info!("Created stream '{}'", OPENWORLD_STREAM),
                Err(e) if is_already_exists_error(&e) => {
                    debug!("Stream already exists (concurrent creation)");
                }
                Err(e) => {
                    return Err(GrooveError::Database(format!(
                        "Failed to create stream: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Ensure all topics exist.
    async fn ensure_topics(&self) -> Result<()> {
        let stream_id = Identifier::named(OPENWORLD_STREAM)
            .map_err(|e| GrooveError::Database(format!("Invalid stream name: {}", e)))?;

        for topic_name in [topics::NOVELTY, topics::GAPS, topics::FEEDBACK] {
            match self
                .client
                .create_topic(
                    &stream_id,
                    topic_name,
                    topics::PARTITION_COUNT,
                    CompressionAlgorithm::None,
                    None,
                    IggyExpiry::NeverExpire,
                    MaxTopicSize::ServerDefault,
                )
                .await
            {
                Ok(_) => info!("Created topic '{}'", topic_name),
                Err(e) if is_already_exists_error(&e) => {
                    debug!("Topic '{}' already exists", topic_name);
                }
                Err(e) => {
                    return Err(GrooveError::Database(format!(
                        "Failed to create topic '{}': {}",
                        topic_name, e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Get current statistics.
    pub fn stats(&self) -> ProducerStats {
        ProducerStats {
            events_produced: self.events_produced.load(Ordering::Relaxed),
            novelty_events: self.novelty_events.load(Ordering::Relaxed),
            gap_events: self.gap_events.load(Ordering::Relaxed),
            feedback_events: self.feedback_events.load(Ordering::Relaxed),
            events_dropped: self.events_dropped.load(Ordering::Relaxed),
        }
    }

    /// Get configuration.
    pub fn config(&self) -> &OpenWorldProducerConfig {
        &self.config
    }

    // =========================================================================
    // Event emission methods
    // =========================================================================

    /// Emit a novelty detected event.
    pub async fn emit_novelty_detected(
        &self,
        fingerprint: &PatternFingerprint,
        cluster: Option<ClusterId>,
    ) -> Result<()> {
        let event = OpenWorldEvent::NoveltyDetected {
            fingerprint: fingerprint.clone(),
            cluster,
        };
        self.emit(topics::NOVELTY, event).await?;
        self.novelty_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Emit a cluster updated event.
    pub async fn emit_cluster_updated(&self, cluster: &AnomalyCluster) -> Result<()> {
        let event = OpenWorldEvent::ClusterUpdated {
            cluster: cluster.clone(),
        };
        self.emit(topics::NOVELTY, event).await?;
        self.novelty_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Emit a gap created event.
    pub async fn emit_gap_created(&self, gap: &CapabilityGap) -> Result<()> {
        let event = OpenWorldEvent::GapCreated { gap: gap.clone() };
        self.emit(topics::GAPS, event).await?;
        self.gap_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Emit a gap status changed event.
    pub async fn emit_gap_status_changed(
        &self,
        gap_id: GapId,
        old: GapStatus,
        new: GapStatus,
    ) -> Result<()> {
        let event = OpenWorldEvent::GapStatusChanged { gap_id, old, new };
        self.emit(topics::GAPS, event).await?;
        self.gap_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Emit solution generated events (one per solution).
    pub async fn emit_solutions_generated(
        &self,
        gap_id: GapId,
        solutions: &[SuggestedSolution],
    ) -> Result<()> {
        for solution in solutions {
            let event = OpenWorldEvent::SolutionGenerated {
                gap_id,
                solution: solution.clone(),
            };
            self.emit(topics::GAPS, event).await?;
            self.gap_events.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Emit a strategy feedback event.
    pub async fn emit_strategy_feedback(
        &self,
        learning_id: LearningId,
        adjustment: f64,
    ) -> Result<()> {
        let event = OpenWorldEvent::StrategyFeedback {
            learning_id,
            adjustment,
        };
        self.emit(topics::FEEDBACK, event).await?;
        self.feedback_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    // =========================================================================
    // Internal methods
    // =========================================================================

    /// Emit an event to a topic.
    async fn emit(&self, topic: &str, event: OpenWorldEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        self.events_produced.fetch_add(1, Ordering::Relaxed);

        if !*self.connected.read().await {
            self.buffer_event(topic, event).await;
            return Ok(());
        }

        match self.try_send(topic, &event).await {
            Ok(()) => {
                debug!(topic, "Emitted OpenWorld event");
                Ok(())
            }
            Err(e) => {
                warn!(topic, error = %e, "Failed to emit event, buffering");
                self.buffer_event(topic, event).await;
                Ok(())
            }
        }
    }

    /// Try to send an event to Iggy.
    async fn try_send(&self, topic: &str, event: &OpenWorldEvent) -> Result<()> {
        let payload =
            serde_json::to_vec(event).map_err(|e| GrooveError::Serialization(e.to_string()))?;

        let message = IggyMessage::builder()
            .payload(payload.into())
            .build()
            .map_err(|e| GrooveError::Database(format!("Failed to build message: {}", e)))?;

        let partitioning = Partitioning::balanced();

        let stream_id = Identifier::named(OPENWORLD_STREAM)
            .map_err(|e| GrooveError::Database(format!("Invalid stream name: {}", e)))?;
        let topic_id = Identifier::named(topic)
            .map_err(|e| GrooveError::Database(format!("Invalid topic name: {}", e)))?;

        let mut messages = [message];
        self.client
            .send_messages(&stream_id, &topic_id, &partitioning, &mut messages)
            .await
            .map_err(|e| GrooveError::Database(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    /// Buffer an event when disconnected.
    async fn buffer_event(&self, topic: &str, event: OpenWorldEvent) {
        let mut buffer = self.reconnect_buffer.write().await;

        if buffer.len() >= MAX_RECONNECT_BUFFER {
            warn!(
                buffer_size = buffer.len(),
                "Reconnect buffer full, dropping oldest event"
            );
            buffer.remove(0);
            self.events_dropped.fetch_add(1, Ordering::Relaxed);
        }

        buffer.push((topic.to_string(), event));
        debug!(
            buffer_size = buffer.len(),
            "Buffered event during disconnect"
        );
    }

    /// Flush buffered events after reconnection.
    async fn flush_buffer(&self) -> Result<()> {
        let events = std::mem::take(&mut *self.reconnect_buffer.write().await);

        if events.is_empty() {
            return Ok(());
        }

        info!(count = events.len(), "Flushing buffered OpenWorld events");

        for (topic, event) in events {
            if let Err(e) = self.try_send(&topic, &event).await {
                warn!(error = %e, "Failed to flush buffered event");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openworld::types::GapCategory;

    // =========================================================================
    // Event serialization tests
    // =========================================================================

    #[test]
    fn test_novelty_detected_serialization() {
        let fingerprint = PatternFingerprint::new(12345, vec![0.1, 0.2, 0.3], "test".to_string());
        let event = OpenWorldEvent::NoveltyDetected {
            fingerprint,
            cluster: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OpenWorldEvent = serde_json::from_str(&json).unwrap();

        if let OpenWorldEvent::NoveltyDetected {
            fingerprint,
            cluster,
        } = deserialized
        {
            assert_eq!(fingerprint.hash, 12345);
            assert!(cluster.is_none());
        } else {
            panic!("Expected NoveltyDetected");
        }
    }

    #[test]
    fn test_gap_created_serialization() {
        let gap = CapabilityGap::new(GapCategory::MissingKnowledge, "test pattern".to_string());
        let event = OpenWorldEvent::GapCreated { gap: gap.clone() };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OpenWorldEvent = serde_json::from_str(&json).unwrap();

        if let OpenWorldEvent::GapCreated { gap: deser_gap } = deserialized {
            assert_eq!(deser_gap.category, GapCategory::MissingKnowledge);
        } else {
            panic!("Expected GapCreated");
        }
    }

    #[test]
    fn test_gap_status_changed_serialization() {
        let event = OpenWorldEvent::GapStatusChanged {
            gap_id: uuid::Uuid::now_v7(),
            old: GapStatus::Detected,
            new: GapStatus::Confirmed,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OpenWorldEvent = serde_json::from_str(&json).unwrap();

        if let OpenWorldEvent::GapStatusChanged { old, new, .. } = deserialized {
            assert_eq!(old, GapStatus::Detected);
            assert_eq!(new, GapStatus::Confirmed);
        } else {
            panic!("Expected GapStatusChanged");
        }
    }

    #[test]
    fn test_strategy_feedback_serialization() {
        let event = OpenWorldEvent::StrategyFeedback {
            learning_id: uuid::Uuid::now_v7(),
            adjustment: 0.15,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OpenWorldEvent = serde_json::from_str(&json).unwrap();

        if let OpenWorldEvent::StrategyFeedback { adjustment, .. } = deserialized {
            assert!((adjustment - 0.15).abs() < f64::EPSILON);
        } else {
            panic!("Expected StrategyFeedback");
        }
    }

    // =========================================================================
    // Config tests
    // =========================================================================

    #[test]
    fn test_config_default() {
        let config = OpenWorldProducerConfig::default();
        assert!(config.enabled);
    }

    // =========================================================================
    // Constants tests
    // =========================================================================

    #[test]
    fn test_stream_name() {
        assert_eq!(OPENWORLD_STREAM, "groove.openworld");
    }

    #[test]
    fn test_topic_names() {
        assert_eq!(topics::NOVELTY, "novelty");
        assert_eq!(topics::GAPS, "gaps");
        assert_eq!(topics::FEEDBACK, "feedback");
    }

    // =========================================================================
    // Stats tests
    // =========================================================================

    #[test]
    fn test_producer_stats_default() {
        let stats = ProducerStats::default();
        assert_eq!(stats.events_produced, 0);
        assert_eq!(stats.novelty_events, 0);
        assert_eq!(stats.gap_events, 0);
        assert_eq!(stats.feedback_events, 0);
        assert_eq!(stats.events_dropped, 0);
    }
}
