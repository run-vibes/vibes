---
id: FEAT0091
title: ModelProvider trait definition
type: feat
status: done
priority: high
scope: models
depends: [FEAT0090]
estimate: 2h
created: 2026-01-13
---

# ModelProvider trait definition

## Summary

Define the `ModelProvider` trait that all model providers (cloud and local) will implement.

## Requirements

- Define `ModelProvider` async trait
- Chat and streaming methods
- Embedding method
- Capability queries (supports_tools, supports_vision)
- Pricing information
- Model listing

## Implementation

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> Vec<ModelInfo>;

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream>;
    async fn embed(&self, request: EmbedRequest) -> Result<EmbedResponse>;

    fn supports_tools(&self) -> bool;
    fn supports_vision(&self) -> bool;
    fn pricing(&self, model: &str) -> Option<Pricing>;
}
```

## Request/Response Types

- `ChatRequest` - messages, model, temperature, max_tokens, tools
- `ChatResponse` - content, usage, tool_calls
- `ChatStream` - async stream of chunks
- `EmbedRequest` - texts, model
- `EmbedResponse` - embeddings, usage

## Acceptance Criteria

- [x] ModelProvider trait defined with async_trait
- [x] Request/Response types defined
- [x] ChatStream type for streaming responses
- [x] Comprehensive documentation
- [x] Tests for type serialization
