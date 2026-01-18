# TUI Theme System Design

## Overview

Design for runtime theme switching and the command infrastructure that supports it.

## Command System Architecture

### Command Trait

```rust
pub enum CommandResult {
    Ok(Option<String>),
    Err(String),
    Quit,
}

pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&mut self, args: &[&str], app: &mut App) -> CommandResult;
    fn completions(&self, args: &[&str], app: &App) -> Vec<String> { vec![] }
    fn help(&self) -> &str { "" }
}
```

### Command Registry

```rust
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn register(&mut self, cmd: Box<dyn Command>);
    pub fn execute(&mut self, input: &str, app: &mut App) -> CommandResult;
    pub fn completions(&self, input: &str, app: &App) -> Vec<String>;
}
```

### Command Input State

```rust
pub struct CommandInput {
    pub buffer: String,
    pub cursor: usize,
    pub completions: Vec<String>,
    pub completion_idx: Option<usize>,
    pub message: Option<(String, bool)>,
}
```

## ThemeCommand

Handles `:theme`, `:theme <name>`, and `:theme save`.

```rust
pub struct ThemeCommand {
    loader: ThemeLoader,
}

impl Command for ThemeCommand {
    fn name(&self) -> &str { "theme" }

    fn execute(&mut self, args: &[&str], app: &mut App) -> CommandResult {
        match args {
            [] => list_themes(),
            ["save"] => save_to_config(),
            [name] => switch_theme(name),
            _ => usage_error(),
        }
    }

    fn completions(&self, args: &[&str], _app: &App) -> Vec<String> {
        // Theme names + "save"
    }
}
```

## Config Persistence

`:theme save` writes to `~/.config/vibes/config.toml`:

- Creates file if missing
- Preserves existing content
- Updates `[theme].active` field

## UI Integration

### Command Bar Widget

Renders at bottom of screen during Command mode:
- Shows `:` prompt with input buffer
- Displays cursor at correct position
- Shows completion dropdown when available
- Displays success/error messages after execution

### App Integration

```rust
pub struct App {
    pub command_input: CommandInput,
    pub command_registry: CommandRegistry,
    // ... existing fields
}
```

Mode::Command handling:
- `:` key enters Command mode
- Keys route to CommandInput methods
- Enter executes via CommandRegistry
- Escape clears and returns to Normal

## Files

### New Files

| File | Purpose |
|------|---------|
| `commands/mod.rs` | Command trait, CommandResult |
| `commands/registry.rs` | CommandRegistry |
| `commands/input.rs` | CommandInput state |
| `commands/theme.rs` | ThemeCommand |
| `widgets/command_bar.rs` | Command bar widget |

### Modified Files

| File | Changes |
|------|---------|
| `app.rs` | Add command fields, wire Mode::Command |
| `lib.rs` | Export commands module |
| `widgets/mod.rs` | Export CommandBarWidget |

---

## Theme Preview in Settings (m45-feat-04)

Visual theme selector with live preview in the Settings view.

### Navigation

**Entry points**:
- Keybinding: `s` from Dashboard → `Action::OpenSettings`
- Command: `:settings` → pushes `View::Settings`

**Exit**:
- `Esc` or `q` cancels and restores original theme
- `Enter` applies and exits

### SettingsView State

```rust
pub struct SettingsState {
    original_theme: String,    // Theme when view opened
    preview_theme: String,     // Currently previewed theme
    selected_index: usize,     // Index in theme list
    focus: SettingsFocus,
}

pub enum SettingsFocus {
    ThemeList,
    ApplyButton,
    CancelButton,
}
```

The `App.theme` field is temporarily swapped to show previews. On Cancel, restore `original_theme`. On Apply, keep current and optionally save.

### Widgets

**ThemeSelector** - Vertical list of theme names:
```rust
pub struct ThemeSelector<'a> {
    themes: &'a [String],
    selected: usize,
    current_theme: &'a str,  // Marked with ●
}
```

**ThemePreview** - Visual color display:
```rust
pub struct ThemePreview<'a> {
    theme: &'a Theme,
}
```

Renders:
1. Color swatches: `████ bg  ████ fg  ████ accent`
2. Semantic colors: `● Success  ● Warning  ● Error`
3. Status colors: `● Running  ● Paused  ● Completed  ● Failed`
4. Sample text in normal, bold, dim styles
5. Border using `theme.border`

### Keyboard Navigation

**ThemeList focused**:
| Key | Action |
|-----|--------|
| `j`/`↓` | Next theme, update preview |
| `k`/`↑` | Previous theme, update preview |
| `Enter` | Apply and exit |
| `Tab` | Focus Apply button |
| `Esc` | Cancel and exit |

**Buttons focused**:
| Key | Action |
|-----|--------|
| `Tab`/`←`/`→` | Move between buttons |
| `Shift+Tab` | Return to ThemeList |
| `Enter` | Execute button |

### Files

**New**:
| File | Purpose |
|------|---------|
| `views/settings.rs` | SettingsView |
| `widgets/theme_selector.rs` | ThemeSelector |
| `widgets/theme_preview.rs` | ThemePreview |
| `commands/settings.rs` | SettingsCommand |

**Modified**:
| File | Changes |
|------|---------|
| `state.rs` | Add SettingsState |
| `keybindings.rs` | Add OpenSettings action, map `s` |
| `app.rs` | Handle Settings view, wire command |
| `views/mod.rs` | Export SettingsView |
| `widgets/mod.rs` | Export new widgets |
| `commands/mod.rs` | Export SettingsCommand |
