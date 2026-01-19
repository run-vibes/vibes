---
id: FEAT0034
title: Strategy types and storage
type: feat
status: done
priority: high
scope: plugin-system
depends: []
estimate: 2h
created: 2026-01-09
---

# Strategy types and storage

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Define core strategy types and CozoDB storage layer for the adaptive strategies system.

## Context

The adaptive strategies system needs structured types to track injection strategies, outcomes, and events. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create strategy module and types

**Files:**
- Create: `plugins/vibes-groove/src/strategy/mod.rs`
- Create: `plugins/vibes-groove/src/strategy/types.rs`

**Steps:**
1. Create `strategy/` module directory
2. Define `InjectionStrategy` enum:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub enum InjectionStrategy {
       MainContext {
           position: ContextPosition,
           format: InjectionFormat,
       },
       Subagent {
           agent_type: SubagentType,
           blocking: bool,
           prompt_template: Option<String>,
       },
       BackgroundSubagent {
           agent_type: SubagentType,
           callback: CallbackMethod,
           timeout_ms: u64,
       },
       Deferred {
           trigger: DeferralTrigger,
           max_wait_ms: Option<u64>,
       },
   }
   ```
3. Define `StrategyVariant` for simplified matching:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub enum StrategyVariant {
       MainContext,
       Subagent,
       BackgroundSubagent,
       Deferred,
   }

   impl From<&InjectionStrategy> for StrategyVariant { ... }
   ```
4. Define outcome types:
   ```rust
   pub struct StrategyOutcome {
       pub value: f64,
       pub confidence: f64,
       pub source: OutcomeSource,
   }

   pub enum OutcomeSource {
       Attribution,
       Direct,
       Both,
   }

   pub struct StrategyEvent {
       pub event_id: EventId,
       pub learning_id: LearningId,
       pub session_id: SessionId,
       pub strategy: InjectionStrategy,
       pub outcome: StrategyOutcome,
       pub timestamp: DateTime<Utc>,
   }
   ```
5. Add module to `lib.rs`
6. Run: `cargo check -p vibes-groove`
7. Commit: `feat(groove): add strategy types`

### Task 2: Define StrategyStore trait

**Files:**
- Create: `plugins/vibes-groove/src/strategy/store.rs`

**Steps:**
1. Define `StrategyStore` trait:
   ```rust
   #[async_trait]
   pub trait StrategyStore: Send + Sync {
       // Distributions
       async fn load_distributions(&self) -> Result<HashMap<(LearningCategory, ContextType), StrategyDistribution>>;
       async fn save_distributions(&self, distributions: &HashMap<(LearningCategory, ContextType), StrategyDistribution>) -> Result<()>;

       // Learning overrides
       async fn load_overrides(&self) -> Result<HashMap<LearningId, LearningStrategyOverride>>;
       async fn save_overrides(&self, overrides: &HashMap<LearningId, LearningStrategyOverride>) -> Result<()>;

       // Strategy events
       async fn store_strategy_event(&self, event: &StrategyEvent) -> Result<()>;
       async fn get_strategy_history(&self, learning_id: LearningId, limit: usize) -> Result<Vec<StrategyEvent>>;

       // Session cache
       async fn cache_strategy(&self, session_id: SessionId, learning_id: LearningId, strategy: &InjectionStrategy) -> Result<()>;
       async fn get_cached_strategy(&self, session_id: SessionId, learning_id: LearningId) -> Result<Option<InjectionStrategy>>;
       async fn clear_session_cache(&self, session_id: SessionId) -> Result<()>;
   }
   ```
2. Add to `strategy/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add StrategyStore trait`

### Task 3: Implement CozoStrategyStore

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/store.rs`

**Steps:**
1. Add CozoDB schema constants:
   ```rust
   pub const STRATEGY_SCHEMA: &str = r#"
   :create strategy_distribution {
       category: String,
       context_type: String =>
       strategy_weights_json: String,
       strategy_params_json: String,
       session_count: Int,
       updated_at: Int
   }

   :create learning_strategy_override {
       learning_id: String =>
       base_category: String,
       specialized_weights_json: String?,
       specialization_threshold: Int,
       session_count: Int,
       updated_at: Int
   }

   :create strategy_event {
       event_id: String =>
       learning_id: String,
       session_id: String,
       strategy_variant: String,
       strategy_params_json: String,
       outcome_value: Float,
       outcome_confidence: Float,
       outcome_source: String,
       timestamp: Int
   }

   :create strategy_session_cache {
       session_id: String,
       learning_id: String =>
       strategy_json: String,
       selected_at: Int
   }
   "#;
   ```
2. Implement `CozoStrategyStore` struct
3. Implement all trait methods with Datalog queries
4. Add index creation
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): implement CozoStrategyStore`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/store.rs`

**Steps:**
1. Write tests:
   - Test distribution CRUD
   - Test override CRUD
   - Test strategy event persistence
   - Test session cache operations
   - Test history queries
2. Run: `cargo test -p vibes-groove strategy::store`
3. Commit: `test(groove): add strategy store tests`

## Acceptance Criteria

- [ ] `InjectionStrategy` enum with all variants
- [ ] `StrategyVariant` for simplified matching
- [ ] `StrategyOutcome` and `StrategyEvent` types
- [ ] `StrategyStore` trait defines storage interface
- [ ] `CozoStrategyStore` implements persistence
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0034`
3. Commit, push, and create PR
