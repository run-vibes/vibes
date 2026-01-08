# Milestone 26: Assessment Framework - Design

## Overview

The Assessment Framework is groove's measurement system - it determines whether learnings help or hurt session outcomes. Without measurement, the learning loop can't improve.

### Core Philosophy

**Platform, not product.** Every major component is trait-based with sensible defaults:
- Intervention mechanisms (hook-based default)
- Session-end detection (hook + timeout default)
- LLM backends (harness subprocess default)
- Circuit breaker algorithms (EMA default)
- Event log storage (Iggy default)

**Zero friction, maximum observability.** Assessment runs automatically with no user configuration required. Power users can tune patterns, backends, and thresholds via `config.toml`.

**Latency is sacred.** Chat sessions must never block on assessment. All persistence is async via fire-and-forget channels to Iggy.

**Event sourcing for auditability.** All assessment signals are written to an immutable log. Views and aggregations are computed from the log. This enables replay, debugging, and algorithm improvements without data loss.

---

## Architecture & Data Flow

### Event Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                           EventBus                                   │
└──────────────┬─────────────────────────────────┬────────────────────┘
               │                                 │
               ▼                                 ▼
    ┌──────────────────┐              ┌─────────────────────┐
    │ SessionCollector │              │ AssessmentProcessor │
    │   (from 4.3)     │              │      (new)          │
    └──────────────────┘              └──────────┬──────────┘
                                                 │
                    ┌────────────────────────────┼────────────────────────────┐
                    │                            │                            │
                    ▼                            ▼                            ▼
         ┌──────────────────┐        ┌───────────────────┐        ┌──────────────────┐
         │ LightweightLayer │        │  CheckpointMgr    │        │  CircuitBreaker  │
         │  (every message) │        │ (medium triggers) │        │  (intervention)  │
         └────────┬─────────┘        └─────────┬─────────┘        └────────┬─────────┘
                  │                            │                           │
                  │         ┌──────────────────┘                           │
                  ▼         ▼                                              ▼
         ┌─────────────────────┐                              ┌────────────────────┐
         │  Fire-and-Forget    │                              │  InterventionMgr   │
         │     Channel         │                              │   (hook injection) │
         └──────────┬──────────┘                              └────────────────────┘
                    │
                    ▼
         ┌─────────────────────┐
         │    Iggy Client      │
         │  (async writer)     │
         └──────────┬──────────┘
                    │
                    ▼
         ┌─────────────────────┐
         │   Iggy Server       │
         │ (supervised child)  │
         └─────────────────────┘
