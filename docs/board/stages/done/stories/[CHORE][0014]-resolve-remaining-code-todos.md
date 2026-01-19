---
id: CHORE0014
title: Resolve remaining code TODOs
type: chore
status: done
priority: medium
scope: cli
depends: [m29-feat-0012-wire-circuit-breaker-intervention]
estimate: 4h
created: 2026-01-08
---

# Resolve remaining code TODOs

## Summary

Audit and resolve all TODO comments in the codebase. A recent review found 12 TODOs, some of which are blocking completion of milestone 29.

## Known TODOs (from review)

1. `processor.rs:302` - Wire circuit breaker intervention (separate story)
2. Various `TODO: implement` stubs
3. Error handling improvements
4. Missing test coverage notes

## Acceptance Criteria

- [ ] All TODOs audited and categorized
- [ ] Critical TODOs resolved or converted to stories
- [ ] Non-actionable TODOs removed
- [ ] No TODOs older than 1 milestone
- [ ] `grep -r "TODO" --include="*.rs"` shows only intentional items

## Implementation Notes

### Audit Process

```bash
# Find all TODOs
grep -rn "TODO" --include="*.rs" | head -20

# Categorize by priority:
# - P1: Blocking functionality
# - P2: Should fix soon
# - P3: Nice to have
# - P4: Convert to story or remove
```

### Resolution Options

1. **Implement**: If small and straightforward
2. **Story**: Create backlog story for larger work
3. **Remove**: If no longer relevant
4. **Document**: If intentional placeholder

### Dependencies

- Wait for m29-feat-0012 (circuit breaker) before this cleanup
- Some TODOs may spawn additional stories
