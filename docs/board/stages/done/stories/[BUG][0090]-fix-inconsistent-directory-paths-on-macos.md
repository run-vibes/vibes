---
id: BUG0090
title: Fix inconsistent directory paths on macOS
type: bug
status: done
priority: medium
epics: [cross-platform]
depends: []
estimate:
created: 2026-01-12
updated: 2026-01-12
---

# Fix inconsistent directory paths on macOS

## Summary

Directory paths are inconsistent between Iggy and plugins on macOS. Iggy uses macOS-native paths (`~/Library/Application Support`) while plugins should use XDG paths (`~/.config`) for CLI tool consistency.

## Problem

**Current State:**
- Iggy data: `~/Library/Application Support/vibes/iggy` (uses `dirs::data_dir()`)
- Plugin loading: `~/Library/Application Support/vibes/plugins` (uses `dirs::config_dir()`)
- Plugin installation: Installs to `~/.config/vibes/plugins/`
- CLI help text: Shows `~/.config/vibes/plugins/`

**The Bug:** `dirs::config_dir()` returns:
- macOS: `~/Library/Application Support` (GUI app convention)
- Linux: `~/.config` (XDG spec)

But for CLI tools, we should use `~/.config/` on ALL platforms (including macOS) following tools like `gh`, `docker`, `kubectl`, etc.

**Why XDG for CLI tools?**
- Cross-platform consistency
- User expectations (developers expect `~/.config/`)
- Easy to find, backup, and version control
- Industry standard for CLI tools (even on macOS)

## Acceptance Criteria

- [ ] All vibes user data uses XDG paths on all platforms:
  - `~/.config/vibes/` for config, plugins
  - `~/.local/share/vibes/` for data (Iggy, cache)
- [ ] Code uses explicit XDG paths, not `dirs::config_dir()`/`dirs::data_dir()`
- [ ] Help text matches actual paths (remove hardcoded paths)
- [ ] Tests pass on both macOS and Linux
- [ ] Documentation updated with correct XDG paths

## Implementation Notes

### Solution: Use XDG Paths Explicitly

Instead of relying on `dirs::config_dir()` which differs per platform, explicitly use XDG paths:

```rust
// Config and plugins
let config_dir = home_dir.join(".config/vibes");
let plugin_dir = config_dir.join("plugins");

// Data and cache
let data_dir = home_dir.join(".local/share/vibes");
let iggy_dir = data_dir.join("iggy");
```

This follows CLI tool conventions (gh, docker, kubectl) and provides consistency across platforms.

### Code Changes Needed

1. **vibes-iggy/src/config.rs** (`default_data_dir`):
   - Change from `dirs::data_dir().join("vibes/iggy")`
   - To `home_dir().join(".local/share/vibes/iggy")`
   - Or use XDG env var: `env::var("XDG_DATA_HOME").or_else(|_| home_dir().join(".local/share")).join("vibes/iggy")`

2. **vibes-core/src/plugins/host.rs** (`PluginHostConfig::default`):
   - Already correct for XDG! Leave as using home + `.config/vibes/plugins`
   - Or make explicit: `home_dir().join(".config/vibes/plugins")`

3. **vibes-cli/src/commands/plugin.rs** (lines 70, 73, 75):
   - Remove hardcoded `~/.config/vibes/plugins/`
   - Use `PluginHostConfig::default().user_plugin_dir.display()` to show actual path

4. **Just commands** (plugin installation):
   - Verify they're using correct paths
   - Update any hardcoded paths to use XDG

### Testing

- Clean install on macOS: verify all data goes to `~/.config/` and `~/.local/share/`
- Clean install on Linux: same paths
- Verify plugin loading works
- Verify Iggy data is accessible
- Run integration tests on both platforms

### Migration

**No backward compatibility needed** - we're the only users. After the fix:

1. Stop daemon
2. Remove old data: `rm -rf ~/Library/Application\ Support/vibes`
3. Remove old plugins: Already in wrong location at `~/.config/vibes/plugins`
4. Rebuild: `just build`
5. Everything will be created fresh in XDG locations
