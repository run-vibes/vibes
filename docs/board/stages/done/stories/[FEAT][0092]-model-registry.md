---
id: FEAT0092
title: ModelRegistry for model discovery
type: feat
status: done
priority: high
scope: models
depends: [FEAT0091]
estimate: 3h
created: 2026-01-13
---

# ModelRegistry for model discovery

## Summary

Implement `ModelRegistry` to catalog available models and manage providers.

## Requirements

- Register providers by name
- Query models by ID, provider, or capabilities
- Model discovery from registered providers
- Filter models by capabilities (chat, vision, tools, embeddings)
- Thread-safe access with Arc

## Implementation

```rust
pub struct ModelRegistry {
    providers: HashMap<String, Arc<dyn ModelProvider>>,
    models: HashMap<ModelId, ModelInfo>,
}

impl ModelRegistry {
    pub fn new() -> Self;
    pub fn register_provider(&mut self, provider: Arc<dyn ModelProvider>);
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn ModelProvider>>;

    pub fn list_models(&self) -> Vec<&ModelInfo>;
    pub fn get_model(&self, id: &ModelId) -> Option<&ModelInfo>;
    pub fn find_by_capability(&self, cap: Capability) -> Vec<&ModelInfo>;
    pub fn find_by_provider(&self, provider: &str) -> Vec<&ModelInfo>;

    pub fn refresh(&mut self) -> Result<()>;
}
```

## Acceptance Criteria

- [x] Provider registration and lookup
- [x] Model catalog populated from providers
- [x] Query by ID, provider, capabilities
- [x] Refresh to update from providers
- [x] Unit tests for all query methods