```

### Key Boundaries

1. **Sync boundary**: EventBus → AssessmentProcessor (lightweight detection runs inline, <10ms)
2. **Async boundary**: LightweightLayer → Channel → Iggy (persistence never blocks)
3. **Intervention boundary**: CircuitBreaker → InterventionMgr → Claude hooks (configurable)

### Parallel Subscriber Model

AssessmentProcessor subscribes to EventBus independently of SessionCollector. They process the same events for different purposes:
- **SessionCollector**: Buffers for learning extraction
- **AssessmentProcessor**: Real-time signal detection and intervention

---

## Three-Tier Assessment Model

### Tier Overview

| Tier | Trigger | Latency Budget | Cost | Purpose |
|------|---------|----------------|------|---------|
| **Lightweight** | Every message | <10ms | $0 | Real-time signals, circuit breaker fuel |
| **Medium** | Checkpoints (~10 msgs) | 0.5-2s async | ~$0.002 | Segment summarization, trend detection |
| **Heavy** | Session end (sampled) | 5-10s async | ~$0.02-0.22 | Full analysis, learning extraction trigger |

### Lightweight Assessment

Runs synchronously on every message. Must be fast.

**Signals detected:**
- **Linguistic patterns** (regex-based, configurable)
  - Negative: "why didn't you", "no, that's not what I meant", "actually, let's go back"
  - Positive: "perfect", "great", "thanks", "exactly"
- **Behavioral counters**
  - Correction count (user edits Claude's output)
  - Retry count (same request repeated)
  - Tool failure count (from Claude's Bash/tool outputs)
- **Running aggregates**
  - Frustration EMA (exponential moving average)
  - Success EMA

**Output:** `LightweightEvent` → fire-and-forget to Iggy

### Medium Assessment

Triggered at checkpoints, runs asynchronously.

**Checkpoint triggers:**
- Every N messages (default: 10)
- Task boundary detected ("done", "next", topic shift)
- Git commit (detected from tool output)
- Build/test pass (detected from tool output)
- Session pause (>5 min inactivity)

**Processing:**
- LLM summarizes the segment (via pluggable backend)
- Compute token metrics for segment
- Calculate segment score

**Output:** `MediumEvent` → Iggy

### Heavy Assessment

Triggered at session end, sampled.

**Session end detection (extensible):**
- Claude `Stop` hook (primary)
- Inactivity timeout (fallback, default: 15 min)

**Sampling strategy:**
- Base rate: 20% of sessions (configurable)
- Boost to 100% when:
  - Burn-in period (first 10 sessions)
  - Explicit user feedback detected
  - High frustration detected (circuit breaker triggered)
  - Unusually long session

**Processing:**
- Full transcript analysis via LLM
- Task outcome classification (success/failure/partial)
- Frustration trajectory analysis
- Triggers learning extraction (4.5's job)

**Output:** `HeavyEvent` → Iggy

---

## Circuit Breaker

### Purpose

Detect "session going bad" signals in real-time and intervene before damage compounds. The circuit breaker consumes lightweight signals and decides when to act.

### State Model (EMA Default)

```rust
pub struct CircuitBreakerState {
    frustration_ema: f64,      // 0.0 - 1.0, higher = worse
    success_ema: f64,          // 0.0 - 1.0, higher = better
    consecutive_failures: u32, // tool failures in a row
    last_intervention: Option<Instant>,
    cooldown: Duration,        // prevent intervention spam
}
```

**EMA update** (on each signal):
```
frustration = α × new_signal + (1 - α) × old_frustration
```

Where `α` (decay rate) is an `AdaptiveParam` that learns from outcomes.

### Trigger Conditions

| Condition | Detection | Default Threshold |
|-----------|-----------|-------------------|
| Frustration spike | `frustration_ema > threshold` | 0.7 (adaptive) |
| Tool failure loop | `consecutive_failures >= N` | 3 |
| Correction storm | `corrections in last M messages > N` | 3 in 5 |
| Repetition | User repeats same instruction verbatim | Exact match |

Thresholds are `AdaptiveParam` values - they learn from session outcomes via Thompson sampling.

### Intervention Pipeline

```
CircuitBreaker (detects)
    → InterventionDecision
    → InterventionManager (trait)
    → HookBasedIntervention (default impl)
    → UserPromptSubmit hook injection
```

**Intervention types:**
- **Clarification pause**: "I notice some confusion. Let me make sure I understand before proceeding..."
- **Approach pivot**: "This approach isn't working. Let me suggest an alternative..."
- **Explicit check-in**: "Should I continue with this direction, or would you like to try something different?"

### Cooldown & Rate Limiting

- Minimum 3 messages between interventions
- Configurable cooldown period (default: 2 minutes)
- Max interventions per session (default: 3) - after that, just log

### Extensibility

```rust
pub trait CircuitBreakerAlgorithm: Send + Sync {
    fn update(&mut self, signal: &LightweightSignal);
    fn should_intervene(&self) -> Option<InterventionType>;
    fn record_outcome(&mut self, outcome: InterventionOutcome);
}

pub trait InterventionMechanism: Send + Sync {
    async fn intervene(&self, intervention: InterventionType) -> Result<()>;
}
```

Default implementations: `EmaCircuitBreaker`, `HookBasedIntervention`

---

## Iggy Integration & Event Log

### Iggy Subprocess Management

vibes daemon owns the Iggy server lifecycle:

```rust
pub struct IggyManager {
    process: Option<Child>,
    client: Option<IggyClient>,
    config: IggyConfig,
    health_check_interval: Duration,
}

impl IggyManager {
    /// Spawn Iggy server from bundled binary
    pub async fn start(&mut self) -> Result<()>;

    /// Graceful shutdown with drain
    pub async fn stop(&mut self) -> Result<()>;

    /// Restart on crash (supervised)
    pub async fn supervise(&self) -> Result<()>;
}
```

**Binary management:**
- Iggy binary bundled in vibes release artifacts
- Version pinned in `Cargo.toml` or manifest file
- Data directory: `~/.vibes/iggy/` (user) or `.vibes/iggy/` (project)
- Auto-spawn on daemon startup

**Health monitoring:**
- Periodic health check (default: 30s)
- Auto-restart on crash with backoff
- Graceful drain on vibes shutdown

### Event Log Trait

```rust
#[async_trait]
pub trait AssessmentLog: Send + Sync {
    /// Append event to immutable log
    async fn append(&self, event: AssessmentEvent) -> Result<EventId>;

    /// Read events for a session
    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>>;

