//! Shared application state for the vibes server

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, broadcast};
use tokio_util::sync::CancellationToken;
use vibes_core::{
    AccessConfig, PluginHost, PluginHostConfig, StoredEvent, SubscriptionStore, TunnelConfig,
    TunnelManager, VapidKeyManager, VibesEvent,
    pty::{PtyConfig, PtyManager},
};
use vibes_evals::{CreateStudy, PeriodType, Study, StudyConfig, StudyId, StudyManager};
use vibes_iggy::{
    EventLog, IggyConfig, IggyEventLog, IggyManager, InMemoryEventLog, Offset, run_preflight_checks,
};
use vibes_models::ModelRegistry;
use vibes_plugin_api::PluginAssessmentResult;

use vibes_observe::TraceEvent;

use crate::ws::{CheckpointInfo, StudyInfo};

/// PTY output event for broadcasting to attached clients
#[derive(Clone, Debug)]
pub enum PtyEvent {
    /// Raw output from a PTY session (base64 encoded for binary safety)
    Output { session_id: String, data: String },
    /// PTY process has exited
    Exit {
        session_id: String,
        exit_code: Option<i32>,
    },
}

use crate::agent_registry::ServerAgentRegistry;
use crate::middleware::AuthLayer;

/// Default capacity for the event broadcast channel
const DEFAULT_BROADCAST_CAPACITY: usize = 1000;

/// Shared application state accessible by all handlers
///
/// # Field Ordering (IMPORTANT)
///
/// `plugin_host` MUST be the last field because it contains dynamically loaded plugins.
/// Other fields (like `assessment_log`) may hold types from those plugins. Rust drops
/// fields in declaration order, so plugins must be unloaded AFTER all plugin types
/// are dropped to avoid invalid vtable dereferences.
#[derive(Clone)]
pub struct AppState {
    /// Event log for persistent event storage
    pub event_log: Arc<dyn EventLog<StoredEvent>>,
    /// Tunnel manager for remote access
    pub tunnel_manager: Arc<RwLock<TunnelManager>>,
    /// Authentication layer
    pub auth_layer: AuthLayer,
    /// When the server started
    pub started_at: DateTime<Utc>,
    /// Broadcast channel for WebSocket event distribution (offset, stored_event)
    event_broadcaster: broadcast::Sender<(Offset, StoredEvent)>,
    /// VAPID key manager for push notifications (optional)
    pub vapid: Option<Arc<VapidKeyManager>>,
    /// Push subscription store (optional)
    pub subscriptions: Option<Arc<SubscriptionStore>>,
    /// PTY session manager for terminal sessions
    pub pty_manager: Arc<RwLock<PtyManager>>,
    /// Broadcast channel for PTY output distribution
    pty_broadcaster: broadcast::Sender<PtyEvent>,
    /// Iggy manager for subprocess lifecycle (optional, only when using Iggy storage)
    iggy_manager: Option<Arc<IggyManager>>,
    /// Broadcast channel for assessment results from plugins
    assessment_broadcaster: broadcast::Sender<PluginAssessmentResult>,
    /// Broadcast channel for trace events from tracing subscriber
    trace_broadcaster: broadcast::Sender<TraceEvent>,
    /// Shutdown token for EventLog consumers
    consumer_shutdown: CancellationToken,
    /// Model registry for AI model discovery
    pub model_registry: Arc<RwLock<ModelRegistry>>,
    /// Agent registry for managing AI agents
    pub agent_registry: Arc<RwLock<ServerAgentRegistry>>,
    /// Study manager for evaluation studies
    study_manager: Option<Arc<StudyManager>>,
    /// Plugin host for managing plugins
    ///
    /// MUST be last - plugins are unloaded when this drops, so all plugin types
    /// (like assessment_log) must be dropped first.
    pub plugin_host: Arc<RwLock<PluginHost>>,
}

