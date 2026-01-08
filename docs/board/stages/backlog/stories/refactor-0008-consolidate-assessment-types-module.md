---
id: REFACTOR0008
title: Consolidate assessment types module
type: refactor
status: backlog
priority: low
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
---

# Consolidate assessment types module

## Summary

The assessment types are spread across multiple locations:
- `vibes-groove/src/assessment/types.rs` - Core types
- `vibes-groove/src/assessment/config.rs` - Config types
- `vibes-groove/src/plugin.rs` - API response types
- `vibes-core` - Shared event types

Consider consolidating or clearly separating concerns.

## Current Issues

1. **Duplication**: Some types defined in multiple places
2. **Coupling**: Plugin-specific types leak into API boundaries
3. **Discovery**: Hard to find the canonical definition

## Acceptance Criteria

- [ ] Clear separation: internal types vs API types
- [ ] Single source of truth for each type
- [ ] Documentation on type organization
- [ ] No breaking API changes (preserve external interface)

## Implementation Notes

### Proposed Structure

```
vibes-groove/src/assessment/
├── types.rs          # Internal assessment types
├── config.rs         # Configuration types (keep separate)
├── api_types.rs      # HTTP/CLI response types (new)
└── mod.rs            # Re-exports public API
```

### Migration Strategy

1. Audit all type usages
2. Identify which types are internal vs external
3. Move types to appropriate modules
4. Update imports across codebase
5. Add module-level documentation
