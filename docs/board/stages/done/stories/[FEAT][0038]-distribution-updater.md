---
id: FEAT0038
title: Distribution updater
type: feat
status: done
priority: high
scope: plugin-system
depends: [FEAT0035, FEAT0037]
estimate: 2h
created: 2026-01-09
---

# Distribution updater

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement distribution updates based on strategy outcomes.

## Context

The distribution updater applies outcomes to both category distributions and per-learning overrides, handling the specialization trigger when a learning accumulates enough data. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create DistributionUpdater struct

**Files:**
- Create: `plugins/vibes-groove/src/strategy/updater.rs`

**Steps:**
1. Define `DistributionUpdater` struct:
   ```rust
   pub struct DistributionUpdater {
       specialization_threshold: u32,  // Sessions before learning specializes (default: 20)
       specialization_confidence: f64, // Minimum confidence (default: 0.6)
   }

   impl Default for DistributionUpdater {
       fn default() -> Self {
           Self {
               specialization_threshold: 20,
               specialization_confidence: 0.6,
           }
       }
   }
   ```
2. Add config struct:
   ```rust
   pub struct UpdaterConfig {
       pub specialization_threshold: u32,
       pub specialization_confidence: f64,
   }
   ```
3. Add to `strategy/mod.rs`
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add DistributionUpdater struct`

### Task 2: Implement main update logic

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/updater.rs`

**Steps:**
1. Implement `update()`:
   ```rust
   pub fn update(
       &self,
       distributions: &mut HashMap<(LearningCategory, ContextType), StrategyDistribution>,
       overrides: &mut HashMap<LearningId, LearningStrategyOverride>,
       learning: &Learning,
       context: &SessionContext,
       strategy: &InjectionStrategy,
       outcome: &StrategyOutcome,
   ) {
       let variant = StrategyVariant::from(strategy);
       let key = (learning.category, context.context_type);

       // Always update category distribution
       self.update_category_distribution(distributions, &key, variant, outcome);

       // Update or create learning override
       self.update_learning_override(overrides, distributions, learning, &key, variant, outcome);
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement main update logic`

### Task 3: Implement category distribution update

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/updater.rs`

**Steps:**
1. Implement `update_category_distribution()`:
   ```rust
   fn update_category_distribution(
       &self,
       distributions: &mut HashMap<(LearningCategory, ContextType), StrategyDistribution>,
       key: &(LearningCategory, ContextType),
       variant: StrategyVariant,
       outcome: &StrategyOutcome,
   ) {
       if let Some(dist) = distributions.get_mut(key) {
           if let Some(param) = dist.strategy_weights.get_mut(&variant) {
               param.update(outcome.value, outcome.confidence);
           }
           dist.session_count += 1;
           dist.updated_at = Utc::now();
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement category update`

### Task 4: Implement learning override update with specialization

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/updater.rs`

**Steps:**
1. Implement `update_learning_override()`:
   ```rust
   fn update_learning_override(
       &self,
       overrides: &mut HashMap<LearningId, LearningStrategyOverride>,
       distributions: &HashMap<(LearningCategory, ContextType), StrategyDistribution>,
       learning: &Learning,
       key: &(LearningCategory, ContextType),
       variant: StrategyVariant,
       outcome: &StrategyOutcome,
   ) {
       // Create override if doesn't exist
       let override_ = overrides.entry(learning.id).or_insert_with(|| {
           LearningStrategyOverride::new(learning.id, learning.category)
       });
       override_.session_count += 1;

       // Check if should specialize
       if override_.session_count >= self.specialization_threshold
           && override_.specialized_weights.is_none()
           && outcome.confidence >= self.specialization_confidence
       {
           if let Some(dist) = distributions.get(key) {
               override_.specialize_from(dist);
           }
       }

       // Update specialized weights if they exist
       if let Some(ref mut specialized) = override_.specialized_weights {
           if let Some(param) = specialized.get_mut(&variant) {
               param.update(outcome.value, outcome.confidence);
           }
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement specialization trigger`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/updater.rs`

**Steps:**
1. Write tests:
   - Test category distribution update
   - Test learning override creation
   - Test specialization threshold trigger
   - Test specialized weights update
   - Test confidence requirement for specialization
   - Test session count tracking
2. Run: `cargo test -p vibes-groove strategy::updater`
3. Commit: `test(groove): add updater tests`

## Acceptance Criteria

- [ ] `DistributionUpdater` updates both levels
- [ ] Category distribution always updated
- [ ] Learning override created on first encounter
- [ ] Specialization triggers at threshold
- [ ] Specialized weights updated independently
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0038`
3. Commit, push, and create PR
