---
id: FEAT0063
title: Configuration and wiring
type: feat
status: done
priority: high
scope: plugin-system
depends: [FEAT0060]
estimate: 2h
created: 2026-01-09
---

# Configuration and wiring

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Wire all open-world components together with comprehensive configuration.

## Context

This is the final story for milestone 34. It connects all open-world components into the groove plugin with configurable parameters. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Define configuration structs

**Files:**
- Modify: `plugins/vibes-groove/src/config.rs`

**Steps:**
1. Add `OpenWorldConfig`:
   ```rust
   #[derive(Debug, Clone, Deserialize)]
   pub struct OpenWorldConfig {
       pub enabled: bool,
       pub novelty: NoveltyConfig,
       pub gaps: GapsConfig,
       pub response: ResponseConfig,
       pub solutions: SolutionsConfig,
   }

   #[derive(Debug, Clone, Deserialize)]
   pub struct NoveltyConfig {
       /// Initial similarity threshold for novelty detection
       pub similarity_threshold: f64,
       /// Minimum pending outliers before clustering
       pub min_pending_for_cluster: usize,
       /// DBSCAN epsilon parameter
       pub eps: f32,
       /// DBSCAN min_points parameter
       pub min_points: usize,
       /// Cache known hashes in memory
       pub cache_known_hashes: bool,
   }

   #[derive(Debug, Clone, Deserialize)]
   pub struct GapsConfig {
       /// Minimum failures before creating gap
       pub min_failures_for_gap: u32,
       /// Confidence threshold for failure detection
       pub failure_confidence_threshold: f64,
       /// Auto-escalate severity
       pub auto_escalate: bool,
   }

   #[derive(Debug, Clone, Deserialize)]
   pub struct ResponseConfig {
       pub monitor_threshold: u32,
       pub cluster_threshold: u32,
       pub auto_adjust_threshold: u32,
       pub surface_threshold: u32,
       pub exploration_adjustment: f64,
       pub max_exploration_bonus: f64,
   }

   #[derive(Debug, Clone, Deserialize)]
   pub struct SolutionsConfig {
       /// Enable automatic solution generation
       pub auto_generate: bool,
       /// Maximum solutions per gap
       pub max_solutions: usize,
       /// Minimum confidence for pattern-based solutions
       pub pattern_min_confidence: f64,
   }
   ```
2. Add default implementations
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add OpenWorldConfig`

### Task 2: Wire components in plugin

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Initialize open-world components:
   ```rust
   // In GroovePlugin::new() or init()
   let openworld_config = config.openworld.clone();

   if openworld_config.enabled {
       // NoveltyDetector (uses M30's embedder)
       let novelty_detector = Arc::new(RwLock::new(
           NoveltyDetector::new(
               embedder.clone(),
               store.clone(),
               openworld_config.novelty,
           )
       ));

       // CapabilityGapDetector
       let gap_detector = Arc::new(RwLock::new(
           CapabilityGapDetector::new(
               store.clone(),
               openworld_config.gaps,
           )
       ));

       // SolutionGenerator
       let solution_generator = Arc::new(
           SolutionGenerator::new(
               store.clone(),
               embedder.clone(),
               openworld_config.solutions,
           )
       );

       // GraduatedResponse
       let graduated_response = Arc::new(
           GraduatedResponse::new(
               novelty_detector.clone(),
               gap_detector.clone(),
               strategy_learner.clone(),
               openworld_config.response,
           )
       );

       // OpenWorldHook
       let openworld_hook = Arc::new(OpenWorldHook::new(
           novelty_detector,
           gap_detector,
           graduated_response,
           solution_generator,
           producer,
       ));

       // Wire into strategy consumer
       strategy_consumer = strategy_consumer.with_novelty_hook(openworld_hook);
   }
   ```
2. Store references for CLI access
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): wire openworld components`

### Task 3: Add documentation

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/mod.rs`

**Steps:**
1. Add module-level documentation:
   - Overview of open-world adaptation
   - Component interaction diagram
   - Configuration reference
2. Document public types and traits
3. Run: `cargo doc -p vibes-groove --no-deps`
4. Commit: `docs(groove): add openworld documentation`

### Task 4: Add integration test

**Files:**
- Create: `plugins/vibes-groove/tests/openworld_integration.rs`

**Steps:**
1. Write full pipeline integration test:
   ```rust
   #[tokio::test]
   async fn test_openworld_full_pipeline() {
       // Setup plugin with openworld enabled
       let plugin = test_plugin_with_openworld();

       // Simulate session outcomes
       // 1. First novel context
       // 2. Repeated novel contexts (clustering)
       // 3. Negative outcomes (gap detection)
       // 4. Check graduated response
       // 5. Verify solution generation

       // Assert events emitted
       // Assert strategy learner adjusted
       // Assert gap created with solutions
   }
   ```
2. Run: `cargo test -p vibes-groove openworld::integration`
3. Commit: `test(groove): add openworld integration test`

### Task 5: Final verification

**Files:**
- None (verification only)

**Steps:**
1. Run full test suite:
   ```bash
   just test
   ```
2. Run pre-commit checks:
   ```bash
   just pre-commit
   ```
3. Verify CLI commands work:
   ```bash
   cargo build -p vibes-groove
   # Manual testing of commands
   ```
4. Commit: `chore(groove): complete milestone 34`

## Acceptance Criteria

- [x] `OpenWorldConfig` with all nested configs
- [x] All components wired in plugin initialization
- [x] Components only created when enabled
- [x] Strategy consumer receives OpenWorldHook
- [x] Module documentation complete
- [x] Integration test passes
- [x] All tests pass
- [x] Pre-commit checks pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0063`
3. Update milestone status: `just board done-milestone 34-open-world-adaptation`
4. Commit, push, and create PR
