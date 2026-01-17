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
