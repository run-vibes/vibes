---
id: FEAT0077
title: Config Save/Load for Wizards
type: feat
status: done
priority: medium
scope: networking
---

# Config Save/Load for Wizards

Ensure wizards can save configuration properly while preserving existing settings.

## Context

Wizards need to update specific sections of the config file (tunnel, auth) without destroying other settings (server, session). This story ensures ConfigLoader can load, modify, and save config atomically.

## Acceptance Criteria

- [x] Add `ConfigLoader::save(config: &VibesConfig)` method
- [x] Save preserves existing config sections not being modified
- [x] Creates config file and parent directories if they don't exist
- [x] Show what was saved to user:
  ```
  Configuration saved:
    tunnel.enabled = true
    tunnel.mode = "quick"
  ```
- [x] Warn if overwriting existing non-default values
- [x] Handle TOML serialization errors gracefully

## Technical Notes

```rust
impl ConfigLoader {
    pub fn save(config: &VibesConfig) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directories
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(config)?;
        std::fs::write(&config_path, toml)?;

        Ok(())
    }
}
```

Preserve comments by:
1. Read existing file as string
2. Parse to RawVibesConfig
3. Merge changes
4. Serialize back

(Or accept losing comments for simplicity in v1)

## Size

S - Small (single function, straightforward file operations)
