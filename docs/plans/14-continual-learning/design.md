# Continual Learning Design

## Overview

Transform vibes from a Claude Code proxy into a **learning harness** that makes any AI coding assistant progressively more effective through accumulated experience. The system captures learnings automatically from session outcomes and injects them into future sessions with no user intervention required.

## Goals

1. **Zero friction** - Automatic outcome-based learning, no user annotation needed
2. **Harness agnostic** - Works with any AI coding assistant, not just Claude Code
3. **Fully adaptive** - No hardcoded thresholds; all parameters learn from outcomes
4. **Hierarchical scope** - Learnings isolated as Global → User → Project
5. **Open-world ready** - Can detect patterns that don't exist yet (like MCP before MCP)

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Storage | CozoDB only | Unified relational + graph + vector in single MPL-2.0 library |
| Learning IDs | UUIDv7 | Time-ordered for natural chronological queries |
| Parameter tuning | Bayesian adaptive | No hardcoded thresholds; Thompson sampling for exploration |
| Capture mechanism | Claude hooks first | Most control; abstract for other harnesses |
| Injection mechanism | CLAUDE.md first | Simple, reliable; abstract for extensibility |
| Level 0 priority | Harness introspection | Must know capabilities before attempting to learn |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         vibes-learning plugin                        │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────────┐  │
│  │ Level 0          │  │ Level 1          │  │ Level 2-3         │  │
│  │ Introspection    │  │ Capture/Inject   │  │ Learn/Adapt       │  │
│  │                  │  │                  │  │                   │  │
│  │ ┌──────────────┐ │  │ ┌──────────────┐ │  │ ┌───────────────┐ │  │
│  │ │   Harness    │ │  │ │   Capture    │ │  │ │  Transcript   │ │  │
│  │ │   trait      │─┼──┼▶│   Adapter    │ │  │ │  Analyzer     │ │  │
│  │ └──────────────┘ │  │ └──────────────┘ │  │ └───────────────┘ │  │
│  │ ┌──────────────┐ │  │ ┌──────────────┐ │  │ ┌───────────────┐ │  │
│  │ │ Capabilities │ │  │ │  Injection   │ │  │ │   Strategy    │ │  │
│  │ │   struct     │─┼──┼▶│   Adapter    │ │  │ │   Learner     │ │  │
│  │ └──────────────┘ │  │ └──────────────┘ │  │ └───────────────┘ │  │
│  └──────────────────┘  └────────┬─────────┘  │ ┌───────────────┐ │  │
│                                 │            │ │   Novelty     │ │  │
│                                 ▼            │ │   Detector    │ │  │
│                        ┌──────────────────┐  │ └───────────────┘ │  │
│                        │     CozoDB       │  └───────────────────┘  │
│                        │  (storage layer) │                         │
│                        └──────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
```

## Level Progression

| Level | Name | Purpose | Milestone |
|-------|------|---------|-----------|
| 0 | Harness Introspection | Discover what we can capture and inject | 4.1 |
| 1 | Capture & Inject | Basic learning pipeline (MVP) | 4.2, 4.3 |
| 2a | Assessment Framework | Measure outcomes, detect signals | 4.4 ⭐ |
| 2b | Learning Extraction | Rich pattern extraction from transcripts | 4.5 |
| 2c | Attribution Engine | Connect learnings to outcomes | 4.6 ⭐ |
| 3a | Adaptive Strategies | Learn HOW to inject, not just WHAT | 4.7 |
| 3b | Observability Dashboard | Surface learnings and metrics to users | 4.8 ⭐ |
| 4 | Open-World Adaptation | Detect unknown unknowns, meta-learning | 4.9 |

---

## Core Types

### Learning Model

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// UUIDv7 provides time-ordered unique identifiers
pub type LearningId = Uuid;

/// Hierarchical scope for learning isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    Global,
    User(String),
    Project(String),
}

/// A captured piece of knowledge
#[derive(Debug, Clone)]
pub struct Learning {
    pub id: LearningId,
    pub scope: Scope,
    pub category: LearningCategory,
    pub content: LearningContent,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: LearningSource,
    pub usage_stats: UsageStats,
}

#[derive(Debug, Clone)]
pub enum LearningCategory {
    CodePattern,
    Preference,
    Solution,
    ErrorRecovery,
    ToolUsage,
    HarnessKnowledge,
}
```

### Adaptive Parameters

**Critical:** No hardcoded coefficients anywhere. All thresholds learn from outcomes.

```rust
/// A parameter that learns via Bayesian updates
#[derive(Debug, Clone)]
pub struct AdaptiveParam {
    pub value: f64,
    pub uncertainty: f64,
    pub observations: u64,
    pub prior_alpha: f64,
    pub prior_beta: f64,
}

impl AdaptiveParam {
    pub fn new_uninformed() -> Self {
        Self {
            value: 0.5,
            uncertainty: 1.0,
            observations: 0,
            prior_alpha: 1.0,  // Uniform prior
            prior_beta: 1.0,
        }
    }

    /// Bayesian update based on outcome
    pub fn update(&mut self, outcome: f64, weight: f64) {
        self.observations += 1;
        let effective_weight = weight / (1.0 + self.uncertainty);
        self.prior_alpha += outcome * effective_weight;
        self.prior_beta += (1.0 - outcome) * effective_weight;
        self.value = self.prior_alpha / (self.prior_alpha + self.prior_beta);
        self.uncertainty = 1.0 / (1.0 + (self.observations as f64).sqrt());
    }

    /// Thompson sampling for exploration
    pub fn sample(&self) -> f64 {
        use rand_distr::{Beta, Distribution};
        let beta = Beta::new(self.prior_alpha, self.prior_beta).unwrap();
        beta.sample(&mut rand::thread_rng())
    }
}
```

---

## Harness Introspection (Level 0)

Before learning anything, we must discover what the harness supports.

```rust
/// A harness is any AI coding assistant we're enhancing
#[async_trait]
pub trait Harness: Send + Sync {
    fn introspect(&self) -> HarnessCapabilities;
    fn capture_adapter(&self) -> Box<dyn CaptureAdapter>;
    fn injection_adapter(&self) -> Box<dyn InjectionAdapter>;
    fn harness_type(&self) -> &str;
    fn version(&self) -> Option<String>;
}

/// What a harness is capable of
#[derive(Debug, Clone)]
pub struct HarnessCapabilities {
    pub hooks: Option<HookCapabilities>,
    pub transcripts: Option<TranscriptCapabilities>,
    pub config: Option<ConfigCapabilities>,
    pub input_mechanisms: Vec<InputMechanism>,
    pub output_format: OutputFormat,
    pub observable_signals: Vec<ObservableSignal>,
}
```

