---
id: FEAT0026
title: Attribution types and storage
type: feat
status: done
priority: high
scope: plugin-system
depends: []
estimate: 2h
created: 2026-01-09
---

# Attribution types and storage

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Define core attribution types and CozoDB storage layer for the attribution engine.

## Context

The attribution engine needs structured types to track how learnings influence sessions and their measured value over time. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Create attribution module and types

**Files:**
- Create: `plugins/vibes-groove/src/attribution/mod.rs`
- Create: `plugins/vibes-groove/src/attribution/types.rs`

**Steps:**
1. Create `attribution/` module directory
2. Define core types in `types.rs`:
   ```rust
   pub struct AttributionRecord {
       pub learning_id: LearningId,
       pub session_id: SessionId,
       pub timestamp: DateTime<Utc>,
       pub was_activated: bool,
       pub activation_confidence: f64,
       pub activation_signals: Vec<ActivationSignal>,
       pub temporal_positive: f64,
       pub temporal_negative: f64,
       pub net_temporal: f64,
       pub was_withheld: bool,
       pub session_outcome: f64,
       pub attributed_value: f64,
   }

   pub enum ActivationSignal {
       EmbeddingSimilarity { score: f64, message_idx: u32 },
       ExplicitReference { pattern: String, message_idx: u32 },
   }
   ```
3. Define `LearningValue` and `LearningStatus`:
   ```rust
   pub struct LearningValue {
       pub learning_id: LearningId,
       pub estimated_value: f64,
       pub confidence: f64,
       pub session_count: u32,
       pub activation_rate: f64,
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
4. Define ablation types:
   ```rust
   pub struct AblationExperiment {
       pub learning_id: LearningId,
       pub started_at: DateTime<Utc>,
       pub sessions_with: Vec<SessionOutcome>,
       pub sessions_without: Vec<SessionOutcome>,
       pub result: Option<AblationResult>,
   }

   pub struct AblationResult {
       pub marginal_value: f64,
       pub confidence: f64,
       pub is_significant: bool,
   }
   ```
5. Add module to `lib.rs`
6. Run: `cargo check -p vibes-groove`
7. Commit: `feat(groove): add attribution types`

### Task 2: Define AttributionStore trait

**Files:**
- Create: `plugins/vibes-groove/src/attribution/store.rs`

**Steps:**
1. Define `AttributionStore` trait:
   ```rust
   #[async_trait]
   pub trait AttributionStore: Send + Sync {
       // Attribution records
       async fn store_attribution(&self, record: &AttributionRecord) -> Result<()>;
       async fn get_attributions_for_learning(&self, id: LearningId) -> Result<Vec<AttributionRecord>>;
       async fn get_attributions_for_session(&self, id: SessionId) -> Result<Vec<AttributionRecord>>;

       // Learning values
       async fn get_learning_value(&self, id: LearningId) -> Result<Option<LearningValue>>;
       async fn update_learning_value(&self, value: &LearningValue) -> Result<()>;
       async fn list_learning_values(&self, limit: usize) -> Result<Vec<LearningValue>>;

       // Ablation experiments
       async fn get_experiment(&self, id: LearningId) -> Result<Option<AblationExperiment>>;
       async fn update_experiment(&self, exp: &AblationExperiment) -> Result<()>;
   }
   ```
2. Add to `attribution/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add AttributionStore trait`

### Task 3: Implement CozoAttributionStore

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/store.rs`

**Steps:**
1. Add CozoDB schema constants:
   ```rust
   pub const ATTRIBUTION_SCHEMA: &str = r#"
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

   :create ablation_experiment {
       learning_id: String =>
       started_at: Int,
       sessions_with_json: String,
       sessions_without_json: String,
       marginal_value: Float?,
       confidence: Float?,
       is_significant: Bool?
   }
   "#;
   ```
2. Implement `CozoAttributionStore` struct with `CozoDb` reference
3. Implement all trait methods with Datalog queries
4. Add index creation for efficient lookups
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): implement CozoAttributionStore`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/store.rs`

**Steps:**
1. Write tests for all store operations:
   - Test store and retrieve attribution record
   - Test learning value CRUD
   - Test ablation experiment tracking
   - Test listing with limits
2. Run: `cargo test -p vibes-groove attribution::store`
3. Commit: `test(groove): add attribution store tests`

## Acceptance Criteria

- [ ] `AttributionRecord` type captures all layer outputs
- [ ] `LearningValue` tracks aggregated lifetime value
- [ ] `AblationExperiment` tracks A/B test state
- [ ] `AttributionStore` trait defines storage interface
- [ ] `CozoAttributionStore` implements persistence
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0026`
3. Commit, push, and create PR