    /// Read events in time range
    async fn read_range(&self, start: DateTime, end: DateTime) -> Result<Vec<AssessmentEvent>>;

    /// Subscribe to real-time events (for dashboard)
    fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent>;
}
```

### Iggy Implementation

```rust
pub struct IggyAssessmentLog {
    client: IggyClient,
    stream_id: u32,
    topic_id: u32,
}
```

**Stream topology:**
- Stream: `groove.assessment`
- Topics: `groove.assessment.lightweight`, `groove.assessment.medium`, `groove.assessment.heavy`
- Partitioning: By session ID (keeps session events together)

**Retention policies:**

| Topic | Retention | Rationale |
|-------|-----------|-----------|
| `groove.assessment.lightweight` | 7 days | High volume, only needed for recent debugging |
| `groove.assessment.medium` | 30 days | Checkpoint summaries, useful for trend analysis |
| `groove.assessment.heavy` | Forever | Full session analyses, training data for improvements |

### Fire-and-Forget Channel

```rust
// In AssessmentProcessor
let (tx, rx) = mpsc::unbounded_channel::<AssessmentEvent>();

// Lightweight detection (sync, fast)
let signal = detect_signals(&message);
tx.send(AssessmentEvent::Lightweight(signal)).ok(); // Never blocks

// Background writer task
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        if let Err(e) = log.append(event).await {
            tracing::warn!("Assessment log write failed: {}", e);
            // Don't retry - accept data loss over latency
        }
    }
});
```

### Future Optimization: Warm Harness Pool

Instead of spawning Claude subprocess per LLM request, keep a pool of warm processes:

```
┌─────────────────┐
│  HarnessPool    │
│  ┌───────────┐  │
│  │ claude #1 │◄─┼── TTY stdin/stdout
│  │ (warm)    │  │
│  └───────────┘  │
│  ┌───────────┐  │
│  │ claude #2 │◄─┼── TTY stdin/stdout
│  │ (warm)    │  │
│  └───────────┘  │
└─────────────────┘
```

Could reduce latency from ~500ms to ~100ms. Deferred to future optimization.

---

## LLM Backend Abstraction

### The Problem

Medium and heavy assessment need LLM calls, but users have different access patterns:
- Some have Claude CLI with Max subscription (no API key)
- Some have Anthropic API keys
- Some want to use OpenAI or local models

### Trait Definition

```rust
#[async_trait]
pub trait AssessmentLLM: Send + Sync {
    /// Summarize a conversation segment (medium assessment)
    async fn summarize_segment(&self, messages: &[Message]) -> Result<SegmentSummary>;

    /// Analyze full session (heavy assessment)
    async fn analyze_session(&self, transcript: &Transcript) -> Result<SessionAnalysis>;

    /// Classify task outcome
    async fn classify_outcome(&self, context: &OutcomeContext) -> Result<Outcome>;
}
```

### Implementations

**1. HarnessSubprocess (Default)**
```rust
pub struct HarnessLLM {
    harness_path: PathBuf,  // e.g., /usr/local/bin/claude
}

impl AssessmentLLM for HarnessLLM {
    async fn summarize_segment(&self, messages: &[Message]) -> Result<SegmentSummary> {
        let prompt = format_summarization_prompt(messages);
        let output = Command::new(&self.harness_path)
            .args(["--print", &prompt])
            .output()
            .await?;
        parse_summary(&output.stdout)
    }
}
```
- Uses `claude --print` (or equivalent for other harnesses)
- Zero additional auth required
- Pro: Works with existing Claude subscription
- Con: Subprocess overhead (~500ms)

**2. AnthropicAPI**
```rust
pub struct AnthropicLLM {
    client: anthropic_sdk::Client,
    model: String,  // default: "claude-3-haiku"
}
```
- Direct API calls, fastest option
- API key from env (`ANTHROPIC_API_KEY`) or config
- Model configurable (Haiku for cost, Sonnet for quality)

**3. OpenAI / Ollama / Others**
- Same pattern, different client
- Registered via config: `backend = "ollama"`, `model = "llama3"`

---

## Attribution Context & Lineage

### Event Context Requirements

Every assessment event must capture:

```rust
pub struct AssessmentContext {
    // Identity
    session_id: SessionId,
    event_id: EventId,          // UUIDv7 for ordering
    timestamp: DateTime<Utc>,

    // Lineage
    active_learnings: Vec<LearningId>,  // What was injected at session start
    injection_method: InjectionMethod,   // CLAUDE.md, hook, both
    injection_scope: Scope,              // Global, User, Project

