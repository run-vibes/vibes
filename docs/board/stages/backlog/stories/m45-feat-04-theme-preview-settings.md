---
id: m45-feat-04
title: Theme preview in settings
type: feat
status: backlog
priority: medium
epics: [tui]
depends: [m45-feat-01, m45-feat-02]
estimate: 3h
milestone: 45-tui-theme-system
---

# Theme Preview in Settings

## Summary

Add a theme selector to the Settings view with a live preview widget. Users can browse available themes, see a preview of each theme's colors before applying, and select their preferred theme.

## UI Layout

```
┌─ Settings ──────────────────────────────────────┐
│                                                  │
│  Appearance                                      │
│  ├─ Theme: [vibes ▼]                            │
│  │                                               │
│  │  ┌─ Preview ─────────────────────┐           │
│  │  │  Sample text in theme colors  │           │
│  │  │  ████ bg  ████ fg  ████ accent│           │
│  │  │  ● Success  ● Warning  ● Error│           │
│  │  │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━│           │
│  │  │  Status: Running | Paused     │           │
│  │  └───────────────────────────────┘           │
│  │                                               │
│  └─ [Apply]  [Cancel]                           │
│                                                  │
└──────────────────────────────────────────────────┘
```

## Features

### ThemeSelector Widget

```rust
pub struct ThemeSelector {
    themes: Vec<Theme>,
    selected: usize,
    preview_theme: Theme,
}

impl ThemeSelector {
    pub fn new(loader: &ThemeLoader, current: &str) -> Self;

    /// Move selection up/down
    pub fn select_prev(&mut self);
    pub fn select_next(&mut self);

    /// Get the currently selected theme
    pub fn selected_theme(&self) -> &Theme;
}

impl StatefulWidget for ThemeSelector {
    type State = ThemeSelectorState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}
```

### ThemePreview Widget

```rust
pub struct ThemePreview<'a> {
    theme: &'a Theme,
}

impl<'a> ThemePreview<'a> {
    pub fn new(theme: &'a Theme) -> Self;
}

impl Widget for ThemePreview<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render color swatches
        // Render sample text
        // Render status indicators
        // Show theme name
    }
}
```

### Preview Elements

The preview widget demonstrates:

1. **Color swatches** - Visual blocks showing bg, fg, accent colors
2. **Sample text** - Text rendered in the theme's styles (normal, bold, dim)
3. **Status indicators** - Colored dots for success/warning/error
4. **Status bar sample** - Running/paused/completed states
5. **Border style** - Box borders in the theme's border color

## Implementation

1. Create `src/widgets/theme_selector.rs`
2. Create `src/widgets/theme_preview.rs`
3. Add appearance section to SettingsView
4. Wire theme list from ThemeLoader
5. Implement keyboard navigation (j/k or ↑/↓)
6. Preview updates on selection change
7. Apply button saves and activates theme
8. Cancel returns without changes
9. Add Enter shortcut to apply
10. Add Escape shortcut to cancel

## Interaction Flow

1. User navigates to Settings → Appearance
2. Theme dropdown shows available themes
3. Arrow keys navigate theme list
4. Preview updates in real-time as user browses
5. Enter applies selected theme
6. Escape cancels and reverts preview

## Acceptance Criteria

- [ ] Settings view has Appearance section
- [ ] Theme selector lists all available themes
- [ ] Current theme is highlighted in the list
- [ ] Preview widget shows selected theme colors
- [ ] Preview updates immediately on selection change
- [ ] Apply button saves theme to config
- [ ] Cancel button reverts to original theme
- [ ] Keyboard navigation works (j/k/↑/↓)
- [ ] Enter applies, Escape cancels
- [ ] Preview shows all key color categories
