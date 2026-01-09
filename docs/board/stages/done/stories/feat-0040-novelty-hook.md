---
id: FEAT0040
title: Novelty hook extension point
type: feat
status: done
priority: medium
epics: [plugin-system]
depends: [FEAT0039]
estimate: 1h
created: 2026-01-09
milestone: 32-adaptive-strategies
---

# Novelty hook extension point

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add clean extension point for future novelty detection capabilities.

## Context

The adaptive strategies system needs hooks for future novelty detection (identifying new patterns or learning opportunities). This story creates the extension point without implementing actual detection. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Define NoveltyHook trait

**Files:**
- Create: `plugins/vibes-groove/src/strategy/novelty.rs`

**Steps:**
1. Define the trait:
   ```rust
   /// Extension point for future novelty detection
   ///
   /// Implementations can monitor strategy outcomes for patterns that
   /// indicate new learning opportunities or strategy innovations.
   #[async_trait]
   pub trait NoveltyHook: Send + Sync {
       /// Called after each strategy outcome is processed
       async fn on_strategy_outcome(
           &self,
           learning: &Learning,
           context: &SessionContext,
           strategy: &InjectionStrategy,
           outcome: &StrategyOutcome,
       ) -> Result<()>;

       /// Called at session boundary
       async fn on_session_end(&self, session_id: SessionId) -> Result<()>;
   }
   ```
2. Add to `strategy/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add NoveltyHook trait`

### Task 2: Implement NoOpNoveltyHook

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/novelty.rs`

**Steps:**
1. Create default no-op implementation:
   ```rust
   /// Default no-op novelty hook
   ///
   /// Does nothing, used when novelty detection is disabled or not configured.
   pub struct NoOpNoveltyHook;

   #[async_trait]
   impl NoveltyHook for NoOpNoveltyHook {
       async fn on_strategy_outcome(
           &self,
           _learning: &Learning,
           _context: &SessionContext,
           _strategy: &InjectionStrategy,
           _outcome: &StrategyOutcome,
       ) -> Result<()> {
           Ok(())
       }

       async fn on_session_end(&self, _session_id: SessionId) -> Result<()> {
           Ok(())
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): add NoOpNoveltyHook`

### Task 3: Document extension point usage

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/novelty.rs`

**Steps:**
1. Add comprehensive documentation:
   ```rust
   //! Novelty detection extension point for adaptive strategies
   //!
   //! This module provides the `NoveltyHook` trait for implementing
   //! novelty detection algorithms. Novelty detection identifies:
   //!
   //! - New patterns in user behavior that could become learnings
   //! - Strategy combinations that perform unexpectedly well
   //! - Context shifts that require strategy adaptation
   //!
   //! # Future Implementation Ideas
   //!
   //! - Statistical change detection (CUSUM, ADWIN)
   //! - Clustering-based anomaly detection
   //! - Reinforcement learning for strategy exploration
   //!
   //! # Example Implementation
   //!
   //! ```rust,ignore
   //! struct StatisticalNoveltyHook {
   //!     change_detector: CusumDetector,
   //!     window_size: usize,
   //! }
   //!
   //! #[async_trait]
   //! impl NoveltyHook for StatisticalNoveltyHook {
   //!     async fn on_strategy_outcome(...) -> Result<()> {
   //!         self.change_detector.observe(outcome.value);
   //!         if self.change_detector.is_change_point() {
   //!             // Trigger strategy re-exploration
   //!         }
   //!         Ok(())
   //!     }
   //! }
   //! ```
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `docs(groove): document novelty hook extension`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/novelty.rs`

**Steps:**
1. Write tests:
   - Test NoOpNoveltyHook does nothing
   - Test trait can be implemented
   - Test hook invocation from consumer
2. Run: `cargo test -p vibes-groove strategy::novelty`
3. Commit: `test(groove): add novelty hook tests`

## Acceptance Criteria

- [x] `NoveltyHook` trait defined with clear interface
- [x] `NoOpNoveltyHook` provides default implementation
- [x] Extension point documented with future ideas
- [x] Consumer invokes hook when registered
- [x] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0040`
3. Commit, push, and create PR
