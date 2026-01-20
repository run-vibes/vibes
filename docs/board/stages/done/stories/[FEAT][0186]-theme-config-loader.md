---
id: FEAT0186
title: Theme config file loader
type: feat
status: done
priority: high
scope: tui/05-customizable-themes
depends: []
estimate: 3h
---

# Theme Config File Loader

## Summary

Implement TOML-based theme configuration loading. Users can define custom themes in a config file, which the TUI will load at startup. This enables personalization without code changes.

## Config Location

Themes are defined in the vibes config file:

```toml
# ~/.config/vibes/config.toml

[theme]
active = "vibes"  # Name of theme to use

# Custom theme definition
[[theme.custom]]
name = "my-theme"

# Base colors (hex format)
bg = "#1a1a2e"
fg = "#eaeaea"
accent = "#e94560"
success = "#00ff88"
warning = "#ffc800"
error = "#ff5555"

# Status colors
running = "#00ff88"
paused = "#ffc800"
completed = "#646464"
failed = "#ff5555"

# UI elements
border = "#3c3c3c"
selection = "#285028"
highlight = "#009664"
```

## Features

### ThemeLoader

```rust
use std::path::Path;
use crate::theme::Theme;

pub struct ThemeLoader {
    builtin: Vec<Theme>,
    custom: Vec<Theme>,
}

impl ThemeLoader {
    /// Load themes from config file
    pub fn from_config(path: &Path) -> Result<Self, ThemeLoadError>;

    /// Get a theme by name (searches custom first, then builtin)
    pub fn get(&self, name: &str) -> Option<&Theme>;

    /// List all available theme names
    pub fn list(&self) -> Vec<&str>;

    /// Get the active theme name from config
    pub fn active_name(&self) -> &str;
}
```

### Color Parsing

```rust
/// Parse hex color string to ratatui Color
fn parse_hex_color(hex: &str) -> Result<Color, ColorParseError> {
    // Supports #RRGGBB format
    // Returns Color::Rgb(r, g, b)
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ThemeLoadError {
    #[error("config file not found: {0}")]
    NotFound(PathBuf),

    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("invalid color format '{0}': expected #RRGGBB")]
    InvalidColor(String),

    #[error("theme '{0}' not found")]
    ThemeNotFound(String),
}
```

## Implementation

1. Add `toml` and `serde` dependencies to vibes-tui
2. Create `src/theme/config.rs` with TOML serde structs
3. Create `src/theme/loader.rs` with ThemeLoader
4. Add `parse_hex_color` helper function
5. Integrate loader into App initialization
6. Add unit tests for color parsing
7. Add integration tests for config loading

## Acceptance Criteria

- [x] ThemeLoader loads themes from TOML config file
- [x] Custom themes override builtin themes with same name
- [x] Invalid config files produce clear error messages
- [x] Hex colors in #RRGGBB format are parsed correctly
- [x] Missing config file falls back to builtin themes gracefully
- [x] `theme.active` selects the startup theme
- [x] All theme fields are configurable via TOML
- [x] Unit tests cover color parsing edge cases
