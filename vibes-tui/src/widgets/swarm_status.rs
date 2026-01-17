//! Swarm coordination status widget.
//!
//! Displays swarm-level status indicators including execution strategy,
//! overall status, agent count breakdown, and aggregate metrics.

/// Overall status of a swarm execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants used in tests and future swarm integration
pub enum SwarmStatus {
    /// Not yet started.
    #[default]
    Pending,
    /// At least one agent active.
    Running,
    /// All agents completed successfully.
    Completed,
    /// At least one agent failed.
    Failed,
    /// Some completed, some failed.
    Partial,
    /// User cancelled.
    Cancelled,
}

/// Spinner animation frames for running status.
const SPINNER_FRAMES: [char; 4] = ['⟳', '◐', '◓', '◑'];

impl SwarmStatus {
    /// Returns the display label for this status.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Partial => "Partial",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Returns the spinner character for the given animation frame.
    ///
    /// Only shows spinner for Running status; other statuses return space.
    pub fn spinner_char(&self, frame: usize) -> char {
        match self {
            Self::Running => SPINNER_FRAMES[frame % SPINNER_FRAMES.len()],
            _ => ' ',
        }
    }
}

/// Execution strategy for a swarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants used in tests and future swarm integration
pub enum SwarmStrategy {
    /// All agents run simultaneously.
    #[default]
    Parallel,
    /// Agents run one after another.
    Sequential,
    /// Output of one feeds into next.
    Pipeline,
    /// Multiple agents, consensus on result.
    Voting,
}

impl SwarmStrategy {
    /// Returns the display label for this strategy.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Parallel => "Parallel",
            Self::Sequential => "Sequential",
            Self::Pipeline => "Pipeline",
            Self::Voting => "Voting",
        }
    }
}

/// Agent count breakdown for swarm status display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AgentCounts {
    /// Total number of agents in the swarm.
    pub total: u32,
    /// Number of currently running agents.
    pub running: u32,
    /// Number of successfully completed agents.
    pub completed: u32,
    /// Number of failed agents.
    pub failed: u32,
}

impl AgentCounts {
    /// Creates a new AgentCounts with the given values.
    pub fn new(total: u32, running: u32, completed: u32, failed: u32) -> Self {
        Self {
            total,
            running,
            completed,
            failed,
        }
    }

    /// Formats the agent counts as a summary string.
    ///
    /// Example: "2/3 running  1 completed" or "1/3 running  1 completed  1 failed"
    pub fn format_summary(&self) -> String {
        let mut parts = vec![format!("{}/{} running", self.running, self.total)];

        if self.completed > 0 {
            parts.push(format!("{} completed", self.completed));
        }

        if self.failed > 0 {
            parts.push(format!("{} failed", self.failed));
        }

        parts.join("  ")
    }
}

/// Aggregate metrics for swarm execution.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(dead_code)] // Used in tests and future swarm integration
pub struct SwarmMetrics {
    /// Total tokens used across all agents.
    pub total_tokens: u64,
    /// Total cost in dollars.
    pub cost_dollars: f64,
    /// Duration in seconds.
    pub duration_secs: u64,
}

#[allow(dead_code)] // Methods used in tests and future swarm integration
impl SwarmMetrics {
    /// Creates new metrics with the given values.
    pub fn new(total_tokens: u64, cost_dollars: f64, duration_secs: u64) -> Self {
        Self {
            total_tokens,
            cost_dollars,
            duration_secs,
        }
    }

    /// Formats tokens with thousands separators.
    pub fn format_tokens(&self) -> String {
        format_with_separators(self.total_tokens)
    }

    /// Formats cost as dollars (e.g., "$1.87").
    pub fn format_cost(&self) -> String {
        format!("${:.2}", self.cost_dollars)
    }

