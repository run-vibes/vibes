---
id: evals
title: Evaluation Framework
status: planned
description: Benchmark mode for industry evals, longitudinal mode for measuring real-world performance over time
---

# Evaluation Framework

Benchmarking system for validating vibes+groove against industry standards (SWE-Bench, Remote Labor Index) AND longitudinal measurement of performance over days/weeks of real work.

## Overview

Evals is **completely separate from groove**. While groove learns and adapts, evals measures:

1. **Benchmark Mode**: Point-in-time performance against industry standards
2. **Longitudinal Mode**: Continuous measurement over extended periods

The north star: measure performance over days or weeks of real work.

## Two Modes

### Benchmark Mode (Point-in-Time)

Run standardized benchmarks to measure current capability:

```rust
pub struct Benchmark {
    pub id: BenchmarkId,
    pub name: String,
    pub suite: BenchmarkSuite,
    pub tasks: Vec<BenchmarkTask>,
}

pub enum BenchmarkSuite {
    SWEBench { split: String },      // SWE-Bench Lite, Full, Verified
    RemoteLaborIndex,                 // Real-world task completion
    HumanEval,                        // Code generation
    Custom { spec: PathBuf },         // User-defined
}

pub struct BenchmarkResult {
    pub benchmark_id: BenchmarkId,
    pub timestamp: DateTime,
    pub scores: HashMap<String, f64>,
    pub passed: u32,
    pub failed: u32,
    pub duration: Duration,
    pub configuration: EvalConfiguration,
}
```

### Longitudinal Mode (Continuous)

Track performance across real work over extended periods:

```rust
pub struct LongitudinalStudy {
    pub id: StudyId,
    pub name: String,
    pub started: DateTime,
    pub period: StudyPeriod,
    pub metrics: Vec<MetricDefinition>,
    pub checkpoints: Vec<Checkpoint>,
}

pub enum StudyPeriod {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Indefinite,  // Until manually stopped
}

pub struct Checkpoint {
    pub timestamp: DateTime,
    pub metrics: LongitudinalMetrics,
    pub events_analyzed: u64,
    pub sessions_included: Vec<SessionId>,
}
```

## Longitudinal Metrics

```rust
pub struct LongitudinalMetrics {
    // Session-level
    pub sessions_completed: u64,
    pub session_success_rate: f64,
    pub avg_session_duration: Duration,

    // Task-level
    pub tasks_completed: u64,
    pub first_attempt_success_rate: f64,
    pub avg_iterations_to_success: f64,

    // Agent-level
    pub agent_efficiency: f64,          // Tasks/time
    pub tool_success_rate: f64,
    pub self_correction_rate: f64,

    // Swarm-level
    pub swarm_coordination_overhead: f64,
    pub parallelism_efficiency: f64,

    // Learning (groove integration point)
    pub learnings_applied: u64,
    pub learning_effectiveness: f64,    // Success rate when learning applied

    // Cost
    pub total_tokens: u64,
    pub total_cost: f64,
    pub cost_per_successful_task: f64,

    // Time window
    pub period: TimePeriod,
}
```

## What Gets Measured

### Session Evaluation

```rust
pub struct SessionEval {
    pub session_id: SessionId,
    pub started: DateTime,
    pub completed: Option<DateTime>,

    // Goal completion
    pub goals_stated: Vec<String>,
    pub goals_achieved: Vec<GoalResult>,

    // Quality
    pub user_interventions: u32,
    pub error_recovery_success: f64,
    pub tool_efficiency: f64,
}
```

### Workflow Evaluation

```rust
pub struct WorkflowEval {
    pub workflow_type: String,          // "debug", "feature", "refactor"
    pub attempts: u32,
    pub successes: u32,
    pub avg_duration: Duration,
    pub common_failure_points: Vec<String>,
}
```

### Swarm Evaluation

