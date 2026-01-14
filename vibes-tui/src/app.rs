//! Main application struct and event loop for vibes TUI.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};

use crate::{VibesTerminal, restore_terminal, setup_terminal};

use crate::{AppState, Theme, vibes_default};

/// Placeholder for view stack (implemented in Story 3).
#[derive(Debug, Clone, Default)]
pub struct ViewStack;

/// Placeholder for keybindings (implemented in Story 4).
#[derive(Debug, Clone, Default)]
pub struct KeyBindings;

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
            views: ViewStack,
            keybindings: KeyBindings,
            theme: vibes_default(),
            client: None,
            running: true,
        }
    }

    /// Handles a key event.
    ///
    /// Currently only handles 'q' to quit. More keybindings in Story 4.
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            _ => {}
        }
    }

    /// Renders the application to the terminal frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .constraints([Constraint::Min(0)])
            .split(area);

        let block = Block::default()
            .title(" vibes TUI ")
            .borders(Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(self.theme.border));

        let text = Paragraph::new("Press 'q' to quit")
            .style(ratatui::style::Style::default().fg(self.theme.fg))
            .block(block);

        frame.render_widget(text, chunks[0]);
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
}
