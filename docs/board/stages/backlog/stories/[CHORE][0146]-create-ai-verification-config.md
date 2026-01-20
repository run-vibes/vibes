---
id: CHORE0146
title: Create AI verification config and templates
type: chore
status: backlog
priority: high
scope: coherence-verification/04-ai-assisted-verification
depends: []
estimate: 1h
created: 2026-01-19
---

# Create AI verification config and templates

## Summary

Set up the configuration file and report template for AI-assisted verification.

## Acceptance Criteria

- [ ] `verification/config.toml` exists with default model configuration
- [ ] `verification/templates/ai-report.md` template exists
- [ ] Config supports `ollama:` and `claude:` model prefixes
- [ ] Confidence thresholds are configurable (high=80, medium=50)

## Implementation Notes

**SRS Requirements:** SRS-10

**Files:**
- Create: `verification/config.toml`
- Create: `verification/templates/ai-report.md`

See [DESIGN.md](../../epics/coherence-verification/milestones/04-ai-assisted-verification/DESIGN.md) for config format.
