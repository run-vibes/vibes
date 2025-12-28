# Continual Learning Plugin Design

**Date:** 2025-12-28
**Status:** Design Complete
**Milestone:** 4.x (Future Phase)

---

## Section 1: Overview & Goals

### Vision

Transform vibes from a Claude Code proxy into a **learning harness** that makes any AI coding assistant progressively more effective through accumulated experience.

### Primary Goals

1. **Claude Learns** - The AI assistant becomes more effective over time through captured learnings
2. **Zero Friction** - Automatic outcome-based learning with no user intervention required
3. **Hierarchical Scope** - Learnings organized as Global → User → Project
4. **Harness Agnostic** - Works across any coding assistant, not just Claude Code
5. **Fully Adaptive** - No hardcoded thresholds; system learns its own parameters
6. **Open-World Ready** - Can detect and adapt to capabilities that don't exist yet

### Non-Goals (Phase 1)

- User learns (educational features)
- System learns about users (personalization beyond preferences)
- Multi-user knowledge sharing

### Success Criteria

The system becomes progressively more autonomous. A concrete benchmark: vibes could dynamically create adapters for any harness and any version of a harness without human intervention.

---

## Section 2: Learning Data Model

### Core Types

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// UUIDv7 provides time-ordered unique identifiers
pub type LearningId = Uuid;

/// Hierarchical scope for learning isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Applies across all users and projects
    Global,
    /// Applies to a specific user across all their projects
    User(String),
    /// Applies to a specific project directory
    Project(String),
}

/// What was learned
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
    /// Coding patterns, idioms, conventions
    CodePattern,
    /// User preferences (formatting, naming, style)
    Preference,
    /// Problem-solution pairs
    Solution,
    /// Error patterns and recovery strategies
    ErrorRecovery,
    /// Tool/capability usage patterns
    ToolUsage,
    /// Harness-specific knowledge
    HarnessKnowledge,
}

