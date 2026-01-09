---
id: FEAT0036
title: Thompson sampling learner
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0035]
estimate: 3h
created: 2026-01-09
milestone: 32-adaptive-strategies
---

# Thompson sampling learner

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement strategy selection via Thompson sampling with lazy caching.

## Context

The strategy learner selects injection strategies by sampling from Beta posteriors (Thompson sampling), balancing exploration and exploitation. Uses lazy evaluation with per-session caching. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create StrategyLearner struct

**Files:**
- Create: `plugins/vibes-groove/src/strategy/learner.rs`

**Steps:**
1. Define `StrategyLearner` struct:
   ```rust
   pub struct StrategyLearner {
       /// Category-level distributions (the priors)
       category_distributions: HashMap<(LearningCategory, ContextType), StrategyDistribution>,

       /// Per-learning overrides (specializations)
       learning_overrides: HashMap<LearningId, LearningStrategyOverride>,

       /// Session cache for lazy + consistent selection
       session_cache: HashMap<(SessionId, LearningId), InjectionStrategy>,

       config: StrategyLearnerConfig,
   }
   ```
2. Define config:
   ```rust
   pub struct StrategyLearnerConfig {
       pub exploration_bonus: f64,
       pub min_samples_for_confidence: u32,
       pub specialization_threshold: u32,
       pub specialization_confidence: f64,
   }
   ```
3. Add to `strategy/mod.rs`
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add StrategyLearner struct`

### Task 2: Implement lazy strategy selection

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/learner.rs`

**Steps:**
1. Implement `select_strategy()`:
   ```rust
   pub async fn select_strategy(
       &mut self,
       learning: &Learning,
       context: &SessionContext,
   ) -> InjectionStrategy {
       let cache_key = (context.session_id, learning.id);

       // Return cached if exists
       if let Some(cached) = self.session_cache.get(&cache_key) {
           return cached.clone();
       }

       // Sample new strategy
       let strategy = self.sample_strategy(learning, context);
       self.session_cache.insert(cache_key, strategy.clone());
       strategy
   }
   ```
2. Implement `clear_session()`:
   ```rust
   pub fn clear_session(&mut self, session_id: SessionId) {
       self.session_cache.retain(|(sid, _), _| *sid != session_id);
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement lazy strategy selection`

### Task 3: Implement Thompson sampling

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/learner.rs`

**Steps:**
1. Implement `sample_strategy()`:
   ```rust
   fn sample_strategy(&self, learning: &Learning, context: &SessionContext) -> InjectionStrategy {
       let weights = self.get_effective_weights(learning, context);

       // Thompson sampling: sample from each Beta posterior, pick highest
       let selected_variant = weights
           .iter()
           .map(|(variant, param)| (*variant, param.sample()))
           .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
           .map(|(v, _)| v)
           .unwrap_or(StrategyVariant::Deferred);

       // Sample parameters for the selected strategy
       self.sample_params(selected_variant, learning, context)
   }
   ```
2. Implement `get_effective_weights()`:
   ```rust
   fn get_effective_weights(&self, learning: &Learning, context: &SessionContext)
       -> &HashMap<StrategyVariant, AdaptiveParam>
   {
       // Use learning override if specialized, otherwise category distribution
       if let Some(override_) = self.learning_overrides.get(&learning.id) {
           if let Some(ref specialized) = override_.specialized_weights {
               return specialized;
           }
       }

       let key = (learning.category, context.context_type);
       &self.category_distributions.get(&key).unwrap().strategy_weights
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement Thompson sampling`

### Task 4: Implement parameter sampling

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/learner.rs`

**Steps:**
1. Implement `sample_params()`:
   ```rust
   fn sample_params(
       &self,
       variant: StrategyVariant,
       learning: &Learning,
       context: &SessionContext,
   ) -> InjectionStrategy {
       match variant {
           StrategyVariant::MainContext => {
               InjectionStrategy::MainContext {
                   position: self.sample_position(learning),
                   format: self.sample_format(learning),
               }
           }
           StrategyVariant::Subagent => {
               InjectionStrategy::Subagent {
                   agent_type: self.sample_agent_type(context),
                   blocking: self.sample_blocking(),
                   prompt_template: None,
               }
           }
           StrategyVariant::BackgroundSubagent => {
               InjectionStrategy::BackgroundSubagent {
                   agent_type: self.sample_agent_type(context),
                   callback: CallbackMethod::Interrupt,
                   timeout_ms: 30_000,
               }
           }
           StrategyVariant::Deferred => {
               InjectionStrategy::Deferred {
                   trigger: DeferralTrigger::Explicit,
                   max_wait_ms: None,
               }
           }
       }
   }
   ```
2. Add helper samplers for each parameter
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement parameter sampling`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/learner.rs`

**Steps:**
1. Write tests:
   - Test caching returns same strategy within session
   - Test Thompson sampling selects from distribution
   - Test exploration/exploitation balance
   - Test hierarchy lookup
   - Test session cache cleanup
2. Run: `cargo test -p vibes-groove strategy::learner`
3. Commit: `test(groove): add learner tests`

## Acceptance Criteria

- [ ] `StrategyLearner` with lazy caching
- [ ] Thompson sampling from Beta posteriors
- [ ] Effective weights from hierarchy
- [ ] Parameter sampling for each variant
- [ ] Session cache cleanup
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0036`
3. Commit, push, and create PR
