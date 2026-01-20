# Milestone 31: Attribution Engine - Design

## Overview

The Attribution Engine determines which learnings help or hurt sessions by analyzing signals across multiple dimensions and time. It uses a 4-layer architecture where each layer is trait-based for future extensibility.

**Core principle**: Start simple, make it swappable. Each layer has a straightforward default implementation with a trait that allows sophisticated strategies later.

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Iggy: groove.assessment.heavy                     │
│            (HeavyEvent with active_learnings, outcome)              │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ consumed by
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Attribution Consumer                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │ Layer 1:        │  │ Layer 2:        │  │ Layer 3:            │ │
│  │ Activation      │─▶│ Temporal        │─▶│ Ablation            │ │
│  │ Detection       │  │ Correlation     │  │ Manager             │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘ │
│           │                   │                      │              │
│           └───────────────────┴──────────────────────┘              │
│                               ▼                                      │
│                    ┌─────────────────────┐                          │
│                    │ Layer 4:            │                          │
│                    │ Value Aggregation   │                          │
│                    └──────────┬──────────┘                          │
└───────────────────────────────┼─────────────────────────────────────┘
                                │ writes to
            ┌───────────────────┼───────────────────┐
            ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐
│ Iggy:           │  │ CozoDB:         │  │ CozoDB:                 │
│ attribution     │  │ attribution     │  │ learning_value          │
│ (raw events)    │  │ (per-session)   │  │ (aggregated)            │
└─────────────────┘  └─────────────────┘  └─────────────────────────┘
                                                    │
                                                    ▼
                                         ┌─────────────────────────┐
                                         │ Auto-deprecation        │
                                         │ (value < -0.3,          │
                                         │  confidence > 0.8)      │
                                         └─────────────────────────┘
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architecture | Full 4-layer system | Complete attribution from day one |
| Layer 1 | Embedding similarity + explicit reference | Captures paraphrased usage, trait-based for LLM later |
| Layer 2 | Exponential decay | Simple, well-understood, trait-based for adaptive later |
| Layer 3 | Conservative ablation | Minimize user impact, only test uncertain learnings |
| Layer 4 | Weighted average | Interpretable, debuggable |
| Trigger | Iggy consumer | Decoupled, supports replay |
| Negative learnings | Auto-deprecate | Protect users, reversible |

---

## Core Types

### Attribution Record

Per-session attribution stored in CozoDB:

```rust
pub struct AttributionRecord {
    pub learning_id: LearningId,
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,

    // Layer 1: Activation
    pub was_activated: bool,
    pub activation_confidence: f64,
    pub activation_signals: Vec<ActivationSignal>,

    // Layer 2: Temporal correlation
    pub temporal_positive: f64,
    pub temporal_negative: f64,
    pub net_temporal: f64,

    // Layer 3: Ablation
    pub was_withheld: bool,

    // Final
    pub session_outcome: Outcome,
    pub attributed_value: f64,
}

pub enum ActivationSignal {
    EmbeddingSimilarity { score: f64, message_idx: u32 },
    ExplicitReference { pattern: String, message_idx: u32 },
}
```

### Learning Value

Aggregated lifetime value:

```rust
pub struct LearningValue {
    pub learning_id: LearningId,
    pub estimated_value: f64,
    pub confidence: f64,
    pub session_count: u32,
    pub activation_rate: f64,

    // Per-source breakdown
    pub temporal_value: f64,
    pub temporal_confidence: f64,
    pub ablation_value: Option<f64>,
    pub ablation_confidence: Option<f64>,

    pub status: LearningStatus,
    pub updated_at: DateTime<Utc>,
}

pub enum LearningStatus {
    Active,
    Deprecated { reason: String },
    Experimental,
}
```

---

## Layer 1: Activation Detection

Detects if a learning influenced Claude's behavior.

### Trait

```rust
#[async_trait]
pub trait ActivationDetector: Send + Sync {
    async fn detect(
        &self,
        learning: &Learning,
        transcript: &Transcript,
        embedder: &dyn Embedder,
    ) -> Result<ActivationResult>;
}

pub struct ActivationResult {
    pub was_activated: bool,
    pub confidence: f64,
    pub signals: Vec<ActivationSignal>,
}
```

### Default: HybridActivationDetector

