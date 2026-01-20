---
id: FEAT0199
title: Implement model router for Ollama and Claude
type: feat
status: done
priority: high
scope: coherence-verification/04-ai-assisted-verification
depends: [FEAT0198]
estimate: 4h
created: 2026-01-19
---

# Implement model router for Ollama and Claude

## Summary

Create TypeScript module to route artifacts to configured multimodal models (Ollama or Claude) and return verdicts with confidence levels.

## Acceptance Criteria

- [ ] Router loads model config from `verification/config.toml`
- [ ] Router supports Ollama models via `ollama` npm package
- [ ] Router supports Claude models via `@anthropic-ai/sdk`
- [ ] Router accepts `--model` override from command line
- [ ] Returns verdict (pass/fail/unclear) with confidence (0-100)
- [ ] Returns evidence text and optional suggestion

## Implementation Notes

**SRS Requirements:** SRS-04, SRS-05, SRS-06, SRS-07, SRS-11

**Files:**
- Create: `verification/scripts/lib/router.ts`
- Update: `package.json` with `ollama`, `@anthropic-ai/sdk`, `toml` dependencies

See [DESIGN.md](../../epics/coherence-verification/milestones/04-ai-assisted-verification/DESIGN.md) for prompt structure.

## Learnings

### L001: Model router abstracts provider differences cleanly

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Implementing router to support both Ollama and Claude APIs |
| **Insight** | **What went well:** A unified interface for different model providers makes it easy to switch between local (Ollama) and cloud (Claude) models • **Harder than expected:** Error handling differs significantly between providers - Ollama fails silently while Claude throws detailed errors • **Would do differently:** Add retry logic with exponential backoff from the start |
| **Suggested Action** | When building multi-provider abstractions, standardize error handling and add retry logic early |
| **Applies To** | Any code that wraps multiple AI/LLM providers |
| **Applied** | |