### Claude Code Harness

```rust
impl Harness for ClaudeCodeHarness {
    fn introspect(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            hooks: self.detect_hooks(),  // ~/.claude/hooks
            transcripts: self.detect_transcripts(),  // Stop hook provides path
            config: Some(ConfigCapabilities {
                claude_md: true,
                settings_json: true,
                environment_vars: true,
            }),
            input_mechanisms: vec![
                InputMechanism::ConfigFile {
                    path: PathBuf::from("CLAUDE.md"),
                    format: ConfigFormat::Markdown,
                },
            ],
            output_format: OutputFormat::Pty,
            observable_signals: vec![
                ObservableSignal::Stdout,
                ObservableSignal::Hooks,
                ObservableSignal::ExitCode,
            ],
        }
    }
}
```

---

## Storage Layer

### CozoDB Schema

```datalog
# Core entities
:create learning {
    id: String =>
    scope_type: String,
    scope_value: String?,
    category: String,
    description: String,
    pattern_json: String,
    insight: String,
    confidence: Float,
    created_at: Int,
    updated_at: Int,
    source_type: String,
    source_json: String
}

# Usage statistics
:create usage_stats {
    learning_id: String =>
    times_injected: Int,
    times_helpful: Int,
    times_ignored: Int,
    times_contradicted: Int,
    last_used: Int?
}

# Vector embeddings for semantic search
:create learning_embeddings {
    learning_id: String =>
    embedding: <F32; 1536>
}

# Adaptive parameters
:create adaptive_params {
    param_name: String =>
    value: Float,
    uncertainty: Float,
    observations: Int,
    prior_alpha: Float,
    prior_beta: Float
}

# Learning relationships (graph)
:create learning_relations {
    from_id: String,
    relation_type: String,
    to_id: String =>
    weight: Float,
    created_at: Int
}

# Indexes
::hnsw create learning_embeddings:semantic_idx {
    dim: 1536,
    m: 16,
    ef_construction: 200,
    fields: [embedding]
}

::index create learning:by_scope { scope_type, scope_value }
::index create learning:by_category { category }
```

### Storage Trait

```rust
#[async_trait]
pub trait LearningStorage: Send + Sync {
    async fn store_learning(&self, learning: &Learning) -> Result<LearningId>;
    async fn get_learning(&self, id: LearningId) -> Result<Option<Learning>>;
    async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>>;
    async fn semantic_search(&self, query: &[f32], limit: usize) -> Result<Vec<Learning>>;
    async fn update_usage(&self, id: LearningId, stats: &UsageStats) -> Result<()>;
    async fn get_related(&self, id: LearningId, relation: &str) -> Result<Vec<Learning>>;
}
```

---

## Capture & Injection

### Capture Adapter

```rust
#[async_trait]
pub trait CaptureAdapter: Send + Sync {
    /// Capture signals from a session
    async fn capture(&self, session_id: &str) -> Result<Vec<CapturedSignal>>;

    /// Get session outcome when complete
    async fn get_outcome(&self, session_id: &str) -> Result<SessionOutcome>;
}

/// Claude Code implementation using hooks
pub struct ClaudeCodeHooksCapture {
    hooks_receiver: HooksReceiver,
}

impl CaptureAdapter for ClaudeCodeHooksCapture {
    async fn capture(&self, session_id: &str) -> Result<Vec<CapturedSignal>> {
        // Receive PreToolUse, PostToolUse, Stop events via hooks
        self.hooks_receiver.get_events(session_id).await
    }
}
```

### Injection Adapter

```rust
#[async_trait]
pub trait InjectionAdapter: Send + Sync {
    /// Prepare learnings for injection into session context
    async fn prepare_context(
        &self,
        storage: &dyn LearningStorage,
        scope: &Scope,
        config: &AdaptiveConfig,
    ) -> Result<SessionContext>;

    /// Inject a learning using the selected strategy
    async fn inject(
        &self,
        learning: &Learning,
        strategy: &InjectionStrategy,
    ) -> Result<()>;
}

/// Claude Code injection via CLAUDE.md
pub struct ClaudeCodeInjector {
    project_path: PathBuf,
}
```

---

## Adaptive Injection Strategies

### Strategy Types

```rust
#[derive(Debug, Clone)]
pub enum InjectionStrategy {
    /// Inject into main Claude context
    MainContext { position: ContextPosition },

    /// Delegate to a subagent
    Subagent { agent_type: SubagentType, blocking: bool },

    /// Run in background, surface later
    BackgroundSubagent { agent_type: SubagentType, callback: CallbackMethod },

    /// Don't inject now, wait for trigger
    Deferred { trigger: DeferralTrigger },
}
```

### Strategy Learning

```rust
/// Learns which strategies work via Thompson sampling
pub struct StrategyLearner {
    strategy_priors: HashMap<(LearningCategory, ContextType), StrategyDistribution>,
}

impl StrategyLearner {
    pub fn select_strategy(
        &self,
        learning: &Learning,
        context: &SessionContext,
    ) -> InjectionStrategy {
        let distribution = self.get_distribution(learning, context);

        // Thompson sampling: sample from each posterior, pick highest
        distribution.weights
            .iter()
            .map(|(strategy, param)| (strategy.clone(), param.sample()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(s, _)| s)
            .unwrap_or(InjectionStrategy::Deferred {
                trigger: DeferralTrigger::Explicit
            })
    }

    pub fn update(&mut self, strategy: &InjectionStrategy, outcome: f64) {
        // Bayesian update based on whether injection helped
    }
}
```

---

## Open-World Adaptation

### Novelty Detection

```rust
/// Detects unknown patterns in session data
pub struct NoveltyDetector {
    threshold: AdaptiveParam,
    anomaly_clusters: Vec<AnomalyCluster>,
    known_fingerprints: HashSet<PatternFingerprint>,
}

impl NoveltyDetector {
    pub async fn detect(&mut self, session: &SessionData) -> Vec<Novelty> {
        let mut novelties = Vec::new();

        for event in &session.events {
            let fingerprint = self.compute_fingerprint(event);

            if self.known_fingerprints.contains(&fingerprint) {
                continue;  // Known pattern
            }

            let embedding = self.embed(event).await?;
            let min_distance = self.min_cluster_distance(&embedding);
            let threshold = self.threshold.sample();

            if min_distance > threshold {
                // Novel pattern discovered!
                novelties.push(Novelty { event, embedding, distance: min_distance });
                self.cluster_novelty(novelty);
            }
        }

        novelties
    }

    /// Surface high-frequency clusters for human review
    pub fn get_emergent_patterns(&self, min_examples: usize) -> Vec<&AnomalyCluster> {
        self.anomaly_clusters
            .iter()
            .filter(|c| c.examples.len() >= min_examples && c.named.is_none())
            .collect()
    }
}
```