    // Harness context
    harness_type: HarnessType,           // ClaudeCode, etc.
    harness_version: Option<String>,

    // Environment
    project_id: Option<ProjectId>,
    user_id: UserId,
}
```

### Per-Tier Context

**Lightweight events:**
```rust
pub struct LightweightEvent {
    context: AssessmentContext,
    message_idx: u32,
    signals: Vec<LightweightSignal>,
    frustration_ema: f64,
    success_ema: f64,
}
```

**Medium events:**
```rust
pub struct MediumEvent {
    context: AssessmentContext,
    checkpoint_id: CheckpointId,
    message_range: (u32, u32),
    trigger: CheckpointTrigger,
    summary: String,
    token_metrics: TokenMetrics,
    segment_score: f64,

    // Attribution hints
    learnings_referenced: Vec<LearningId>,  // If Claude mentioned a learning
}
```

**Heavy events:**
```rust
pub struct HeavyEvent {
    context: AssessmentContext,
    outcome: Outcome,
    task_summary: String,
    frustration_trajectory: Vec<f64>,

    // Attribution data (computed)
    learning_attributions: HashMap<LearningId, AttributionScore>,

    // Lineage for extracted learnings (fed to 4.5)
    extraction_candidates: Vec<ExtractionCandidate>,
}
```

### Attribution Flow

```
Session Start
    │
    ├── Record: active_learnings = [L1, L2, L3]
    │
    ▼
Assessment Events (carry active_learnings in context)
    │
    ▼
Session End (Heavy Assessment)
    │
    ├── Outcome: Success/Failure/Partial
    ├── Compute attribution: L1 → +0.8, L2 → -0.1, L3 → +0.3
    │
    ▼
4.6 Attribution Engine
    │
    ├── Update learning scores
    ├── Thompson sampling update
    └── May deprecate/boost learnings
```

### Lineage Chain

For audit and debugging, every learning knows its origin:

```rust
pub struct LearningLineage {
    learning_id: LearningId,
    extracted_from: SessionId,           // Which session produced this
    extraction_event: EventId,           // Which heavy assessment
    extraction_confidence: f64,

    // Usage history
    activations: Vec<ActivationRecord>,  // Every time it was injected
    attribution_history: Vec<AttributionRecord>, // Outcome correlations
}
```

This feeds the dashboard (4.8) with full traceability: "This learning came from session X, has been used Y times, and correlates with Z% success rate."

---

## Configuration Schema

### Full Configuration Structure

```toml
# ~/.config/vibes/config.toml (user level)
# .vibes/config.toml (project level - overrides user)

[plugins.groove.assessment]
enabled = true
intervention_enabled = true

[plugins.groove.assessment.sampling]
base_rate = 0.2           # 20% of sessions get heavy assessment
burnin_sessions = 10      # 100% sampling for first N sessions

[plugins.groove.assessment.session_end]
# Extensible - can add more detectors later
hook_enabled = true       # Use Claude Stop hook
timeout_enabled = true    # Inactivity fallback
timeout_minutes = 15

[plugins.groove.assessment.circuit_breaker]
enabled = true
cooldown_seconds = 120
max_interventions_per_session = 3

[plugins.groove.assessment.llm]
backend = "harness"       # "harness" | "anthropic" | "openai" | "ollama"
model = "claude-3-haiku"  # Model selection (ignored for harness)
# API keys read from: ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.

[plugins.groove.assessment.patterns]
# Custom patterns (merged with defaults)
negative = [
    "my custom frustration phrase",
]
positive = [
    "my custom success phrase",
]

[plugins.groove.assessment.retention]
lightweight_days = 7
medium_days = 30
heavy_days = -1           # -1 = forever

[plugins.groove.assessment.iggy]
# Usually don't need to touch these
data_dir = "~/.vibes/iggy"  # or ".vibes/iggy" for project scope
port = 8090
```

### Scope Resolution

1. Load system defaults (compiled in)
2. Merge user config (`~/.config/vibes/config.toml`)
3. Merge project config (`.vibes/config.toml`)
4. Apply environment variable overrides (`VIBES_GROOVE_*`)

Later values override earlier. Arrays (like patterns) are merged, not replaced.

### Adaptive Parameters (Not in Config)

These are *not* user-configurable - they learn from outcomes:
- `frustration_threshold` - circuit breaker trigger point
- `ema_alpha` - decay rate for EMA
- `checkpoint_interval` - optimal messages between checkpoints

Stored in CozoDB as `AdaptiveParam` (from 4.2), updated via Thompson sampling.

---

## CLI Commands

### Initial Commands (Progressive Disclosure)

**`vibes groove assess status`**

Shows current assessment configuration and circuit breaker state:

```
$ vibes groove assess status

