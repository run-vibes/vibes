---
id: FEAT0029
title: Ablation manager (Layer 3)
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0026]
estimate: 3h
created: 2026-01-09
milestone: 31-attribution-engine
---

# Ablation manager (Layer 3)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement ablation testing to run A/B experiments by withholding uncertain learnings.

## Context

Layer 3 of the attribution engine runs controlled experiments to measure true causal impact of learnings. It withholds uncertain learnings from some sessions and compares outcomes. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Define AblationStrategy trait

**Files:**
- Create: `plugins/vibes-groove/src/attribution/ablation.rs`

**Steps:**
1. Define the trait:
   ```rust
   pub trait AblationStrategy: Send + Sync {
       /// Decide if learning should be withheld from this session
       fn should_withhold(&self, learning: &Learning, value: &LearningValue) -> bool;

       /// Check if experiment has enough data
       fn is_experiment_complete(&self, experiment: &AblationExperiment) -> bool;

       /// Compute marginal value from experiment results
       fn compute_marginal_value(&self, experiment: &AblationExperiment) -> Option<AblationResult>;
   }
   ```
2. Add to `attribution/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add AblationStrategy trait`

### Task 2: Implement ConservativeAblation

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/ablation.rs`

**Steps:**
1. Create `ConservativeAblation` struct:
   ```rust
   pub struct ConservativeAblation {
       uncertainty_threshold: f64,    // only ablate if confidence < this, default 0.7
       ablation_rate: f64,            // fraction of sessions to withhold, default 0.10
       min_sessions_per_arm: usize,   // default 20
       significance_level: f64,       // p-value threshold, default 0.05
   }
   ```
2. Implement `should_withhold()`:
   - Only consider if learning confidence < uncertainty_threshold
   - Random selection based on ablation_rate
   - Return true if selected for ablation
3. Implement `is_experiment_complete()`:
   - Check sessions_with.len() >= min_sessions_per_arm
   - Check sessions_without.len() >= min_sessions_per_arm
4. Implement `compute_marginal_value()`:
   - Calculate mean outcome for with/without arms
   - Implement Welch's t-test for significance
   - Return AblationResult with marginal_value and confidence
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): implement ConservativeAblation`

### Task 3: Add experiment management

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/ablation.rs`

**Steps:**
1. Create `AblationManager` struct:
   ```rust
   pub struct AblationManager<S: AblationStrategy> {
       strategy: S,
       store: Arc<dyn AttributionStore>,
   }
   ```
2. Implement experiment lifecycle methods:
   ```rust
   impl<S: AblationStrategy> AblationManager<S> {
       /// Record session outcome for a learning
       async fn record_outcome(
           &self,
           learning_id: LearningId,
           outcome: f64,
           was_withheld: bool,
       ) -> Result<()>;

       /// Check and finalize complete experiments
       async fn check_experiments(&self) -> Result<Vec<(LearningId, AblationResult)>>;
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add AblationManager`

### Task 4: Add configuration

**Files:**
- Modify: `plugins/vibes-groove/src/config.rs`

**Steps:**
1. Add ablation config:
   ```rust
   pub struct AblationConfig {
       pub enabled: bool,
       pub uncertainty_threshold: f64,
       pub ablation_rate: f64,
       pub min_sessions_per_arm: usize,
   }

   impl Default for AblationConfig {
       fn default() -> Self {
           Self {
               enabled: true,
               uncertainty_threshold: 0.7,
               ablation_rate: 0.10,
               min_sessions_per_arm: 20,
           }
       }
   }
   ```
2. Wire into `GrooveConfig` under `attribution` section
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add ablation config`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/ablation.rs`

**Steps:**
1. Write tests:
   - Test should_withhold respects uncertainty threshold
   - Test ablation rate probability
   - Test experiment completion criteria
   - Test Welch's t-test calculation
   - Test marginal value computation
   - Test experiment lifecycle
2. Run: `cargo test -p vibes-groove attribution::ablation`
3. Commit: `test(groove): add ablation tests`

## Acceptance Criteria

- [ ] `AblationStrategy` trait defined
- [ ] `ConservativeAblation` only ablates uncertain learnings
- [ ] Welch's t-test for significance testing
- [ ] Configurable thresholds and rates
- [ ] `AblationManager` tracks experiment state
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0029`
3. Commit, push, and create PR
