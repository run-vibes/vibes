//! Keybindings system for vibes TUI.
//!
//! Provides vim-style keybindings with global and view-specific layers.
//! View-specific bindings override global bindings when defined.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::views::View;

/// Actions that can be triggered by key presses.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    Restart,
    ViewDiff,

    // Connection
    Retry,
}

/// Keybindings configuration with global and view-specific layers.
#[derive(Debug, Clone)]
pub struct KeyBindings {
    /// Global keybindings that apply to all views.
    pub global: HashMap<KeyEvent, Action>,
    /// View-specific keybindings that override global bindings.
    pub view_specific: HashMap<View, HashMap<KeyEvent, Action>>,
    /// Agent view bindings (applies to any Agent view regardless of agent ID).
    agent_bindings: HashMap<KeyEvent, Action>,
}

impl KeyBindings {
    /// Resolve a key press to an action using global bindings only.
    pub fn resolve_global(&self, key: KeyEvent) -> Option<Action> {
        self.global.get(&key).cloned()
    }

    /// Resolve a key press to an action, view-specific takes precedence over global.
    pub fn resolve(&self, key: KeyEvent, current_view: &View) -> Option<Action> {
        // Check view-specific first
        if let Some(view_bindings) = self.view_specific.get(current_view)
            && let Some(action) = view_bindings.get(&key)
        {
            return Some(action.clone());
        }

        // Check agent bindings for any Agent view
        if matches!(current_view, View::Agent(_))
            && let Some(action) = self.agent_bindings.get(&key)
        {
            return Some(action.clone());
        }

        // Fall back to global
        self.global.get(&key).cloned()
    }

    /// Add a view-specific keybinding.
    pub fn add_view_binding(&mut self, view: View, key: KeyEvent, action: Action) {
        self.view_specific
            .entry(view)
            .or_default()
            .insert(key, action);
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut global = HashMap::new();

        // Navigation - vim style
        global.insert(key('j'), Action::NavigateDown);
        global.insert(key('k'), Action::NavigateUp);
        global.insert(key('h'), Action::NavigateLeft);
        global.insert(key('l'), Action::NavigateRight);

        // Navigation - arrow keys
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

        // Connection
        global.insert(key('r'), Action::Retry);

        // Jump to views (1-9)
        for i in 1..=9 {
            global.insert(
                key(char::from_digit(i, 10).unwrap()),
                Action::JumpToView(i as usize),
            );
        }

        // Agent view-specific bindings
        let mut agent_bindings = HashMap::new();
        agent_bindings.insert(key('y'), Action::Approve);
        agent_bindings.insert(key('n'), Action::Deny);
        agent_bindings.insert(key('v'), Action::ViewDiff);
        agent_bindings.insert(key('p'), Action::Pause);
        agent_bindings.insert(key('c'), Action::Cancel);
        agent_bindings.insert(key('r'), Action::Restart);

        Self {
            global,
            view_specific: HashMap::new(),
            agent_bindings,
        }
    }
}

/// Helper to create a KeyEvent from a character.
fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

