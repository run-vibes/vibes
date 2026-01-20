---
id: FEAT0208
title: Add CLI options for non-interactive learning capture
type: feat
status: done
priority: high
scope: coherence-verification/05-learnings-capture
depends: [FEAT0203]
estimate: 1h
created: 2026-01-20
updated: 2026-01-20
---

# Add CLI options for non-interactive learning capture

## Summary

Add CLI options to `just board done` to pass learnings non-interactively, enabling subagents and automation to capture learnings without interactive prompts.

## Acceptance Criteria

- [x] `WELL="text" just board done <id>` passes "what went well" non-interactively
- [x] `HARD="text" just board done <id>` passes "what was harder" non-interactively
- [x] `DIFFERENT="text" just board done <id>` passes "what would you do differently"
- [x] Options can be combined: `WELL="..." HARD="..." DIFFERENT="..." just board done <id>`
- [x] When any environment variable is set, skip interactive prompts
- [x] `NO_LEARNINGS=1 just board done <id>` explicitly skips learning capture
- [x] Interactive mode still works when no env vars provided
- [x] Documentation updated in board.just help

## Implementation Notes

**Approach:** Environment variables instead of CLI args because `just` variadic args (`*ARGS`) don't preserve quotes in multi-word values.

Modified `.justfiles/board.just` done recipe to:
1. Read `WELL`, `HARD`, `DIFFERENT`, `NO_LEARNINGS` environment variables
2. If any learning env vars are set, use those values instead of prompting
3. If `NO_LEARNINGS=1`, skip learning capture entirely
4. Otherwise, fall back to interactive prompts (current behavior)

Example usage:
```bash
# Interactive (current behavior)
just board done 0146

# Non-interactive with learnings
WELL="Config was straightforward" HARD="TOML edge cases" just board done 0146

# Explicitly skip learnings (rare, for automation that truly has nothing to report)
NO_LEARNINGS=1 just board done 0146
```

## Learnings

### L001: Just variadic args dont preserve quotes

| Field | Value |
|-------|-------|
| **Category** | tooling |
| **Context** | Implementing non-interactive CLI options for just recipes |
| **Insight** | **What went well:** Environment variables provide a clean way to pass multi-word values to bash scripts in just recipes • **Harder than expected:** Just variadic args (`*ARGS`) split on whitespace even with quotes, making `--flag "multi word"` impossible to parse correctly • **Would do differently:** Start with environment variables from the beginning when values might contain spaces |
| **Suggested Action** | Use environment variables (not CLI args) when passing multi-word values to just recipes |
| **Applies To** | All just recipes that need to accept string values with spaces |
| **Applied** | |
