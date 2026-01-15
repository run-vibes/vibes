//! Stack-based view navigation for vibes TUI.
//!
//! Provides a stack model for "drill down and back" navigation patterns,
//! similar to lazygit.

use crate::state::{AgentId, SessionId, SwarmId};

/// Available views in the TUI.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum View {
    #[default]
    Dashboard,
    Session(SessionId),
    Agent(AgentId),
    Swarm(SwarmId),
    Models,
    Observe,
    Evals,
    Settings,
}

/// Stack-based view navigation.
///
/// Maintains the current view and a history stack for back navigation.
#[derive(Debug, Clone)]
pub struct ViewStack {
    pub current: View,
    pub history: Vec<View>,
}

impl ViewStack {
    /// Creates a new ViewStack starting at Dashboard.
    pub fn new() -> Self {
        Self {
            current: View::Dashboard,
            history: Vec::new(),
        }
    }

    /// Push new view, saving current to history.
    pub fn push(&mut self, view: View) {
        self.history
            .push(std::mem::replace(&mut self.current, view));
    }

    /// Pop to previous view, returns false if at root.
    pub fn pop(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current = prev;
            true
        } else {
            false
        }
    }

    /// Replace current view without affecting history.
    pub fn replace(&mut self, view: View) {
        self.current = view;
    }

    /// Check if we can go back.
    pub fn can_pop(&self) -> bool {
        !self.history.is_empty()
    }
}

impl Default for ViewStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_defaults_to_dashboard() {
        let view = View::default();
        assert_eq!(view, View::Dashboard);
    }

    #[test]
    fn view_enum_has_all_variants() {
        // Verify all variants exist and are constructible
        let _dashboard = View::Dashboard;
        let _session = View::Session("sess-1".into());
        let _agent = View::Agent("agent-1".into());
        let _swarm = View::Swarm("swarm-1".into());
        let _models = View::Models;
        let _observe = View::Observe;
        let _evals = View::Evals;
        let _settings = View::Settings;
    }

    #[test]
    fn viewstack_starts_with_dashboard() {
        let stack = ViewStack::new();
        assert_eq!(stack.current, View::Dashboard);
        assert!(stack.history.is_empty());
    }

    #[test]
    fn push_saves_current_to_history_and_sets_new_current() {
        let mut stack = ViewStack::new();
        assert_eq!(stack.current, View::Dashboard);

        stack.push(View::Settings);

        assert_eq!(stack.current, View::Settings);
        assert_eq!(stack.history.len(), 1);
        assert_eq!(stack.history[0], View::Dashboard);
    }

    #[test]
    fn push_multiple_views_builds_history() {
        let mut stack = ViewStack::new();

        stack.push(View::Session("sess-1".into()));
        stack.push(View::Agent("agent-1".into()));

        assert_eq!(stack.current, View::Agent("agent-1".into()));
        assert_eq!(stack.history.len(), 2);
        assert_eq!(stack.history[0], View::Dashboard);
        assert_eq!(stack.history[1], View::Session("sess-1".into()));
    }

    #[test]
    fn pop_restores_previous_view_from_history() {
        let mut stack = ViewStack::new();
        stack.push(View::Settings);
        stack.push(View::Models);

        let popped = stack.pop();

        assert!(popped);
        assert_eq!(stack.current, View::Settings);
        assert_eq!(stack.history.len(), 1);
    }

    #[test]
    fn pop_returns_false_when_history_empty() {
        let mut stack = ViewStack::new();

        let popped = stack.pop();

        assert!(!popped);
        assert_eq!(stack.current, View::Dashboard);
    }

    #[test]
    fn pop_returns_false_at_root_after_push_pop() {
        let mut stack = ViewStack::new();
        stack.push(View::Settings);
        stack.pop();

        let popped = stack.pop();

        assert!(!popped);
        assert_eq!(stack.current, View::Dashboard);
    }

    #[test]
    fn replace_changes_current_without_affecting_history() {
        let mut stack = ViewStack::new();
        stack.push(View::Settings);

        stack.replace(View::Models);

        assert_eq!(stack.current, View::Models);
        assert_eq!(stack.history.len(), 1);
        assert_eq!(stack.history[0], View::Dashboard);
    }

    #[test]
    fn replace_on_fresh_stack_changes_current_keeps_empty_history() {
        let mut stack = ViewStack::new();

        stack.replace(View::Evals);

        assert_eq!(stack.current, View::Evals);
        assert!(stack.history.is_empty());
    }

    #[test]
    fn can_pop_returns_true_when_history_not_empty() {
        let mut stack = ViewStack::new();
        stack.push(View::Settings);

        assert!(stack.can_pop());
    }

    #[test]
    fn can_pop_returns_false_when_history_empty() {
        let stack = ViewStack::new();

        assert!(!stack.can_pop());
    }

    #[test]
    fn can_pop_updates_after_pop() {
        let mut stack = ViewStack::new();
        stack.push(View::Settings);
        assert!(stack.can_pop());

        stack.pop();

        assert!(!stack.can_pop());
    }

    #[test]
    fn viewstack_default_equals_new() {
        let stack1 = ViewStack::new();
        let stack2 = ViewStack::default();

        assert_eq!(stack1.current, stack2.current);
        assert_eq!(stack1.history.len(), stack2.history.len());
    }

    #[test]
    fn view_with_ids_are_equal_when_ids_match() {
        let sess1 = View::Session("sess-1".into());
        let sess1_copy = View::Session("sess-1".into());
        let sess2 = View::Session("sess-2".into());

        assert_eq!(sess1, sess1_copy);
        assert_ne!(sess1, sess2);
    }
}
