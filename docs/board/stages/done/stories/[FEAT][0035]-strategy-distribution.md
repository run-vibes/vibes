---
id: FEAT0035
title: Strategy distribution hierarchy
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0034]
estimate: 2h
created: 2026-01-09
milestone: 32-adaptive-strategies
---

# Strategy distribution hierarchy

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement hierarchical distributions with category-level priors and per-learning specialization.

## Context

The adaptive strategies system uses a two-level hierarchy: category distributions provide cold-start defaults, while individual learnings specialize over time. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create StrategyDistribution

**Files:**
- Create: `plugins/vibes-groove/src/strategy/distribution.rs`

**Steps:**
1. Define `StrategyDistribution` struct:
   ```rust
   pub struct StrategyDistribution {
       pub category: LearningCategory,
       pub context_type: ContextType,

       /// Weights for each strategy variant (Beta distributions via AdaptiveParam)
       pub strategy_weights: HashMap<StrategyVariant, AdaptiveParam>,

       /// Parameters within each strategy (also adaptive)
       pub strategy_params: HashMap<StrategyVariant, StrategyParams>,

       pub session_count: u32,
       pub updated_at: DateTime<Utc>,
   }
   ```
2. Implement constructor with default priors:
   ```rust
   impl StrategyDistribution {
       pub fn new(category: LearningCategory, context_type: ContextType, config: &StrategyConfig) -> Self {
           // Initialize with default weights from config
           // MainContext: 0.3, Subagent: 0.2, Background: 0.1, Deferred: 0.4
       }

       pub fn get_weight(&self, variant: StrategyVariant) -> &AdaptiveParam { ... }
       pub fn update_weight(&mut self, variant: StrategyVariant, value: f64, confidence: f64) { ... }
   }
   ```
3. Add serialization for CozoDB storage
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add StrategyDistribution`

### Task 2: Create LearningStrategyOverride

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/distribution.rs`

**Steps:**
1. Define `LearningStrategyOverride` struct:
   ```rust
   pub struct LearningStrategyOverride {
       pub learning_id: LearningId,
       pub base_category: LearningCategory,

       /// Only populated once learning has enough data
       pub specialized_weights: Option<HashMap<StrategyVariant, AdaptiveParam>>,
       pub specialization_threshold: u32,

       pub session_count: u32,
   }
   ```
2. Implement constructor:
   ```rust
   impl LearningStrategyOverride {
       pub fn new(learning_id: LearningId, category: LearningCategory) -> Self {
           Self {
               learning_id,
               base_category: category,
               specialized_weights: None,
               specialization_threshold: 20,
               session_count: 0,
           }
       }
   }
   ```
3. Implement specialization:
   ```rust
   pub fn specialize_from(&mut self, category_dist: &StrategyDistribution) {
       // Copy weights from category distribution
       // These will now diverge as this learning accumulates data
       self.specialized_weights = Some(category_dist.strategy_weights.clone());
   }

   pub fn is_specialized(&self) -> bool {
       self.specialized_weights.is_some()
   }
   ```
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add LearningStrategyOverride`

### Task 3: Implement effective weights lookup

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/distribution.rs`

**Steps:**
1. Add helper function for weight resolution:
   ```rust
   pub fn get_effective_weights<'a>(
       override_: Option<&'a LearningStrategyOverride>,
       category_dist: &'a StrategyDistribution,
   ) -> &'a HashMap<StrategyVariant, AdaptiveParam> {
       // Use learning override if specialized, otherwise category distribution
       if let Some(override_) = override_ {
           if let Some(ref specialized) = override_.specialized_weights {
               return specialized;
           }
       }
       &category_dist.strategy_weights
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): add effective weights lookup`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/distribution.rs`

**Steps:**
1. Write tests:
   - Test default distribution initialization
   - Test weight update propagation
   - Test specialization trigger
   - Test effective weights delegation
   - Test serialization round-trip
2. Run: `cargo test -p vibes-groove strategy::distribution`
3. Commit: `test(groove): add distribution tests`

## Acceptance Criteria

- [ ] `StrategyDistribution` with configurable default priors
- [ ] `LearningStrategyOverride` inherits from category
- [ ] Specialization copies and diverges weights
- [ ] Effective weights lookup handles hierarchy
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0035`
3. Commit, push, and create PR
