//! File watcher for capability changes with debouncing

use crate::{Harness, HarnessCapabilities, Result};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Watches config files and re-introspects on changes with debouncing
pub struct CapabilityWatcher {
    harness: Arc<dyn Harness>,
    capabilities: Arc<RwLock<HarnessCapabilities>>,
    project_root: Option<PathBuf>,
    _watcher: notify::RecommendedWatcher,
    _task: tokio::task::JoinHandle<()>,
}

impl CapabilityWatcher {
    /// Create a new watcher that performs initial introspection and watches for changes
    ///
    /// # Arguments
    /// * `harness` - The harness to introspect
    /// * `project_root` - Optional project root for project-scoped config
    /// * `debounce_ms` - Milliseconds to wait for quiet period before re-introspecting
    pub async fn new(
        harness: Arc<dyn Harness>,
        project_root: Option<PathBuf>,
        debounce_ms: u64,
    ) -> Result<Self> {
        // Initial introspection
        let initial_caps = harness.introspect(project_root.as_deref()).await?;
        let capabilities = Arc::new(RwLock::new(initial_caps));

        // Set up file watching
        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>(100);

        let watcher = recommended_watcher(move |event| {
            // Use blocking_send since this callback runs in the notify thread
            let _ = tx.blocking_send(event);
        })?;

        // Get config paths to watch
        let config_paths = harness.config_paths(project_root.as_deref())?;

        let mut watcher = watcher;

        // Watch all existing config paths
        if let Some(system) = &config_paths.system
            && system.exists()
        {
            tracing::debug!("Watching system config: {:?}", system);
            watcher.watch(system, RecursiveMode::Recursive)?;
        }

        if config_paths.user.exists() {
            tracing::debug!("Watching user config: {:?}", config_paths.user);
            watcher.watch(&config_paths.user, RecursiveMode::Recursive)?;
        }

        if let Some(project) = &config_paths.project
            && project.exists()
        {
            tracing::debug!("Watching project config: {:?}", project);
            watcher.watch(project, RecursiveMode::Recursive)?;
        }

        // Spawn the debounce loop
        let harness_clone = harness.clone();
        let capabilities_clone = capabilities.clone();
        let project_root_clone = project_root.clone();

        let task = tokio::spawn(async move {
            Self::debounce_loop(rx, harness_clone, capabilities_clone, project_root_clone, debounce_ms).await;
        });

        Ok(Self {
            harness,
            capabilities,
            project_root,
            _watcher: watcher,
            _task: task,
        })
    }

    /// Get the current capabilities
    pub async fn capabilities(&self) -> HarnessCapabilities {
        self.capabilities.read().await.clone()
    }

    /// Force a re-introspection
    pub async fn refresh(&self) -> Result<()> {
        tracing::info!("Forcing capability refresh");
        let new_caps = self.harness.introspect(self.project_root.as_deref()).await?;
        *self.capabilities.write().await = new_caps;
        Ok(())
    }

