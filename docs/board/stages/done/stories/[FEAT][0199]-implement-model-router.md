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

### L001: Model

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Model | **Harder than expected:** Handling | **Would do differently:** Add |
| **Suggested Action** | Add |
| **Applies To** | (to be determined) |
| **Applied** | |
