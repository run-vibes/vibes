---
id: FEAT0105
title: Add Ollama autostart config option
type: feat
status: backlog
priority: medium
epics: []
depends: []
estimate:
created: 2026-01-13
updated: 2026-01-13
---

# Add Ollama autostart config option

## Summary

Add an `[ollama]` section to the vibes config TOML that optionally autostarts
Ollama when vibes starts. This enables a seamless local LLM experience without
requiring users to manually start Ollama in a separate terminal.

## Acceptance Criteria

- [ ] New `[ollama]` section in config with `enabled` and `host` fields
- [ ] When `ollama.enabled = true`, vibes spawns `ollama serve` on startup
- [ ] Respects `ollama.host` if set, otherwise uses default `localhost:11434`
- [ ] Gracefully handles Ollama already running (detect and skip spawn)
- [ ] Gracefully handles Ollama not installed (warn but don't fail)
- [ ] Ollama process terminates when vibes exits (child process management)
- [ ] Works with both `vibes serve` and `vibes claude` commands

## Implementation Notes

**Config structure** (follows existing pattern in `vibes-cli/src/config/types.rs`):

```toml
[ollama]
enabled = false         # Default: don't autostart
host = "localhost:11434"  # Optional: custom host
```

**Add to types.rs:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OllamaConfigSection {
    #[serde(default)]
    pub enabled: bool,
    pub host: Option<String>,
}
```

**Process management:**
- Use `std::process::Command` to spawn `ollama serve`
- Check if Ollama is already running before spawning (HTTP ping to `/api/tags`)
- Store child process handle for cleanup on shutdown
- Use `OLLAMA_HOST` env var if `host` is configured

**Reference:** Similar pattern to `server.auto_start` in existing config.
