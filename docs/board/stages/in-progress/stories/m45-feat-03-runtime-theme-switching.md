---
id: m45-feat-03
title: Runtime theme switching
type: feat
status: in-progress
priority: medium
epics: [tui]
depends: [m45-feat-01, m45-feat-02]
estimate: 2h
milestone: 45-tui-theme-system
---

# Runtime Theme Switching

## Summary

Implement the `:theme <name>` command for changing themes at runtime without restarting the TUI. The theme change takes effect immediately and can optionally persist to the config file.

## Commands

### `:theme`
Lists available themes and shows current selection:

```
Available themes:
  vibes (current)
  dark
  light
  high-contrast
  my-custom-theme

Use :theme <name> to switch
```

### `:theme <name>`
Switches to the specified theme immediately:

```
:theme dark
# Theme changed to 'dark'

:theme unknown
# Error: theme 'unknown' not found
```

### `:theme save`
Persists the current theme to config file:

```
:theme save
# Theme 'dark' saved to config
```

## Features

### Command Handler

```rust
impl App {
    /// Handle :theme command
    pub fn cmd_theme(&mut self, args: &[&str]) -> CommandResult {
        match args {
            [] => self.list_themes(),
            ["save"] => self.save_current_theme(),
            [name] => self.switch_theme(name),
            _ => Err(CommandError::InvalidArgs(":theme [name|save]")),
        }
    }

    fn switch_theme(&mut self, name: &str) -> CommandResult {
        let theme = self.theme_loader
            .get(name)
            .ok_or(CommandError::ThemeNotFound(name.into()))?;

        self.theme = theme.clone();
        Ok(format!("Theme changed to '{}'", name))
    }

    fn save_current_theme(&self) -> CommandResult {
        // Update config file with current theme name
        self.config.set_theme(&self.theme.name)?;
        self.config.save()?;
        Ok(format!("Theme '{}' saved to config", self.theme.name))
    }
}
```

### Immediate Repaint

Theme changes trigger an immediate full repaint:

```rust
fn switch_theme(&mut self, name: &str) -> CommandResult {
    // ... switch theme
    self.needs_full_repaint = true;  // Force complete redraw
    Ok(...)
}
```

## Implementation

1. Add `:theme` command to command parser
2. Implement `list_themes()` with current indicator
3. Implement `switch_theme()` with validation
4. Implement `save_current_theme()` with config update
5. Add `needs_full_repaint` flag for theme changes
6. Wire command through existing command dispatch
7. Add unit tests for command parsing
8. Add integration test for theme persistence

## Keybinding (Optional)

Consider adding a quick-switch keybinding:

```rust
// Ctrl+T cycles through themes
KeyCode::Char('t') if modifiers.contains(KeyModifiers::CONTROL) => {
    self.cycle_theme();
}
```

## Acceptance Criteria

- [ ] `:theme` lists all available themes
- [ ] `:theme` shows current theme indicator
- [ ] `:theme <name>` switches theme immediately
- [ ] `:theme <name>` shows error for unknown themes
- [ ] `:theme save` persists current theme to config
- [ ] Theme change triggers full UI repaint
- [ ] All UI elements update to new theme colors
- [ ] Tab completion works for theme names
