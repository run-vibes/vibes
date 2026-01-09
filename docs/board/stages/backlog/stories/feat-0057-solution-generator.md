---
id: FEAT0057
title: SolutionGenerator
type: feat
status: pending
priority: medium
epics: [plugin-system]
depends: [FEAT0055]
estimate: 2h
created: 2026-01-09
milestone: 34-open-world-adaptation
---

# SolutionGenerator

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement solution generation for capability gaps using templates and pattern analysis.

## Context

The SolutionGenerator produces actionable suggestions for resolving capability gaps. It uses predefined templates for common gap categories and analyzes similar successful patterns to find solutions. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Create SolutionGenerator struct

**Files:**
- Create: `plugins/vibes-groove/src/openworld/solutions.rs`

**Steps:**
1. Implement `SolutionGenerator` struct:
   ```rust
   pub struct SolutionGenerator {
       templates: HashMap<GapCategory, Vec<SolutionTemplate>>,
       pattern_analyzer: PatternAnalyzer,
       config: SolutionsConfig,
   }
   ```
2. Implement `SolutionTemplate`:
   ```rust
   struct SolutionTemplate {
       action: SolutionAction,
       description: String,
       prerequisites: Vec<String>,
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add SolutionGenerator struct`

### Task 2: Implement default templates

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/solutions.rs`

**Steps:**
1. Implement `default_templates()`:
   ```rust
   fn default_templates() -> HashMap<GapCategory, Vec<SolutionTemplate>> {
       let mut templates = HashMap::new();

       templates.insert(GapCategory::MissingKnowledge, vec![
           SolutionTemplate {
               action: SolutionAction::CreateLearning {
                   content: "{context_pattern}".into(),
                   category: LearningCategory::Pattern,
               },
               description: "Create a new learning for this pattern".into(),
               prerequisites: vec![],
           },
           SolutionTemplate {
               action: SolutionAction::RequestHumanInput {
                   question: "What should happen in this context?".into(),
               },
               description: "Ask user for guidance".into(),
               prerequisites: vec![],
           },
       ]);

       templates.insert(GapCategory::IncorrectPattern, vec![
           SolutionTemplate {
               action: SolutionAction::DisableLearning { id: Default::default() },
               description: "Disable the problematic learning".into(),
               prerequisites: vec!["identified_learning".into()],
           },
           SolutionTemplate {
               action: SolutionAction::ModifyLearning {
                   id: Default::default(),
                   change: "{suggested_modification}".into(),
               },
               description: "Modify the learning to correct the pattern".into(),
               prerequisites: vec!["identified_learning".into()],
           },
       ]);

       // ... templates for other categories

       templates
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): add default solution templates`

### Task 3: Implement solution generation

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/solutions.rs`

**Steps:**
1. Implement `generate()`:
   ```rust
   pub fn generate(&self, gap: &CapabilityGap) -> Vec<SuggestedSolution> {
       let mut solutions = Vec::new();

       // Get templates for this category
       if let Some(templates) = self.templates.get(&gap.category) {
           for template in templates {
               if self.prerequisites_met(template, gap) {
                   let action = self.specialize_action(&template.action, gap);
                   solutions.push(SuggestedSolution {
                       action,
                       source: SolutionSource::Template,
                       confidence: 0.7,
                       applied: false,
                   });
               }
           }
       }

       // Find solutions from similar contexts
       solutions.extend(
           self.pattern_analyzer.find_solutions_from_similar_contexts(gap)
       );

       // Sort by confidence
       solutions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
       solutions
   }
   ```
2. Implement `specialize_action()`:
   - Fill in gap-specific details in template
   - Replace placeholders with actual values
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement solution generation`

### Task 4: Implement PatternAnalyzer

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/solutions.rs`

**Steps:**
1. Implement `PatternAnalyzer`:
   ```rust
   pub struct PatternAnalyzer {
       store: Arc<dyn OpenWorldStore>,
       embedder: Arc<dyn Embedder>,
   }

   impl PatternAnalyzer {
       pub fn find_solutions_from_similar_contexts(
           &self,
           gap: &CapabilityGap,
       ) -> Vec<SuggestedSolution> {
           // Find resolved gaps with similar context
           // Extract what worked for them
           // Adapt to current gap
       }
   }
   ```
2. Implement similarity search using embeddings
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement PatternAnalyzer`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/solutions.rs`

**Steps:**
1. Write tests:
   - Test template lookup by category
   - Test action specialization
   - Test prerequisite checking
   - Test confidence sorting
   - Test pattern analysis
2. Run: `cargo test -p vibes-groove openworld::solutions`
3. Commit: `test(groove): add solution generator tests`

## Acceptance Criteria

- [ ] Templates defined for all gap categories
- [ ] `generate()` returns specialized solutions
- [ ] Prerequisites are checked before suggesting
- [ ] Pattern analyzer finds similar resolved gaps
- [ ] Solutions sorted by confidence
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0057`
3. Commit, push, and create PR
