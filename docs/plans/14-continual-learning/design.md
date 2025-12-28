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
| 2a | Learning Extraction | Rich pattern extraction from transcripts | 4.4 |
| 2b | Adaptive Strategies | Learn HOW to inject, not just WHAT | 4.5 |
| 3 | Open-World Adaptation | Detect unknown unknowns, meta-learning | 4.6 |

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

## Milestones

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

### 4.4 Learning Extraction
- [ ] Transcript parser for Claude JSONL
- [ ] Error recovery pattern extraction
- [ ] User correction detection
- [ ] `Embedder` trait with implementations
- [ ] Semantic search via HNSW

### 4.5 Adaptive Strategies
- [ ] `InjectionStrategy` enum
- [ ] `StrategyLearner` with Thompson sampling
- [ ] Subagent injection support
- [ ] Outcome-based parameter updates

### 4.6 Open-World Adaptation
- [ ] `NoveltyDetector` for unknown patterns
- [ ] `AnomalyCluster` for grouping unknowns
- [ ] `CapabilityGapDetector`
- [ ] Emergent pattern surfacing
- [ ] Meta-learning metrics

---

## Open Questions

1. **Embedding model** - Local (fast, private) vs API (better quality)?
2. **Initial priors** - How to bootstrap adaptive parameters?
3. **Privacy** - Should learnings ever leave the local machine?
4. **Transcript access** - Parse Claude's JSONL or request structured export?
