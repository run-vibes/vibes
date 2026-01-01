//! Pre-flight checks for Iggy server requirements.
//!
//! Iggy uses io_uring on Linux which requires sufficient locked memory.
//! These checks help diagnose system configuration issues before the
//! server fails to start.

use crate::error::{Error, Result};

/// Minimum recommended locked memory limit for io_uring (64 MB).
/// This is a conservative estimate; actual requirements depend on workload.
const MIN_MEMLOCK_BYTES: u64 = 64 * 1024 * 1024;

/// Result of a pre-flight check.
#[derive(Debug, Clone)]
pub struct PreflightResult {
    /// Whether the check passed.
    pub passed: bool,
    /// Current value (human-readable).
    pub current: String,
    /// Required value (human-readable).
    pub required: String,
    /// Help message if the check failed.
    pub help: Option<String>,
}

/// Check if the system has sufficient locked memory limit for io_uring.
///
/// Iggy uses the compio async runtime which requires io_uring on Linux.
/// io_uring needs locked memory for its ring buffers.
///
/// # Returns
///
/// A `PreflightResult` indicating whether the limit is sufficient.
#[cfg(target_os = "linux")]
pub fn check_memlock_limit() -> PreflightResult {
    use rlimit::{Resource, getrlimit};

    match getrlimit(Resource::MEMLOCK) {
        Ok((soft, _hard)) => {
            let passed = soft == rlimit::INFINITY || soft >= MIN_MEMLOCK_BYTES;
            let current = if soft == rlimit::INFINITY {
                "unlimited".to_string()
            } else {
                format_bytes(soft)
            };

            PreflightResult {
                passed,
                current,
                required: format_bytes(MIN_MEMLOCK_BYTES),
                help: if passed {
                    None
                } else {
                    Some(MEMLOCK_HELP.to_string())
                },
            }
        }
        Err(_) => PreflightResult {
            passed: false,
            current: "unknown".to_string(),
            required: format_bytes(MIN_MEMLOCK_BYTES),
            help: Some("Could not query memlock limit".to_string()),
        },
    }
}

/// On non-Linux systems, io_uring is not used.
#[cfg(not(target_os = "linux"))]
pub fn check_memlock_limit() -> PreflightResult {
    PreflightResult {
        passed: true,
        current: "N/A".to_string(),
        required: "N/A".to_string(),
        help: None,
    }
}

/// Run all pre-flight checks and return an error if any fail.
///
/// # Errors
///
/// Returns an error with a detailed help message if any check fails.
pub fn run_preflight_checks() -> Result<()> {
    let memlock = check_memlock_limit();

    if !memlock.passed {
        let msg = format!(
            "Insufficient locked memory limit for io_uring.\n\
             Current: {}\n\
             Required: {} (minimum)\n\n\
             {}",
            memlock.current,
            memlock.required,
            memlock.help.unwrap_or_default()
        );
        return Err(Error::Connection(msg));
    }

    Ok(())
}

/// Format bytes as a human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{} KB", bytes / KB)
    } else {
        format!("{} bytes", bytes)
    }
}

const MEMLOCK_HELP: &str = r#"Iggy requires sufficient locked memory for io_uring.

To fix this, run one of the following:

  1. Temporary (current session only):
     ulimit -l unlimited

  2. Persistent (add to /etc/security/limits.conf as root):
     * soft memlock unlimited
     * hard memlock unlimited
     (Then log out and back in)

  3. For systemd services (add to service file):
     LimitMEMLOCK=infinity"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_memlock_returns_result() {
        let result = check_memlock_limit();
        // Just verify it runs without panicking and returns sensible data
        assert!(!result.current.is_empty());
        assert!(!result.required.is_empty());
    }

    #[test]
    fn format_bytes_formats_correctly() {
        assert_eq!(format_bytes(512), "512 bytes");
        assert_eq!(format_bytes(1024), "1 KB");
        assert_eq!(format_bytes(8 * 1024 * 1024), "8 MB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2 GB");
    }

    #[test]
    fn preflight_result_has_help_when_failed() {
        let result = PreflightResult {
            passed: false,
            current: "8 MB".to_string(),
            required: "64 MB".to_string(),
            help: Some("Fix it".to_string()),
        };
        assert!(!result.passed);
        assert!(result.help.is_some());
    }
}