```rust
pub struct SwarmEval {
    pub strategy: SwarmStrategy,
    pub tasks_run: u32,
    pub coordination_overhead: Duration,
    pub quality_vs_single_agent: f64,   // Relative improvement
    pub optimal_agent_count: u32,       // For this task type
}
```

## Trend Analysis

```rust
pub struct TrendAnalysis {
    pub metric: String,
    pub period: TimePeriod,
    pub data_points: Vec<(DateTime, f64)>,
    pub trend: Trend,
    pub forecast: Option<Forecast>,
}

pub enum Trend {
    Improving { rate: f64 },
    Stable { variance: f64 },
    Declining { rate: f64 },
    Insufficient { minimum_needed: u32 },
}
```

## Configuration

```rust
pub struct EvalConfiguration {
    // What to measure
    pub benchmark_mode: bool,
    pub longitudinal_mode: bool,

    // Benchmark settings
    pub benchmarks: Vec<BenchmarkSuite>,

    // Longitudinal settings
    pub checkpoint_interval: Duration,
    pub metrics_to_track: Vec<MetricDefinition>,
    pub retention_period: Duration,

    // Model/Agent configuration
    pub model: ModelId,
    pub agent_config: AgentConfig,
    pub groove_enabled: bool,
}
```

## Storage

```rust
pub struct EvalStorage {
    // Benchmark results
    pub benchmarks: Table<BenchmarkResult>,

    // Longitudinal data
    pub studies: Table<LongitudinalStudy>,
    pub checkpoints: Table<Checkpoint>,
    pub raw_metrics: TimeSeriesStore,

    // Analysis
    pub trends: Table<TrendAnalysis>,
}
```

## CLI Commands

```
# Benchmark mode
vibes eval run swe-bench              # Run SWE-Bench
vibes eval run remote-labor-index     # Run Remote Labor Index
vibes eval run custom <spec>          # Run custom benchmark
vibes eval results                    # Show benchmark results
vibes eval compare <a> <b>            # Compare two runs

# Longitudinal mode
vibes eval study start <name>         # Start longitudinal study
vibes eval study stop <id>            # Stop study
vibes eval study status               # Current study status
vibes eval study report <id>          # Generate report

# Analysis
vibes eval trends                     # Show performance trends
vibes eval trends --metric <name>     # Specific metric trend
vibes eval export <format>            # Export data (csv, json)
```

## Reports

```rust
pub struct EvalReport {
    pub title: String,
    pub period: TimePeriod,

    // Summary
    pub executive_summary: String,

    // Benchmark results
    pub benchmark_scores: Vec<BenchmarkResult>,

    // Longitudinal insights
    pub key_metrics: LongitudinalMetrics,
    pub trends: Vec<TrendAnalysis>,
    pub insights: Vec<Insight>,

    // Recommendations
    pub recommendations: Vec<Recommendation>,
}

pub struct Insight {
    pub category: InsightCategory,
    pub finding: String,
    pub evidence: Vec<DataPoint>,
    pub significance: Significance,
}
```

## Milestones

| ID | Milestone | Description | Status |
|----|-----------|-------------|--------|
| 39 | [Eval Core](../../milestones/39-eval-core/) | Metrics definitions, study lifecycle, storage | in-progress |
| 50 | [Longitudinal Mode](../../milestones/50-longitudinal-mode/) | Continuous measurement over real work | planned |
| 51 | [Eval Reports](../../milestones/51-eval-reports/) | Export, visualization, summaries | planned |
| 52 | [Trend Analysis](../../milestones/52-trend-analysis/) | Forecasting, anomaly detection, patterns | planned |
| 53 | [Benchmark Harness](../../milestones/53-benchmark-harness/) | SWE-Bench + extensible TaskRunner infrastructure | planned |
| 54 | [Additional Benchmarks](../../milestones/54-additional-benchmarks/) | HumanEval, Remote Labor Index, Custom | planned |

**Sequencing:** M39 → M50 → M51 → M52 → M53 → M54
