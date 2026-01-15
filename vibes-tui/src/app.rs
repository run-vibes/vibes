//! Main application struct and event loop for vibes TUI.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::keybindings::{Action, KeyBindings};
use crate::views::{DashboardView, View, ViewRenderer, ViewStack};
use crate::{AppState, Theme, VibesTerminal, restore_terminal, setup_terminal, vibes_default};

/// Placeholder for WebSocket client (implemented in Story 5).
#[derive(Debug)]
pub struct VibesClient;

/// Main TUI application.
#[derive(Debug)]
pub struct App {
    pub state: AppState,
    pub views: ViewStack,
    pub keybindings: KeyBindings,
    pub theme: Theme,
    pub client: Option<VibesClient>,
    pub running: bool,
}

impl App {
    /// Creates a new App instance with default settings.
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            views: ViewStack::new(),
            keybindings: KeyBindings::default(),
            theme: vibes_default(),
            client: None,
            running: true,
        }
    }

    /// Handles a key event.
    ///
    /// Resolves the key to an action using keybindings, then executes the action.
    /// Ctrl-C is handled as a special case to always quit.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl-C always quits (special case, not in keybindings)
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        // Resolve key to action using keybindings
        if let Some(action) = self.keybindings.resolve(key, &self.views.current) {
            self.execute_action(action);
        }
    }

    /// Executes an action.
    fn execute_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::Back => {
                self.views.pop();
            }
            // Navigation and other actions will be wired up in future stories
            Action::NavigateUp
            | Action::NavigateDown
            | Action::NavigateLeft
            | Action::NavigateRight
            | Action::Select
            | Action::CommandMode
            | Action::SearchMode
            | Action::HelpMode
            | Action::JumpToView(_)
            | Action::Approve
            | Action::Deny
            | Action::Pause
            | Action::Resume
            | Action::Cancel
            | Action::ViewDiff => {
                // Placeholder - will be implemented in future stories
            }
        }
    }

    /// Renders the application to the terminal frame.
    ///
    /// Delegates to the current view's renderer based on the view stack.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .constraints([Constraint::Min(0)])
            .split(area);

        // Delegate rendering to the current view
        self.render_current_view(frame, chunks[0]);
    }

    /// Renders the current view from the view stack.
    fn render_current_view(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match &self.views.current {
            View::Dashboard => DashboardView.render(frame, area, self),
            // Other views will be implemented in later stories
            _ => DashboardView.render(frame, area, self),
        }
    }

    /// Processes async updates (WebSocket messages, timers, etc.).
    ///
    /// Currently a no-op placeholder. Expanded in Story 5.
    pub async fn tick(&mut self) {
        // Placeholder for async event processing
    }

    /// Runs the main event loop.
    ///
    /// Sets up the terminal, enters the render/input loop, and restores
    /// the terminal on exit. Returns an error if terminal setup fails.
    pub async fn run(&mut self) -> io::Result<()> {
        let mut terminal = setup_terminal()?;

        let result = self.event_loop(&mut terminal).await;

        // Always restore terminal, even if event loop failed
        restore_terminal(&mut terminal)?;

        result
    }

    /// The core event loop. Separated from `run` for testability.
    async fn event_loop(&mut self, terminal: &mut VibesTerminal) -> io::Result<()> {
        while self.running {
            // Render
            terminal.draw(|f| self.render(f))?;

            // Handle input with timeout for tick
            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
            {
                self.handle_key(key);
            }

            // Process async updates
            self.tick().await;
        }

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_new_starts_running() {
        let app = App::new();
        assert!(app.running);
    }

    #[test]
    fn app_new_has_default_state() {
        let app = App::new();
        assert!(app.state.session.is_none());
        assert!(app.client.is_none());
    }

    #[test]
    fn app_new_has_vibes_theme() {
        let app = App::new();
        assert_eq!(app.theme.name, "vibes");
    }

    #[test]
    fn handle_key_q_stops_running() {
        let mut app = App::new();
        assert!(app.running);

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        app.handle_key(key);

        assert!(!app.running);
    }

    #[test]
    fn handle_key_ctrl_c_stops_running() {
        let mut app = App::new();
        assert!(app.running);

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        app.handle_key(key);

        assert!(!app.running);
    }

    #[test]
    fn handle_key_other_keys_dont_stop() {
        let mut app = App::new();

        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(key);
        assert!(app.running);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.handle_key(key);
        assert!(app.running);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(key);
        assert!(app.running);
    }

    #[test]
    fn app_default_equals_new() {
        let app1 = App::new();
        let app2 = App::default();

        assert_eq!(app1.running, app2.running);
        assert_eq!(app1.theme.name, app2.theme.name);
    }

    #[tokio::test]
    async fn tick_completes_without_error() {
        let mut app = App::new();
        app.tick().await;
        // Should complete without panicking
    }

    #[test]
    fn app_new_has_dashboard_as_current_view() {
        let app = App::new();
        assert_eq!(app.views.current, View::Dashboard);
    }

    #[test]
    fn app_new_has_empty_view_history() {
        let app = App::new();
        assert!(app.views.history.is_empty());
    }

    #[test]
    fn app_viewstack_push_works() {
        let mut app = App::new();
        app.views.push(View::Settings);

        assert_eq!(app.views.current, View::Settings);
        assert_eq!(app.views.history.len(), 1);
    }

    #[test]
    fn app_viewstack_pop_returns_to_dashboard() {
        let mut app = App::new();
        app.views.push(View::Settings);
        app.views.pop();

        assert_eq!(app.views.current, View::Dashboard);
        assert!(app.views.history.is_empty());
    }

    #[test]
    fn handle_key_esc_pops_view_stack() {
        let mut app = App::new();
        app.views.push(View::Settings);
        assert_eq!(app.views.current, View::Settings);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(key);

        // Esc triggers Back action which pops the view stack
        assert_eq!(app.views.current, View::Dashboard);
        assert!(app.views.history.is_empty());
        // But app should still be running
        assert!(app.running);
    }

    #[test]
    fn handle_key_esc_at_root_does_nothing() {
        let mut app = App::new();
        assert_eq!(app.views.current, View::Dashboard);
        assert!(app.views.history.is_empty());

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(key);

        // At root, Esc does nothing (pop returns false)
        assert_eq!(app.views.current, View::Dashboard);
        assert!(app.running);
    }
}
