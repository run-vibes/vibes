---
id: DOCS0004
title: Event schema versioning strategy
type: docs
status: icebox
priority: low
scope: plugin-system
depends: []
estimate: 2h
created: 2026-01-08
---

# Event schema versioning strategy

## Summary

Document a strategy for evolving event schemas stored in Iggy without breaking existing data. Events are persisted and queried over time, so schema changes need careful handling.

## Problem Statement

Events are serialized to Iggy using bincode/JSON. When schemas change:
- Old events may fail to deserialize
- New fields may be missing from old events
- Renamed fields break compatibility

## Acceptance Criteria

- [ ] Schema evolution strategy documented
- [ ] Backwards compatibility rules defined
- [ ] Migration tooling approach outlined
- [ ] Event versioning pattern established
- [ ] Testing strategy for schema changes

## Implementation Notes

### Schema Evolution Options

1. **Additive Only**: Only add optional fields, never remove
2. **Version Field**: Each event carries schema version
3. **Schema Registry**: Separate schema definitions from data
4. **Multiple Deserializers**: Try schemas newest-to-oldest

### Recommended Approach: Version Field

```rust
#[derive(Serialize, Deserialize)]
pub struct VibesEvent {
    pub schema_version: u32,  // Always present
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl VibesEvent {
    pub fn deserialize_payload<T: DeserializeOwned>(&self) -> Result<T> {
        match self.schema_version {
            1 => deserialize_v1(&self.payload),
            2 => deserialize_v2(&self.payload),
            _ => Err(UnknownSchemaVersion),
        }
    }
}
```

### Migration Tooling

- `vibes events migrate` - Upgrade old events to new schema
- Run as batch job during maintenance windows
- Preserve original events, write upgraded copies

### Documentation Location

`docs/EVENT-SCHEMA-VERSIONING.md`
