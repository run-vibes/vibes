//! Plugin event consumer for assessment processing.
//!
//! This consumer polls events from the EventLog and routes them to plugins
//! via `dispatch_raw_event`. Results are broadcast via AppState.
//!
//! This is the bridge between the host's event stream and the plugin callback
//! interface, working around TypeId mismatch issues with dynamic libraries.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace};
use vibes_core::plugins::PluginHost;

use super::{ConsumerConfig, ConsumerManager, Result};
use crate::AppState;

/// Start the plugin event consumer.
///
/// This consumer polls events from the EventLog and routes them to all loaded
/// plugins via `plugin_host.dispatch_raw_event()`. The returned assessment
/// results are broadcast via AppState for WebSocket clients.
///
/// # Arguments
///
/// * `manager` - The consumer manager to register with
/// * `plugin_host` - The plugin host containing loaded plugins
/// * `state` - The AppState for broadcasting results
/// * `shutdown` - Cancellation token for graceful shutdown
pub async fn start_plugin_event_consumer(
    manager: &mut ConsumerManager,
    plugin_host: Arc<RwLock<PluginHost>>,
    state: Arc<AppState>,
    shutdown: CancellationToken,
) -> Result<()> {
    let config =
        ConsumerConfig::replay("plugin-events").with_poll_timeout(Duration::from_millis(100));

    let handler = create_plugin_event_handler(plugin_host, state, shutdown);

    manager.spawn_consumer(config, handler).await
}

/// Create the event handler that dispatches to plugins.
fn create_plugin_event_handler(
    plugin_host: Arc<RwLock<PluginHost>>,
    state: Arc<AppState>,
    _shutdown: CancellationToken,
) -> super::EventHandler {
    Arc::new(move |stored| {
        let plugin_host = Arc::clone(&plugin_host);
        let state = Arc::clone(&state);

        Box::pin(async move {
            // Dispatch event to all plugins via the generic plugin interface
            let results = {
                let mut host = plugin_host.write().await;
                host.dispatch_raw_event(&stored)
            };

            if results.is_empty() {
                trace!(event_id = %stored.event_id, "No assessment results from plugins");
                return;
            }

            debug!(
                event_id = %stored.event_id,
                result_count = results.len(),
                "Plugin returned assessment results"
            );

            // Broadcast results to WebSocket clients
            for result in results {
                state.broadcast_assessment_result(result);
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use vibes_plugin_api::PluginAssessmentResult;

    #[test]
    fn test_plugin_assessment_result_creation() {
        let result = PluginAssessmentResult::lightweight("test-session", r#"{"test": true}"#);
        assert_eq!(result.result_type, "lightweight");
        assert_eq!(result.session_id, "test-session");
        assert_eq!(result.payload, r#"{"test": true}"#);
    }

    #[test]
    fn test_plugin_assessment_result_checkpoint() {
        let result = PluginAssessmentResult::checkpoint("session-1", r#"{"summary": "ok"}"#);
        assert_eq!(result.result_type, "checkpoint");
        assert_eq!(result.session_id, "session-1");
    }
}