impl AppState {
    /// Create a new AppState with default components
    pub fn new() -> Self {
        let event_log: Arc<dyn EventLog<StoredEvent>> =
            Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (assessment_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (trace_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager: None,
            assessment_broadcaster,
            trace_broadcaster,
            consumer_shutdown: CancellationToken::new(),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            agent_registry: Arc::new(RwLock::new(ServerAgentRegistry::new())),
            study_manager: None,
            plugin_host,
        }
    }

    /// Create a new AppState for testing that doesn't load external plugins.
    ///
    /// This avoids issues with dynamically loaded plugins whose background tasks
    /// can outlive the test runtime and cause memory corruption.
    pub fn new_for_testing() -> Self {
        use std::path::PathBuf;

        // Use a non-existent directory so no external plugins are loaded
        let plugin_config = PluginHostConfig {
            user_plugin_dir: PathBuf::from("/nonexistent/vibes/plugins"),
            project_plugin_dir: None,
            handler_timeout: std::time::Duration::from_secs(5),
        };

        let event_log: Arc<dyn EventLog<StoredEvent>> =
            Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let plugin_host = Arc::new(RwLock::new(PluginHost::new(plugin_config)));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (assessment_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (trace_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            pty_broadcaster,
            iggy_manager: None,
            assessment_broadcaster,
            trace_broadcaster,
            consumer_shutdown: CancellationToken::new(),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            agent_registry: Arc::new(RwLock::new(ServerAgentRegistry::new())),
            study_manager: None,
            plugin_host,
        }
    }

    /// Create a new AppState with a custom EventLog (for testing)
    #[cfg(test)]
    pub fn with_event_log(event_log: Arc<dyn EventLog<StoredEvent>>) -> Self {
        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (assessment_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (trace_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager: None,
            assessment_broadcaster,
            trace_broadcaster,
            consumer_shutdown: CancellationToken::new(),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            agent_registry: Arc::new(RwLock::new(ServerAgentRegistry::new())),
            study_manager: None,
            plugin_host,
        }
    }

    /// Create a new AppState with Iggy-backed persistent storage.
    ///
    /// Attempts to start and connect to the bundled Iggy server.
    ///
    /// # Errors
    ///
    /// Returns an error if Iggy cannot be started (missing binary, insufficient
    /// system resources like ulimit, connection failure, etc.)
    pub async fn new_with_iggy() -> Result<Self, vibes_iggy::Error> {
        let (log, manager) = Self::try_start_iggy().await?;
        tracing::info!("Using Iggy for persistent event storage");
        let event_log: Arc<dyn EventLog<StoredEvent>> = Arc::new(log);
        let iggy_manager = Some(manager);

        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (assessment_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (trace_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Ok(Self {
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager,
            assessment_broadcaster,
            trace_broadcaster,
            consumer_shutdown: CancellationToken::new(),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            agent_registry: Arc::new(RwLock::new(ServerAgentRegistry::new())),
            study_manager: None,
            plugin_host,
        })
    }

    /// Create AppState with Iggy using custom configuration.
    ///
    /// This is useful for tests that need isolated ports and data directories.
    #[doc(hidden)]
    pub async fn new_with_iggy_config(iggy_config: IggyConfig) -> Result<Self, vibes_iggy::Error> {
        let (log, manager) = Self::try_start_iggy_with_config(iggy_config).await?;
        tracing::info!("Using Iggy for persistent event storage");
        let event_log: Arc<dyn EventLog<StoredEvent>> = Arc::new(log);
        let iggy_manager = Some(manager);

        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (assessment_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (trace_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Ok(Self {
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager,
            assessment_broadcaster,
            trace_broadcaster,
            consumer_shutdown: CancellationToken::new(),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            agent_registry: Arc::new(RwLock::new(ServerAgentRegistry::new())),
            study_manager: None,
            plugin_host,
        })
    }

    /// Try to start the Iggy server and create an event log.
    ///
    /// Returns both the event log and a reference to the manager for shutdown.
    async fn try_start_iggy()
    -> Result<(IggyEventLog<StoredEvent>, Arc<IggyManager>), vibes_iggy::Error> {
        Self::try_start_iggy_with_config(IggyConfig::default()).await
    }

    /// Try to start the Iggy server with custom configuration.
    async fn try_start_iggy_with_config(
        config: IggyConfig,
    ) -> Result<(IggyEventLog<StoredEvent>, Arc<IggyManager>), vibes_iggy::Error> {
        // Check if binary is available before trying to start
        if config.find_binary().is_none() {
            return Err(vibes_iggy::Error::BinaryNotFound);
        }

        // Check system requirements (ulimit for io_uring)
        run_preflight_checks()?;

        let manager = Arc::new(IggyManager::new(config));
        manager.start().await?;

        // Spawn the supervisor loop to monitor the process and handle restarts
        // The supervisor calls try_wait() which reaps zombie processes
        let supervisor_manager = Arc::clone(&manager);
        tokio::spawn(async move {
            if let Err(e) = supervisor_manager.supervise().await {
                tracing::error!("Iggy supervisor exited with error: {}", e);
            }
        });

        // Wait for Iggy to be fully ready (HTTP + TCP)
        // This replaces the fixed grace period to fix the race condition where
        // CLI hooks (using HTTP) could fire before HTTP was accepting connections.
        manager.wait_for_ready().await?;

        // Create event log from the manager (cloning the Arc)
        let event_log = IggyEventLog::new(Arc::clone(&manager));
        event_log.connect().await?;

        Ok((event_log, manager))
    }

    /// Configure authentication for this state
    pub fn with_auth(mut self, config: AccessConfig) -> Self {
        self.auth_layer = AuthLayer::new(config);
        self
    }

    /// Configure push notifications for this state
    pub fn with_push(
        mut self,
        vapid: Arc<VapidKeyManager>,
        subscriptions: Arc<SubscriptionStore>,
    ) -> Self {
        self.vapid = Some(vapid);
        self.subscriptions = Some(subscriptions);
        self
    }

    /// Configure PTY settings for this state
    pub fn with_pty_config(mut self, config: PtyConfig) -> Self {
        self.pty_manager = Arc::new(RwLock::new(PtyManager::new(config)));
        self
    }

    /// Create AppState with a custom plugin host (for testing)
    #[cfg(test)]
    pub fn with_plugin_host(plugin_host: Arc<RwLock<PluginHost>>) -> Self {
        let event_log: Arc<dyn EventLog<StoredEvent>> =
            Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (assessment_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (trace_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager: None,
            assessment_broadcaster,
            trace_broadcaster,
            consumer_shutdown: CancellationToken::new(),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            agent_registry: Arc::new(RwLock::new(ServerAgentRegistry::new())),
            study_manager: None,
            plugin_host,
        }
    }

    /// Create AppState with custom components (for testing)
    pub fn with_components(
        plugin_host: Arc<RwLock<PluginHost>>,
        tunnel_manager: Arc<RwLock<TunnelManager>>,
    ) -> Self {
        let event_log: Arc<dyn EventLog<StoredEvent>> =
            Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (assessment_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (trace_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager: None,
            assessment_broadcaster,
            trace_broadcaster,
            consumer_shutdown: CancellationToken::new(),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            agent_registry: Arc::new(RwLock::new(ServerAgentRegistry::new())),
            study_manager: None,
            plugin_host,
        }
    }

    /// Set the Iggy manager for shutdown coordination
    #[must_use]
    pub fn with_iggy_manager(mut self, manager: Arc<IggyManager>) -> Self {
        self.iggy_manager = Some(manager);
        self
    }

    /// Get the Iggy manager if available.
    ///
    /// Returns None if Iggy is not being used for persistence.
    pub fn iggy_manager(&self) -> Option<Arc<IggyManager>> {
        self.iggy_manager.clone()
    }

    /// Subscribe to assessment results broadcast from plugins.
    ///
    /// Returns a receiver that will receive `PluginAssessmentResult` events
    /// as they are produced by the plugin event consumer.
    pub fn subscribe_assessment_results(&self) -> broadcast::Receiver<PluginAssessmentResult> {
        self.assessment_broadcaster.subscribe()
    }

    /// Broadcast an assessment result from a plugin.
    ///
    /// Called by the plugin event consumer when a plugin returns assessment results.
    /// Returns the number of receivers that received the result.
    pub fn broadcast_assessment_result(&self, result: PluginAssessmentResult) -> usize {
        self.assessment_broadcaster.send(result).unwrap_or(0)
    }

    /// Gracefully shutdown the server, stopping all managed subprocesses and consumers.
    ///
    /// This should be called before the server exits to ensure clean termination
    /// of the Iggy server subprocess and EventLog consumers.
    pub async fn shutdown(&self) {
        // Signal EventLog consumers to stop
        tracing::info!("Cancelling consumer shutdown token");
        self.consumer_shutdown.cancel();

        if let Some(manager) = &self.iggy_manager {
            tracing::info!("Stopping Iggy server subprocess");
            if let Err(e) = manager.stop().await {
                tracing::error!("Error stopping Iggy server: {}", e);
            }
        }
    }

    /// Get the consumer shutdown token for EventLog consumers.
    ///
    /// This token is cancelled when `shutdown()` is called, allowing consumers
    /// to gracefully stop processing.
    pub fn consumer_shutdown_token(&self) -> CancellationToken {
        self.consumer_shutdown.clone()
    }

    /// Get a reference to the tunnel manager
    pub fn tunnel_manager(&self) -> &Arc<RwLock<TunnelManager>> {
        &self.tunnel_manager
    }

    /// Get a reference to the plugin host
    pub fn plugin_host(&self) -> &Arc<RwLock<PluginHost>> {
        &self.plugin_host
    }

    /// Get the event broadcaster sender for consumer integration.
    ///
    /// This is used by EventLog consumers to broadcast events to WebSocket clients.
    pub fn event_broadcaster(&self) -> broadcast::Sender<(Offset, StoredEvent)> {
        self.event_broadcaster.clone()
    }

    /// Subscribe to events broadcast to WebSocket clients
    ///
    /// Returns a receiver that will receive (offset, stored_event) tuples published
    /// through the event broadcaster.
    pub fn subscribe_events(&self) -> broadcast::Receiver<(Offset, StoredEvent)> {
        self.event_broadcaster.subscribe()
    }

    /// Append an event to the EventLog.
    ///
    /// This is the primary way to publish events. The event will be:
    /// 1. Wrapped in a StoredEvent with a unique UUIDv7 event_id
    /// 2. Persisted to the EventLog (Iggy when available, in-memory otherwise)
    /// 3. Picked up by consumers (WebSocket, Notification, Assessment)
    /// 4. Broadcast to connected clients via the WebSocket consumer
    ///
    /// This method spawns a task to avoid blocking the caller.
    /// If persistence fails, the error is logged but not propagated.
    pub fn append_event(&self, event: VibesEvent) {
        let event_log = Arc::clone(&self.event_log);
        let stored = StoredEvent::new(event);
        tokio::spawn(async move {
            if let Err(e) = event_log.append(stored).await {
                tracing::warn!("Failed to append event to EventLog: {}", e);
            }
        });
    }

    /// Broadcast a stored event with its offset to all subscribed WebSocket clients.
    ///
    /// **Internal API:** Event producers should NOT call this directly.
    /// Use [`append_event`] instead, which writes to the EventLog.
    /// The WebSocket consumer will then call this method after reading
    /// from the log.
    ///
    /// Returns the number of receivers that received the event.
    /// Returns 0 if there are no active subscribers.
    pub fn broadcast_event(&self, offset: Offset, stored: StoredEvent) -> usize {
        self.event_broadcaster.send((offset, stored)).unwrap_or(0)
    }

    /// Subscribe to PTY events broadcast to WebSocket clients
    ///
    /// Returns a receiver that will receive all PtyEvents published
    /// through the PTY broadcaster.
    pub fn subscribe_pty_events(&self) -> broadcast::Receiver<PtyEvent> {
        self.pty_broadcaster.subscribe()
    }

    /// Publish a PTY event to all subscribed WebSocket clients
    ///
    /// Returns the number of receivers that received the event.
    /// Returns 0 if there are no active subscribers.
    pub fn broadcast_pty_event(&self, event: PtyEvent) -> usize {
        self.pty_broadcaster.send(event).unwrap_or(0)
    }

    /// Subscribe to trace events from the tracing subscriber
    ///
    /// Returns a receiver that will receive TraceEvents as spans complete.
    pub fn subscribe_traces(&self) -> broadcast::Receiver<TraceEvent> {
        self.trace_broadcaster.subscribe()
    }

    /// Get the trace broadcaster sender for TraceBroadcaster integration.
    ///
    /// This sender is passed to the TraceBroadcaster layer during tracing setup
    /// so it can broadcast span completions to WebSocket clients.
    pub fn trace_broadcaster(&self) -> broadcast::Sender<TraceEvent> {
        self.trace_broadcaster.clone()
    }

    /// Returns how long the server has been running
    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }

    // === Study Management Methods ===

    /// Create a new longitudinal study.
    pub async fn create_study(
        &self,
        name: &str,
        period_type: &str,
        period_value: Option<u32>,
        description: Option<String>,
    ) -> Result<StudyInfo, String> {
        let manager = self
            .study_manager
            .as_ref()
            .ok_or_else(|| "Eval studies not enabled".to_string())?;

        let parsed_period = PeriodType::parse(period_type)
            .ok_or_else(|| format!("Invalid period type: {}", period_type))?;

        let cmd = CreateStudy {
            name: name.to_string(),
            period_type: parsed_period,
            period_value,
            config: StudyConfig {
                description,
                ..Default::default()
            },
        };

        let study_id = manager.create_study(cmd).await.map_err(|e| e.to_string())?;

        // Also start the study immediately
        manager
            .start_study(study_id)
            .await
            .map_err(|e| e.to_string())?;

        // Fetch the created study to return info
        let study = manager
            .get_study(study_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Study not found after creation".to_string())?;

        let checkpoints = manager
            .get_checkpoints(study_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(study_to_info(&study, checkpoints.len() as u32))
    }

    /// Start a pending study.
    pub async fn start_study(&self, study_id: &str) -> Result<(), String> {
        let manager = self
            .study_manager
            .as_ref()
            .ok_or_else(|| "Eval studies not enabled".to_string())?;

        let id = parse_study_id(study_id)?;
        manager.start_study(id).await.map_err(|e| e.to_string())
    }

    /// Stop a running study.
    pub async fn stop_study(&self, study_id: &str) -> Result<(), String> {
        let manager = self
            .study_manager
            .as_ref()
            .ok_or_else(|| "Eval studies not enabled".to_string())?;

        let id = parse_study_id(study_id)?;
        manager.stop_study(id).await.map_err(|e| e.to_string())
    }

    /// List all studies.
    pub async fn list_studies(&self) -> Result<Vec<StudyInfo>, String> {
        let manager = self
            .study_manager
            .as_ref()
            .ok_or_else(|| "Eval studies not enabled".to_string())?;

        let studies = manager.list_studies().await.map_err(|e| e.to_string())?;

        let mut result = Vec::with_capacity(studies.len());
        for study in studies {
            let checkpoints = manager
                .get_checkpoints(study.id)
                .await
                .map_err(|e| e.to_string())?;
            result.push(study_to_info(&study, checkpoints.len() as u32));
        }

        Ok(result)
    }

    /// Get detailed study information.
    pub async fn get_study(
        &self,
        study_id: &str,
    ) -> Result<Option<(StudyInfo, Vec<CheckpointInfo>)>, String> {
        let manager = self
            .study_manager
            .as_ref()
            .ok_or_else(|| "Eval studies not enabled".to_string())?;

        let id = parse_study_id(study_id)?;
        let study = manager.get_study(id).await.map_err(|e| e.to_string())?;

        match study {
            Some(study) => {
                let checkpoints = manager
                    .get_checkpoints(id)
                    .await
                    .map_err(|e| e.to_string())?;

                let study_info = study_to_info(&study, checkpoints.len() as u32);
                let checkpoint_infos: Vec<_> = checkpoints.iter().map(checkpoint_to_info).collect();

                Ok(Some((study_info, checkpoint_infos)))
            }
            None => Ok(None),
        }
    }

    /// Record a checkpoint for a study.
    pub async fn record_checkpoint(&self, study_id: &str) -> Result<CheckpointInfo, String> {
        let manager = self
            .study_manager
            .as_ref()
            .ok_or_else(|| "Eval studies not enabled".to_string())?;

        let id = parse_study_id(study_id)?;

        // Create a checkpoint with default metrics
        // In a real implementation, this would collect actual metrics
        let cmd = vibes_evals::RecordCheckpoint {
            study_id: id,
            metrics: vibes_evals::LongitudinalMetrics::default(),
            events_analyzed: 0,
            sessions_included: vec![],
        };

        let checkpoint_id = manager
            .record_checkpoint(cmd)
            .await
            .map_err(|e| e.to_string())?;

        // Fetch the created checkpoint
        let checkpoints = manager
            .get_checkpoints(id)
            .await
            .map_err(|e| e.to_string())?;

        checkpoints
            .into_iter()
            .find(|c| c.id == checkpoint_id)
            .map(|c| checkpoint_to_info(&c))
            .ok_or_else(|| "Checkpoint not found after creation".to_string())
    }
}

/// Parse a study ID from string.
fn parse_study_id(s: &str) -> Result<StudyId, String> {
    uuid::Uuid::parse_str(s)
        .map(StudyId)
        .map_err(|_| format!("Invalid study ID: {}", s))
}

/// Convert a Study to StudyInfo for the WebSocket protocol.
fn study_to_info(study: &Study, checkpoint_count: u32) -> StudyInfo {
    StudyInfo {
        id: study.id.0.to_string(),
        name: study.name.clone(),
        status: study.status.as_str().to_string(),
        period_type: study.period_type.as_str().to_string(),
        period_value: study.period_value,
        description: study.config.description.clone(),
        created_at: study.created_at.timestamp(),
        started_at: study.started_at.map(|t| t.timestamp()),
        stopped_at: study.stopped_at.map(|t| t.timestamp()),
        checkpoint_count,
    }
}

/// Convert a Checkpoint to CheckpointInfo for the WebSocket protocol.
fn checkpoint_to_info(checkpoint: &vibes_evals::Checkpoint) -> CheckpointInfo {
    CheckpointInfo {
        id: checkpoint.id.0.to_string(),
        study_id: checkpoint.study_id.0.to_string(),
        timestamp: checkpoint.timestamp.timestamp(),
        sessions_completed: checkpoint.metrics.sessions_completed as u32,
        success_rate: Some(checkpoint.metrics.session_success_rate),
        first_attempt_rate: Some(checkpoint.metrics.first_attempt_success_rate),
        avg_iterations: Some(checkpoint.metrics.avg_iterations_to_success),
        cost_per_task: Some(checkpoint.metrics.cost_per_successful_task),
        events_analyzed: checkpoint.events_analyzed,
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::ClaudeEvent;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(state.uptime_seconds() >= 0);
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.uptime_seconds() >= 0);
    }

    #[test]
    fn test_app_state_with_components() {
        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));

        let state = AppState::with_components(plugin_host, tunnel_manager);
        assert!(state.uptime_seconds() >= 0);
    }

    #[tokio::test]
    async fn test_app_state_has_tunnel_manager() {
        let state = AppState::new();
        let tunnel = state.tunnel_manager.read().await;
        assert!(!tunnel.is_enabled());
    }

    // ==================== Model Registry Tests ====================

    #[tokio::test]
    async fn test_app_state_has_model_registry() {
        let state = AppState::new();
        let registry = state.model_registry.read().await;
        assert_eq!(registry.model_count(), 0); // Empty by default
    }

    // ==================== Event Broadcasting Tests ====================

    #[tokio::test]
    async fn test_subscribe_events_returns_receiver() {
        let state = AppState::new();
        let _receiver = state.subscribe_events();
        // Receiver created successfully
    }

    #[tokio::test]
    async fn test_broadcast_event_with_no_subscribers_returns_zero() {
        let state = AppState::new();
        // No subscribers, should return 0
        let stored = StoredEvent::new(VibesEvent::SessionCreated {
            session_id: "sess-1".to_string(),
            name: None,
        });
        let count = state.broadcast_event(0, stored);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_event_to_subscriber() {
        let state = AppState::new();
        let mut receiver = state.subscribe_events();

        let stored = StoredEvent::new(VibesEvent::Claude {
            session_id: "sess-1".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        });
        let expected_event_id = stored.event_id;

        let count = state.broadcast_event(42, stored);
        assert_eq!(count, 1);

        let (offset, received) = receiver.recv().await.unwrap();
        assert_eq!(offset, 42);
        assert_eq!(received.event_id, expected_event_id);
        match &received.event {
            VibesEvent::Claude { session_id, event } => {
                assert_eq!(session_id, "sess-1");
                match event {
                    ClaudeEvent::TextDelta { text } => assert_eq!(text, "Hello"),
                    _ => panic!("Expected TextDelta"),
                }
            }
            _ => panic!("Expected Claude event"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_to_multiple_subscribers() {
        let state = AppState::new();
        let mut receiver1 = state.subscribe_events();
        let mut receiver2 = state.subscribe_events();

        let stored = StoredEvent::new(VibesEvent::SessionStateChanged {
            session_id: "sess-1".to_string(),
            state: "processing".to_string(),
        });

        let count = state.broadcast_event(99, stored);
        assert_eq!(count, 2);

        // Both receivers should get the event with offset
        let (offset1, received1) = receiver1.recv().await.unwrap();
        let (offset2, received2) = receiver2.recv().await.unwrap();

        assert_eq!(offset1, 99);
        assert_eq!(offset2, 99);

        match &received1.event {
            VibesEvent::SessionStateChanged { session_id, state } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(state, "processing");
            }
            _ => panic!("Expected SessionStateChanged"),
        }

        match &received2.event {
            VibesEvent::SessionStateChanged { session_id, state } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(state, "processing");
            }
            _ => panic!("Expected SessionStateChanged"),
        }
    }

    // ==================== Shutdown Tests ====================

    #[tokio::test]
    async fn test_shutdown_stops_iggy_manager() {
        // Create an IggyManager without starting a process
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));

        // Create AppState with the manager
        let state = AppState::new().with_iggy_manager(manager.clone());

        // Call shutdown
        state.shutdown().await;

        // Verify the manager's shutdown signal was set
        assert_eq!(manager.state().await, vibes_iggy::IggyState::Stopped);
    }

    #[tokio::test]
    async fn test_shutdown_without_iggy_is_safe() {
        // AppState without Iggy should not panic on shutdown
        let state = AppState::new();
        state.shutdown().await;
        // No panic = success
    }

    #[tokio::test]
    async fn test_shutdown_cancels_consumer_token() {
        let state = AppState::new();
        let shutdown_token = state.consumer_shutdown_token();

        // Token should not be cancelled initially
        assert!(!shutdown_token.is_cancelled());

        // Call shutdown
        state.shutdown().await;

        // Token should now be cancelled
        assert!(shutdown_token.is_cancelled());
    }

    #[tokio::test]
    async fn test_consumer_shutdown_token_is_shared() {
        let state = AppState::new();
        let token1 = state.consumer_shutdown_token();
        let token2 = state.consumer_shutdown_token();

        // Cancel via one token
        token1.cancel();

        // Both should see the cancellation
        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
    }

    // ==================== Event Appending Tests ====================

    #[tokio::test]
    async fn test_append_event_writes_to_log() {
        use vibes_iggy::SeekPosition;

        let state = AppState::new();

        let event = VibesEvent::SessionCreated {
            session_id: "test-session".to_string(),
            name: Some("Test".to_string()),
        };

        state.append_event(event);

        // Give the spawned task time to complete
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Create a consumer and verify the event is in the log
        let mut consumer = state.event_log.consumer("test-reader").await.unwrap();
        consumer.seek(SeekPosition::Beginning).await.unwrap();

        let batch = consumer
            .poll(10, std::time::Duration::from_millis(100))
            .await
            .unwrap();
        assert_eq!(batch.events.len(), 1);

        let stored = &batch.events[0].1;

        // Verify the event has a valid UUIDv7 event_id
        assert_eq!(stored.event_id.get_version(), Some(uuid::Version::SortRand));

        // Verify the inner event data
        match &stored.event {
            VibesEvent::SessionCreated { session_id, .. } => {
                assert_eq!(session_id, "test-session");
            }
            _ => panic!("Expected SessionCreated event"),
        }
    }
}