### Capability Gap Detection

```rust
/// Surfaces what the system cannot learn
pub struct CapabilityGapDetector {
    gaps: Vec<CapabilityGap>,
}

#[derive(Debug, Clone)]
pub struct CapabilityGap {
    pub id: Uuid,
    pub description: String,
    pub frequency: u32,
    pub first_seen: DateTime<Utc>,
    pub example_contexts: Vec<String>,
    pub potential_solutions: Vec<String>,
}
```

---

## Crate Structure

```
vibes-learning/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── plugin.rs              # Plugin trait implementation
│   │
│   ├── model/                 # Core types (Milestone 4.2)
│   │   ├── mod.rs
│   │   ├── learning.rs
│   │   ├── scope.rs
│   │   └── adaptive.rs
│   │
│   ├── storage/               # CozoDB layer (Milestone 4.2)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── cozo.rs
│   │   └── migrations.rs
│   │
│   ├── introspection/         # Harness discovery (Milestone 4.1)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── claude_code.rs
│   │   └── generic.rs
│   │
│   ├── capture/               # Capture adapters (Milestone 4.3)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   └── claude_code.rs
│   │
│   ├── inject/                # Injection adapters (Milestone 4.3)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── claude_code.rs
│   │   └── strategy.rs
│   │
│   ├── analysis/              # Transcript analysis (Milestone 4.4)
│   │   ├── mod.rs
│   │   ├── transcript.rs
│   │   ├── patterns.rs
│   │   └── embedder.rs
│   │
│   └── novelty/               # Open-world (Milestone 4.6)
│       ├── mod.rs
│       ├── detector.rs
│       ├── cluster.rs
│       └── gaps.rs
```

---

## Dependencies

### vibes-learning/Cargo.toml

```toml
[dependencies]
vibes-plugin-api = { path = "../vibes-plugin-api" }

# Storage
cozo = { version = "0.7", features = ["storage-rocksdb"] }

# Identifiers
uuid = { version = "1.0", features = ["v7"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Random (for Thompson sampling)
rand = "0.8"
rand_distr = "0.4"

# Error handling
thiserror = "1.0"
```

---

## Testing Strategy

### Unit Tests

| Component | Coverage |
|-----------|----------|
| AdaptiveParam | Bayesian updates converge correctly |
| CozoDB queries | CRUD operations, semantic search |
| PatternFingerprint | Deterministic for same input |
| StrategyLearner | Thompson sampling selects correctly |

### Integration Tests

| Test | Description |
|------|-------------|
| capture_inject_roundtrip | Capture learning, store, retrieve, inject |
| scope_isolation | Project learnings don't leak to other projects |
| adaptive_convergence | Parameters stabilize with consistent outcomes |
| novelty_clustering | Unknown patterns cluster together |

---

## Assessment Framework

The assessment framework is foundational to the entire learning system. Without measuring outcomes, we cannot know if learnings help or hurt. The framework uses a **tiered approach** that balances signal quality against latency and cost.

### Design Philosophy

Two distinct systems work together:
- **Post-hoc learning loop** (heavy, async): Extract learnings, compute attribution, update parameters
- **Real-time circuit breaker** (lightweight, inline): Detect "going bad" signals, intervene before damage

The circuit breaker must be extremely lightweight to avoid degrading the conversation.

### Assessment Tiers

| Tier | What It Does | Cost | Latency | When | Blocking? |
|------|--------------|------|---------|------|-----------|
| **Lightweight** | Regex, counters, lexicon sentiment | $0 | <10ms | Every message | Yes |
| **Medium** | LLM summarize segment, compute metrics | ~$0.002 | 0.5-2s | Checkpoints | No (async) |
| **Heavy** | Full transcript analysis, learning extraction | ~$0.02-0.22 | 5-10s | Sampled | No (async) |

### Cost Analysis

Using Anthropic pricing (as of 2024):

| Model | Input | Output |
|-------|-------|--------|
| Haiku | $0.25/1M tokens | $1.25/1M tokens |
| Sonnet | $3/1M tokens | $15/1M tokens |

Per-assessment costs:

| Assessment Type | Tokens (in/out) | Haiku | Sonnet |
|-----------------|-----------------|-------|--------|
| Medium | 5K / 1K | $0.002 | $0.03 |
| Heavy | 50K / 5K | $0.02 | $0.22 |

Monthly projections (100 sessions/month, ~20 checkpoints each):

| Strategy | Heavy Assessments | Medium Assessments | Haiku Cost | Sonnet Cost |
|----------|-------------------|-------------------|------------|-------------|
| Aggressive | 100 | 2,000 | ~$6 | ~$82 |
| Random 20% | 20 | 400 | ~$1.20 | ~$16 |
| Importance 10% | 10 | 200 | ~$0.60 | ~$8 |

**Key insight**: Costs are trivial. The constraint is latency, not cost.

### Outcome Signals

#### Token Economics
```rust
pub struct TokenMetrics {
    /// Tokens spent on corrections / total tokens
    pub correction_ratio: f64,
    /// How many times did user ask for same thing?
    pub retry_count: u32,
    /// Did the first response satisfy?
    pub first_attempt_success: bool,
    /// Tokens to completion vs task complexity
    pub token_velocity: f64,
}
```

#### Linguistic Signals

| Pattern | Signal | Weight |
|---------|--------|--------|
| "Why didn't you..." | Missed convention | High negative |
| "No, I meant..." | Misunderstanding | Medium negative |
| "Actually, let's go back" | Backtracking | High negative |
| "Perfect" / "Great" / "Thanks" | Success | High positive |
| User repeats instruction verbatim | Failed to understand | Very high negative |
| "Remember for next time" | Explicit learning opportunity | Capture trigger |

```rust
#[derive(Debug, Clone)]
pub struct LinguisticSignal {
    pub pattern: SignalPattern,
    pub message_index: usize,
    pub confidence: f64,
    pub sentiment_delta: f64,
}

#[derive(Debug, Clone)]
pub enum SignalPattern {
    Positive(PositivePattern),
    Negative(NegativePattern),
    Neutral,
}

#[derive(Debug, Clone)]
pub enum NegativePattern {
    WhyDidntYou,       // "Why didn't you..."
    NoIMeant,          // "No, I meant..."
    Backtracking,      // "Actually, let's go back"
    Repetition,        // User repeats instruction
    Frustration,       // Expletives, caps, "!!!"
}

#[derive(Debug, Clone)]
pub enum PositivePattern {
    Praise,            // "Perfect", "Great", "Thanks"
    Continuation,      // Quick follow-up without correction
    ExplicitSuccess,   // "That worked!"
    RememberThis,      // "Remember for next time"
}
```