#[derive(Debug, Clone)]
pub struct LearningContent {
    /// Human-readable description
    pub description: String,
    /// Structured representation for matching
    pub pattern: Pattern,
    /// Context required for this learning to apply
    pub context_requirements: Vec<ContextRequirement>,
    /// The actual insight/knowledge
    pub insight: String,
    /// Embedding vector for semantic search
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub enum LearningSource {
    /// Inferred from session outcomes
    SessionOutcome {
        session_id: String,
        outcome: Outcome,
    },
    /// Extracted from transcript analysis
    TranscriptAnalysis {
        transcript_path: String,
        extraction_method: String,
    },
    /// Derived from harness introspection
    HarnessIntrospection {
        harness_type: String,
        discovery_method: String,
    },
    /// Meta-learned from other learnings
    MetaLearning {
        source_learning_ids: Vec<LearningId>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    pub times_injected: u64,
    pub times_helpful: u64,
    pub times_ignored: u64,
    pub times_contradicted: u64,
    pub last_used: Option<DateTime<Utc>>,
}
```

### Adaptive Parameters

**Critical Design Principle:** No hardcoded coefficients. All thresholds and weights are learned.

```rust
/// An adaptive parameter that learns from outcomes
#[derive(Debug, Clone)]
pub struct AdaptiveParam {
    /// Current value
    pub value: f64,
    /// Uncertainty (decreases with more observations)
    pub uncertainty: f64,
    /// Number of observations
    pub observations: u64,
    /// Running statistics for Bayesian updates
    pub prior_alpha: f64,
    pub prior_beta: f64,
}

impl AdaptiveParam {
    pub fn new_uninformed() -> Self {
        Self {
            value: 0.5,           // Start neutral
            uncertainty: 1.0,     // Maximum uncertainty
            observations: 0,
            prior_alpha: 1.0,     // Uniform prior
            prior_beta: 1.0,
        }
    }

    /// Bayesian update based on outcome
    pub fn update(&mut self, outcome: f64, weight: f64) {
        self.observations += 1;

        // Bayesian update (simplified Beta-Bernoulli)
        let effective_weight = weight / (1.0 + self.uncertainty);
        self.prior_alpha += outcome * effective_weight;
        self.prior_beta += (1.0 - outcome) * effective_weight;

        // Update value as posterior mean
        self.value = self.prior_alpha / (self.prior_alpha + self.prior_beta);

        // Uncertainty decreases with observations
        self.uncertainty = 1.0 / (1.0 + (self.observations as f64).sqrt());
    }

    /// Sample from posterior for exploration
    pub fn sample(&self) -> f64 {
        use rand_distr::{Beta, Distribution};
        let beta = Beta::new(self.prior_alpha, self.prior_beta).unwrap();
        beta.sample(&mut rand::thread_rng())
    }
}

/// System-wide adaptive parameters
pub struct AdaptiveConfig {
    /// Minimum confidence for injection
    pub injection_threshold: AdaptiveParam,
    /// Balance between exploitation and exploration
    pub exploration_rate: AdaptiveParam,
    /// How quickly old learnings decay
    pub decay_rate: AdaptiveParam,
    /// Novelty detection sensitivity
    pub novelty_threshold: AdaptiveParam,
    /// Per-category parameters
    pub category_weights: HashMap<LearningCategory, AdaptiveParam>,
    /// Per-scope parameters
    pub scope_weights: HashMap<Scope, AdaptiveParam>,
}
```

---

## Section 3: Capture Mechanism

### Philosophy

Learnings are captured from **outcomes**, not explicit user annotation. The user runs their harness normally; vibes observes and learns.

### Capture Sources for Claude Code

```rust
/// Signals captured from Claude Code hooks
pub struct ClaudeCodeCapture {
    /// Tools used and their outcomes
    pub tool_events: Vec<ToolEvent>,
    /// Session termination reason
    pub stop_reason: Option<String>,
    /// Path to full transcript (Claude still writes these!)
    pub transcript_path: Option<String>,
}

pub struct ToolEvent {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}
```

### Outcome Signals

```rust
/// Observable outcome signals
pub enum OutcomeSignal {
    /// Session completed successfully (user exit)
    SuccessfulCompletion,
    /// Error occurred
    ErrorOccurred(String),
    /// User approved a tool use
    ToolApproved { tool: String },
    /// User denied a tool use
    ToolDenied { tool: String },
    /// Same mistake repeated
    RepeatedError { pattern: String },
    /// User manually corrected output
    UserCorrection { before: String, after: String },
    /// Task was abandoned
    TaskAbandoned,
    /// Rapid task completion (efficiency signal)
    RapidCompletion { duration: Duration },
}

/// Inferred session outcome
pub struct SessionOutcome {
    pub session_id: String,
    pub signals: Vec<OutcomeSignal>,
    pub inferred_success: f64,  // 0.0 to 1.0
    pub learnable_patterns: Vec<LearnablePattern>,
}
```

### Important Discovery: Transcripts Still Available

The Stop hook provides `transcript_path` - Claude Code still writes full transcripts. Vibes removed its own SQLite history storage, but we can still analyze Claude's transcripts for rich learning extraction.

---

## Section 4: Storage Layer (CozoDB)

### Why CozoDB

After deep analysis of storage options, CozoDB was selected as the primary storage:

| Requirement | CozoDB Capability |
|-------------|-------------------|
| Relational queries | Native Datalog with SQL-like syntax |
| Graph traversal | First-class recursive queries |
| Vector search | Built-in HNSW index |
| Embedded | Single-file, no external process |
| License | MPL-2.0 (permissive, compatible) |
| Rust-native | 97% Rust, excellent integration |

### Schema

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
    created_at: Int,  # Unix timestamp
    updated_at: Int,
    source_type: String,
    source_json: String
}

# Usage statistics (separate for efficient updates)
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
    embedding: <F32; 1536>  # OpenAI embedding dimension
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
    relation_type: String,  # "derived_from", "supersedes", "conflicts_with"
    to_id: String =>
    weight: Float,
    created_at: Int
}

# Harness capabilities (introspection results)
:create harness_capabilities {
    harness_type: String,
    harness_version: String? =>
    capabilities_json: String,
    discovered_at: Int
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
::index create learning:by_confidence { confidence }
```

### Query Examples

```datalog
# Find relevant learnings for a context
?[id, description, confidence] :=
    *learning{id, scope_type, scope_value, description, confidence},
    scope_type = 'project',
    scope_value = '/path/to/project',
    confidence > 0.6  # This threshold is adaptive!

# Graph traversal: find all learnings derived from a source
?[derived_id, description] :=
    *learning_relations{from_id: source, relation_type: 'derived_from', to_id: derived_id},
    *learning{id: derived_id, description},
    source = 'learning-uuid-here'

# Semantic search with vector similarity
?[id, description, distance] :=
    ~learning_embeddings:semantic_idx{
        embedding: $query_vector,
        k: 10,
        ef: 50,
        bind_distance: distance,
        learning_id: id
    },
    *learning{id, description}
```

---

## Section 5: Storage Architecture

### Migration Paths

While starting with CozoDB-only, we document clear migration paths:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Storage Architecture                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Phase 1: CozoDB Only                                               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                        CozoDB                                │   │
│  │  • Relational (Datalog)                                     │   │
│  │  • Graph (recursive queries)                                │   │
│  │  • Vector (HNSW index)                                      │   │
│  │  Single file: ~/.vibes/learning.cozo                        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  Migration Trigger Criteria:                                        │
│  • If vector queries > 10ms p95 → Consider LanceDB                 │
│  • If relational queries complex → Consider SQLite                 │
│  • If graph depth > 10 hops slow → Consider dedicated graph DB     │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Phase 2: Hybrid (if needed)                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │   SQLite     │  │   LanceDB    │  │   CozoDB     │              │
│  │  Relational  │  │   Vectors    │  │    Graph     │              │
│  │  Audit log   │  │  Embeddings  │  │  Relations   │              │
│  │  Config      │  │  Similarity  │  │  Inference   │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
│         ↑                 ↑                 ↑                       │
│         └─────────────────┼─────────────────┘                       │
│                           │                                         │
│              ┌────────────────────────┐                            │
│              │   Unified Query Layer   │                            │
│              │   StorageAdapter trait  │                            │
│              └────────────────────────┘                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Storage Abstraction

```rust
/// Storage adapter trait for future migration flexibility
#[async_trait]
pub trait LearningStorage: Send + Sync {
    async fn store_learning(&self, learning: &Learning) -> Result<LearningId>;
    async fn get_learning(&self, id: LearningId) -> Result<Option<Learning>>;
    async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>>;
    async fn semantic_search(&self, query: &[f32], limit: usize) -> Result<Vec<Learning>>;
    async fn update_usage(&self, id: LearningId, stats: &UsageStats) -> Result<()>;
    async fn get_related(&self, id: LearningId, relation: &str) -> Result<Vec<Learning>>;
}

/// CozoDB implementation
pub struct CozoStorage {
    db: cozo::DbInstance,
}

impl CozoStorage {
    pub fn new(path: &Path) -> Result<Self> {
        let db = cozo::DbInstance::new("rocksdb", path.to_str().unwrap(), "")?;
        // Initialize schema on first run
        Self::ensure_schema(&db)?;
        Ok(Self { db })
    }
}
```

---

## Section 6: Injection Mechanism

### Injection via Hooks

For Claude Code, we use the existing hook infrastructure:

```rust
/// Hook-based injection adapter for Claude Code
pub struct ClaudeCodeInjector {
    storage: Arc<dyn LearningStorage>,
    config: Arc<AdaptiveConfig>,
}

impl ClaudeCodeInjector {
    /// Called before session starts to prepare context
    pub async fn prepare_session_context(
        &self,
        project_path: &Path,
        user_id: &str,
    ) -> Result<SessionContext> {
        // Gather relevant learnings from all applicable scopes
        let scopes = vec![
            Scope::Global,
            Scope::User(user_id.to_string()),
            Scope::Project(project_path.to_string_lossy().to_string()),
        ];

        let mut learnings = Vec::new();
        for scope in &scopes {
            let scope_learnings = self.storage.find_by_scope(scope).await?;
            learnings.extend(scope_learnings);
        }

        // Filter by adaptive confidence threshold
        let threshold = self.config.injection_threshold.sample();
        let filtered: Vec<_> = learnings
            .into_iter()
            .filter(|l| l.confidence > threshold)
            .collect();

        // Rank and select top learnings
        let selected = self.rank_and_select(filtered).await?;

        Ok(SessionContext { learnings: selected })
    }
}
```

### Injection Points

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Code Session                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Pre-Session (CLAUDE.md injection)                       │
│     └─ High-confidence, always-relevant learnings           │
│                                                              │
│  2. Pre-Tool-Use Hook                                        │
│     └─ Tool-specific learnings (e.g., "Write file X...")    │
│                                                              │
│  3. On-Demand (via MCP tool if available)                   │
│     └─ "recall_learning" tool for explicit queries          │
│                                                              │
│  4. Post-Tool-Use Hook                                       │
│     └─ Capture outcomes, don't inject                       │
│                                                              │
│  5. Stop Hook                                                │
│     └─ Final learning extraction from transcript            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Section 7: Agentic Injection Strategy

### The Challenge

Not all learnings belong in the main context. Some should be:
- Injected directly into main Claude context
- Delegated to a background subagent for processing
- Deferred until explicitly relevant
- Handled by a specialized verification subagent

### Injection Strategy Taxonomy

```rust
/// How a learning should be injected
#[derive(Debug, Clone)]
pub enum InjectionStrategy {
    /// Inject directly into main context (highest priority learnings)
    MainContext {
        /// Where in context: system prompt, pre-message, inline
        position: ContextPosition,
    },

    /// Delegate to a subagent for processing
    Subagent {
        /// Type of subagent to spawn
        agent_type: SubagentType,
        /// Whether to block main execution
        blocking: bool,
    },

    /// Run in background, surface results asynchronously
    BackgroundSubagent {
        agent_type: SubagentType,
        /// How to surface results when ready
        callback: CallbackMethod,
    },

    /// Don't inject now, but remember for later
    Deferred {
        trigger: DeferralTrigger,
    },
}

#[derive(Debug, Clone)]
pub enum SubagentType {
    /// Verify learning still applies to current context
    Verifier,
    /// Synthesize multiple learnings into coherent guidance
    Synthesizer,
    /// Research whether learning is outdated
    Researcher,
    /// Custom agent type discovered through meta-learning
    Custom(String),
}
```

### Fully Adaptive Strategy Selection

**No hardcoded rules.** The system learns which strategies work:

```rust
/// Learns optimal injection strategy for each learning type
pub struct StrategyLearner {
    /// Maps (category, context_type) → strategy distribution
    strategy_priors: HashMap<(LearningCategory, ContextType), StrategyDistribution>,
}

#[derive(Debug, Clone)]
pub struct StrategyDistribution {
    /// Probability weights for each strategy
    pub weights: HashMap<InjectionStrategy, AdaptiveParam>,
}

impl StrategyLearner {
    /// Select strategy using Thompson sampling (exploration vs exploitation)
    pub fn select_strategy(
        &self,
        learning: &Learning,
        context: &SessionContext,
    ) -> InjectionStrategy {
        let key = (learning.category.clone(), context.context_type());

        let distribution = self.strategy_priors
            .get(&key)
            .cloned()
            .unwrap_or_default();

        // Thompson sampling: sample from each posterior, pick highest
        distribution.weights
            .iter()
            .map(|(strategy, param)| (strategy.clone(), param.sample()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(s, _)| s)
            .unwrap_or(InjectionStrategy::Deferred {
                trigger: DeferralTrigger::Explicit,
            })
    }

    /// Update based on outcome
    pub fn update(
        &mut self,
        learning: &Learning,
        context: &SessionContext,
        strategy_used: &InjectionStrategy,
        outcome: f64,  // 0.0 = harmful, 0.5 = neutral, 1.0 = helpful
    ) {
        let key = (learning.category.clone(), context.context_type());

        let distribution = self.strategy_priors
            .entry(key)
            .or_default();

        if let Some(param) = distribution.weights.get_mut(strategy_used) {
            param.update(outcome, 1.0);
        }
    }
}
```

### Strategy Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Injection Decision Flow                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Learning Retrieved                                              │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  StrategyLearner.select_strategy()                       │    │
│  │  (Thompson sampling from learned distributions)          │    │
│  └─────────────────────────────────────────────────────────┘    │
│       │                                                          │
│       ├──▶ MainContext                                          │
│       │       └─ Inject directly, track outcome                 │
│       │                                                          │
│       ├──▶ Subagent (blocking)                                  │
│       │       └─ Spawn verifier/synthesizer, wait for result    │
│       │                                                          │
│       ├──▶ BackgroundSubagent                                   │
│       │       └─ Spawn async, callback when ready               │
│       │                                                          │
│       └──▶ Deferred                                             │
│               └─ Store trigger condition, inject later          │
│                                                                  │
│  Session Ends                                                    │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  StrategyLearner.update()                                │    │
│  │  (Bayesian update based on session outcome)              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Section 8: Transcript Analysis & Learning Extraction

### Post-Session Analysis

When a session ends, the Stop hook provides `transcript_path`. We analyze the full transcript to extract learnings:

```rust
/// Extracts learnings from Claude Code transcripts
pub struct TranscriptAnalyzer {
    storage: Arc<dyn LearningStorage>,
    embedder: Arc<dyn Embedder>,
}

impl TranscriptAnalyzer {
    pub async fn analyze(&self, transcript_path: &Path) -> Result<Vec<Learning>> {
        let transcript = self.parse_transcript(transcript_path).await?;

        let mut learnings = Vec::new();

        // Pattern 1: Error → Recovery sequences
        learnings.extend(self.extract_error_recoveries(&transcript).await?);

        // Pattern 2: User corrections
        learnings.extend(self.extract_user_corrections(&transcript).await?);

        // Pattern 3: Successful tool usage patterns
        learnings.extend(self.extract_tool_patterns(&transcript).await?);

        // Pattern 4: Conversation-level insights
        learnings.extend(self.extract_conversation_insights(&transcript).await?);

        Ok(learnings)
    }

    async fn extract_error_recoveries(
        &self,
        transcript: &Transcript,
    ) -> Result<Vec<Learning>> {
        let mut learnings = Vec::new();

        // Find error → success sequences
        for window in transcript.messages.windows(3) {
            if let [action, error, recovery] = window {
                if error.is_error() && recovery.resolves_error(error) {
                    let learning = Learning {
                        id: Uuid::now_v7(),
                        category: LearningCategory::ErrorRecovery,
                        content: LearningContent {
                            description: format!(
                                "When {} fails with {}, recover by {}",
                                action.summarize(),
                                error.summarize(),
                                recovery.summarize()
                            ),
                            pattern: Pattern::ErrorRecovery {
                                trigger: action.to_pattern(),
                                error: error.to_pattern(),
                                recovery: recovery.to_pattern(),
                            },
                            insight: recovery.extract_insight(),
                            ..Default::default()
                        },
                        confidence: 0.5,  // Will be updated by AdaptiveParam
                        source: LearningSource::TranscriptAnalysis {
                            transcript_path: transcript.path.to_string_lossy().to_string(),
                            extraction_method: "error_recovery_window".to_string(),
                        },
                        ..Default::default()
                    };
                    learnings.push(learning);
                }
            }
        }

        Ok(learnings)
    }
}
```

### Embedding Generation

```rust
/// Generate embeddings for semantic search
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

/// Local embedding using sentence-transformers
pub struct LocalEmbedder {
    model: SentenceTransformer,
}

/// Remote embedding using OpenAI API (optional)
pub struct OpenAIEmbedder {
    client: OpenAIClient,
    model: String,  // "text-embedding-3-small"
}
```

---

## Section 9: Capability Gap Surfacing

### The Problem

When the system encounters patterns it can't learn from, it should surface this as a capability gap rather than silently failing.

### Capability Gap Detection

```rust
/// Identifies what the system cannot currently learn
pub struct CapabilityGapDetector {
    known_categories: HashSet<LearningCategory>,
    known_patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct CapabilityGap {
    pub id: Uuid,
    pub description: String,
    pub frequency: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub example_contexts: Vec<String>,
    pub potential_solutions: Vec<String>,
}

impl CapabilityGapDetector {
    /// Called when transcript analysis can't extract learnings
    pub fn report_gap(
        &mut self,
        context: &AnalysisContext,
        reason: &str,
    ) -> CapabilityGap {
        // Check if this is a known gap
        if let Some(existing) = self.find_similar_gap(context, reason) {
            existing.frequency += 1;
            existing.last_seen = Utc::now();
            return existing.clone();
        }

        // New gap discovered
        let gap = CapabilityGap {
            id: Uuid::now_v7(),
            description: format!("Cannot learn from: {}", reason),
            frequency: 1,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            example_contexts: vec![context.summarize()],
            potential_solutions: self.suggest_solutions(context, reason),
        };

        self.gaps.push(gap.clone());
        gap
    }

    /// Surface high-frequency gaps for potential future implementation
    pub fn get_priority_gaps(&self, min_frequency: u32) -> Vec<&CapabilityGap> {
        self.gaps
            .iter()
            .filter(|g| g.frequency >= min_frequency)
            .collect()
    }
}
```

### Gap Visualization (Web UI)

```
┌─────────────────────────────────────────────────────────────────┐
│  Capability Gaps                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ⚠️  High Priority Gaps (seen 10+ times)                         │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ MCP Tool Usage Patterns                           Freq: 47 │ │
│  │ Cannot extract learnings from custom MCP tools            │ │
│  │ Potential: Add MCP introspection adapter                  │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Multi-File Refactoring Strategies                 Freq: 23 │ │
│  │ Cannot identify optimal refactoring sequences              │ │
│  │ Potential: Add AST analysis to transcript analyzer         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  📊 Gap Trends Over Time                                         │
│  [Chart showing gap frequency changes]                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Section 10: Open-World Adaptation

### The Challenge

The system must detect and adapt to capabilities that don't exist yet. When MCP was invented, the system should have noticed "there's a new pattern here I don't understand" before anyone told it about MCP.

### Novelty Detection

```rust
/// Detects unknown patterns in session data
pub struct NoveltyDetector {
    /// Sensitivity to unknown patterns (adaptive)
    threshold: AdaptiveParam,
    /// Clustering for grouping similar anomalies
    anomaly_clusters: Vec<AnomalyCluster>,
    /// Known pattern fingerprints
    known_fingerprints: HashSet<PatternFingerprint>,
}

#[derive(Debug, Clone)]
pub struct AnomalyCluster {
    pub id: Uuid,
    pub centroid: Vec<f32>,  // Embedding centroid
    pub examples: Vec<NoveltyExample>,
    pub first_seen: DateTime<Utc>,
    pub frequency: u32,
    pub named: Option<String>,  // Once we understand it
}

impl NoveltyDetector {
    /// Analyze session for unknown patterns
    pub async fn detect(&mut self, session: &SessionData) -> Vec<Novelty> {
        let mut novelties = Vec::new();

        for event in &session.events {
            let fingerprint = self.compute_fingerprint(event);

            // Known pattern?
            if self.known_fingerprints.contains(&fingerprint) {
                continue;
            }

            // Compute embedding for similarity
            let embedding = self.embed(event).await?;

            // Distance to nearest known cluster
            let min_distance = self.min_cluster_distance(&embedding);

            // Thompson sampling for threshold (exploration)
            let threshold = self.threshold.sample();

            if min_distance > threshold {
                // This is novel!
                let novelty = Novelty {
                    event: event.clone(),
                    embedding,
                    distance: min_distance,
                };
                novelties.push(novelty);

                // Add to or create cluster
                self.cluster_novelty(novelty);
            }
        }

        novelties
    }

    /// When a cluster reaches critical mass, surface it
    pub fn get_emergent_patterns(&self, min_examples: usize) -> Vec<&AnomalyCluster> {
        self.anomaly_clusters
            .iter()
            .filter(|c| c.examples.len() >= min_examples && c.named.is_none())
            .collect()
    }
}
```

### Emergent Pattern Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                 Open-World Adaptation Flow                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Session Data                                                    │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  NoveltyDetector.detect()                                │    │
│  │  • Fingerprint each event                                │    │
│  │  • Compare to known patterns                             │    │
│  │  • Cluster unknown patterns                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│       │                                                          │
│       ├──▶ Known Pattern → Normal learning flow                 │
│       │                                                          │
│       └──▶ Unknown Pattern                                      │
│               │                                                  │
│               ▼                                                  │
│       ┌─────────────────────────────────────────────────────┐   │
│       │  Add to AnomalyCluster                               │   │
│       │  (group similar unknowns together)                   │   │
│       └─────────────────────────────────────────────────────┘   │
│               │                                                  │
│               ▼                                                  │
│       ┌─────────────────────────────────────────────────────┐   │
│       │  Cluster reaches critical mass (N examples)          │   │
│       │                                                       │   │
│       │  → Surface as "Emergent Pattern"                     │   │
│       │  → Suggest new LearningCategory                       │   │
│       │  → Propose new CaptureAdapter                        │   │
│       └─────────────────────────────────────────────────────┘   │
│               │                                                  │
│               ▼                                                  │
│       ┌─────────────────────────────────────────────────────┐   │
│       │  Human reviews emergent pattern                      │   │
│       │  (or autonomous agent in Level 3)                    │   │
│       │                                                       │   │
│       │  → Name the pattern                                  │   │
│       │  → Add to known_fingerprints                         │   │
│       │  → Create learning schema                            │   │
│       └─────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Example: Discovering MCP

```
Timeline:
─────────────────────────────────────────────────────────────────

Day 1-30: System sees only standard Claude Code tools
          (Bash, Read, Write, Edit, etc.)
          NoveltyDetector knows these patterns

Day 31:   New tool appears: "mcp__github__create_issue"
          NoveltyDetector: "Unknown pattern! Clustering..."

Day 32:   More: "mcp__notion__search", "mcp__slack__send"
          NoveltyDetector: "Cluster growing. Similar naming pattern."

Day 45:   Cluster has 50+ examples
          → Surfaced as "Emergent Pattern #7"
          → Characteristics:
            • Prefix: "mcp__"
            • Structure: mcp__<server>__<action>
            • Appears in tool_name field
            • New parameter patterns

Day 46:   System creates CapabilityGap:
          "Cannot learn from mcp__* tools - unknown semantics"

Day 47:   System suggests:
          "Create new LearningCategory::McpToolUsage"
          "Add McpIntrospectionAdapter to discover servers"

          Human confirms, system now learns MCP patterns!

─────────────────────────────────────────────────────────────────
```

---

## Section 11: Plugin Structure & Integration

### Crate Organization

```
vibes-learning/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── plugin.rs          # Plugin trait implementation
│   │
│   ├── model/             # Core data types
│   │   ├── mod.rs
│   │   ├── learning.rs
│   │   ├── scope.rs
│   │   ├── adaptive.rs    # AdaptiveParam
│   │   └── strategy.rs    # InjectionStrategy
│   │
│   ├── storage/           # Storage layer
│   │   ├── mod.rs
│   │   ├── trait.rs       # LearningStorage trait
│   │   ├── cozo.rs        # CozoDB implementation
│   │   └── migrations.rs  # Schema versioning
│   │
│   ├── capture/           # Capture adapters
│   │   ├── mod.rs
│   │   ├── trait.rs       # CaptureAdapter trait
│   │   ├── claude_code.rs # Claude Code hooks adapter
│   │   └── generic.rs     # Fallback for unknown harnesses
│   │
│   ├── inject/            # Injection adapters
│   │   ├── mod.rs
│   │   ├── trait.rs       # InjectionAdapter trait
│   │   ├── claude_code.rs # Claude Code injection
│   │   └── strategy.rs    # StrategyLearner
│   │
│   ├── analysis/          # Learning extraction
│   │   ├── mod.rs
│   │   ├── transcript.rs  # Transcript analyzer
│   │   ├── patterns.rs    # Pattern extraction
│   │   └── embedder.rs    # Embedding generation
│   │
│   ├── introspection/     # Harness introspection
│   │   ├── mod.rs
│   │   ├── trait.rs       # Harness trait
│   │   ├── claude_code.rs # Claude Code introspector
│   │   └── capabilities.rs
│   │
│   ├── novelty/           # Open-world adaptation
│   │   ├── mod.rs
│   │   ├── detector.rs    # NoveltyDetector
│   │   ├── cluster.rs     # AnomalyCluster
│   │   └── fingerprint.rs
│   │
│   └── gaps/              # Capability gap tracking
│       ├── mod.rs
│       └── detector.rs
│
└── tests/
    ├── storage_tests.rs
    ├── capture_tests.rs
    └── integration_tests.rs
```

### Plugin Entry Point

```rust
// src/plugin.rs

use vibes_plugin_api::{Plugin, PluginInfo, EventContext};

pub struct LearningPlugin {
    storage: Arc<dyn LearningStorage>,
    harness: Arc<dyn Harness>,
    capture_adapter: Box<dyn CaptureAdapter>,
    injection_adapter: Box<dyn InjectionAdapter>,
    strategy_learner: StrategyLearner,
    novelty_detector: NoveltyDetector,
    gap_detector: CapabilityGapDetector,
    config: Arc<AdaptiveConfig>,
}

impl Plugin for LearningPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "vibes-learning".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Continual learning for AI coding assistants".to_string(),
        }
    }

    fn on_load(&mut self, ctx: &mut EventContext) -> Result<()> {
        // Level 0: Harness introspection
        let capabilities = self.harness.introspect();

        // Select adapters based on discovered capabilities
        self.capture_adapter = self.harness.capture_adapter();
        self.injection_adapter = self.harness.injection_adapter();

        // Subscribe to relevant events
        ctx.subscribe("session.start");
        ctx.subscribe("session.end");
        ctx.subscribe("hook.pre_tool_use");
        ctx.subscribe("hook.post_tool_use");
        ctx.subscribe("hook.stop");

        Ok(())
    }

    fn on_event(&mut self, event: &Event, ctx: &mut EventContext) -> Result<()> {
        match event.name.as_str() {
            "session.start" => self.on_session_start(event, ctx),
            "session.end" => self.on_session_end(event, ctx),
            "hook.stop" => self.on_stop_hook(event, ctx),
            _ => Ok(()),
        }
    }
}