/// Helper to create a KeyEvent from a KeyCode.
fn key_code(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_enum_has_navigation_variants() {
        // All navigation actions should exist
        let _quit = Action::Quit;
        let _up = Action::NavigateUp;
        let _down = Action::NavigateDown;
        let _left = Action::NavigateLeft;
        let _right = Action::NavigateRight;
        let _select = Action::Select;
        let _back = Action::Back;
    }

    #[test]
    fn action_enum_has_mode_variants() {
        let _cmd = Action::CommandMode;
        let _search = Action::SearchMode;
        let _help = Action::HelpMode;
    }

    #[test]
    fn action_enum_has_jump_variant() {
        let _jump = Action::JumpToView(1);
    }

    #[test]
    fn action_enum_has_view_specific_variants() {
        let _approve = Action::Approve;
        let _deny = Action::Deny;
        let _pause = Action::Pause;
        let _resume = Action::Resume;
        let _cancel = Action::Cancel;
        let _diff = Action::ViewDiff;
    }

    #[test]
    fn action_is_clone_and_eq() {
        let action = Action::Quit;
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    // KeyBindings tests
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    fn key_code(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn keybindings_default_has_vim_navigation() {
        let bindings = KeyBindings::default();

        assert_eq!(
            bindings.resolve_global(key('j')),
            Some(Action::NavigateDown)
        );
        assert_eq!(bindings.resolve_global(key('k')), Some(Action::NavigateUp));
        assert_eq!(
            bindings.resolve_global(key('h')),
            Some(Action::NavigateLeft)
        );
        assert_eq!(
            bindings.resolve_global(key('l')),
            Some(Action::NavigateRight)
        );
    }

    #[test]
    fn keybindings_default_has_arrow_keys() {
        let bindings = KeyBindings::default();

        assert_eq!(
            bindings.resolve_global(key_code(KeyCode::Down)),
            Some(Action::NavigateDown)
        );
        assert_eq!(
            bindings.resolve_global(key_code(KeyCode::Up)),
            Some(Action::NavigateUp)
        );
        assert_eq!(
            bindings.resolve_global(key_code(KeyCode::Left)),
            Some(Action::NavigateLeft)
        );
        assert_eq!(
            bindings.resolve_global(key_code(KeyCode::Right)),
            Some(Action::NavigateRight)
        );
    }

    #[test]
    fn keybindings_default_has_action_keys() {
        let bindings = KeyBindings::default();

        assert_eq!(
            bindings.resolve_global(key_code(KeyCode::Enter)),
            Some(Action::Select)
        );
        assert_eq!(
            bindings.resolve_global(key_code(KeyCode::Esc)),
            Some(Action::Back)
        );
        assert_eq!(bindings.resolve_global(key('q')), Some(Action::Quit));
    }

    #[test]
    fn keybindings_default_has_mode_keys() {
        let bindings = KeyBindings::default();

        assert_eq!(bindings.resolve_global(key(':')), Some(Action::CommandMode));
        assert_eq!(bindings.resolve_global(key('/')), Some(Action::SearchMode));
        assert_eq!(bindings.resolve_global(key('?')), Some(Action::HelpMode));
    }

    #[test]
    fn keybindings_default_has_jump_keys() {
        let bindings = KeyBindings::default();

        for i in 1..=9 {
            let c = char::from_digit(i, 10).unwrap();
            assert_eq!(
                bindings.resolve_global(key(c)),
                Some(Action::JumpToView(i as usize))
            );
        }
    }

    #[test]
    fn keybindings_resolve_global_returns_none_for_unmapped_keys() {
        let bindings = KeyBindings::default();

        assert_eq!(bindings.resolve_global(key('z')), None);
        assert_eq!(bindings.resolve_global(key('x')), None);
        assert_eq!(bindings.resolve_global(key_code(KeyCode::F(1))), None);
    }

    // resolve() with view-specific tests
    use crate::views::View;

    #[test]
    fn keybindings_resolve_returns_global_for_unmapped_view() {
        let bindings = KeyBindings::default();

        // Dashboard has no view-specific bindings by default
        assert_eq!(
            bindings.resolve(key('j'), &View::Dashboard),
            Some(Action::NavigateDown)
        );
    }

    #[test]
    fn keybindings_resolve_view_specific_overrides_global() {
        let mut bindings = KeyBindings::default();

        // Override 'j' for Settings view to mean Approve
        bindings.add_view_binding(View::Settings, key('j'), Action::Approve);

        // For Dashboard, 'j' is still NavigateDown
        assert_eq!(
            bindings.resolve(key('j'), &View::Dashboard),
            Some(Action::NavigateDown)
        );

        // For Settings, 'j' is now Approve
        assert_eq!(
            bindings.resolve(key('j'), &View::Settings),
            Some(Action::Approve)
        );
    }

    #[test]
    fn keybindings_resolve_falls_back_to_global_when_not_overridden() {
        let mut bindings = KeyBindings::default();

        // Add a view-specific binding for 'a' in Settings
        bindings.add_view_binding(View::Settings, key('a'), Action::Approve);

        // 'q' should still work (fall back to global)
        assert_eq!(
            bindings.resolve(key('q'), &View::Settings),
            Some(Action::Quit)
        );
    }

    #[test]
    fn keybindings_resolve_returns_none_for_unmapped_key() {
        let bindings = KeyBindings::default();

        assert_eq!(bindings.resolve(key('z'), &View::Dashboard), None);
    }

    // ==================== Agent View Keybinding Tests ====================

    #[test]
    fn keybindings_agent_view_y_maps_to_approve() {
        let bindings = KeyBindings::default();
        assert_eq!(
            bindings.resolve(key('y'), &View::Agent("test".into())),
            Some(Action::Approve)
        );
    }

    #[test]
    fn keybindings_agent_view_n_maps_to_deny() {
        let bindings = KeyBindings::default();
        assert_eq!(
            bindings.resolve(key('n'), &View::Agent("test".into())),
            Some(Action::Deny)
        );
    }

    #[test]
    fn keybindings_agent_view_v_maps_to_view_diff() {
        let bindings = KeyBindings::default();
        assert_eq!(
            bindings.resolve(key('v'), &View::Agent("test".into())),
            Some(Action::ViewDiff)
        );
    }

    #[test]
    fn keybindings_agent_view_p_maps_to_pause() {
        let bindings = KeyBindings::default();
        assert_eq!(
            bindings.resolve(key('p'), &View::Agent("test".into())),
            Some(Action::Pause)
        );
    }

    #[test]
    fn keybindings_agent_view_c_maps_to_cancel() {
        let bindings = KeyBindings::default();
        assert_eq!(
            bindings.resolve(key('c'), &View::Agent("test".into())),
            Some(Action::Cancel)
        );
    }

    #[test]
    fn keybindings_agent_view_r_maps_to_restart() {
        let bindings = KeyBindings::default();
        assert_eq!(
            bindings.resolve(key('r'), &View::Agent("test".into())),
            Some(Action::Restart)
        );
    }

    #[test]
    fn action_enum_has_restart_variant() {
        let _restart = Action::Restart;
    }
}
