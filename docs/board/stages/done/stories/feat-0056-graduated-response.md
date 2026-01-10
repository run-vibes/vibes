---
id: FEAT0056
title: GraduatedResponse system
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0053, FEAT0055]
estimate: 3h
created: 2026-01-09
milestone: 34-open-world-adaptation
---

# GraduatedResponse system

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement progressive response to novelty with stages: Monitor → Cluster → AutoAdjust → Surface.

## Context

The GraduatedResponse system determines how to respond to novel patterns based on observation count. Early stages just monitor; later stages adjust strategy parameters; persistent novelty gets surfaced to users. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Create GraduatedResponse struct

**Files:**
- Create: `plugins/vibes-groove/src/openworld/response.rs`

**Steps:**
1. Implement `GraduatedResponse` struct:
   ```rust
   pub struct GraduatedResponse {
       novelty_detector: Arc<RwLock<NoveltyDetector>>,
       gap_detector: Arc<RwLock<CapabilityGapDetector>>,
       strategy_learner: Arc<RwLock<StrategyLearner>>,
       config: ResponseConfig,
   }
   ```
2. Implement `ResponseConfig`:
   ```rust
   pub struct ResponseConfig {
       pub monitor_threshold: u32,      // < 3 observations
       pub cluster_threshold: u32,      // 3-10 observations
       pub auto_adjust_threshold: u32,  // 10-25 observations
       pub surface_threshold: u32,      // > 25 observations

       pub exploration_adjustment: f64,
       pub max_exploration_bonus: f64,
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add GraduatedResponse struct`

### Task 2: Implement stage determination

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/response.rs`

**Steps:**
1. Implement `determine_stage()`:
   ```rust
   fn determine_stage(&self, cluster: &AnomalyCluster) -> ResponseStage {
       let count = cluster.members.len() as u32;
       match count {
           n if n < self.config.monitor_threshold => ResponseStage::Monitor,
           n if n < self.config.cluster_threshold => ResponseStage::Cluster,
           n if n < self.config.auto_adjust_threshold => ResponseStage::AutoAdjust,
           _ => ResponseStage::Surface,
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement stage determination`

### Task 3: Implement stage responses

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/response.rs`

**Steps:**
1. Implement `respond_to_cluster()`:
   ```rust
   pub async fn respond_to_cluster(
       &self,
       cluster: &AnomalyCluster,
   ) -> Result<ResponseAction> {
       let stage = self.determine_stage(cluster);

       match stage {
           ResponseStage::Monitor => Ok(ResponseAction::None),

           ResponseStage::Cluster => {
               // Just ensure clustering is running
               Ok(ResponseAction::None)
           }

           ResponseStage::AutoAdjust => {
               // Increase exploration for this context
               let adjustment = self.adjust_exploration(cluster).await?;
               Ok(ResponseAction::AdjustExploration(adjustment))
           }

           ResponseStage::Surface => {
               // Create capability gap and notify
               let gap = self.create_gap_from_cluster(cluster).await?;
               Ok(ResponseAction::CreateGap(gap))
           }
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement stage responses`

### Task 4: Implement strategy feedback

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/response.rs`

**Steps:**
1. Implement `adjust_exploration()`:
   ```rust
   async fn adjust_exploration(&self, cluster: &AnomalyCluster) -> Result<f64> {
       let mut learner = self.strategy_learner.write().await;

       // Calculate exploration bonus based on cluster size
       let bonus = (cluster.members.len() as f64 / 10.0)
           .min(self.config.max_exploration_bonus);

       // Apply to relevant category distributions
       for member in &cluster.members {
           if let Some(learning_id) = member.associated_learning {
               learner.increase_exploration(learning_id, bonus)?;
           }
       }

       Ok(bonus)
   }
   ```
2. Implement `create_gap_from_cluster()`:
   - Create CapabilityGap from persistent cluster
   - Classify category from member patterns
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement strategy feedback`

### Task 5: Implement main entry and tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/response.rs`

**Steps:**
1. Implement `respond()` main entry:
   ```rust
   pub async fn respond(
       &self,
       novelty_result: &NoveltyResult,
   ) -> Result<Vec<ResponseAction>> {
       let actions = Vec::new();

       match novelty_result {
           NoveltyResult::Novel { cluster: Some(id), .. } => {
               let cluster = self.novelty_detector.read().await
                   .get_cluster(*id)?;
               actions.push(self.respond_to_cluster(&cluster).await?);
           }
           _ => {}
       }

       // Also check gaps
       let gaps = self.gap_detector.read().await.get_active_gaps();
       for gap in gaps {
           if gap.needs_response() {
               actions.push(ResponseAction::NotifyUser(gap.summary()));
           }
       }

       Ok(actions)
   }
   ```
2. Write tests for each response stage
3. Run: `cargo test -p vibes-groove openworld::response`
4. Commit: `test(groove): add graduated response tests`

## Acceptance Criteria

- [ ] Stage determination based on observation count
- [ ] Monitor stage takes no action
- [ ] Cluster stage ensures clustering runs
- [ ] AutoAdjust stage increases exploration
- [ ] Surface stage creates capability gap
- [ ] Strategy learner receives exploration feedback
- [ ] Gap creation from persistent clusters
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0056`
3. Commit, push, and create PR