impl LearningPlugin {
    async fn on_session_start(&mut self, event: &Event, ctx: &mut EventContext) -> Result<()> {
        // Prepare session context with relevant learnings
        let session_ctx = self.injection_adapter
            .prepare_session_context(&self.storage, &self.config)
            .await?;

        // Inject learnings based on strategy
        for learning in session_ctx.learnings {
            let strategy = self.strategy_learner.select_strategy(&learning, &session_ctx);
            self.injection_adapter.inject(&learning, &strategy, ctx).await?;
        }

        Ok(())
    }

    async fn on_stop_hook(&mut self, event: &Event, ctx: &mut EventContext) -> Result<()> {
        // Extract transcript path from stop hook
        if let Some(transcript_path) = event.data.get("transcript_path") {
            // Analyze transcript for learnings
            let analyzer = TranscriptAnalyzer::new(
                Arc::clone(&self.storage),
                self.create_embedder(),
            );
            let learnings = analyzer.analyze(Path::new(transcript_path)).await?;

            // Store learnings
            for learning in learnings {
                self.storage.store_learning(&learning).await?;
            }

            // Check for novelties
            let session_data = self.capture_adapter.get_session_data(event)?;
            let novelties = self.novelty_detector.detect(&session_data).await?;

            for novelty in novelties {
                // Report as potential capability gap
                self.gap_detector.report_gap(&novelty.context, &novelty.reason);
            }
        }

        Ok(())
    }
}
```

---

## Section 12: Core Concepts Summary

### Concept Glossary

| Concept | Definition |
|---------|------------|
| **Learning** | A captured piece of knowledge that can improve future sessions |
| **Scope** | Hierarchical isolation level (Global → User → Project) |
| **AdaptiveParam** | A parameter that learns from outcomes via Bayesian updates |
| **CaptureAdapter** | Harness-specific mechanism for observing sessions |
| **InjectionAdapter** | Harness-specific mechanism for injecting learnings |
| **InjectionStrategy** | How to inject a learning (MainContext, Subagent, Deferred) |
| **StrategyLearner** | Learns which injection strategies work best |
| **NoveltyDetector** | Identifies patterns the system doesn't understand |
| **AnomalyCluster** | Grouped unknown patterns that may represent new capabilities |
| **CapabilityGap** | Something the system cannot currently learn from |
| **Harness** | The AI coding assistant being enhanced (Claude Code, etc.) |
| **HarnessCapabilities** | What a harness supports (hooks, transcripts, config) |

### Level Progression

```
Level 0: Harness Introspection
         ├─ Discover what the harness can do
         ├─ Select appropriate adapters
         └─ Prerequisite for all other levels

