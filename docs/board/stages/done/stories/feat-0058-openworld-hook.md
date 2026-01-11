---
id: FEAT0058
title: OpenWorldHook (M32 integration)
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0056, FEAT0057]
estimate: 3h
created: 2026-01-09
milestone: 34-open-world-adaptation
---

# OpenWorldHook (M32 integration)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement NoveltyHook trait for M32 integration, connecting open-world detection to strategy learning.

## Context

The OpenWorldHook implements M32's NoveltyHook trait, creating a closed feedback loop. When strategy outcomes are processed, the hook runs novelty detection and gap analysis, feeding back exploration adjustments to the strategy learner. See [design.md](../../../milestones/34-open-world-adaptation/design.md) and [M32 design](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create OpenWorldHook struct

**Files:**
- Create: `plugins/vibes-groove/src/openworld/hook.rs`

**Steps:**
1. Implement `OpenWorldHook` struct:
   ```rust
   pub struct OpenWorldHook {
       novelty_detector: Arc<RwLock<NoveltyDetector>>,
       gap_detector: Arc<RwLock<CapabilityGapDetector>>,
       graduated_response: Arc<GraduatedResponse>,
       solution_generator: Arc<SolutionGenerator>,
       producer: Arc<OpenWorldProducer>,
   }
   ```
2. Implement constructor
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add OpenWorldHook struct`

### Task 2: Implement NoveltyHook trait

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/hook.rs`

**Steps:**
1. Implement `NoveltyHook` trait:
   ```rust
   #[async_trait]
   impl NoveltyHook for OpenWorldHook {
       async fn on_strategy_outcome(
           &self,
           learning: &Learning,
           context: &SessionContext,
           strategy: &InjectionStrategy,
           outcome: &StrategyOutcome,
       ) -> Result<()> {
           // Step 1: Novelty detection
           let novelty_result = self.novelty_detector.write().await
               .detect(context).await?;

           // Step 2: Gap detection
           let session_outcome = SessionOutcome::from_strategy_outcome(outcome);
           let gap = self.gap_detector.write().await
               .process_outcome(&session_outcome, &[]).await?;

           // Step 3: Graduated response
           let actions = self.graduated_response
               .respond(&novelty_result).await?;

           // Step 4: Execute response actions
           for action in actions {
               self.execute_action(action).await?;
           }

           // Step 5: Generate solutions for new gaps
           if let Some(gap) = gap {
               let solutions = self.solution_generator.generate(&gap);
               self.update_gap_solutions(gap.id, solutions).await?;
           }

           Ok(())
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement NoveltyHook trait`

### Task 3: Implement action execution

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/hook.rs`

**Steps:**
1. Implement `execute_action()`:
   ```rust
   async fn execute_action(&self, action: ResponseAction) -> Result<()> {
       match action {
           ResponseAction::None => Ok(()),

           ResponseAction::AdjustExploration(adjustment) => {
               self.feedback_to_strategy_learner(adjustment).await
           }

           ResponseAction::CreateGap(gap) => {
               self.producer.emit_gap_created(&gap).await?;
               Ok(())
           }

           ResponseAction::SuggestSolution(solution) => {
               // Solutions are attached to gaps, not directly executed
               Ok(())
           }

           ResponseAction::NotifyUser(message) => {
               self.producer.emit_user_notification(&message).await?;
               Ok(())
           }
       }
   }
   ```
2. Implement `feedback_to_strategy_learner()`:
   - Adjust category priors based on gap feedback
   - Increase exploration for problematic contexts
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement action execution`

### Task 4: Wire into M32 StrategyConsumer

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/consumer.rs`
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Update `StrategyConsumer` to accept `OpenWorldHook`:
   ```rust
   impl StrategyConsumer {
       pub fn with_novelty_hook(mut self, hook: Arc<dyn NoveltyHook>) -> Self {
           self.novelty_hook = Some(hook);
           self
       }
   }
   ```
2. Wire in plugin initialization:
   ```rust
   let openworld_hook = Arc::new(OpenWorldHook::new(...));
   let strategy_consumer = StrategyConsumer::new(...)
       .with_novelty_hook(openworld_hook);
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): wire OpenWorldHook into StrategyConsumer`

### Task 5: Add integration tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/hook.rs`

**Steps:**
1. Write integration tests:
   - Test full pipeline from outcome to response
   - Test novelty detection triggers exploration adjustment
   - Test gap creation triggers solution generation
   - Test feedback loop to strategy learner
2. Run: `cargo test -p vibes-groove openworld::hook`
3. Commit: `test(groove): add OpenWorldHook integration tests`

## Acceptance Criteria

- [x] `OpenWorldHook` implements `NoveltyHook` trait
- [x] `on_strategy_outcome()` runs full pipeline
- [x] Response actions are executed correctly
- [x] Strategy learner receives exploration feedback
- [x] Gap solutions are generated and attached
- [x] Wired into M32 `StrategyConsumer`
- [x] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0058`
3. Commit, push, and create PR
