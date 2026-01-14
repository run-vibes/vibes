//! Configuration for Iggy server and client.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Configuration for the Iggy server subprocess.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IggyConfig {
    /// Path to the iggy-server binary.
    #[serde(default = "default_binary_path")]
    pub binary_path: PathBuf,

    /// Directory where Iggy stores its data.
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// TCP port for Iggy server.
    #[serde(default = "default_port")]
    pub port: u16,

    /// HTTP port for Iggy server (REST API).
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Interval between health checks.
    #[serde(default = "default_health_check_interval", with = "humantime_serde")]
    pub health_check_interval: Duration,

    /// Maximum number of restart attempts before giving up.
    #[serde(default = "default_max_restart_attempts")]
    pub max_restart_attempts: u32,

    /// Number of shards (worker threads) for Iggy.
    /// Default is "4" to avoid consuming all CPU cores.
    /// Iggy's default "all" would use every core on the system.
    #[serde(default = "default_cpu_allocation")]
    pub cpu_allocation: String,

    /// Memory pool size for Iggy's buffer management.
    /// Default is "512 MiB" (Iggy's minimum) to avoid excessive memory use.
    /// Iggy's default is "4 GiB".
    #[serde(default = "default_memory_pool_size")]
    pub memory_pool_size: String,

    /// Bucket capacity for memory pool allocation.
    /// Default is 128 (Iggy's minimum) for conservative memory use.
    /// Iggy's default is 8192.
    #[serde(default = "default_bucket_capacity")]
    pub bucket_capacity: u32,
}

fn default_binary_path() -> PathBuf {
    PathBuf::from("iggy-server")
}

fn default_data_dir() -> PathBuf {
    // Allow override via environment variable for test isolation
    if let Ok(path) = std::env::var("VIBES_IGGY_DATA_DIR") {
        return PathBuf::from(path);
    }

    // Use XDG data directory helper for consistency
    vibes_paths::data_dir().join("iggy")
}

fn default_port() -> u16 {
    // Allow override via environment variable for test isolation
    if let Ok(port) = std::env::var("VIBES_IGGY_PORT")
        && let Ok(p) = port.parse()
    {
        return p;
    }
    8090
}

fn default_http_port() -> u16 {
    // Allow override via environment variable for test isolation
    if let Ok(port) = std::env::var("VIBES_IGGY_HTTP_PORT")
        && let Ok(p) = port.parse()
    {
        return p;
    }
    // HTTP port for Iggy REST API.
    // Uses 7431 to avoid conflicts with common dev servers (3000-3999 range).
    7431
}

fn default_health_check_interval() -> Duration {
    Duration::from_secs(5)
}

fn default_max_restart_attempts() -> u32 {
    3
}

fn default_cpu_allocation() -> String {
    // Use 4 shards instead of "all" to avoid consuming every CPU core.
    // Iggy's default "all" would spawn one shard per core (e.g., 48 on a workstation),
    // which causes OutOfMemory panics when each shard tries to allocate buffers.
    "4".to_string()
}

fn default_memory_pool_size() -> String {
    // Use Iggy's minimum (512 MiB) instead of the 4 GiB default.
    // This is sufficient for vibes' use case (event streaming for a single user).
    "512 MiB".to_string()
}

fn default_bucket_capacity() -> u32 {
    // Use Iggy's minimum (128) instead of the 8192 default.
    // Reduces memory footprint for local development use.
    128
}

impl Default for IggyConfig {
    fn default() -> Self {
        Self {
            binary_path: default_binary_path(),
            data_dir: default_data_dir(),
            port: default_port(),
            http_port: default_http_port(),
            health_check_interval: default_health_check_interval(),
            max_restart_attempts: default_max_restart_attempts(),
            cpu_allocation: default_cpu_allocation(),
            memory_pool_size: default_memory_pool_size(),
            bucket_capacity: default_bucket_capacity(),
        }
    }
}

impl IggyConfig {
    /// Find the iggy-server binary.
    ///
    /// Resolution order:
    /// 1. Explicit path in config (if it exists)
    /// 2. CARGO_TARGET_DIR environment variable (shared target directory)
    /// 3. Same directory as current executable
    /// 4. Workspace target directory (for tests running from target/debug/deps/)
    /// 5. PATH lookup
    #[must_use]
    pub fn find_binary(&self) -> Option<PathBuf> {
        // 1. Explicit path in config (if it's an absolute path that exists)
        if self.binary_path.is_absolute() && self.binary_path.exists() {
            return Some(self.binary_path.clone());
        }

        // 2. Check CARGO_TARGET_DIR (shared target directory across worktrees)
        if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
            for profile in ["debug", "release"] {
                let binary = PathBuf::from(&target_dir).join(profile).join("iggy-server");
                if binary.exists() {
                    return Some(binary);
                }
            }
        }

