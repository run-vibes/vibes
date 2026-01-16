//! Main application struct and event loop for vibes TUI.

use std::io;
use std::time::Duration;

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::client::{ReconnectConfig, TuiClient};
use crate::keybindings::{Action, KeyBindings};
use crate::views::{AgentView, DashboardView, SwarmView, View, ViewRenderer, ViewStack};
use crate::widgets::{
    ActivityEvent, ActivityFeedWidget, ConnectionStatus, SessionInfo, SessionListWidget,
    SessionStatus, StatsBarWidget,
};
use crate::{
    AppState, Mode, Theme, VibesTerminal, restore_terminal, setup_terminal, vibes_default,
};

/// Main TUI application.
#[derive(Debug)]
pub struct App {
    pub state: AppState,
    pub views: ViewStack,
    pub keybindings: KeyBindings,
    pub theme: Theme,
    pub client: Option<TuiClient>,
    pub running: bool,
    /// Error message to display (e.g., connection errors).
    pub error_message: Option<String>,
    /// Server URL for reconnection attempts.
    pub server_url: Option<String>,
    /// Flag indicating a retry was requested (for command to handle).
    pub retry_requested: bool,
    /// Session list widget for the dashboard.
    pub session_widget: SessionListWidget,
    /// Stats summary bar widget for the dashboard.
    pub stats_widget: StatsBarWidget,
    /// Activity feed widget for the dashboard.
    pub activity_widget: ActivityFeedWidget,
    /// Reconnection configuration.
    pub reconnect_config: ReconnectConfig,
    /// Current reconnection attempt number.
    reconnect_attempt: u32,
    /// Time of last reconnection attempt (for backoff).
    last_reconnect_attempt: Option<std::time::Instant>,
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
            error_message: None,
            server_url: None,
            retry_requested: false,
            session_widget: SessionListWidget::new(),
            stats_widget: StatsBarWidget::default(),
            activity_widget: ActivityFeedWidget::new(),
            reconnect_config: ReconnectConfig::default(),
            reconnect_attempt: 0,
            last_reconnect_attempt: None,
        }
    }

    /// Creates a new App with a connected client.
    pub fn with_client(client: TuiClient) -> Self {
        Self {
            state: AppState::default(),
            views: ViewStack::new(),
            keybindings: KeyBindings::default(),
            theme: vibes_default(),
            client: Some(client),
            running: true,
            error_message: None,
            server_url: None,
            retry_requested: false,
            session_widget: SessionListWidget::new(),
            stats_widget: StatsBarWidget {
                connection_status: ConnectionStatus::Connected,
                ..Default::default()
            },
            activity_widget: ActivityFeedWidget::new(),
            reconnect_config: ReconnectConfig::default(),
            reconnect_attempt: 0,
            last_reconnect_attempt: None,
        }
    }

    /// Creates a new App with a connected client and stores the server URL.
    pub fn with_client_url(client: TuiClient, url: String) -> Self {
        Self {
            state: AppState::default(),
            views: ViewStack::new(),
            keybindings: KeyBindings::default(),
            theme: vibes_default(),
            client: Some(client),
            running: true,
            error_message: None,
            server_url: Some(url),
            retry_requested: false,
            session_widget: SessionListWidget::new(),
            stats_widget: StatsBarWidget {
                connection_status: ConnectionStatus::Connected,
                ..Default::default()
            },
            activity_widget: ActivityFeedWidget::new(),
            reconnect_config: ReconnectConfig::default(),
            reconnect_attempt: 0,
            last_reconnect_attempt: None,
        }
    }

    /// Sets the client for this app.
    pub fn set_client(&mut self, client: TuiClient) {
        self.client = Some(client);
        self.error_message = None;
        self.stats_widget.connection_status = ConnectionStatus::Connected;
    }

    /// Handles a connection error by showing an error message.
    pub fn handle_connection_error(&mut self, error: &str) {
        self.state.mode = Mode::Normal;
        self.stats_widget.connection_status = ConnectionStatus::Disconnected;
        self.error_message = Some(format!(
            "Connection lost: {}. Press 'r' to retry or 'q' to quit.",
            error
        ));
    }

    /// Clears the current error message.
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Updates the stats widget based on current session data.
    fn update_stats(&mut self) {
        // Count active sessions (Running status)
        let active_sessions = self
            .session_widget
            .sessions
            .iter()
            .filter(|s| s.status == SessionStatus::Running)
            .count() as u32;

        // Sum agent counts from all sessions
        let running_agents = self
            .session_widget
            .sessions
            .iter()
            .map(|s| s.agent_count as u32)
            .sum();

        self.stats_widget.session_count = active_sessions;
        self.stats_widget.agent_count = running_agents;
        // total_cost is updated separately when cost data arrives
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
            Action::Retry => {
                // Only retry if there's an error and we have a server URL
                if self.error_message.is_some() && self.server_url.is_some() {
                    self.retry_requested = true;
                    self.running = false; // Exit to allow command to handle reconnection
                }
            }
            // Navigation actions for session list
            Action::NavigateUp => {
                if self.views.current == View::Dashboard {
                    self.session_widget.select_prev();
                }
            }
            Action::NavigateDown => {
                if self.views.current == View::Dashboard {
                    self.session_widget.select_next();
                }
            }
            Action::Select => {
                if self.views.current == View::Dashboard {
                    // Navigate to session detail view (future story)
                    if let Some(session) = self.session_widget.selected_session() {
                        tracing::debug!("Selected session: {}", session.id);
                        // TODO: Push session detail view
                    }
                }
            }
            // Agent control actions
            Action::Pause | Action::Resume => {
                // Both Pause and Resume map to 'p' key - toggles between states
                if let View::Agent(agent_id) = &self.views.current
                    && let Some(agent_state) = self.state.agents.get_mut(agent_id)
                {
                    use crate::widgets::AgentStatus;
                    let current = agent_state.control_bar.status();
                    match current {
                        AgentStatus::Running | AgentStatus::WaitingForPermission => {
                            agent_state.control_bar.set_status(AgentStatus::Paused);
                            // TODO: Send pause command to server
                        }
                        AgentStatus::Paused => {
                            agent_state.control_bar.set_status(AgentStatus::Running);
                            // TODO: Send resume command to server
                        }
                        // Completed/Failed/Cancelled states don't support pause/resume
                        _ => {}
                    }
                }
            }
            Action::Cancel => {
                if let View::Agent(agent_id) = &self.views.current
                    && let Some(agent_state) = self.state.agents.get_mut(agent_id)
                {
                    use crate::widgets::ConfirmationType;
                    // Show confirmation dialog for cancel
                    agent_state.confirmation.show(ConfirmationType::Cancel);
                }
            }
            Action::Restart => {
                if let View::Agent(agent_id) = &self.views.current
                    && let Some(agent_state) = self.state.agents.get_mut(agent_id)
                {
                    use crate::widgets::ConfirmationType;
                    // Show confirmation dialog for restart
                    agent_state.confirmation.show(ConfirmationType::Restart);
                }
            }
            // Other actions - will be wired up in future stories
            Action::NavigateLeft
            | Action::NavigateRight
            | Action::CommandMode
            | Action::SearchMode
            | Action::HelpMode
            | Action::JumpToView(_)
            | Action::Approve
            | Action::Deny
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
            View::Agent(agent_id) => AgentView::new(agent_id.clone()).render(frame, area, self),
            View::Swarm(swarm_id) => SwarmView::new(swarm_id.clone()).render(frame, area, self),
            // Other views will be implemented in later stories
            _ => DashboardView.render(frame, area, self),
        }
    }

    /// Processes async updates (WebSocket messages, timers, etc.).
    ///
    /// Polls the client for incoming messages and updates state accordingly.
    /// Also handles auto-reconnection when the connection is lost.
    pub async fn tick(&mut self) {
        // Check connection status and attempt reconnection if needed
        self.check_connection().await;

        // Collect messages first to avoid borrow issues
        let messages: Vec<_> = if let Some(client) = &mut self.client {
            std::iter::from_fn(|| client.try_recv()).collect()
        } else {
            Vec::new()
        };

        // Then process them
        for msg in messages {
            self.handle_server_message(msg);
        }
    }

    /// Checks the connection status and attempts reconnection if needed.
    async fn check_connection(&mut self) {
        // Check if we have a client and if it's still connected
        let is_connected = self.client.as_ref().is_some_and(|c| c.is_connected());

        if !is_connected && self.server_url.is_some() {
            // Connection lost, attempt to reconnect
            self.stats_widget.connection_status = ConnectionStatus::Reconnecting;

            // Check if enough time has passed since last attempt (exponential backoff)
            let should_attempt = self.last_reconnect_attempt.is_none_or(|last| {
                let required_delay = self
                    .reconnect_config
                    .delay_for_attempt(self.reconnect_attempt);
                last.elapsed() >= required_delay
            });

            if should_attempt {
                self.attempt_reconnect().await;
            }
        } else if is_connected {
            // Connection is healthy, reset reconnect state
            self.reconnect_attempt = 0;
            self.last_reconnect_attempt = None;
            self.stats_widget.connection_status = ConnectionStatus::Connected;
        }
    }

    /// Attempts to reconnect to the server.
    async fn attempt_reconnect(&mut self) {
        let url = match &self.server_url {
            Some(url) => url.clone(),
            None => return,
        };

        // Check max attempts if configured
        if let Some(max) = self.reconnect_config.max_attempts
            && self.reconnect_attempt >= max
        {
            self.stats_widget.connection_status = ConnectionStatus::Disconnected;
            self.error_message =
                Some("Max reconnection attempts reached. Press 'r' to retry.".to_string());
            return;
        }

        tracing::info!(
            attempt = self.reconnect_attempt + 1,
            "Attempting to reconnect to {}",
            url
        );

        self.last_reconnect_attempt = Some(std::time::Instant::now());

        match TuiClient::connect(&url).await {
            Ok(client) => {
                tracing::info!("Reconnected successfully");
                self.client = Some(client);
                self.stats_widget.connection_status = ConnectionStatus::Connected;
                self.reconnect_attempt = 0;
                self.error_message = None;

                // Request fresh data after reconnect
                if let Some(client) = &self.client {
                    let _ = client.list_sessions("reconnect-sessions").await;
                    let _ = client.list_agents("reconnect-agents").await;
                }
            }
            Err(e) => {
                tracing::warn!(
                    attempt = self.reconnect_attempt + 1,
                    error = %e,
                    "Reconnection failed"
                );
                self.reconnect_attempt += 1;
                self.client = None;
            }
        }
    }

    /// Handles a server message.
    fn handle_server_message(&mut self, msg: vibes_server::ws::ServerMessage) {
        use vibes_server::ws::ServerMessage;

        match msg {
            ServerMessage::SessionList { sessions, .. } => {
                // Update session widget with session list
                tracing::debug!("Received {} sessions", sessions.len());
                self.session_widget.sessions = sessions
                    .into_iter()
                    .map(|s| SessionInfo {
                        id: s.id,
                        status: match s.state.as_str() {
                            "running" | "active" => SessionStatus::Running,
                            "paused" => SessionStatus::Paused,
                            "completed" | "closed" => SessionStatus::Completed,
                            "failed" | "error" => SessionStatus::Failed,
                            _ => SessionStatus::Running,
                        },
                        agent_count: 0, // Will be populated when agents are fetched
                        branch: None,   // Branch info not in current protocol
                        name: s.name,
                    })
                    .collect();

                // Update stats widget based on sessions
                self.update_stats();
            }
            ServerMessage::AgentList { agents, .. } => {
                // Update state with agent list
                tracing::debug!("Received {} agents", agents.len());
            }
            ServerMessage::Error { message, .. } => {
                self.error_message = Some(message);
            }
            // Real-time session notifications
            ServerMessage::SessionNotification { session_id, name } => {
                tracing::debug!("Session created: {}", session_id);

                // Add new session to the list
                let display_name = name
                    .clone()
                    .unwrap_or_else(|| session_id[..8.min(session_id.len())].to_string());
                self.session_widget.sessions.push(SessionInfo {
                    id: session_id.clone(),
                    status: SessionStatus::Running,
                    agent_count: 0,
                    branch: None,
                    name: name.clone(),
                });

                // Add to activity feed
                self.activity_widget.push_event(ActivityEvent {
                    time: Local::now().format("%H:%M").to_string(),
                    source: "session".into(),
                    description: format!("created \"{}\"", display_name),
                });

                self.update_stats();
            }
            ServerMessage::SessionState { session_id, state } => {
                tracing::debug!("Session {} state changed to: {}", session_id, state);

                // Update session status in the list
                if let Some(session) = self
                    .session_widget
                    .sessions
                    .iter_mut()
                    .find(|s| s.id == session_id)
                {
                    session.status = match state.as_str() {
                        "running" | "active" => SessionStatus::Running,
                        "paused" => SessionStatus::Paused,
                        "completed" | "closed" => SessionStatus::Completed,
                        "failed" | "error" => SessionStatus::Failed,
                        _ => session.status,
                    };
                }

                // Add to activity feed
                let display_id = &session_id[..8.min(session_id.len())];
                self.activity_widget.push_event(ActivityEvent {
                    time: Local::now().format("%H:%M").to_string(),
                    source: display_id.to_string(),
                    description: format!("state → {}", state),
                });

                self.update_stats();
            }
            ServerMessage::SessionRemoved { session_id, reason } => {
                tracing::debug!("Session {} removed: {:?}", session_id, reason);

                // Remove session from the list
                self.session_widget.sessions.retain(|s| s.id != session_id);

                // Add to activity feed
                let display_id = &session_id[..8.min(session_id.len())];
                let reason_str = match reason {
                    vibes_server::ws::RemovalReason::OwnerDisconnected => "disconnected",
                    vibes_server::ws::RemovalReason::Killed => "killed",
                    vibes_server::ws::RemovalReason::SessionFinished => "finished",
                };
                self.activity_widget.push_event(ActivityEvent {
                    time: Local::now().format("%H:%M").to_string(),
                    source: display_id.to_string(),
                    description: reason_str.to_string(),
                });

                self.update_stats();
            }
            // Other messages will be handled as views are implemented
            _ => {}
        }
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

    #[test]
    fn app_new_has_no_error_message() {
        let app = App::new();
        assert!(app.error_message.is_none());
    }

    #[test]
    fn handle_connection_error_sets_message() {
        let mut app = App::new();
        app.handle_connection_error("Connection refused");

        assert!(app.error_message.is_some());
        assert!(
            app.error_message
                .as_ref()
                .unwrap()
                .contains("Connection refused")
        );
        assert!(app.error_message.as_ref().unwrap().contains("'r' to retry"));
    }

    #[test]
    fn clear_error_removes_message() {
        let mut app = App::new();
        app.handle_connection_error("Test error");
        assert!(app.error_message.is_some());

        app.clear_error();
        assert!(app.error_message.is_none());
    }

    // ==================== Agent Control Action Tests ====================

    #[test]
    fn handle_key_p_in_agent_view_toggles_pause_when_running() {
        use crate::widgets::AgentStatus;

        let mut app = App::new();
        // Set up agent state
        let agent_id = "test-agent".to_string();
        app.state
            .agents
            .insert(agent_id.clone(), Default::default());
        // Navigate to agent view
        app.views.push(View::Agent(agent_id.clone()));

        // Verify initial state is Running
        assert_eq!(
            app.state
                .agents
                .get(&agent_id)
                .unwrap()
                .control_bar
                .status(),
            AgentStatus::Running
        );

        // Press 'p' to pause
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        app.handle_key(key);

        // Should now be Paused
        assert_eq!(
            app.state
                .agents
                .get(&agent_id)
                .unwrap()
                .control_bar
                .status(),
            AgentStatus::Paused
        );
    }

    #[test]
    fn handle_key_p_in_agent_view_toggles_resume_when_paused() {
        use crate::widgets::AgentStatus;

        let mut app = App::new();
        let agent_id = "test-agent".to_string();
        let mut agent_state = crate::state::AgentState::default();
        agent_state.control_bar.set_status(AgentStatus::Paused);
        app.state.agents.insert(agent_id.clone(), agent_state);
        app.views.push(View::Agent(agent_id.clone()));

        // Press 'p' to resume
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        app.handle_key(key);

        // Should now be Running
        assert_eq!(
            app.state
                .agents
                .get(&agent_id)
                .unwrap()
                .control_bar
                .status(),
            AgentStatus::Running
        );
    }

    #[test]
    fn handle_key_c_in_agent_view_shows_cancel_confirmation() {
        use crate::widgets::ConfirmationType;

        let mut app = App::new();
        let agent_id = "test-agent".to_string();
        app.state
            .agents
            .insert(agent_id.clone(), Default::default());
        app.views.push(View::Agent(agent_id.clone()));

        // Verify confirmation is not visible initially
        assert!(
            !app.state
                .agents
                .get(&agent_id)
                .unwrap()
                .confirmation
                .is_visible()
        );

        // Press 'c' to show cancel confirmation
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        app.handle_key(key);

        // Confirmation should now be visible with Cancel type
        let confirmation = &app.state.agents.get(&agent_id).unwrap().confirmation;
        assert!(confirmation.is_visible());
        assert_eq!(
            confirmation.confirmation_type(),
            Some(ConfirmationType::Cancel)
        );
    }

    #[test]
    fn handle_key_r_in_agent_view_shows_restart_confirmation() {
        use crate::widgets::ConfirmationType;

        let mut app = App::new();
        let agent_id = "test-agent".to_string();
        app.state
            .agents
            .insert(agent_id.clone(), Default::default());
        app.views.push(View::Agent(agent_id.clone()));

        // Press 'r' to show restart confirmation
        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        app.handle_key(key);

        // Confirmation should now be visible with Restart type
        let confirmation = &app.state.agents.get(&agent_id).unwrap().confirmation;
        assert!(confirmation.is_visible());
        assert_eq!(
            confirmation.confirmation_type(),
            Some(ConfirmationType::Restart)
        );
    }
}