Level 1: Capture & Inject
         ├─ Capture session outcomes
         ├─ Store learnings
         └─ Inject relevant learnings

Level 2: Learning from Outcomes
         ├─ Analyze transcripts
         ├─ Extract patterns
         └─ Update confidence based on usage

Level 3: Meta-Learning
         ├─ Learn how to learn better
         ├─ Detect unknown unknowns
         └─ Generate new adapters autonomously
```

---

## Section 13: Harness Introspection (Level 0)

### Why Level 0?

Before learning anything, we must know:
1. What can we observe? (Capture mechanisms)
2. How can we inject? (Injection mechanisms)
3. What does this harness support? (Capabilities)

This is **Level 0** because everything else depends on it.

### Harness Abstraction

```rust
/// A harness is any AI coding assistant we're enhancing
#[async_trait]
pub trait Harness: Send + Sync {
    /// Discover what this harness supports
    fn introspect(&self) -> HarnessCapabilities;

    /// Get the appropriate capture adapter
    fn capture_adapter(&self) -> Box<dyn CaptureAdapter>;

    /// Get the appropriate injection adapter
    fn injection_adapter(&self) -> Box<dyn InjectionAdapter>;

    /// Harness identifier
    fn harness_type(&self) -> &str;

    /// Version if detectable
    fn version(&self) -> Option<String>;
}