#### Behavioral Signals
```rust
pub struct BehavioralMetrics {
    /// User immediately reverts/modifies Claude's edits
    pub edit_rejection_rate: f64,
    /// Same tool failing repeatedly
    pub tool_failure_cascades: u32,
    /// Claude adds features user didn't ask for
    pub scope_creep_detected: bool,
    /// User simplifies Claude's solution
    pub over_engineering_detected: bool,
}
```

#### Code Quality Signals
```rust
pub struct CodeQualityDelta {
    /// New lint warnings after changes
    pub lint_delta: i32,
    /// New type errors after changes
    pub type_error_delta: i32,
    /// Build status changed
    pub build_broken: bool,
    /// Existing tests now fail
    pub test_regression: bool,
}
```

### Checkpoint Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                    CONTINUOUS (every msg)                    │
│  • Lightweight heuristics (regex, counters)                  │
│  • Cost: $0, Latency: <10ms, Blocking: yes                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              CHECKPOINT (async, ~every 10 msgs)              │
│  • Medium assessment: summarize segment, compute metrics    │
│  • Triggered by: message count, task boundary, commit       │
│  • Cost: ~$0.002, Latency: N/A (async), Blocking: no        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 SESSION END (sampled 20%)                    │
│  • Heavy assessment: full analysis, learning extraction     │
│  • Random 20% baseline + boost for high-signal sessions     │
│  • Cost: ~$0.02, Latency: N/A (async), Blocking: no         │
└─────────────────────────────────────────────────────────────┘
```

Checkpoint triggers:

| Checkpoint Type | Detection | Assessment Tier | Blocking? |
|-----------------|-----------|-----------------|-----------|
| Every message | Always | Lightweight | Yes |
| N messages (10) | Counter | Medium | No (async) |
| Task boundary | "done", "next", topic shift | Medium | No (async) |
| Commit | Git hook | Medium | No (async) |
| Build/test pass | Tool success | Medium | No (async) |
| Session pause (>5 min) | Timer | Medium | N/A |
| Session end | Stop hook | Heavy | N/A |

### Sampling Strategy

```rust
pub struct SamplingConfig {
    /// Base probability for heavy assessment
    pub base_rate: f64,  // 0.20 = 20%

    /// Boost to 100% for these conditions
    pub boost_conditions: BoostConditions,
}

pub struct BoostConditions {
    /// Explicit user feedback given
    pub explicit_feedback: bool,
    /// High frustration detected in lightweight assessment
    pub high_frustration: bool,
    /// Session unusually long
    pub long_session_threshold: Duration,
    /// First N sessions (burn-in period)
    pub burn_in_sessions: u32,  // e.g., first 10 sessions
}

impl SamplingConfig {
    pub fn should_run_heavy_assessment(&self, session: &SessionMetadata) -> bool {
        // Always assess during burn-in
        if session.session_number <= self.boost_conditions.burn_in_sessions {
            return true;
        }

        // Boost conditions trigger 100% assessment
        if session.has_explicit_feedback
            || session.frustration_score > FRUSTRATION_THRESHOLD
            || session.duration > self.boost_conditions.long_session_threshold {
            return true;
        }

        // Otherwise random sample
        rand::random::<f64>() < self.base_rate
    }
}
```

### Real-Time Circuit Breaker

Lightweight heuristics that run every message to detect "going bad":

| Signal | Detection | Intervention |
|--------|-----------|--------------|
| Frustration spike | 3+ corrections in N messages | Pause: "Let me make sure I understand..." |
| Tool failure loop | Same tool failing 3+ times | Suggest alternative approach |
| Build/test regression | Was passing, now failing | Offer rollback to last working state |
| Token burn | High tokens, low progress | "Should I step back and reconsider?" |
| Scope explosion | Task complexity increasing | "This is getting complex. Break it down?" |
| Repetition | User repeating same instruction | Full stop, explicit clarification |

```rust
pub struct CircuitBreaker {
    /// Rolling window of recent signals
    window: VecDeque<LightweightSignal>,
    /// Thresholds (adaptive)
    frustration_threshold: AdaptiveParam,
    failure_threshold: AdaptiveParam,
}

impl CircuitBreaker {
    pub fn check(&mut self, signal: LightweightSignal) -> Option<Intervention> {
        self.window.push_back(signal);
        self.trim_window();

        // Check frustration
        let frustration_count = self.count_frustration_signals();
        if frustration_count >= self.frustration_threshold.sample() as u32 {
            return Some(Intervention::Clarify);
        }

        // Check tool failures
        let failure_count = self.count_consecutive_failures();
        if failure_count >= self.failure_threshold.sample() as u32 {
            return Some(Intervention::SuggestAlternative);
        }

        None
    }
}
```

### Assessment Types

```rust
/// Core types for the assessment system
pub mod assessment {
    use super::*;

    /// Result of a lightweight assessment (runs every message)
    #[derive(Debug, Clone)]
    pub struct LightweightAssessment {
        pub message_index: usize,
        pub timestamp: DateTime<Utc>,
        pub signals: Vec<LightweightSignal>,
        pub running_frustration: f64,
        pub running_success: f64,
    }

    /// Result of a medium assessment (runs at checkpoints)
    #[derive(Debug, Clone)]
    pub struct MediumAssessment {
        pub checkpoint_id: Uuid,
        pub message_range: (usize, usize),
        pub timestamp: DateTime<Utc>,
        pub segment_summary: String,
        pub token_metrics: TokenMetrics,
        pub code_quality_delta: Option<CodeQualityDelta>,
        pub segment_score: f64,
    }

    /// Result of a heavy assessment (runs at session end, sampled)
    #[derive(Debug, Clone)]
    pub struct HeavyAssessment {
        pub session_id: SessionId,
        pub timestamp: DateTime<Utc>,
        pub full_analysis: SessionAnalysis,
        pub extracted_learnings: Vec<PotentialLearning>,
        pub attributions: Vec<Attribution>,
        pub session_score: f64,
        pub confidence: f64,
    }

    /// Comprehensive session analysis from heavy assessment
    #[derive(Debug, Clone)]
    pub struct SessionAnalysis {
        pub task_summary: String,
        pub success_indicators: Vec<String>,
        pub failure_indicators: Vec<String>,
        pub correction_patterns: Vec<CorrectionPattern>,
        pub tool_usage_patterns: Vec<ToolUsagePattern>,
        pub user_sentiment_trajectory: Vec<(usize, f64)>,
    }
}
```

---

## Attribution Engine

The attribution engine connects learnings to outcomes, answering: "Which learnings actually helped?"

### The Attribution Problem

```
Session with 30 injected learnings
         │
         ▼
