//! Restart policy for cloudflared process supervision

use std::time::{Duration, Instant};

/// Policy for restarting failed processes with exponential backoff
#[derive(Debug)]
pub struct RestartPolicy {
    attempts: Vec<Instant>,
    max_attempts_per_window: u32,
    window_duration: Duration,
    current_backoff_idx: usize,
}

/// Backoff delays: immediate, 1s, 5s, 15s, 30s
const BACKOFF_DELAYS: [Duration; 5] = [
    Duration::from_secs(0),
    Duration::from_secs(1),
    Duration::from_secs(5),
    Duration::from_secs(15),
    Duration::from_secs(30),
];

impl RestartPolicy {
    /// Create a new restart policy
    ///
    /// - `max_attempts`: Maximum restart attempts within the window
    /// - `window`: Time window to count attempts
    pub fn new(max_attempts: u32, window: Duration) -> Self {
        Self {
            attempts: Vec::new(),
            max_attempts_per_window: max_attempts,
            window_duration: window,
            current_backoff_idx: 0,
        }
    }

    /// Default policy: 5 attempts per 60 seconds
    pub fn default_policy() -> Self {
        Self::new(5, Duration::from_secs(60))
    }

    /// Check if we should restart and get the delay
    ///
    /// Returns `Some(delay)` if restart is allowed, `None` if we should give up
    pub fn should_restart(&mut self) -> Option<Duration> {
        let now = Instant::now();

        // Clean old attempts outside window
        self.attempts
            .retain(|t| now.duration_since(*t) < self.window_duration);

        if self.attempts.len() >= self.max_attempts_per_window as usize {
            return None; // Give up
        }

        let delay = BACKOFF_DELAYS
            .get(self.current_backoff_idx)
            .copied()
            .unwrap_or(BACKOFF_DELAYS[BACKOFF_DELAYS.len() - 1]);

        self.attempts.push(now);
        self.current_backoff_idx = (self.current_backoff_idx + 1).min(BACKOFF_DELAYS.len() - 1);

        Some(delay)
    }

    /// Reset the policy after successful connection
    pub fn reset(&mut self) {
        self.attempts.clear();
        self.current_backoff_idx = 0;
    }

    /// Get the number of recent attempts
    pub fn recent_attempts(&self) -> usize {
        self.attempts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_policy_first_attempt_immediate() {
        let mut policy = RestartPolicy::default_policy();
        let delay = policy.should_restart().unwrap();
        assert_eq!(delay, Duration::from_secs(0));
    }

    #[test]
    fn restart_policy_backoff_increases() {
        let mut policy = RestartPolicy::default_policy();

        let d1 = policy.should_restart().unwrap();
        let d2 = policy.should_restart().unwrap();
        let d3 = policy.should_restart().unwrap();

        assert_eq!(d1, Duration::from_secs(0));
        assert_eq!(d2, Duration::from_secs(1));
        assert_eq!(d3, Duration::from_secs(5));
    }

    #[test]
    fn restart_policy_gives_up_after_max_attempts() {
        let mut policy = RestartPolicy::new(3, Duration::from_secs(60));

        assert!(policy.should_restart().is_some());
        assert!(policy.should_restart().is_some());
        assert!(policy.should_restart().is_some());
        assert!(policy.should_restart().is_none()); // 4th attempt denied
    }

    #[test]
    fn restart_policy_reset_clears_state() {
        let mut policy = RestartPolicy::default_policy();

        policy.should_restart();
        policy.should_restart();
        assert_eq!(policy.recent_attempts(), 2);

        policy.reset();
        assert_eq!(policy.recent_attempts(), 0);

        // Should start from immediate again
        let delay = policy.should_restart().unwrap();
        assert_eq!(delay, Duration::from_secs(0));
    }

    #[test]
    fn restart_policy_caps_at_max_backoff() {
        let mut policy = RestartPolicy::new(10, Duration::from_secs(120));

        // Exhaust all backoff levels
        for _ in 0..8 {
            policy.should_restart();
        }

        // Should stay at max backoff (30s)
        let delay = policy.should_restart().unwrap();
        assert_eq!(delay, Duration::from_secs(30));
    }
}
