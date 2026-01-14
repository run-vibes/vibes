---
id: FEAT0104
title: Show Ollama models in web UI
type: feat
status: in-progress
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-13
updated: 2026-01-13
---

# Show Ollama models in web UI

## Summary

The CLI now shows Ollama models via `vibes models list` (commit abe7499), but the
web UI's Models page returns empty data. Wire up the server to register
OllamaProvider and expose models through the WebSocket API so the web UI can
display locally installed Ollama models.

## Acceptance Criteria

- [ ] Server registers OllamaProvider on startup (respects `OLLAMA_HOST`)
- [ ] WebSocket API supports a `models.list` message returning available models
- [ ] `useModels` hook fetches from server instead of returning empty data
- [ ] Models page displays Ollama models with size and modified date
- [ ] Gracefully handles Ollama being unavailable (no error, just no local models)

## Implementation Notes

**Server changes:**
- Add `ModelRegistry` to server state (`vibes-server/src/state.rs`)
- Register OllamaProvider similar to CLI's `build_registry()` in `commands/models.rs`
- Add WebSocket handler for `models.list` message

**Web UI changes:**
- Update `ModelInfo` interface to include `size_bytes` and `modified_at` fields
- Connect `useModels` hook to WebSocket message
- Update Models page to display size (human-readable) and modification date

**Reference:** See CLI implementation in `vibes-cli/src/commands/models.rs` for
the pattern of registering OllamaProvider with fallback when unavailable.