    /// Debounce loop that waits for events, then waits for quiet period before re-introspecting
    async fn debounce_loop(
        mut rx: mpsc::Receiver<notify::Result<notify::Event>>,
        harness: Arc<dyn Harness>,
        capabilities: Arc<RwLock<HarnessCapabilities>>,
        project_root: Option<PathBuf>,
        debounce_ms: u64,
    ) {
        let debounce_duration = std::time::Duration::from_millis(debounce_ms);

        loop {
            // Wait for first event
            let event = rx.recv().await;
            if event.is_none() {
                // Channel closed, exit
                break;
            }

            tracing::debug!("Config change detected, starting debounce");

            // Drain any pending events and wait for quiet period
            loop {
                match tokio::time::timeout(debounce_duration, rx.recv()).await {
                    Ok(Some(_)) => {
                        // More events, keep waiting
                        tracing::debug!("More events during debounce, resetting timer");
                    }
                    Ok(None) => {
                        // Channel closed
                        return;
                    }
                    Err(_) => {
                        // Timeout - quiet period elapsed, time to re-introspect
                        break;
                    }
                }
            }

            tracing::info!("Debounce complete, re-introspecting capabilities");

            match harness.introspect(project_root.as_deref()).await {
                Ok(new_caps) => {
                    *capabilities.write().await = new_caps;
                    tracing::info!("Capabilities refreshed successfully");
                }
                Err(e) => {
                    tracing::error!("Failed to refresh capabilities: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConfigPaths, HarnessCapabilities, ScopedCapabilities};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock harness for testing
    struct MockHarness {
        introspect_count: AtomicUsize,
    }

    impl MockHarness {
        fn new() -> Self {
            Self {
                introspect_count: AtomicUsize::new(0),
            }
        }

        fn introspect_count(&self) -> usize {
            self.introspect_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl Harness for MockHarness {
        fn harness_type(&self) -> &'static str {
            "mock"
        }

        async fn version(&self) -> Option<String> {
            Some("1.0.0".to_string())
        }

        fn config_paths(&self, project_root: Option<&std::path::Path>) -> Result<ConfigPaths> {
            Ok(ConfigPaths {
                system: None,
                user: PathBuf::from("/tmp/mock-harness-user"),
                project: project_root.map(|p| p.join(".mock")),
            })
        }

        async fn introspect(&self, _project_root: Option<&std::path::Path>) -> Result<HarnessCapabilities> {
            self.introspect_count.fetch_add(1, Ordering::SeqCst);
            Ok(HarnessCapabilities {
                harness_type: "mock".to_string(),
                version: Some("1.0.0".to_string()),
                system: None,
                user: ScopedCapabilities::default(),
                project: None,
            })
        }
    }

    #[tokio::test]
    async fn test_watcher_performs_initial_introspection() {
        let harness = Arc::new(MockHarness::new());
        let watcher = CapabilityWatcher::new(harness.clone(), None, 100)
            .await
            .unwrap();

        // Should have introspected once during creation
        assert_eq!(harness.introspect_count(), 1);

        // Capabilities should be available
        let caps = watcher.capabilities().await;
        assert_eq!(caps.harness_type, "mock");
        assert_eq!(caps.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_watcher_refresh_re_introspects() {
        let harness = Arc::new(MockHarness::new());
        let watcher = CapabilityWatcher::new(harness.clone(), None, 100)
            .await
            .unwrap();

        // Initial introspection
        assert_eq!(harness.introspect_count(), 1);

        // Force refresh
        watcher.refresh().await.unwrap();

        // Should have introspected again
        assert_eq!(harness.introspect_count(), 2);
    }

    #[tokio::test]
    async fn test_watcher_with_project_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Create the project config directory so it can be watched
        std::fs::create_dir_all(project_root.join(".mock")).unwrap();

        let harness = Arc::new(MockHarness::new());
        let watcher = CapabilityWatcher::new(harness.clone(), Some(project_root), 100)
            .await
            .unwrap();

        assert_eq!(harness.introspect_count(), 1);

        let caps = watcher.capabilities().await;
        assert_eq!(caps.harness_type, "mock");
    }

    /// Mock harness that uses a specific config directory for file watching tests
    struct MockHarnessWithConfigDir {
        config_dir: PathBuf,
        introspect_count: Arc<RwLock<usize>>,
    }

    #[async_trait]
    impl Harness for MockHarnessWithConfigDir {
        fn harness_type(&self) -> &'static str {
            "mock"
        }

        async fn version(&self) -> Option<String> {
            Some("1.0.0".to_string())
        }

        fn config_paths(&self, _project_root: Option<&std::path::Path>) -> Result<ConfigPaths> {
            Ok(ConfigPaths {
                system: None,
                user: self.config_dir.clone(),
                project: None,
            })
        }

        async fn introspect(&self, _project_root: Option<&std::path::Path>) -> Result<HarnessCapabilities> {
            *self.introspect_count.write().await += 1;
            Ok(HarnessCapabilities {
                harness_type: "mock".to_string(),
                version: Some("1.0.0".to_string()),
                system: None,
                user: ScopedCapabilities::default(),
                project: None,
            })
        }
    }

    #[tokio::test]
    async fn test_watcher_re_introspects_on_file_change() {
        use std::time::Duration;
        use tokio::fs;

        let tmp = tempfile::TempDir::new().unwrap();
        let config_dir = tmp.path().join(".mock");
        fs::create_dir(&config_dir).await.unwrap();

        let introspect_count = Arc::new(RwLock::new(0));
        let harness = Arc::new(MockHarnessWithConfigDir {
            config_dir: config_dir.clone(),
            introspect_count: introspect_count.clone(),
        });

        let _watcher = CapabilityWatcher::new(harness, None, 50).await.unwrap();
        assert_eq!(*introspect_count.read().await, 1);

        // Modify a file in the watched directory
        fs::write(config_dir.join("test.txt"), "trigger").await.unwrap();

        // Wait for debounce + processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Should have triggered re-introspection
        assert!(
            *introspect_count.read().await >= 2,
            "Expected at least 2 introspections after file change"
        );
    }
}