Assessment Framework
────────────────────
Status: Active
Intervention: Enabled (hook-based)
LLM Backend: harness (claude)
Sampling Rate: 20% (burn-in complete)

Circuit Breaker
───────────────
State: Normal
Frustration EMA: 0.12
Success EMA: 0.78
Consecutive Failures: 0
Last Intervention: None this session

Active Learnings (3)
────────────────────
• L-abc123: "Use conventional commits..." (project)
• L-def456: "Prefer Result over unwrap..." (user)
• L-ghi789: "Run just pre-commit before..." (project)

Iggy Status
───────────
Server: Running (pid 12345)
Events Today: 1,247
Storage: 42 MB
```

**`vibes groove assess history`**

Shows recent session assessments:

```
$ vibes groove assess history

Recent Sessions
───────────────
SESSION     DATE        OUTCOME   FRUSTRATION  LEARNINGS  ASSESSED
─────────────────────────────────────────────────────────────────────
abc-123     Today 2:30p Success   Low (0.15)   3 active   Heavy ✓
def-456     Today 11:00a Partial  Med (0.42)   2 active   Sampled
ghi-789     Yesterday   Success   Low (0.08)   3 active   Heavy ✓
jkl-012     Yesterday   Failure   High (0.71)  1 active   Heavy ✓

Use 'vibes groove assess history <session-id>' for details.
```

**`vibes groove assess history <session-id>`**

Detailed view of a specific session:

```
$ vibes groove assess history abc-123

Session abc-123
───────────────
Date: 2025-12-30 14:30:00
Duration: 23 minutes
Messages: 47
Outcome: Success

Frustration Trajectory
──────────────────────
     │    ╭─╮
0.5 ─┤   ╭╯ ╰╮
     │  ╭╯   ╰─────────────
0.0 ─┴──┴─────────────────────
     0   10   20   30   40  msg

Checkpoints (5)
───────────────
#1 (msg 1-10):  "Initial setup, exploring codebase" [0.72]
#2 (msg 11-20): "Implementing auth, one correction" [0.68]
#3 (msg 21-30): "Debugging JWT issue, some frustration" [0.45]
#4 (msg 31-40): "Resolved, tests passing" [0.85]
#5 (msg 41-47): "Cleanup and commit" [0.91]

Active Learnings & Attribution
──────────────────────────────
• L-abc123: +0.8 (helpful - mentioned in checkpoint #2)
• L-def456: +0.1 (neutral)
• L-ghi789: +0.5 (helpful - user followed advice in #5)
```

### Future Commands (Add As Needed)

Reserved for later based on user feedback:
- `vibes groove assess trigger` - manual assessment
- `vibes groove assess replay` - debug event replay
- `vibes groove assess patterns` - list active patterns
- `vibes groove assess calibrate` - reset adaptive params

---

## Decision Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Scope** | Full three-tier + circuit breaker | Comprehensive measurement system |
| **Intervention mechanism** | Hook-based, pluggable | Affects Claude behavior, extensible |
| **Session-end detection** | Hook + timeout, extensible | Covers normal exits + abandonment |
| **Linguistic signals** | Fuzzy regex + user-configurable | Fast, flexible, improvable |
| **Code quality signals** | Piggyback on tool outputs | Zero overhead, uses harness capabilities |
| **LLM backend** | Pluggable, harness subprocess default | Zero config, works with existing auth |
| **Circuit breaker state** | EMA default, swappable | Simple, adaptive, extensible |
| **Event storage** | Immutable log via Iggy | Event sourcing, replay, auditability |
| **Iggy management** | Bundled binary, auto-spawn | Zero config, pinned version |
| **Configuration** | `[plugins.groove.assessment]` in config.toml | Consistent with plugin namespace |
| **Scope resolution** | File location determines scope | Simple mental model |
| **CLI commands** | Minimal (status, history) | Progressive disclosure |
| **Integration** | Parallel EventBus subscriber | Clean separation from capture |
| **Sub-milestones** | 4.4.1 (infra) → 4.4.2 (logic) | De-risk Iggy first |
| **Stream naming** | Dot notation (`groove.assessment.*`) | Hierarchical consistency |
| **Retention** | Per-tier (7d/30d/forever) | Match data value to storage cost |
| **Attribution context** | Full lineage on every event | Enables 4.6 attribution engine |
