# Milestone 30: Learning Extraction - Design

## Overview

Learning Extraction transforms raw session outcomes into reusable learnings. It sits between Assessment (which measures sessions) and Attribution (which tracks learning value).

**Core principle**: Iggy stores the immutable event stream (what happened), CozoDB stores the queryable derived state (what we learned). If the extraction algorithm improves, we can replay Iggy and rebuild CozoDB.

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Heavy Assessment                                 │
│   (produces HeavyEvent with extraction_candidates)                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ writes to Iggy
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Iggy: groove.assessment.heavy                     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ consumed by
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  Learning Extraction Consumer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │ Pattern Detector │  │ LLM Candidates  │  │ Dedup + Embedding   │ │
│  │ (corrections,   │  │ (from HeavyEvent│  │ (semantic similarity│ │
│  │  error→fix)     │  │  candidates)    │  │  check, merge)      │ │
│  └────────┬────────┘  └────────┬────────┘  └──────────┬──────────┘ │
│           └────────────────────┴─────────────────────►│            │
└──────────────────────────────────────────────────────┬┴────────────┘
                           │ writes to
           ┌───────────────┴───────────────┐
           ▼                               ▼
┌─────────────────────┐         ┌─────────────────────┐
│ Iggy: extraction    │         │ CozoDB: learning    │
│ (raw events, audit) │         │ (materialized,      │
│                     │         │  with embeddings)   │
└─────────────────────┘         └─────────────────────┘
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Extraction sources | LLM + Pattern-based | LLM for nuanced patterns, heuristics for high-confidence mechanical patterns |
| Pattern types | Corrections + Error→Fix | Clear signals with low false-positive rates |
| Embeddings | Local-only (gte-small) | Privacy, no API costs, fast enough for similarity search |
| Model delivery | Download on first use | Keeps initial binary small |
| Storage | Iggy + CozoDB | Raw events for audit/replay, CozoDB for queryable derived state |
| Trigger | Separate Iggy consumer | Decoupled, replay capability, independent scaling |
| Deduplication | Semantic with trait | Configurable strategy, LLM merge can be added later |
| Scope levels | Project + User + Global | Matches existing config resolution pattern |

---

## Core Types

### Learning

The materialized output stored in CozoDB:

```rust
pub struct Learning {
    pub id: LearningId,                    // UUIDv7
    pub scope: LearningScope,              // Project, User, or Global
    pub category: LearningCategory,        // Correction, ErrorRecovery, Pattern, Preference
    pub description: String,               // Human-readable summary
    pub pattern: LearningPattern,          // What triggers this learning
    pub insight: String,                   // What Claude should know/do
    pub confidence: f64,                   // 0.0-1.0, updated by attribution
    pub source: LearningSource,            // Which session/event produced this
    pub embedding: Vec<f32>,               // For semantic search (384 dims for gte-small)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum LearningScope {
    Project(ProjectId),   // .vibes/learnings/
    User,                 // ~/.config/vibes/learnings/
    Global,               // Shared (future)
}

pub enum LearningCategory {
    Correction,           // User corrected Claude's behavior
    ErrorRecovery,        // Claude recovered from tool failure
    Pattern,              // LLM-extracted general pattern
    Preference,           // User preference detected
}

pub struct LearningSource {
    pub session_id: SessionId,
    pub event_id: EventId,                 // The HeavyEvent that triggered extraction
    pub message_range: Option<(u32, u32)>, // Where in the transcript
    pub extraction_method: ExtractionMethod,
}

pub enum ExtractionMethod {
    Pattern(PatternType),  // Which pattern detector found it
    Llm,                   // LLM-generated candidate
}
```

### ExtractionEvent

Raw event written to Iggy for audit:

```rust
pub struct ExtractionEvent {
    pub event_id: EventId,
    pub timestamp: DateTime<Utc>,
    pub source_heavy_event: EventId,       // Which HeavyEvent triggered this
    pub candidates_processed: u32,
    pub learnings_created: Vec<LearningId>,
    pub learnings_merged: Vec<(LearningId, LearningId)>, // (new, merged_into)
    pub learnings_rejected: u32,           // Below confidence threshold
}
```

---

## Pattern Detection

### Correction Detector

Scans transcripts for user corrections:

