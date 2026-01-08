---
id: DOCS0002
title: Update PRD for current architecture
type: docs
status: backlog
priority: medium
epics: [core]
depends: []
estimate: 1h
created: 2026-01-08
updated: 2026-01-08
---

# Update PRD for current architecture

## Summary

The PRD (docs/PRD.md) was written early in the project and some sections don't reflect the current architecture. Update it to accurately describe what vibes does today.

## Areas to Update

1. **Groove Plugin**: PRD predates the assessment framework - add section on vibes-groove
2. **EventLog/Iggy**: PRD mentions event system but Iggy integration is more mature now
3. **CLI Commands**: Document the full command surface (`vibes assess`, `vibes events`, etc.)
4. **Three-tier Assessment**: Lightweight/Medium/Heavy model should be prominent
5. **Plugin System**: Current plugin API and loading mechanism

## Acceptance Criteria

- [ ] PRD reflects current project state
- [ ] All major features documented
- [ ] Architecture diagrams updated if needed
- [ ] Future/planned items clearly marked as such
- [ ] No references to deprecated patterns

## Implementation Notes

1. Read current PRD thoroughly
2. Cross-reference with ARCHITECTURE.md and CLAUDE.md
3. Update outdated sections
4. Add new sections for groove, assessment, etc.
5. Mark clearly which features are implemented vs planned