┌────────────────────────────────┐
│  Learning A: "Use Rust"        │──┐
│  Learning B: "Test first"      │──┤
│  Learning C: "Small PRs"       │──┤     ┌─────────────┐
│  Learning D: "Use axum"        │──┼────▶│   Session   │────▶ Score: 0.85
│  ...26 more...                 │──┘     └─────────────┘
└────────────────────────────────┘

Question: Which learnings get credit for the 0.85?
```

### Attribution Methods Comparison

| Method | Pros | Cons | Use Case |
|--------|------|------|----------|
| Simple correlation | Easy | Popularity bias, no causation | Never use alone |
| Temporal attribution | Cheap, distinguishes use vs presence | Still correlation | Layer 1-2 |
| Ablation testing | Measures causal impact | Slow to converge, expensive | Ground truth |
| Shapley values | Theoretically fair | Computationally intractable | Deprecation decisions |
| Attention-based | Cheap, real-time | Similarity ≠ causation | Activation detection |

### Layered Attribution System

```
┌─────────────────────────────────────────────────────────────┐
│  LAYER 1: Activation Detection (every session)              │
│  • Embedding similarity between output and learning         │
│  • Binary: was this learning "used" or ignored?             │
│  • Cost: cheap (just embeddings)                            │
│  • Purpose: Filter - only attribute to activated learnings  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  LAYER 2: Temporal Correlation (every session)              │
│  • Given activation, how close to positive/negative signal? │
│  • Credit learnings whose activation preceded good outcomes │
│  • Cost: cheap (parsing + timestamps)                       │
│  • Purpose: Weighted attribution based on timing            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  LAYER 3: Ablation Testing (sampled, ~5% of sessions)       │
│  • Occasionally withhold specific learnings                 │
│  • Measure actual marginal value over time                  │
│  • Cost: medium (lost value from withholding)               │
│  • Purpose: Ground truth causal impact                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  LAYER 4: Value Aggregation                                  │
│  • Combine all layers into final learning value             │
│  • Weight ablation data heavily when available              │
│  • Fall back to temporal correlation when ablation sparse   │
│  • Purpose: Best estimate of each learning's value          │
└─────────────────────────────────────────────────────────────┘
```

### Layer 1: Activation Detection

Determines whether Claude actually **used** a learning, not just whether it was present.

```rust
/// Detect if a learning was activated in Claude's output
pub struct ActivationDetector {
    embedder: Box<dyn Embedder>,
    activation_threshold: AdaptiveParam,
}

impl ActivationDetector {
    /// Compute activation score for a learning given Claude's output
    pub async fn detect_activation(
        &self,
        output: &str,
        learning: &Learning,
    ) -> ActivationResult {
        // Semantic similarity between output and learning
        let output_embedding = self.embedder.embed(output).await;
        let learning_embedding = self.embedder.embed(&learning.content.text()).await;
        let similarity = cosine_similarity(&output_embedding, &learning_embedding);

        // Also check for explicit references (keywords, phrases)
        let explicit_refs = self.find_explicit_references(output, learning);

        // Combine signals
        let activation_score = self.combine_signals(similarity, explicit_refs);
        let threshold = self.activation_threshold.sample();

        ActivationResult {
            learning_id: learning.id,
            activated: activation_score > threshold,
            confidence: activation_score,
            explicit_references: explicit_refs,
        }
    }

