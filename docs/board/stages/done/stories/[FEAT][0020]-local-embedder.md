---
id: FEAT0020
title: Local embedder
type: feat
status: done
priority: high
epics: [plugin-system]
depends: []
estimate: 3h
created: 2026-01-08
updated: 2026-01-09
milestone: 30-learning-extraction
---

# Local embedder

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement local embedding using the gte-small model for semantic similarity and deduplication.

## Context

The embedder generates 384-dimensional vectors for learnings, enabling semantic search via HNSW index. Uses ONNX Runtime for inference. See [design.md](../../../milestones/30-learning-extraction/design.md) for configuration.

## Tasks

### Task 1: Add dependencies

**Files:**
- Modify: `plugins/vibes-groove/Cargo.toml`

**Steps:**
1. Add dependencies:
   ```toml
   ort = { version = "2", features = ["download-binaries"] }
   tokenizers = "0.19"
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `chore(groove): add onnx and tokenizer dependencies`

### Task 2: Create Embedder trait

**Files:**
- Create: `plugins/vibes-groove/src/extraction/embedder.rs`

**Steps:**
1. Define `Embedder` trait:
   ```rust
   #[async_trait]
   pub trait Embedder: Send + Sync {
       async fn embed(&self, text: &str) -> Result<Vec<f32>>;
       async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
       fn dimensions(&self) -> usize;
   }
   ```
2. Add module to `extraction/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add Embedder trait`

### Task 3: Implement LocalEmbedder

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/embedder.rs`

**Steps:**
1. Create `LocalEmbedder` struct:
   - Model path (default `~/.cache/vibes/models/gte-small/`)
   - ONNX session
   - Tokenizer
2. Implement model loading:
   - Check if model exists in cache
   - Download from HuggingFace if missing
   - Show progress indicator
3. Implement `embed()`:
   - Tokenize input text
   - Run ONNX inference
   - Apply mean pooling to get single vector
4. Implement `embed_batch()`:
   - Batch tokenization
   - Batch inference
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): implement LocalEmbedder with gte-small`

### Task 4: Add model download

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/embedder.rs`

**Steps:**
1. Implement `download_model()`:
   - Download `model.onnx` from HuggingFace
   - Download `tokenizer.json`
   - Show download progress
   - Verify file integrity
2. Add `health_check()` method:
   - Verify model loads
   - Run test embedding
   - Return model info
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add model download and health check`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/embedder.rs`

**Steps:**
1. Write unit tests:
   - Test embedding dimensions (384)
   - Test embedding determinism (same input = same output)
   - Test batch embedding
   - Test cosine similarity properties
2. Run: `cargo test -p vibes-groove extraction::embedder`
3. Commit: `test(groove): add embedder tests`

## Acceptance Criteria

- [ ] `LocalEmbedder` loads gte-small model
- [ ] Model auto-downloads on first use
- [ ] `embed()` returns 384-dimensional vector
- [ ] `embed_batch()` handles multiple texts efficiently
- [ ] `health_check()` verifies model is working
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0020`
3. Commit, push, and create PR