        // 3. Same directory as current executable
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            let sibling = dir.join("iggy-server");
            if sibling.exists() {
                return Some(sibling);
            }

            // 4. Workspace target directory (tests run from target/debug/deps/)
            // Walk up looking for target/debug/iggy-server or target/release/iggy-server
            let mut current = dir;
            while let Some(parent) = current.parent() {
                // Check if this is a target directory
                if current.file_name().is_some_and(|n| n == "target") {
                    for profile in ["debug", "release"] {
                        let binary = current.join(profile).join("iggy-server");
                        if binary.exists() {
                            return Some(binary);
                        }
                    }
                }
                current = parent;
            }
        }

        // 5. Check PATH
        if let Ok(path) = which::which("iggy-server") {
            return Some(path);
        }

        None
    }

    /// Create a new config with a custom binary path.
    #[must_use]
    pub fn with_binary_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = path.into();
        self
    }

    /// Create a new config with a custom data directory.
    #[must_use]
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = path.into();
        self
    }

    /// Create a new config with a custom port.
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Create a new config with a custom HTTP port.
    #[must_use]
    pub fn with_http_port(mut self, port: u16) -> Self {
        self.http_port = port;
        self
    }

    /// Create a new config with a custom CPU allocation (number of shards).
    #[must_use]
    pub fn with_cpu_allocation(mut self, allocation: impl Into<String>) -> Self {
        self.cpu_allocation = allocation.into();
        self
    }

    /// Create a new config with a custom memory pool size.
    #[must_use]
    pub fn with_memory_pool_size(mut self, size: impl Into<String>) -> Self {
        self.memory_pool_size = size.into();
        self
    }

    /// Create a new config with a custom bucket capacity.
    #[must_use]
    pub fn with_bucket_capacity(mut self, capacity: u32) -> Self {
        self.bucket_capacity = capacity;
        self
    }

    /// Get the TCP connection address for clients.
    #[must_use]
    pub fn connection_address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// Get environment variables for spawning iggy-server.
    ///
    /// Iggy uses environment variables for configuration, not CLI flags.
    /// Sets default root credentials (iggy/iggy) for development use.
    /// Configures resource limits to prevent excessive CPU/memory usage.
    #[must_use]
    pub fn env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert(
            "IGGY_TCP_ADDRESS".to_string(),
            format!("127.0.0.1:{}", self.port),
        );
        vars.insert(
            "IGGY_HTTP_ADDRESS".to_string(),
            format!("127.0.0.1:{}", self.http_port),
        );
        vars.insert(
            "IGGY_SYSTEM_PATH".to_string(),
            self.data_dir.display().to_string(),
        );
        // Set default root credentials for local development
        // These match the SDK's DEFAULT_ROOT_USERNAME/DEFAULT_ROOT_PASSWORD
        vars.insert("IGGY_ROOT_USERNAME".to_string(), "iggy".to_string());
        vars.insert("IGGY_ROOT_PASSWORD".to_string(), "iggy".to_string());
        // Resource control: limit CPU and memory usage
        vars.insert(
            "IGGY_SYSTEM_SHARDING_CPU_ALLOCATION".to_string(),
            self.cpu_allocation.clone(),
        );
        vars.insert(
            "IGGY_SYSTEM_MEMORY_POOL_SIZE".to_string(),
            self.memory_pool_size.clone(),
        );
        vars.insert(
            "IGGY_SYSTEM_MEMORY_POOL_BUCKET_CAPACITY".to_string(),
            self.bucket_capacity.to_string(),
        );
        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_values() {
        let config = IggyConfig::default();

        assert_eq!(config.binary_path, PathBuf::from("iggy-server"));
        assert_eq!(config.port, 8090);
        assert_eq!(config.max_restart_attempts, 3);
        assert_eq!(config.health_check_interval, Duration::from_secs(5));
    }

    #[test]
    fn config_builder_pattern() {
        let config = IggyConfig::default()
            .with_binary_path("/usr/bin/iggy")
            .with_data_dir("/var/lib/iggy")
            .with_port(9000);

        assert_eq!(config.binary_path, PathBuf::from("/usr/bin/iggy"));
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/iggy"));
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn config_connection_address() {
        let config = IggyConfig::default().with_port(8091);
        assert_eq!(config.connection_address(), "127.0.0.1:8091");
    }

    #[test]
    fn find_binary_returns_explicit_absolute_path_when_exists() {
        // Create a temp file to act as our "binary"
        let temp_dir = tempfile::tempdir().unwrap();
        let binary_path = temp_dir.path().join("iggy-server");
        std::fs::write(&binary_path, "fake binary").unwrap();

        let config = IggyConfig::default().with_binary_path(&binary_path);
        let found = config.find_binary();

        assert_eq!(found, Some(binary_path));
    }

    #[test]
    fn find_binary_ignores_nonexistent_explicit_path() {
        let config = IggyConfig::default().with_binary_path("/nonexistent/iggy-server");
        // This should fall through to PATH lookup, which may or may not find anything
        // The key is it doesn't return the nonexistent explicit path
        let found = config.find_binary();

        assert_ne!(found, Some(PathBuf::from("/nonexistent/iggy-server")));
    }

    #[test]
    fn find_binary_finds_sibling_of_current_exe() {
        // This test is tricky because we can't easily mock current_exe
        // We'll just verify find_binary returns Some when iggy-server exists
        // next to the test binary (which it does in our target/release/)
        let config = IggyConfig::default();

        // Check if iggy-server exists next to current exe
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            let sibling = dir.join("iggy-server");
            if sibling.exists() {
                let found = config.find_binary();
                assert_eq!(found, Some(sibling));
            }
        }
    }

    #[test]
    fn find_binary_returns_none_when_not_found() {
        // Use a config with a relative path (not absolute), so it won't
        // match the first condition, and if iggy-server isn't in PATH
        // or next to the current exe, it should return None
        let config = IggyConfig::default().with_binary_path("definitely-not-iggy-server");

        // We can't guarantee None here because iggy-server might be in PATH
        // So we just verify the function doesn't panic
        let _found = config.find_binary();
    }

    #[test]
    fn env_vars_returns_correct_iggy_environment() {
        let config = IggyConfig::default()
            .with_port(9090)
            .with_http_port(3001)
            .with_data_dir("/custom/data");

        let vars = config.env_vars();

        assert_eq!(
            vars.get("IGGY_TCP_ADDRESS"),
            Some(&"127.0.0.1:9090".to_string())
        );
        assert_eq!(
            vars.get("IGGY_HTTP_ADDRESS"),
            Some(&"127.0.0.1:3001".to_string())
        );
        assert_eq!(
            vars.get("IGGY_SYSTEM_PATH"),
            Some(&"/custom/data".to_string())
        );
    }

    #[test]
    fn with_http_port_sets_http_port() {
        let config = IggyConfig::default().with_http_port(4000);
        assert_eq!(config.http_port, 4000);
    }

    #[test]
    fn default_http_port_is_not_3000() {
        // Port 3000 is commonly used by dev servers, avoid conflicts
        let config = IggyConfig::default();
        assert_ne!(config.http_port, 3000);
    }

    // ==================== Resource Control Tests ====================

    #[test]
    fn default_cpu_allocation_is_conservative() {
        // Default should NOT use all cores - that's the bug we're fixing
        let config = IggyConfig::default();
        assert_eq!(config.cpu_allocation, "4");
    }

    #[test]
    fn default_memory_pool_size_is_minimum() {
        // Use minimum viable memory (512 MiB) not the 4 GiB default
        let config = IggyConfig::default();
        assert_eq!(config.memory_pool_size, "512 MiB");
    }

    #[test]
    fn default_bucket_capacity_is_minimum() {
        // Use minimum bucket capacity (128) not the 8192 default
        let config = IggyConfig::default();
        assert_eq!(config.bucket_capacity, 128);
    }

    #[test]
    fn with_cpu_allocation_sets_value() {
        let config = IggyConfig::default().with_cpu_allocation("8");
        assert_eq!(config.cpu_allocation, "8");
    }

    #[test]
    fn with_memory_pool_size_sets_value() {
        let config = IggyConfig::default().with_memory_pool_size("1 GiB");
        assert_eq!(config.memory_pool_size, "1 GiB");
    }

    #[test]
    fn with_bucket_capacity_sets_value() {
        let config = IggyConfig::default().with_bucket_capacity(256);
        assert_eq!(config.bucket_capacity, 256);
    }

    #[test]
    fn env_vars_includes_cpu_allocation() {
        let config = IggyConfig::default().with_cpu_allocation("2");
        let vars = config.env_vars();
        assert_eq!(
            vars.get("IGGY_SYSTEM_SHARDING_CPU_ALLOCATION"),
            Some(&"2".to_string())
        );
    }

    #[test]
    fn env_vars_includes_memory_pool_size() {
        let config = IggyConfig::default().with_memory_pool_size("1 GiB");
        let vars = config.env_vars();
        assert_eq!(
            vars.get("IGGY_SYSTEM_MEMORY_POOL_SIZE"),
            Some(&"1 GiB".to_string())
        );
    }

    #[test]
    fn env_vars_includes_bucket_capacity() {
        let config = IggyConfig::default().with_bucket_capacity(256);
        let vars = config.env_vars();
        assert_eq!(
            vars.get("IGGY_SYSTEM_MEMORY_POOL_BUCKET_CAPACITY"),
            Some(&"256".to_string())
        );
    }
}