/// What a harness is capable of
#[derive(Debug, Clone)]
pub struct HarnessCapabilities {
    /// Hook support (pre/post tool use, stop, etc.)
    pub hooks: Option<HookCapabilities>,

    /// Transcript/history access
    pub transcripts: Option<TranscriptCapabilities>,

    /// Configuration injection points
    pub config: Option<ConfigCapabilities>,

    /// Ways to send input to the harness
    pub input_mechanisms: Vec<InputMechanism>,

    /// Output format (streaming JSON, raw text, etc.)
    pub output_format: OutputFormat,

    /// What we can observe without hooks
    pub observable_signals: Vec<ObservableSignal>,
}

#[derive(Debug, Clone)]
pub struct HookCapabilities {
    pub pre_tool_use: bool,
    pub post_tool_use: bool,
    pub stop: bool,
    pub custom_hooks: Vec<String>,
    pub hook_location: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TranscriptCapabilities {
    pub format: TranscriptFormat,
    pub location: PathBuf,
    pub realtime_access: bool,
    pub post_session_access: bool,
}

#[derive(Debug, Clone)]
pub enum InputMechanism {
    /// Can inject via CLAUDE.md or similar
    ConfigFile { path: PathBuf, format: ConfigFormat },
    /// Can inject via environment variables
    Environment { prefix: String },
    /// Can inject via MCP tools
    McpTool { server: String, tool: String },
    /// Direct stdin if we control the process
    Stdin,
}

#[derive(Debug, Clone)]
pub enum ObservableSignal {
    /// Can observe stdout
    Stdout,
    /// Can observe stderr
    Stderr,
    /// Can observe file system changes
    FileSystem { paths: Vec<PathBuf> },
    /// Can observe process exit code
    ExitCode,
    /// Can observe via hooks
    Hooks,
}
```

### Claude Code Introspector

```rust
/// Introspects Claude Code specifically
pub struct ClaudeCodeHarness {
    claude_dir: PathBuf,
}

impl ClaudeCodeHarness {
    pub fn new() -> Result<Self> {
        let claude_dir = dirs::home_dir()
            .ok_or_else(|| Error::NoHomeDir)?
            .join(".claude");
        Ok(Self { claude_dir })
    }