- Compares learning embedding to Claude's response embeddings
- Checks for explicit keyword references
- Combines both signals for confidence score

Future: `LlmActivationDetector` for direct model classification.

---

## Layer 2: Temporal Correlation

Measures positive/negative signals near activation points.

### Trait

```rust
pub trait TemporalCorrelator: Send + Sync {
    fn correlate(
        &self,
        activation_points: &[u32],
        lightweight_events: &[LightweightEvent],
    ) -> TemporalResult;
}

pub struct TemporalResult {
    pub positive_score: f64,
    pub negative_score: f64,
    pub net_score: f64,
}
```

### Default: ExponentialDecayCorrelator

- Weight = e^(-λ * distance) where λ = 0.2
- Signals closer to activation have more weight
- Max distance: 10 messages

Future: `AdaptiveDecayCorrelator` learns optimal decay rate.

---

## Layer 3: Ablation Testing

Runs A/B experiments by withholding uncertain learnings.

### Trait

```rust
pub trait AblationStrategy: Send + Sync {
    fn should_withhold(&self, learning: &Learning, value: &LearningValue) -> bool;
    fn is_experiment_complete(&self, experiment: &AblationExperiment) -> bool;
    fn compute_marginal_value(&self, experiment: &AblationExperiment) -> Option<AblationResult>;
}

pub struct AblationResult {
    pub marginal_value: f64,
    pub confidence: f64,
    pub is_significant: bool,
}
```

### Default: ConservativeAblation

- Only ablate learnings with confidence < 0.7
- 10% of applicable sessions
- Minimum 20 sessions per arm
- Welch's t-test for significance (p < 0.05)

Future: `ThompsonSamplingAblation` for optimal exploration/exploitation.

### Experiment Tracking

```rust
pub struct AblationExperiment {
    pub learning_id: LearningId,
    pub started_at: DateTime<Utc>,
    pub sessions_with: Vec<SessionOutcome>,
    pub sessions_without: Vec<SessionOutcome>,
    pub result: Option<AblationResult>,
}
```

---

## Layer 4: Value Aggregation

Combines signals from all sources.

### Logic

```rust
pub struct ValueAggregator {
    temporal_weight: f64,        // 0.6
    ablation_weight: f64,        // 0.4
    deprecation_threshold: f64,  // -0.3
    deprecation_confidence: f64, // 0.8
}
```

- Confidence-weighted average of temporal and ablation values
- Auto-deprecates when value < -0.3 and confidence > 0.8
- Deprecation is reversible via CLI

---

## Storage

### CozoDB Schema

```datalog
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

:create ablation_experiment {
    learning_id: String =>
    started_at: Int,
    sessions_with_json: String,
    sessions_without_json: String,
    marginal_value: Float?,
    confidence: Float?,
    is_significant: Bool?
}

:create learning_value {
    learning_id: String =>
    estimated_value: Float,
    confidence: Float,
    session_count: Int,
    activation_rate: Float,
    temporal_value: Float,
    temporal_confidence: Float,
    ablation_value: Float?,
    ablation_confidence: Float?,
    status: String,
    updated_at: Int
}

::index create attribution:by_learning { learning_id }
::index create attribution:by_session { session_id }
::index create learning_value:by_value { estimated_value }
::index create learning_value:by_status { status }
```

---

## Configuration

```toml
[plugins.groove.attribution]
enabled = true

[plugins.groove.attribution.activation]
similarity_threshold = 0.75
reference_boost = 0.15

[plugins.groove.attribution.temporal]
decay_rate = 0.2
max_distance = 10

[plugins.groove.attribution.ablation]
enabled = true
uncertainty_threshold = 0.7
ablation_rate = 0.10
min_sessions_per_arm = 20

[plugins.groove.attribution.deprecation]
auto_deprecate = true
value_threshold = -0.3
confidence_threshold = 0.8
```

---

## CLI Commands

```
vibes groove attr status              # Show attribution engine status
vibes groove attr values              # List learning values
vibes groove attr show <learning-id>  # Detailed attribution breakdown
vibes groove attr explain <learning> <session>  # Why this attribution?
vibes groove learn enable <id>        # Re-enable deprecated learning
vibes groove learn disable <id>       # Manually deprecate
```
