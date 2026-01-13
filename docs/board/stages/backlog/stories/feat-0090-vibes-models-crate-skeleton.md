---
id: FEAT0090
title: vibes-models crate skeleton
type: feat
status: pending
priority: high
epics: [models]
depends: []
estimate: 2h
created: 2026-01-13
milestone: 37-models-registry-auth
---

# vibes-models crate skeleton

## Summary

Create the foundational `vibes-models` crate with module structure and core types.

## Requirements

- Create `vibes-models` crate in workspace
- Set up module structure:
  - `registry/` - Model catalog and discovery
  - `auth/` - Credential management
  - `providers/` - Provider implementations
  - `types/` - Core types (ModelId, ModelInfo, Capabilities, Pricing)
- Define core types from epic design
- Add to workspace Cargo.toml
- Basic error types

## Implementation Notes

```
vibes-models/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── types.rs          # ModelId, ModelInfo, Capabilities, Pricing
│   ├── error.rs          # Error types
│   ├── registry/
│   │   └── mod.rs
│   ├── auth/
│   │   └── mod.rs
│   └── providers/
│       └── mod.rs
```

## Acceptance Criteria

- [ ] Crate compiles and is part of workspace
- [ ] Core types defined (ModelId, ModelInfo, Capabilities, Pricing)
- [ ] Module structure matches epic design
- [ ] Error types with thiserror
- [ ] Basic documentation
