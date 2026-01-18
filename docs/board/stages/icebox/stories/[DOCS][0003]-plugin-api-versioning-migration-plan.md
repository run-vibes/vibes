---
id: DOCS0003
title: Plugin API versioning migration plan
type: docs
status: icebox
priority: low
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-18
---

# Plugin API versioning migration plan

## Summary

Document a strategy for versioning the plugin API (`vibes-plugin-api` crate) as it evolves. Currently there's no formal versioning strategy, which could cause issues as third-party plugins are developed.

## Questions to Answer

1. How do we version the plugin API crate?
2. What constitutes a breaking change?
3. How do plugins detect incompatible host versions?
4. How do we deprecate old APIs?
5. How long do we support old API versions?

## Acceptance Criteria

- [ ] ADR (Architecture Decision Record) for plugin versioning
- [ ] Semantic versioning policy documented
- [ ] Breaking change detection strategy
- [ ] Migration guide template for API updates
- [ ] Runtime version checking mechanism designed

## Implementation Notes

### Versioning Strategy Options

1. **Semantic Versioning**: Standard approach, bump major on breaking changes
2. **API Version Field**: Plugin declares required API version
3. **Feature Flags**: Opt-in new APIs, preserve old behavior

### Runtime Compatibility Check

```rust
// In vibes-plugin-api
pub const API_VERSION: &str = "0.1.0";

pub trait Plugin {
    fn api_version(&self) -> &str { API_VERSION }
    // ...
}

// In host
fn load_plugin(path: &Path) -> Result<Box<dyn Plugin>> {
    let plugin = unsafe { load_dynamic(path)? };
    if !semver_compatible(plugin.api_version(), API_VERSION) {
        return Err(IncompatibleApiVersion);
    }
    Ok(plugin)
}
```

### Documentation Location

`docs/PLUGIN-API-VERSIONING.md` or ADR in `docs/adr/`
