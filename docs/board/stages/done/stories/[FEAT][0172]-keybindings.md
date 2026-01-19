---
id: FEAT0172
title: KeyBindings system
type: feat
status: done
priority: high
scope: tui/41-terminal-ui-framework
depends: [m41-feat-03]
estimate: 3h
---

# KeyBindings System

## Summary

Implement vim-style keybindings with global and view-specific layers. Keybindings are the primary input mechanism, and the layered resolution (view-specific overrides global) is essential for the vim-like experience.

## Features

### Action Enum

```rust
pub enum Action {
    // Navigation
    Quit,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Select,
    Back,

    // Modes
    CommandMode,
    SearchMode,
    HelpMode,

    // Jump
    JumpToView(usize),

    // View-specific (for future milestones)
    Approve,
    Deny,
    Pause,
    Resume,
    Cancel,
    ViewDiff,
}
```

### KeyBindings Struct

```rust
pub struct KeyBindings {
    pub global: HashMap<KeyEvent, Action>,
    pub view_specific: HashMap<View, HashMap<KeyEvent, Action>>,
}

impl KeyBindings {
    pub fn default() -> Self {
        let mut global = HashMap::new();

        // Navigation
        global.insert(key('j'), Action::NavigateDown);
        global.insert(key('k'), Action::NavigateUp);
        global.insert(key('h'), Action::NavigateLeft);
        global.insert(key('l'), Action::NavigateRight);
        global.insert(key_code(KeyCode::Down), Action::NavigateDown);
        global.insert(key_code(KeyCode::Up), Action::NavigateUp);
        global.insert(key_code(KeyCode::Left), Action::NavigateLeft);
        global.insert(key_code(KeyCode::Right), Action::NavigateRight);

        // Actions
        global.insert(key_code(KeyCode::Enter), Action::Select);
        global.insert(key_code(KeyCode::Esc), Action::Back);
        global.insert(key('q'), Action::Quit);

        // Modes
        global.insert(key(':'), Action::CommandMode);
        global.insert(key('/'), Action::SearchMode);
        global.insert(key('?'), Action::HelpMode);

        // Jump to views (1-9)
        for i in 1..=9 {
            global.insert(key(char::from_digit(i, 10).unwrap()), Action::JumpToView(i as usize));
        }

        Self {
            global,
            view_specific: HashMap::new(),
        }
    }

    /// Resolve key to action, view-specific takes precedence
    pub fn resolve(&self, key: KeyEvent, current_view: &View) -> Option<Action> {
        // Check view-specific first
        if let Some(view_bindings) = self.view_specific.get(current_view) {
            if let Some(action) = view_bindings.get(&key) {
                return Some(action.clone());
            }
        }

        // Fall back to global
        self.global.get(&key).cloned()
    }
}
```

### Helper Functions

```rust
fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn key_code(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
```

## Implementation

1. Create `src/keybindings.rs` with Action enum
2. Implement KeyBindings struct with default()
3. Implement resolve() with layered lookup
4. Integrate into App::handle_key()
5. Wire NavigateUp/Down to work with placeholder list (if available)
6. Wire Back (Esc) to ViewStack::pop()
7. Wire Quit (q) to stop event loop
8. Add unit tests for resolution logic

## Acceptance Criteria

- [x] All global keys from epic spec mapped (j/k/h/l, Enter, Esc, :, /, ?, q, 1-9)
- [x] Arrow keys work as alternatives to h/j/k/l
- [x] `q` quits the application
- [x] `Esc` pops ViewStack (goes back)
- [x] View-specific bindings override global when defined
- [x] resolve() returns None for unmapped keys
- [x] Unit tests for resolution logic pass
