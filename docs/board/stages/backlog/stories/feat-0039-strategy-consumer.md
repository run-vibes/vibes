---
id: FEAT0039
title: Strategy consumer
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0036, FEAT0038]
estimate: 3h
created: 2026-01-09
milestone: 32-adaptive-strategies
---

# Strategy consumer

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement Iggy consumer that orchestrates the strategy learning pipeline.

## Context

The strategy consumer subscribes to attribution events and updates strategy distributions based on outcomes. It integrates the outcome router, distribution updater, and novelty hook. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create StrategyConsumer struct

**Files:**
- Create: `plugins/vibes-groove/src/strategy/consumer.rs`

**Steps:**
1. Define `StrategyConsumer` struct:
   ```rust
   pub struct StrategyConsumer {
       outcome_router: OutcomeRouter,
       distribution_updater: DistributionUpdater,
       store: Arc<dyn StrategyStore>,

       /// Hook for future novelty detection
       novelty_hook: Option<Box<dyn NoveltyHook>>,
   }
   ```
2. Add config struct:
   ```rust
   pub struct StrategyConsumerConfig {
       pub outcome: OutcomeRouterConfig,
       pub updater: UpdaterConfig,
       pub enabled: bool,
   }
   ```
3. Implement constructor:
   ```rust
   impl StrategyConsumer {
       pub async fn new(store: Arc<dyn StrategyStore>, config: StrategyConsumerConfig) -> Result<Self> {
           Ok(Self {
               outcome_router: OutcomeRouter::from_config(&config.outcome),
               distribution_updater: DistributionUpdater::from_config(&config.updater),
               store,
               novelty_hook: None,
           })
       }

       pub fn with_novelty_hook(mut self, hook: Box<dyn NoveltyHook>) -> Self {
           self.novelty_hook = Some(hook);
           self
       }
   }
   ```
4. Add to `strategy/mod.rs`
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): add StrategyConsumer struct`

### Task 2: Implement attribution event processing

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/consumer.rs`

**Steps:**
1. Implement `process_attribution_event()`:
   ```rust
   pub async fn process_attribution_event(&self, event: &AttributionEvent) -> Result<()> {
       // Load current distributions
       let mut distributions = self.store.load_distributions().await?;
       let mut overrides = self.store.load_overrides().await?;

       for record in &event.attribution_records {
           let learning = self.store.get_learning(record.learning_id).await?;
           let context = self.store.get_session_context(record.session_id).await?;
           let strategy = self.store.get_used_strategy(record.session_id, record.learning_id).await?;

           // Route outcome from available signals
           let outcome = self.outcome_router.compute_outcome(
               Some(record),
               &event.lightweight_events,
               &strategy,
           );

           if let Some(outcome) = outcome {
               // Update distributions
               self.distribution_updater.update(
                   &mut distributions,
                   &mut overrides,
                   &learning,
                   &context,
                   &strategy,
                   &outcome,
               );

               // Write strategy event to Iggy
               self.write_strategy_event(&learning, &strategy, &outcome).await?;

               // Invoke novelty hook if present
               if let Some(ref hook) = self.novelty_hook {
                   hook.on_strategy_outcome(&learning, &context, &strategy, &outcome).await?;
               }
           }
       }

       // Persist updated distributions
       self.store.save_distributions(&distributions).await?;
       self.store.save_overrides(&overrides).await?;

       Ok(())
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement attribution event processing`

### Task 3: Wire Iggy consumer

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/consumer.rs`

**Steps:**
1. Implement Iggy subscription setup:
   ```rust
   pub async fn start(&self, iggy: &IggyClient) -> Result<JoinHandle<()>> {
       // Subscribe to groove.attribution
       // Resume from last acknowledged offset
       // Spawn consumer task
   }

   async fn write_strategy_event(
       &self,
       learning: &Learning,
       strategy: &InjectionStrategy,
       outcome: &StrategyOutcome,
   ) -> Result<()> {
       // Write to groove.strategy topic
   }
   ```
2. Add Iggy topic `groove.strategy` for output events
3. Implement consumer loop with error handling
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): wire Iggy consumer`

### Task 4: Integrate with plugin lifecycle

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add `StrategyConsumer` to `GroovePlugin`
2. Start consumer in plugin initialization
3. Graceful shutdown handling
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): integrate strategy consumer`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/consumer.rs`

**Steps:**
1. Write tests:
   - Test attribution event processing pipeline
   - Test strategy event creation
   - Test distribution persistence
   - Test novelty hook invocation
   - Test error handling
2. Run: `cargo test -p vibes-groove strategy::consumer`
3. Commit: `test(groove): add consumer tests`

## Acceptance Criteria

- [ ] Subscribes to `groove.attribution`
- [ ] Routes outcomes through pipeline
- [ ] Updates distributions and overrides
- [ ] Publishes to `groove.strategy`
- [ ] Invokes novelty hook if registered
- [ ] Integrated with plugin lifecycle
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0039`
3. Commit, push, and create PR
