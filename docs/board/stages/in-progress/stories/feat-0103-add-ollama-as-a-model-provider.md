---
id: FEAT0103
title: Add Ollama as a model provider
type: feat
status: in-progress
priority: medium
epics: [models]
depends: []
estimate: M
created: 2026-01-13
updated: 2026-01-13
---

# Add Ollama as a model provider

## Summary

Implement Ollama as a local model provider, enabling users to run models locally via Ollama's API. This brings local LLM support to vibes, complementing the existing cloud providers (Anthropic, OpenAI).

Ollama provides a local REST API (default: `http://localhost:11434`) for running models like Llama, Mistral, and others.

## Acceptance Criteria

- [x] Create `OllamaProvider` implementing `ModelProvider` trait
- [x] Discover installed models via Ollama's `/api/tags` endpoint
- [x] Implement `chat()` using `/api/chat` endpoint
- [x] Implement `chat_stream()` for streaming responses
- [x] Handle Ollama connection errors gracefully (service not running)
- [x] Support configurable base URL (not just localhost)
- [x] Mark models as `local: true` in ModelInfo
- [x] Add integration test that runs if Ollama is available
- [x] Unit tests with mocked HTTP responses

## Implementation Notes

### Ollama API

- Base URL: `http://localhost:11434` (configurable)
- List models: `GET /api/tags` → `{ "models": [{ "name": "llama3:latest", ... }] }`
- Chat: `POST /api/chat` with `{ "model": "llama3", "messages": [...], "stream": false }`
- Streaming: Same endpoint with `"stream": true`, returns NDJSON

### Provider Structure

```
vibes-models/src/providers/
├── mod.rs          # trait + re-exports
├── types.rs        # shared types
└── ollama.rs       # NEW: OllamaProvider
```

### Key Differences from Cloud Providers

- No API key required
- Models discovered dynamically (user installs via `ollama pull`)
- No pricing information (local = free)
- Context window/capabilities vary by model (may need model metadata)

### Error Handling

- Connection refused → "Ollama not running. Start with `ollama serve`"
- Model not found → "Model not installed. Run `ollama pull <model>`"
