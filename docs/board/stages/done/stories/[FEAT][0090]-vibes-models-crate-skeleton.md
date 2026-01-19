---
id: FEAT0090
title: vibes-models crate skeleton
type: feat
status: done
priority: high
scope: models
depends: []
estimate: 2h
created: 2026-01-13
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

- [x] Crate compiles and is part of workspace
- [x] Core types defined (ModelId, ModelInfo, Capabilities, Pricing)
- [x] Module structure matches epic design
- [x] Error types with thiserror
- [x] Basic documentation
