---
id: FEAT0187
title: Built-in theme variants
type: feat
status: done
priority: high
epics: [tui]
depends: [m45-feat-01]
estimate: 2h
milestone: 45
---

# Built-in Theme Variants

## Summary

Add additional built-in themes beyond the default "vibes" theme. Provide dark, light, and high-contrast variants for accessibility and user preference. These themes are always available without configuration.

## Themes

### vibes (default)
The original CRT phosphor green theme - already implemented.

### dark
A neutral dark theme for reduced eye strain:

```rust
pub fn dark() -> Theme {
    Theme {
        name: "dark".into(),
        bg: Color::Rgb(30, 30, 30),      // #1e1e1e
        fg: Color::Rgb(212, 212, 212),   // #d4d4d4
        accent: Color::Rgb(86, 156, 214), // #569cd6
        success: Color::Rgb(78, 201, 176), // #4ec9b0
        warning: Color::Rgb(220, 220, 170), // #dcdcaa
        error: Color::Rgb(244, 71, 71),   // #f44747
        // ... status and UI colors
    }
}
```

### light
A light theme for bright environments:

```rust
pub fn light() -> Theme {
    Theme {
        name: "light".into(),
        bg: Color::Rgb(255, 255, 255),   // #ffffff
        fg: Color::Rgb(51, 51, 51),      // #333333
        accent: Color::Rgb(0, 122, 204), // #007acc
        success: Color::Rgb(22, 163, 74), // #16a34a
        warning: Color::Rgb(202, 138, 4), // #ca8a04
        error: Color::Rgb(220, 38, 38),  // #dc2626
        // ... status and UI colors
    }
}
```

### high-contrast
An accessibility theme with maximum contrast:

```rust
pub fn high_contrast() -> Theme {
    Theme {
        name: "high-contrast".into(),
        bg: Color::Black,
        fg: Color::White,
        accent: Color::Rgb(0, 255, 255), // #00ffff cyan
        success: Color::Rgb(0, 255, 0),  // #00ff00 green
        warning: Color::Rgb(255, 255, 0), // #ffff00 yellow
        error: Color::Rgb(255, 0, 0),    // #ff0000 red
        // ... status and UI colors with high contrast
    }
}
```

## Features

### Theme Registry

```rust
/// Get all builtin themes
pub fn builtin_themes() -> Vec<Theme> {
    vec![
        vibes_default(),
        dark(),
        light(),
        high_contrast(),
    ]
}

/// Get a builtin theme by name
pub fn builtin_theme(name: &str) -> Option<Theme> {
    builtin_themes().into_iter().find(|t| t.name == name)
}
```

## Implementation

1. Create `src/theme/variants.rs` for additional themes
2. Implement `dark()` theme constructor
3. Implement `light()` theme constructor
4. Implement `high_contrast()` theme constructor
5. Add `builtin_themes()` registry function
6. Update ThemeLoader to include all builtin themes
7. Add visual tests for each theme rendering

## Design Considerations

- Each theme should be internally consistent (e.g., status colors align with semantic meaning)
- Light theme needs inverted selection/highlight logic
- High-contrast theme prioritizes readability over aesthetics
- All themes must pass WCAG AA contrast requirements for text

## Acceptance Criteria

- [x] `dark` theme provides neutral gray tones
- [x] `light` theme has white background with dark text
- [x] `high-contrast` theme uses pure black/white with saturated colors
- [x] All themes have consistent field mappings
- [x] `builtin_themes()` returns all 4 themes
- [x] `builtin_theme("name")` returns correct theme
- [x] Each theme has appropriate selection/highlight contrast
- [x] Light theme borders are visible on white background
