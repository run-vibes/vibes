---
id: FEAT0052
title: Core types and traits
type: feat
status: done
priority: high
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-09
milestone: 34-open-world-adaptation
---

# Core types and traits

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Define all core types for open-world adaptation including novelty detection, capability gaps, and response actions.

## Context

The open-world adaptation system detects unknown patterns and surfaces capability gaps. This story defines the foundational types used throughout the system. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Create openworld module

**Files:**
- Create: `plugins/vibes-groove/src/openworld/mod.rs`
- Create: `plugins/vibes-groove/src/openworld/types.rs`

**Steps:**
1. Create openworld module directory
2. Define pattern types:
   ```rust
   pub struct PatternFingerprint {
       pub hash: u64,                    // Fast pre-filter
       pub embedding: Vec<f32>,          // From M30 embedder
       pub context_summary: String,      // Human-readable
       pub created_at: DateTime<Utc>,
   }

   pub struct AnomalyCluster {
       pub id: ClusterId,
       pub centroid: Vec<f32>,
       pub members: Vec<PatternFingerprint>,
       pub created_at: DateTime<Utc>,
       pub last_seen: DateTime<Utc>,
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add openworld pattern types`

### Task 2: Define novelty and gap types

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/types.rs`

**Steps:**
1. Define novelty result types:
   ```rust
   pub enum NoveltyResult {
       Known { fingerprint: PatternFingerprint },
       Novel { cluster: Option<ClusterId>, embedding: Vec<f32> },
       PendingClassification { embedding: Vec<f32> },
   }
   ```
2. Define capability gap types:
   ```rust
   pub struct CapabilityGap {
       pub id: GapId,
       pub category: GapCategory,
       pub severity: GapSeverity,
       pub status: GapStatus,
       pub context_pattern: String,
       pub failure_count: u32,
       pub first_seen: DateTime<Utc>,
       pub last_seen: DateTime<Utc>,
       pub suggested_solutions: Vec<SuggestedSolution>,
   }

   pub enum GapCategory {
       MissingKnowledge,
       IncorrectPattern,
       ContextMismatch,
       ToolGap,
   }

   pub enum GapSeverity {
       Low,      // < 3 failures
       Medium,   // 3-10 failures
       High,     // > 10 failures
       Critical, // User escalated
   }

   pub enum GapStatus {
       Detected,
       Confirmed,
       InProgress,
       Resolved,
       Dismissed,
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add openworld gap types`

### Task 3: Define failure and solution types

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/types.rs`

**Steps:**
1. Define failure types:
   ```rust
   pub struct FailureRecord {
       pub id: FailureId,
       pub session_id: SessionId,
       pub failure_type: FailureType,
       pub context_hash: u64,
       pub learning_ids: Vec<LearningId>,
       pub timestamp: DateTime<Utc>,
   }

   pub enum FailureType {
       LearningNotActivated,  // Should have been, wasn't
       NegativeAttribution,   // Activated but hurt
       LowConfidence,         // Uncertain outcome
       ExplicitFeedback,      // User marked as wrong
   }
   ```
2. Define solution types:
   ```rust
   pub struct SuggestedSolution {
       pub action: SolutionAction,
       pub source: SolutionSource,
       pub confidence: f64,
       pub applied: bool,
   }

   pub enum SolutionAction {
       CreateLearning { content: String, category: LearningCategory },
       ModifyLearning { id: LearningId, change: String },
       DisableLearning { id: LearningId },
       AdjustStrategy { category: LearningCategory, change: StrategyChange },
       RequestHumanInput { question: String },
   }

   pub enum SolutionSource {
       Template,          // From predefined templates
       PatternAnalysis,   // From similar contexts
       UserSuggestion,    // From user feedback
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add openworld solution types`

### Task 4: Define response and event types

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/types.rs`
- Create: `plugins/vibes-groove/src/openworld/traits.rs`

**Steps:**
1. Define response types:
   ```rust
   pub enum ResponseAction {
       None,                           // Just monitor
       AdjustExploration(f64),        // Increase/decrease exploration
       CreateGap(CapabilityGap),      // Promote to gap
       SuggestSolution(SuggestedSolution),
       NotifyUser(String),            // Surface to dashboard
   }

   pub enum ResponseStage {
       Monitor,           // < 3 observations
       Cluster,           // 3-10 observations
       AutoAdjust,        // 10-25 observations
       Surface,           // > 25 observations
   }
   ```
2. Define event types for Iggy:
   ```rust
   pub enum OpenWorldEvent {
       NoveltyDetected { fingerprint: PatternFingerprint, cluster: Option<ClusterId> },
       ClusterUpdated { cluster: AnomalyCluster },
       GapCreated { gap: CapabilityGap },
       GapStatusChanged { gap_id: GapId, old: GapStatus, new: GapStatus },
       SolutionGenerated { gap_id: GapId, solution: SuggestedSolution },
       StrategyFeedback { learning_id: LearningId, adjustment: f64 },
   }
   ```
3. Create traits.rs with `OpenWorldStore` trait
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add openworld response and event types`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/types.rs`

**Steps:**
1. Write serialization tests for all types
2. Write conversion tests
3. Run: `cargo test -p vibes-groove openworld::types`
4. Commit: `test(groove): add openworld type tests`

## Acceptance Criteria

- [ ] `PatternFingerprint` and `AnomalyCluster` types defined
- [ ] `NoveltyResult` enum for detection outcomes
- [ ] `CapabilityGap` with category, severity, status
- [ ] `FailureRecord` and `FailureType` for tracking
- [ ] `SuggestedSolution` and `SolutionAction` for remediation
- [ ] `ResponseAction` and `ResponseStage` for graduated response
- [ ] `OpenWorldEvent` for Iggy streaming
- [ ] `OpenWorldStore` trait defined
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0052`
3. Commit, push, and create PR