    /// Formats duration as minutes and seconds (e.g., "4m 32s").
    pub fn format_duration(&self) -> String {
        let minutes = self.duration_secs / 60;
        let seconds = self.duration_secs % 60;
        if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Formats the metrics as a summary string.
    pub fn format_summary(&self) -> String {
        format!(
            "Tokens: {}   Cost: {}   Duration: {}",
            self.format_tokens(),
            self.format_cost(),
            self.format_duration()
        )
    }
}

/// Formats a number with thousands separators.
#[allow(dead_code)] // Used by SwarmMetrics in tests
fn format_with_separators(n: u64) -> String {
    let s = n.to_string();
    let bytes: Vec<_> = s.bytes().rev().collect();
    let chunks: Vec<_> = bytes
        .chunks(3)
        .map(|chunk| String::from_utf8(chunk.iter().copied().rev().collect()).unwrap())
        .collect();
    chunks.into_iter().rev().collect::<Vec<_>>().join(",")
}

#[cfg(test)]
mod tests {
    // === SwarmStatus enum tests ===

    #[test]
    fn swarm_status_default_is_pending() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::default(), SwarmStatus::Pending);
    }

    #[test]
    fn swarm_status_has_all_variants() {
        use super::SwarmStatus;
        let _pending = SwarmStatus::Pending;
        let _running = SwarmStatus::Running;
        let _completed = SwarmStatus::Completed;
        let _failed = SwarmStatus::Failed;
        let _partial = SwarmStatus::Partial;
        let _cancelled = SwarmStatus::Cancelled;
    }

    #[test]
    fn swarm_status_is_copy() {
        use super::SwarmStatus;
        let status = SwarmStatus::Running;
        let copied = status;
        assert_eq!(status, copied);
    }

    #[test]
    fn swarm_status_label_pending() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Pending.label(), "Pending");
    }

    #[test]
    fn swarm_status_label_running() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Running.label(), "Running");
    }

    #[test]
    fn swarm_status_label_completed() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Completed.label(), "Completed");
    }

    #[test]
    fn swarm_status_label_failed() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Failed.label(), "Failed");
    }

    #[test]
    fn swarm_status_label_partial() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Partial.label(), "Partial");
    }

    #[test]
    fn swarm_status_label_cancelled() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Cancelled.label(), "Cancelled");
    }

    // === SwarmStrategy enum tests ===

    #[test]
    fn swarm_strategy_default_is_parallel() {
        use super::SwarmStrategy;
        assert_eq!(SwarmStrategy::default(), SwarmStrategy::Parallel);
    }

    #[test]
    fn swarm_strategy_has_all_variants() {
        use super::SwarmStrategy;
        let _parallel = SwarmStrategy::Parallel;
        let _sequential = SwarmStrategy::Sequential;
        let _pipeline = SwarmStrategy::Pipeline;
        let _voting = SwarmStrategy::Voting;
    }

    #[test]
    fn swarm_strategy_is_copy() {
        use super::SwarmStrategy;
        let strategy = SwarmStrategy::Parallel;
        let copied = strategy;
        assert_eq!(strategy, copied);
    }

    #[test]
    fn swarm_strategy_label_parallel() {
        use super::SwarmStrategy;
        assert_eq!(SwarmStrategy::Parallel.label(), "Parallel");
    }

    #[test]
    fn swarm_strategy_label_sequential() {
        use super::SwarmStrategy;
        assert_eq!(SwarmStrategy::Sequential.label(), "Sequential");
    }

    #[test]
    fn swarm_strategy_label_pipeline() {
        use super::SwarmStrategy;
        assert_eq!(SwarmStrategy::Pipeline.label(), "Pipeline");
    }

    #[test]
    fn swarm_strategy_label_voting() {
        use super::SwarmStrategy;
        assert_eq!(SwarmStrategy::Voting.label(), "Voting");
    }

    // === SwarmStatus spinner indicator tests ===

    #[test]
    fn swarm_status_running_shows_spinner() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Running.spinner_char(0), '⟳');
    }

    #[test]
    fn swarm_status_pending_no_spinner() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Pending.spinner_char(0), ' ');
    }

    #[test]
    fn swarm_status_completed_no_spinner() {
        use super::SwarmStatus;
        assert_eq!(SwarmStatus::Completed.spinner_char(0), ' ');
    }

    #[test]
    fn swarm_status_spinner_animates() {
        use super::SwarmStatus;
        let frame0 = SwarmStatus::Running.spinner_char(0);
        let frame1 = SwarmStatus::Running.spinner_char(1);
        let frame2 = SwarmStatus::Running.spinner_char(2);
        // Spinner should cycle through frames
        assert!(frame0 != ' ');
        assert!(frame1 != ' ');
        assert!(frame2 != ' ');
    }

    // === AgentCounts tests ===

    #[test]
    fn agent_counts_default() {
        use super::AgentCounts;
        let counts = AgentCounts::default();
        assert_eq!(counts.total, 0);
        assert_eq!(counts.running, 0);
        assert_eq!(counts.completed, 0);
        assert_eq!(counts.failed, 0);
    }

    #[test]
    fn agent_counts_new() {
        use super::AgentCounts;
        let counts = AgentCounts::new(3, 1, 1, 0);
        assert_eq!(counts.total, 3);
        assert_eq!(counts.running, 1);
        assert_eq!(counts.completed, 1);
        assert_eq!(counts.failed, 0);
    }

    #[test]
    fn agent_counts_format_running_and_completed() {
        use super::AgentCounts;
        let counts = AgentCounts::new(3, 2, 1, 0);
        assert_eq!(counts.format_summary(), "2/3 running  1 completed");
    }

    #[test]
    fn agent_counts_format_with_failed() {
        use super::AgentCounts;
        let counts = AgentCounts::new(3, 1, 1, 1);
        assert_eq!(
            counts.format_summary(),
            "1/3 running  1 completed  1 failed"
        );
    }

    #[test]
    fn agent_counts_format_all_completed() {
        use super::AgentCounts;
        let counts = AgentCounts::new(3, 0, 3, 0);
        assert_eq!(counts.format_summary(), "0/3 running  3 completed");
    }

    // === SwarmMetrics tests ===

    #[test]
    fn swarm_metrics_default() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::default();
        assert_eq!(metrics.total_tokens, 0);
        assert!((metrics.cost_dollars - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.duration_secs, 0);
    }

    #[test]
    fn swarm_metrics_new() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::new(125432, 1.87, 272);
        assert_eq!(metrics.total_tokens, 125432);
        assert!((metrics.cost_dollars - 1.87).abs() < f64::EPSILON);
        assert_eq!(metrics.duration_secs, 272);
    }

    #[test]
    fn swarm_metrics_format_tokens_with_separators() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::new(125432, 0.0, 0);
        assert_eq!(metrics.format_tokens(), "125,432");
    }

    #[test]
    fn swarm_metrics_format_tokens_small() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::new(42, 0.0, 0);
        assert_eq!(metrics.format_tokens(), "42");
    }

    #[test]
    fn swarm_metrics_format_cost() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::new(0, 1.87, 0);
        assert_eq!(metrics.format_cost(), "$1.87");
    }

    #[test]
    fn swarm_metrics_format_duration_minutes() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::new(0, 0.0, 272); // 4m 32s
        assert_eq!(metrics.format_duration(), "4m 32s");
    }

    #[test]
    fn swarm_metrics_format_duration_seconds_only() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::new(0, 0.0, 45);
        assert_eq!(metrics.format_duration(), "45s");
    }

    #[test]
    fn swarm_metrics_format_summary() {
        use super::SwarmMetrics;
        let metrics = SwarmMetrics::new(125432, 1.87, 272);
        assert_eq!(
            metrics.format_summary(),
            "Tokens: 125,432   Cost: $1.87   Duration: 4m 32s"
        );
    }
}