    fn detect_hooks(&self) -> Option<HookCapabilities> {
        let hooks_dir = self.claude_dir.join("hooks");
        if hooks_dir.exists() {
            Some(HookCapabilities {
                pre_tool_use: true,
                post_tool_use: true,
                stop: true,
                custom_hooks: vec![],
                hook_location: hooks_dir,
            })
        } else {
            None
        }
    }

    fn detect_transcripts(&self) -> Option<TranscriptCapabilities> {
        // Claude writes transcripts to ~/.claude/projects/<hash>/
        let projects_dir = self.claude_dir.join("projects");
        if projects_dir.exists() {
            Some(TranscriptCapabilities {
                format: TranscriptFormat::Jsonl,
                location: projects_dir,
                realtime_access: false,
                post_session_access: true,
            })
        } else {
            None
        }
    }
}

impl Harness for ClaudeCodeHarness {
    fn introspect(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            hooks: self.detect_hooks(),
            transcripts: self.detect_transcripts(),
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
                InputMechanism::Environment {
                    prefix: "CLAUDE_".to_string(),
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

    fn capture_adapter(&self) -> Box<dyn CaptureAdapter> {
        let caps = self.introspect();
        if caps.hooks.is_some() {
            Box::new(ClaudeCodeHooksCapture::new())
        } else {
            // Fallback to output parsing
            Box::new(OutputParsingCapture::new())
        }
    }

    fn injection_adapter(&self) -> Box<dyn InjectionAdapter> {
        Box::new(ClaudeCodeInjector::new())
    }

    fn harness_type(&self) -> &str {
        "claude-code"
    }

    fn version(&self) -> Option<String> {
        // Could parse from `claude --version`
        None
    }
}
```

### Generic Harness Discovery

For unknown harnesses, we attempt discovery:

```rust
/// Attempts to introspect an unknown harness
pub struct GenericHarnessDiscovery;

impl GenericHarnessDiscovery {
    /// Try to figure out what this harness supports
    pub fn discover(process_name: &str, working_dir: &Path) -> HarnessCapabilities {
        let mut caps = HarnessCapabilities::default();

        // Check for common config files
        let config_files = [
            ("CLAUDE.md", ConfigFormat::Markdown),
            (".cursor/rules", ConfigFormat::Text),
            (".aider/config.yml", ConfigFormat::Yaml),
        ];

        for (file, format) in config_files {
            if working_dir.join(file).exists() {
                caps.input_mechanisms.push(InputMechanism::ConfigFile {
                    path: working_dir.join(file),
                    format,
                });
            }
        }

        // Check for hook directories
        let hook_dirs = [
            dirs::home_dir().map(|h| h.join(".claude/hooks")),
            dirs::home_dir().map(|h| h.join(".cursor/hooks")),
        ];

        for hook_dir in hook_dirs.into_iter().flatten() {
            if hook_dir.exists() {
                caps.hooks = Some(HookCapabilities {
                    hook_location: hook_dir,
                    ..Default::default()
                });
                break;
            }
        }

        // Always have stdout
        caps.observable_signals.push(ObservableSignal::Stdout);
        caps.observable_signals.push(ObservableSignal::ExitCode);

        caps
    }
}
```

### Introspection Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                 Level 0: Harness Introspection                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  vibes starts                                                    │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Detect harness type                                     │    │
│  │  • Check process name (claude, cursor, aider, etc.)     │    │
│  │  • Check environment variables                          │    │
│  │  • Check working directory                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│       │                                                          │
│       ├──▶ Known harness (Claude Code)                          │
│       │       └─ Use ClaudeCodeHarness                          │
│       │                                                          │
│       └──▶ Unknown harness                                      │
│               └─ Use GenericHarnessDiscovery                    │
│                                                                  │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  harness.introspect()                                    │    │
│  │  • Check for hooks support                              │    │
│  │  • Check for transcript access                          │    │
│  │  • Check for config injection points                    │    │
│  │  • Identify observable signals                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Select adapters based on capabilities                   │    │
│  │  • If hooks → ClaudeCodeHooksCapture                    │    │
│  │  • If transcripts → TranscriptCapture                   │    │
│  │  • Fallback → OutputParsingCapture                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Store capabilities for future reference                 │    │
│  │  • Cache in CozoDB                                      │    │
│  │  • Log for debugging                                    │    │
│  │  • Surface gaps if limited capabilities                 │    │
│  └─────────────────────────────────────────────────────────┘    │
│       │                                                          │
│       ▼                                                          │
│  Ready for Level 1 (Capture & Inject)                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Foundation (MVP)
- [ ] CozoDB storage layer with schema
- [ ] Claude Code harness introspection
- [ ] Basic capture adapter (hooks)
- [ ] Basic injection adapter (CLAUDE.md)
- [ ] Learning model with UUIDv7
- [ ] Adaptive parameters infrastructure

### Phase 2: Learning Extraction
- [ ] Transcript analyzer
- [ ] Pattern extraction (error recovery, tool usage)
- [ ] Embedding generation (local or API)
- [ ] Semantic search in CozoDB

### Phase 3: Adaptive Injection
- [ ] Strategy learner with Thompson sampling
- [ ] Subagent injection support
- [ ] Outcome-based parameter updates
- [ ] Confidence calibration

### Phase 4: Open-World Adaptation
- [ ] Novelty detector
- [ ] Anomaly clustering
- [ ] Capability gap surfacing
- [ ] Emergent pattern detection

### Phase 5: Meta-Learning
- [ ] Learning from learning outcomes
- [ ] Autonomous adapter generation
- [ ] Cross-harness learning transfer

---

## Testing Strategy

### Unit Tests
- Adaptive parameter Bayesian updates
- CozoDB query correctness
- Pattern fingerprinting
- Scope hierarchy resolution

### Integration Tests
- End-to-end capture → store → retrieve → inject
- Strategy selection and outcome updates
- Novelty detection with synthetic data

### Property-Based Tests
- Adaptive parameters converge appropriately
- Scope isolation is maintained
- Strategy distribution updates are monotonic

### Simulation Tests
- Multi-session learning scenarios
- Emergent pattern detection with synthetic data
- Cross-harness capability discovery

---

## Open Questions

1. **Embedding model**: Local (fast, private) vs API (better quality)?
2. **Initial priors**: How to bootstrap adaptive parameters?
3. **Privacy**: Should learnings ever leave the local machine?
4. **Harness detection**: How to reliably detect unknown harnesses?

---

## References

- [CozoDB Documentation](https://www.cozodb.org/)
- [Bayesian Optimization](https://arxiv.org/abs/1807.02811)
- [Thompson Sampling](https://en.wikipedia.org/wiki/Thompson_sampling)
- [Claude Code Hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)