    fn find_explicit_references(&self, output: &str, learning: &Learning) -> Vec<String> {
        learning.keywords()
            .filter(|kw| output.to_lowercase().contains(&kw.to_lowercase()))
            .map(String::from)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ActivationResult {
    pub learning_id: LearningId,
    pub activated: bool,
    pub confidence: f64,
    pub explicit_references: Vec<String>,
}
```

### Layer 2: Temporal Attribution

Given that a learning was activated, how close was it to positive/negative signals?

```rust
/// Temporal attribution based on signal proximity
pub struct TemporalAttributor {
    /// How quickly attribution decays with distance
    decay_rate: AdaptiveParam,
    /// Maximum messages to look ahead for signals
    window_size: usize,
}

impl TemporalAttributor {
    /// Compute temporal attribution for an activation
    pub fn attribute(
        &self,
        activation: &ActivationResult,
        activation_index: usize,
        signals: &[LinguisticSignal],
    ) -> TemporalAttribution {
        let mut positive_proximity = 0.0;
        let mut negative_proximity = 0.0;

        for signal in signals {
            // Only consider signals after activation
            if signal.message_index <= activation_index {
                continue;
            }

            // Decay based on distance
            let distance = signal.message_index - activation_index;
            if distance > self.window_size {
                break;
            }

            let decay = (-self.decay_rate.value * distance as f64).exp();

            match signal.pattern {
                SignalPattern::Positive(_) => positive_proximity += decay * signal.confidence,
                SignalPattern::Negative(_) => negative_proximity += decay * signal.confidence,
                SignalPattern::Neutral => {},
            }
        }

        TemporalAttribution {
            learning_id: activation.learning_id,
            positive_proximity,
            negative_proximity,
            net_attribution: positive_proximity - negative_proximity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemporalAttribution {
    pub learning_id: LearningId,
    pub positive_proximity: f64,
    pub negative_proximity: f64,
    pub net_attribution: f64,
}
```

### Layer 3: Ablation Testing

Occasionally withhold specific learnings to measure causal impact.

```rust
/// Manages ablation experiments for learnings
pub struct AblationManager {
    /// Active experiments by learning ID
    experiments: HashMap<LearningId, AblationExperiment>,
    /// Probability of running ablation for any learning
    ablation_rate: AdaptiveParam,
    /// Minimum samples needed for significance
    min_samples: u32,
}

#[derive(Debug, Clone)]
pub struct AblationExperiment {
    pub learning_id: LearningId,
    pub started_at: DateTime<Utc>,
    /// Sessions where learning was present
    pub sessions_with: Vec<SessionOutcome>,
    /// Sessions where learning was deliberately withheld
    pub sessions_without: Vec<SessionOutcome>,
}

#[derive(Debug, Clone)]
pub struct SessionOutcome {
    pub session_id: SessionId,
    pub score: f64,
    pub timestamp: DateTime<Utc>,
}

impl AblationExperiment {
    /// Compute marginal value of this learning
    pub fn marginal_value(&self) -> Option<MarginalValue> {
        if self.sessions_without.len() < 10 {
            return None; // Not enough data
        }

        let mean_with: f64 = self.sessions_with.iter()
            .map(|s| s.score)
            .sum::<f64>() / self.sessions_with.len() as f64;

        let mean_without: f64 = self.sessions_without.iter()
            .map(|s| s.score)
            .sum::<f64>() / self.sessions_without.len() as f64;

        let marginal = mean_with - mean_without;
        let p_value = self.welch_t_test();

        Some(MarginalValue {
            learning_id: self.learning_id,
            value: marginal,
            confidence: 1.0 - p_value,
            sample_size_with: self.sessions_with.len(),
            sample_size_without: self.sessions_without.len(),
            is_significant: p_value < 0.05,
        })
    }

    fn welch_t_test(&self) -> f64 {
        // Welch's t-test for unequal variances
        // Returns p-value
        todo!("implement Welch's t-test")
    }
}

#[derive(Debug, Clone)]
pub struct MarginalValue {
    pub learning_id: LearningId,
    pub value: f64,
    pub confidence: f64,
    pub sample_size_with: usize,
    pub sample_size_without: usize,
    pub is_significant: bool,
}

impl AblationManager {
    /// Decide whether to withhold a learning for this session
    pub fn should_ablate(&self, learning_id: LearningId) -> bool {
        let experiment = self.experiments.get(&learning_id);

        // If we have enough data and it's significant, reduce ablation rate
        if let Some(exp) = experiment {
            if let Some(mv) = exp.marginal_value() {
                if mv.is_significant {
                    // Still ablate occasionally to detect drift
                    return rand::random::<f64>() < 0.01;
                }
            }
        }

        // Otherwise, ablate at standard rate
        rand::random::<f64>() < self.ablation_rate.sample()
    }
}
```

### Layer 4: Value Aggregation

Combine all attribution layers into a final learning value estimate.

```rust
/// Aggregates attribution data into learning value
pub struct ValueAggregator {
    /// Weight for temporal attribution when ablation unavailable
    temporal_weight: f64,
    /// Weight for activation rate
    activation_weight: f64,
}

impl ValueAggregator {
    /// Compute final value estimate for a learning
    pub fn compute_value(&self, learning_id: LearningId, data: &AttributionData) -> LearningValue {
        // Ablation is ground truth when available
        if let Some(marginal) = &data.ablation_marginal {
            if marginal.is_significant {
                return LearningValue {
                    learning_id,
                    estimated_value: marginal.value,
                    confidence: marginal.confidence,
                    source: ValueSource::Ablation,
                    activation_rate: data.activation_rate,
                    details: data.clone(),
                };
            }
        }

        // Fall back to temporal attribution
        let temporal_value = data.temporal_attributions.iter()
            .map(|t| t.net_attribution)
            .sum::<f64>() / data.temporal_attributions.len().max(1) as f64;

        // Weight by activation rate (if never activated, value is 0)
        let weighted_value = temporal_value * data.activation_rate;

        LearningValue {
            learning_id,
            estimated_value: weighted_value,
            confidence: 0.5, // Lower confidence than ablation
            source: ValueSource::Temporal,
            activation_rate: data.activation_rate,
            details: data.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LearningValue {
    pub learning_id: LearningId,
    pub estimated_value: f64,
    pub confidence: f64,
    pub source: ValueSource,
    pub activation_rate: f64,
    pub details: AttributionData,
}

#[derive(Debug, Clone)]
pub enum ValueSource {
    Ablation,
    Temporal,
    Prior, // No data yet
}

#[derive(Debug, Clone)]
pub struct AttributionData {
    pub learning_id: LearningId,
    pub activation_rate: f64,
    pub temporal_attributions: Vec<TemporalAttribution>,
    pub ablation_marginal: Option<MarginalValue>,
    pub session_count: u32,
}
```

### Per-Session Attribution Record

```rust
/// Complete attribution record for a learning in a single session
#[derive(Debug, Clone)]
pub struct Attribution {
    pub learning_id: LearningId,
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,

    // Layer 1: Activation
    pub was_activated: bool,
    pub activation_confidence: f64,
    pub activation_signals: Vec<String>,

    // Layer 2: Temporal
    pub temporal_proximity_to_positive: f64,
    pub temporal_proximity_to_negative: f64,
    pub net_temporal_attribution: f64,

    // Layer 3: Ablation
    pub was_withheld: bool,
    pub session_outcome: f64,

    // Combined
    pub attributed_value: f64,
}
```

### Edge Cases

| Scenario | Solution |
|----------|----------|
| **Cold start** (new learning) | Wide prior, high exploration via Thompson sampling |
| **Negative learning** (hurts) | Ablation shows negative marginal value → deprecate |
| **Correlated learnings** | If A and B always co-injected, ablate one to separate |
| **Delayed effects** | Extend temporal window for architectural learnings |
| **Context-dependent** | Scope-specific values (learning helps in project A, hurts in B) |

### Negative Learning Detection

Critical: detect learnings that **hurt** performance.

```rust
impl LearningValue {
    /// Check if this learning should be deprecated
    pub fn should_deprecate(&self) -> bool {
        // Strong evidence it hurts
        if self.source == ValueSource::Ablation
            && self.estimated_value < -0.1
            && self.confidence > 0.8 {
            return true;
        }

        // Temporal evidence of harm
        if self.source == ValueSource::Temporal
            && self.estimated_value < -0.2
            && self.details.session_count > 20 {
            return true;
        }

        false
    }

    /// Check if learning should be flagged for human review
    pub fn needs_review(&self) -> bool {
        // Inconsistent results
        if self.confidence < 0.3 && self.details.session_count > 30 {
            return true;
        }

        // Borderline negative
        if self.estimated_value < 0.0 && self.estimated_value > -0.1 {
            return true;
        }

        false
    }
}
```

---

## Observability Dashboard

The dashboard surfaces learning and assessment data for power users, while the system remains invisible by default.

### UX Philosophy

- **Default**: Magic happens invisibly, sessions just feel better over time
- **Power users**: Dashboard shows what's being learned, assessment trends, attribution
- **Optional**: Toggle for real-time indicator (🧠 learning...)

### Dashboard Sections

| Section | Content | User Value |
|---------|---------|------------|
| **Session Scores** | Trend line of session quality over time | "Am I actually improving?" |
| **Learnings** | List with confidence, scope, usage stats | "What has it figured out?" |
| **Metrics Breakdown** | Correction ratio, sentiment, first-attempt success | "Why that score?" |
| **Attribution** | Which learnings contributed to outcomes | "What's actually helping?" |
| **System Health** | Adaptive parameters, novelty detections, capability gaps | "Is the system working?" |

### Dashboard Data Model

```rust
/// Data for the observability dashboard
pub mod dashboard {
    use super::*;

    /// Overview of system learning progress
    #[derive(Debug, Clone, Serialize)]
    pub struct LearningOverview {
        pub total_learnings: u32,
        pub learnings_by_scope: HashMap<Scope, u32>,
        pub learnings_by_category: HashMap<LearningCategory, u32>,
        pub recent_learnings: Vec<LearningSummary>,
        pub top_performing: Vec<LearningSummary>,
        pub needs_review: Vec<LearningSummary>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct LearningSummary {
        pub id: LearningId,
        pub description: String,
        pub scope: Scope,
        pub category: LearningCategory,
        pub estimated_value: f64,
        pub confidence: f64,
        pub times_injected: u32,
        pub activation_rate: f64,
        pub created_at: DateTime<Utc>,
    }

    /// Session quality trends
    #[derive(Debug, Clone, Serialize)]
    pub struct SessionTrends {
        pub sessions: Vec<SessionDataPoint>,
        pub moving_average: Vec<f64>,
        pub trend_direction: TrendDirection,
        pub improvement_since_start: f64,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct SessionDataPoint {
        pub session_id: SessionId,
        pub timestamp: DateTime<Utc>,
        pub score: f64,
        pub correction_ratio: f64,
        pub first_attempt_success_rate: f64,
    }

    #[derive(Debug, Clone, Serialize)]
    pub enum TrendDirection {
        Improving,
        Stable,
        Declining,
    }

    /// Attribution insights
    #[derive(Debug, Clone, Serialize)]
    pub struct AttributionInsights {
        pub top_contributors: Vec<LearningContribution>,
        pub negative_learnings: Vec<LearningContribution>,
        pub inconclusive: Vec<LearningId>,
        pub ablation_coverage: f64, // % of learnings with significant ablation data
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct LearningContribution {
        pub learning: LearningSummary,
        pub marginal_value: f64,
        pub confidence: f64,
        pub source: ValueSource,
    }

    /// System health metrics
    #[derive(Debug, Clone, Serialize)]
    pub struct SystemHealth {
        pub assessment_coverage: f64,  // % of sessions assessed
        pub ablation_rate: f64,
        pub novelty_detections: u32,
        pub capability_gaps: Vec<CapabilityGapSummary>,
        pub adaptive_params: Vec<AdaptiveParamSummary>,
    }
}
```

### Real-Time Indicator

Optional UI element users can toggle:

```rust
/// State for the real-time learning indicator
#[derive(Debug, Clone, Serialize)]
pub struct LearningIndicator {
    pub visible: bool,
    pub state: IndicatorState,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub enum IndicatorState {
    /// Normal operation, learning in background
    Learning,
    /// Just captured a new learning
    Captured { learning_preview: String },
    /// Just injected learnings
    Injected { count: u32 },
    /// Assessment running
    Assessing,
    /// Idle
    Idle,
}
```

---

## Updated Crate Structure

```
vibes-learning/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── plugin.rs              # Plugin trait implementation
│   │
│   ├── model/                 # Core types (Milestone 4.2)
│   │   ├── mod.rs
│   │   ├── learning.rs
│   │   ├── scope.rs
│   │   └── adaptive.rs
│   │
│   ├── storage/               # CozoDB layer (Milestone 4.2)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── cozo.rs
│   │   └── migrations.rs
│   │
│   ├── introspection/         # Harness discovery (Milestone 4.1)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── claude_code.rs
│   │   └── generic.rs
│   │
│   ├── capture/               # Capture adapters (Milestone 4.3)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   └── claude_code.rs
│   │
│   ├── inject/                # Injection adapters (Milestone 4.3)
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── claude_code.rs
│   │   └── strategy.rs
│   │
│   ├── assessment/            # Assessment framework (Milestone 4.4) ⭐ NEW
│   │   ├── mod.rs
│   │   ├── lightweight.rs     # Every-message heuristics
│   │   ├── medium.rs          # Checkpoint assessments
│   │   ├── heavy.rs           # Full session analysis
│   │   ├── signals.rs         # Signal types and detection
│   │   ├── circuit_breaker.rs # Real-time intervention
│   │   ├── sampling.rs        # Sampling strategy
│   │   └── metrics.rs         # Metric computation
│   │
│   ├── analysis/              # Transcript analysis (Milestone 4.5)
│   │   ├── mod.rs
│   │   ├── transcript.rs
│   │   ├── patterns.rs
│   │   └── embedder.rs
│   │
│   ├── attribution/           # Attribution engine (Milestone 4.6) ⭐ NEW
│   │   ├── mod.rs
│   │   ├── activation.rs      # Layer 1: Activation detection
│   │   ├── temporal.rs        # Layer 2: Temporal correlation
│   │   ├── ablation.rs        # Layer 3: Ablation testing
│   │   ├── aggregation.rs     # Layer 4: Value aggregation
│   │   └── negative.rs        # Negative learning detection
│   │
│   ├── strategy/              # Adaptive strategies (Milestone 4.7)
│   │   ├── mod.rs
│   │   ├── learner.rs
│   │   └── thompson.rs
│   │
│   ├── dashboard/             # Observability (Milestone 4.8) ⭐ NEW
│   │   ├── mod.rs
│   │   ├── data.rs            # Dashboard data models
│   │   ├── api.rs             # API endpoints
│   │   └── indicator.rs       # Real-time indicator
│   │
│   └── novelty/               # Open-world (Milestone 4.9)
│       ├── mod.rs
│       ├── detector.rs
│       ├── cluster.rs
│       └── gaps.rs
```

---

## Updated Milestones

### 4.1 Harness Introspection
- [ ] `Harness` trait and `HarnessCapabilities`
- [ ] `ClaudeCodeHarness` implementation
- [ ] `GenericHarnessDiscovery` for unknown harnesses
- [ ] Capability caching in storage

### 4.2 Storage Foundation
- [ ] CozoDB setup with schema
- [ ] `Learning` model with UUIDv7
- [ ] `LearningStorage` trait and implementation
- [ ] `AdaptiveParam` with Bayesian updates
- [ ] `AdaptiveConfig` for system-wide parameters

### 4.3 Capture & Inject (MVP)
- [ ] `CaptureAdapter` trait
- [ ] `ClaudeCodeHooksCapture` implementation
- [ ] `InjectionAdapter` trait
- [ ] `ClaudeCodeInjector` via CLAUDE.md
- [ ] Session context preparation

### 4.4 Assessment Framework ⭐ NEW
- [ ] Lightweight assessment (every message)
  - [ ] Frustration detector (regex patterns)
  - [ ] Correction counter
  - [ ] Tool failure tracker
  - [ ] Lexicon-based sentiment
- [ ] Medium assessment (checkpoints)
  - [ ] Checkpoint detection (N messages, task boundary, commit)
  - [ ] Async assessment runner (non-blocking)
  - [ ] Segment summarizer (LLM-based)
  - [ ] Token ratio computation
  - [ ] Code quality delta (lint, types)
- [ ] Heavy assessment (session end, sampled)
  - [ ] Full transcript analyzer
  - [ ] Session scoring model
  - [ ] Sampling strategy (20% + signal boost)
  - [ ] Assessment result storage
- [ ] Circuit breaker
  - [ ] Real-time intervention triggers
  - [ ] Intervention types
- [ ] Metrics & baselines
  - [ ] Metric definitions
  - [ ] Baseline establishment (burn-in)
  - [ ] Trend computation

### 4.5 Learning Extraction
- [ ] Transcript parser for Claude JSONL
- [ ] Error recovery pattern extraction
- [ ] User correction detection
- [ ] `Embedder` trait with implementations
- [ ] Semantic search via HNSW

### 4.6 Attribution Engine ⭐ NEW
- [ ] Layer 1: Activation detection
  - [ ] Embedding similarity computation
  - [ ] Explicit reference detection
  - [ ] Activation threshold (adaptive)
- [ ] Layer 2: Temporal correlation
  - [ ] Signal proximity calculation
  - [ ] Decay rate (adaptive)
  - [ ] Net attribution computation
- [ ] Layer 3: Ablation testing
  - [ ] Ablation manager
  - [ ] Experiment tracking
  - [ ] Marginal value computation
  - [ ] Welch's t-test for significance
- [ ] Layer 4: Value aggregation
  - [ ] Multi-source value estimation
  - [ ] Confidence computation
  - [ ] Negative learning detection
- [ ] Attribution storage schema

### 4.7 Adaptive Strategies
- [ ] `InjectionStrategy` enum
- [ ] `StrategyLearner` with Thompson sampling
- [ ] Subagent injection support
- [ ] Outcome-based parameter updates

### 4.8 Observability Dashboard ⭐ NEW
- [ ] Dashboard data models
- [ ] API endpoints for dashboard
- [ ] Session trends visualization
- [ ] Learning list with filtering
- [ ] Attribution insights view
- [ ] System health metrics
- [ ] Real-time indicator (toggle)
- [ ] Settings for visibility

### 4.9 Open-World Adaptation
- [ ] `NoveltyDetector` for unknown patterns
- [ ] `AnomalyCluster` for grouping unknowns
- [ ] `CapabilityGapDetector`
- [ ] Emergent pattern surfacing
- [ ] Meta-learning metrics

---

## Updated CozoDB Schema

```datalog
# Core entities
:create learning {
    id: String =>
    scope_type: String,
    scope_value: String?,
    category: String,
    description: String,
    pattern_json: String,
    insight: String,
    confidence: Float,
    created_at: Int,
    updated_at: Int,
    source_type: String,
    source_json: String
}

# Usage statistics
:create usage_stats {
    learning_id: String =>
    times_injected: Int,
    times_helpful: Int,
    times_ignored: Int,
    times_contradicted: Int,
    last_used: Int?
}

# Vector embeddings for semantic search
:create learning_embeddings {
    learning_id: String =>
    embedding: <F32; 1536>
}

# Adaptive parameters
:create adaptive_params {
    param_name: String =>
    value: Float,
    uncertainty: Float,
    observations: Int,
    prior_alpha: Float,
    prior_beta: Float
}

# Learning relationships (graph)
:create learning_relations {
    from_id: String,
    relation_type: String,
    to_id: String =>
    weight: Float,
    created_at: Int
}

# ⭐ NEW: Assessment results
:create assessment_lightweight {
    session_id: String,
    message_index: Int =>
    timestamp: Int,
    signals_json: String,
    running_frustration: Float,
    running_success: Float
}

:create assessment_medium {
    checkpoint_id: String =>
    session_id: String,
    message_range_start: Int,
    message_range_end: Int,
    timestamp: Int,
    segment_summary: String,
    token_metrics_json: String,
    code_quality_delta_json: String?,
    segment_score: Float
}

:create assessment_heavy {
    session_id: String =>
    timestamp: Int,
    analysis_json: String,
    extracted_learnings_json: String,
    session_score: Float,
    confidence: Float
}

# ⭐ NEW: Attribution records
:create attribution {
    learning_id: String,
    session_id: String =>
    timestamp: Int,
    was_activated: Bool,
    activation_confidence: Float,
    activation_signals_json: String,
    temporal_positive: Float,
    temporal_negative: Float,
    net_temporal: Float,
    was_withheld: Bool,
    session_outcome: Float,
    attributed_value: Float
}

# ⭐ NEW: Ablation experiments
:create ablation_experiment {
    learning_id: String =>
    started_at: Int,
    sessions_with_json: String,
    sessions_without_json: String,
    marginal_value: Float?,
    confidence: Float?,
    is_significant: Bool?
}

# ⭐ NEW: Learning values (aggregated)
:create learning_value {
    learning_id: String =>
    estimated_value: Float,
    confidence: Float,
    source: String,
    activation_rate: Float,
    session_count: Int,
    updated_at: Int
}

# Indexes
::hnsw create learning_embeddings:semantic_idx {
    dim: 1536,
    m: 16,
    ef_construction: 200,
    fields: [embedding]
}

::index create learning:by_scope { scope_type, scope_value }
::index create learning:by_category { category }
::index create assessment_lightweight:by_session { session_id }
::index create assessment_medium:by_session { session_id }
::index create attribution:by_learning { learning_id }
::index create attribution:by_session { session_id }
::index create learning_value:by_value { estimated_value }
```

---

## Open Questions

### Resolved
1. ~~**Assessment timing**~~ → Tiered: lightweight (every msg), medium (checkpoints), heavy (sampled)
2. ~~**Sampling strategy**~~ → 20% random + boost for high-signal sessions
3. ~~**Dashboard visibility**~~ → Invisible default, dashboard for power users, optional indicator
4. ~~**Attribution method**~~ → Layered: activation → temporal → ablation → aggregation

### Remaining
1. **Embedding model** - Local (fast, private) vs API (better quality)?
2. **Initial priors** - How to bootstrap adaptive parameters? (Leaning: uninformed + burn-in)
3. **Privacy** - Should learnings ever leave the local machine?
4. **Transcript access** - Parse Claude's JSONL or request structured export?
