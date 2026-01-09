---
id: FEAT0022
title: Correction detector
type: feat
status: backlog
priority: high
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
milestone: 30-learning-extraction
---

# Correction detector

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Detect user corrections in transcripts to extract preference learnings.

## Context

When users correct Claude ("no, use X instead of Y"), this signals a preference worth learning. The correction detector scans transcripts for these patterns and extracts learnings. See [design.md](../../../milestones/30-learning-extraction/design.md).

## Tasks

### Task 1: Create patterns module

**Files:**
- Create: `plugins/vibes-groove/src/extraction/patterns/mod.rs`
- Create: `plugins/vibes-groove/src/extraction/patterns/correction.rs`

**Steps:**
1. Create `patterns/` directory
2. Define `PatternDetector` trait:
   ```rust
   #[async_trait]
   pub trait PatternDetector: Send + Sync {
       type Candidate;

       /// Detect patterns in a transcript
       async fn detect(&self, transcript: &Transcript) -> Result<Vec<Self::Candidate>>;

       /// Convert candidate to learning
       fn to_learning(&self, candidate: &Self::Candidate) -> Learning;
   }
   ```
3. Add module to `extraction/mod.rs`
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add PatternDetector trait`

### Task 2: Define correction patterns

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/patterns/correction.rs`

**Steps:**
1. Define default correction patterns:
   ```rust
   const DEFAULT_PATTERNS: &[&str] = &[
       r"^[Nn]o,?\s+",           // "No, ..."
       r"^[Aa]ctually,?\s+",     // "Actually, ..."
       r"[Ii] meant\s+",         // "I meant ..."
       r"[Ii] want(ed)?\s+",     // "I want ..."
       r"[Uu]se\s+.+\s+instead", // "use X instead"
       r"[Nn]ot\s+.+,\s+",       // "not X, Y"
   ];
   ```
2. Create `CorrectionCandidate` struct:
   - User message with correction
   - Claude acknowledgment (next message)
   - Extracted preference
   - Confidence score
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add correction patterns`

### Task 3: Implement CorrectionDetector

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/patterns/correction.rs`

**Steps:**
1. Create `CorrectionDetector` struct:
   - Regex patterns (compiled)
   - Custom patterns from config
2. Implement `detect()`:
   - Iterate through transcript turns
   - Find user messages matching patterns
   - Look for Claude acknowledgment in next turn
   - Extract the correction (what changed)
   - Calculate confidence based on:
     - Pattern match strength
     - Claude's acknowledgment clarity
     - Whether change was applied
3. Implement `to_learning()`:
   - Category: `Correction`
   - Description: extracted preference
   - Source: session ID, turn numbers
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement CorrectionDetector`

### Task 4: Add configuration

**Files:**
- Modify: `plugins/vibes-groove/src/config.rs`

**Steps:**
1. Add correction config:
   ```rust
   pub struct CorrectionConfig {
       pub enabled: bool,
       pub patterns: Vec<String>,  // Custom patterns
       pub min_confidence: f64,
   }
   ```
2. Add defaults
3. Wire into extraction config
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add correction detector config`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/patterns/correction.rs`

**Steps:**
1. Create test transcripts with corrections:
   - "No, use tabs not spaces"
   - "Actually, I want TypeScript"
   - "Not that file, the other one"
2. Write tests:
   - Test pattern matching
   - Test acknowledgment detection
   - Test confidence calculation
   - Test learning extraction
3. Run: `cargo test -p vibes-groove extraction::patterns::correction`
4. Commit: `test(groove): add correction detector tests`

## Acceptance Criteria

- [ ] Default correction patterns defined
- [ ] Custom patterns configurable
- [ ] Detects corrections in transcripts
- [ ] Extracts meaningful preferences
- [ ] Calculates reasonable confidence scores
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0022`
3. Commit, push, and create PR