```rust
pub struct CorrectionDetector {
    patterns: Vec<Regex>,  // Configurable correction phrases
}

impl CorrectionDetector {
    pub fn detect(&self, transcript: &Transcript) -> Vec<CorrectionCandidate>;
}

pub struct CorrectionCandidate {
    pub message_range: (u32, u32),
    pub user_correction: String,       // "No, use snake_case not camelCase"
    pub claude_acknowledgment: String, // "You're right, I'll use snake_case"
    pub extracted_insight: String,     // "Use snake_case for variable names"
    pub confidence: f64,
}
```

### Error Recovery Detector

Finds tool failure → success sequences:

```rust
pub struct ErrorRecoveryDetector;

impl ErrorRecoveryDetector {
    pub fn detect(&self, transcript: &Transcript) -> Vec<ErrorRecoveryCandidate>;
}

pub struct ErrorRecoveryCandidate {
    pub message_range: (u32, u32),
    pub error_type: String,            // "compilation error", "test failure"
    pub error_message: String,
    pub recovery_strategy: String,     // What Claude did to fix it
    pub confidence: f64,
}
```

---

## Embedding & Deduplication

### Local Embedder

Using `gte-small` (384 dimensions, ~33M params, ~67MB model file):

```rust
pub struct LocalEmbedder {
    model: ort::Session,   // ONNX Runtime
    tokenizer: Tokenizer,
}

#[async_trait]
impl Embedder for LocalEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn dimensions(&self) -> usize { 384 }
}
```

Model downloaded on first use to `~/.cache/vibes/models/`.

### Deduplication Strategy

Configurable trait for future LLM-based merging:

```rust
#[async_trait]
pub trait DeduplicationStrategy: Send + Sync {
    async fn find_duplicate(
        &self,
        candidate: &LearningCandidate,
        embedder: &dyn Embedder,
        store: &dyn LearningStore,
    ) -> Result<Option<LearningId>>;

    async fn merge(
        &self,
        existing: &Learning,
        new_candidate: &LearningCandidate,
    ) -> Result<Learning>;
}

pub struct SemanticDedup {
    similarity_threshold: f64,  // Default: 0.85
}
```

---

## Storage

### CozoDB Schema

```datalog
:create learning {
    id: String =>
    scope_type: String,
    scope_value: String?,
    category: String,
    description: String,
    pattern_json: String,
    insight: String,
    confidence: Float,
    source_json: String,
    created_at: Int,
    updated_at: Int
}

:create learning_embedding {
    learning_id: String =>
    embedding: <F32; 384>
}

::hnsw create learning_embedding:semantic_idx {
    dim: 384,
    m: 16,
    ef_construction: 200,
    fields: [embedding]
}

::index create learning:by_scope { scope_type, scope_value }
::index create learning:by_category { category }
::index create learning:by_confidence { confidence }
```

### LearningStore Trait

```rust
#[async_trait]
pub trait LearningStore: Send + Sync {
    async fn insert(&self, learning: &Learning) -> Result<()>;
    async fn get(&self, id: LearningId) -> Result<Option<Learning>>;
    async fn update(&self, learning: &Learning) -> Result<()>;
    async fn delete(&self, id: LearningId) -> Result<()>;

    async fn find_by_scope(&self, scope: &LearningScope) -> Result<Vec<Learning>>;
    async fn find_similar(&self, embedding: &[f32], threshold: f64) -> Result<Vec<Learning>>;
    async fn find_for_injection(&self, context: &SessionContext) -> Result<Vec<Learning>>;
}
```

---

## Configuration

```toml
[plugins.groove.extraction]
enabled = true
min_confidence = 0.6

[plugins.groove.extraction.patterns]
corrections_enabled = true
error_recovery_enabled = true
correction_phrases = ["no, use", "actually I want"]

[plugins.groove.extraction.embedding]
model = "gte-small"
cache_dir = "~/.cache/vibes/models"

[plugins.groove.extraction.dedup]
strategy = "semantic"
similarity_threshold = 0.85
```

---

## CLI Commands

```
vibes groove learn status           # Show extraction status and counts
vibes groove learn list             # List learnings with filters
vibes groove learn show <id>        # Full learning details
vibes groove learn delete <id>      # Remove a learning
vibes groove learn export           # Export learnings as JSON
```
