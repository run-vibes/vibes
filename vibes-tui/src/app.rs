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
use crate::commands::{
    CommandInput, CommandRegistry, CommandResult, SettingsCommand, ThemeCommand,
};
use crate::keybindings::{Action, KeyBindings};
use crate::views::{
    AgentView, DashboardView, SettingsView, SwarmView, View, ViewRenderer, ViewStack,
};
use crate::widgets::{
    ActivityEvent, ActivityFeedWidget, CommandBarWidget, ConnectionStatus, SessionInfo,
    SessionListWidget, SessionStatus, StatsBarWidget,
};
use crate::{
    AppState, Mode, SettingsState, Theme, ThemeLoader, VibesTerminal, restore_terminal,
    setup_terminal, vibes_default,
};

/// Main TUI application.
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
    /// Command input state for command mode.
    pub command_input: CommandInput,
    /// Registry of available commands.
    pub command_registry: CommandRegistry,
    /// Theme loader for accessing themes.
    pub theme_loader: ThemeLoader,
    /// Settings view state (when in settings view).
    pub settings_state: Option<SettingsState>,
}

impl App {
    /// Creates a new App instance with default settings.
    pub fn new() -> Self {
        let loader = ThemeLoader::from_default_config();
        let theme = loader.active().cloned().unwrap_or_else(vibes_default);
        let command_registry = Self::create_command_registry(loader.clone());

        Self {
            state: AppState::default(),
            views: ViewStack::new(),
            keybindings: KeyBindings::default(),
            theme,
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
            command_input: CommandInput::default(),
            command_registry,
            theme_loader: loader,
            settings_state: None,
        }
    }

    /// Creates the command registry with all available commands.
    fn create_command_registry(loader: ThemeLoader) -> CommandRegistry {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(ThemeCommand::new(loader)));
        registry.register(Box::new(SettingsCommand));
        registry
    }

    /// Creates a new App with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        let loader = ThemeLoader::new();
        let command_registry = Self::create_command_registry(loader.clone());

        Self {
            state: AppState::default(),
            views: ViewStack::new(),
            keybindings: KeyBindings::default(),
            theme,
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
            command_input: CommandInput::default(),
            command_registry,
            theme_loader: loader,
            settings_state: None,
        }
    }

    /// Creates a new App with a connected client.
    pub fn with_client(client: TuiClient) -> Self {
        let loader = ThemeLoader::from_default_config();
        let theme = loader.active().cloned().unwrap_or_else(vibes_default);
        let command_registry = Self::create_command_registry(loader.clone());

        Self {
            state: AppState::default(),
            views: ViewStack::new(),
            keybindings: KeyBindings::default(),
            theme,
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
            command_input: CommandInput::default(),
            command_registry,
            theme_loader: loader,
            settings_state: None,
        }
    }

    /// Creates a new App with a connected client and stores the server URL.
    pub fn with_client_url(client: TuiClient, url: String) -> Self {
        let loader = ThemeLoader::from_default_config();
        let theme = loader.active().cloned().unwrap_or_else(vibes_default);
        let command_registry = Self::create_command_registry(loader.clone());

        Self {
            state: AppState::default(),
            views: ViewStack::new(),
            keybindings: KeyBindings::default(),
            theme,
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
            command_input: CommandInput::default(),
            command_registry,
            theme_loader: loader,
            settings_state: None,
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
    /// In Command mode, keys are routed to the command input instead.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl-C always quits (special case, not in keybindings)
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        // In command mode, handle keys specially
        if self.state.mode == Mode::Command {
            self.handle_command_mode_key(key);
            return;
        }

        // Resolve key to action using keybindings
        if let Some(action) = self.keybindings.resolve(key, &self.views.current) {
            self.execute_action(action);
        }
    }

    /// Handle keys when in command mode.
    fn handle_command_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Exit command mode
                self.command_input.clear();
                self.state.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                // Execute the command
                let input = self.command_input.buffer.clone();
                self.command_input.clear();
                self.state.mode = Mode::Normal;

                if !input.trim().is_empty() {
                    // Take registry out temporarily to avoid borrow conflict
                    let mut registry = std::mem::take(&mut self.command_registry);
                    let result = registry.execute(&input, self);
                    self.command_registry = registry;

                    match result {
                        CommandResult::Ok(Some(msg)) => {
                            self.command_input.set_message(&msg, false);
                        }
                        CommandResult::Err(msg) => {
                            self.command_input.set_message(&msg, true);
                        }
                        CommandResult::Quit => {
                            self.running = false;
                        }
                        CommandResult::Ok(None) => {}
                    }
                }
            }
            KeyCode::Backspace => {
                self.command_input.backspace();
            }
            KeyCode::Delete => {
                self.command_input.delete();
            }
            KeyCode::Left => {
                self.command_input.move_left();
            }
            KeyCode::Right => {
                self.command_input.move_right();
            }
            KeyCode::Home => {
                self.command_input.move_to_start();
            }
            KeyCode::End => {
                self.command_input.move_to_end();
            }
            KeyCode::Tab => {
                // Trigger tab completion
                if self.command_input.completion_idx.is_some() {
                    self.command_input.next_completion();
                } else {
                    let buffer = self.command_input.buffer.clone();
                    let completions = self.command_registry.completions(&buffer, self);
                    self.command_input.set_completions(completions);
                    self.command_input.next_completion();
                }
            }
            KeyCode::Char(c) => {
                self.command_input.insert(c);
                // Clear completions when typing
                self.command_input.completions.clear();
                self.command_input.completion_idx = None;
            }
            _ => {}
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
                } else if self.views.current == View::Settings
                    && let Some(ref mut settings) = self.settings_state
                {
                    let idx = settings.selected_index();
                    if idx > 0 {
                        settings.set_selected_index(idx - 1);
                        // Update preview theme
                        let themes = self.theme_loader.list();
                        if let Some(name) = themes.get(idx - 1) {
                            settings.set_preview_theme(name);
                        }
                    }
                }
            }
            Action::NavigateDown => {
                if self.views.current == View::Dashboard {
                    self.session_widget.select_next();
                } else if self.views.current == View::Settings
                    && let Some(ref mut settings) = self.settings_state
                {
                    let idx = settings.selected_index();
                    let themes = self.theme_loader.list();
                    if idx + 1 < themes.len() {
                        settings.set_selected_index(idx + 1);
                        // Update preview theme
                        if let Some(name) = themes.get(idx + 1) {
                            settings.set_preview_theme(name);
                        }
                    }
                }
            }
            Action::Select => {
                if self.views.current == View::Dashboard {
                    // Navigate to session detail view (future story)
                    if let Some(session) = self.session_widget.selected_session() {
                        tracing::debug!("Selected session: {}", session.id);
                        // TODO: Push session detail view
                    }
                } else if self.views.current == View::Settings {
                    // Apply the selected theme
                    if let Some(ref settings) = self.settings_state {
                        let preview = settings.preview_theme().to_string();
                        if let Some(theme) = self.theme_loader.get(&preview) {
                            self.theme = theme.clone();
                        }
                    }
                    // Clear settings state and return to previous view
                    self.settings_state = None;
                    self.views.pop();
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
            // Swarm merge actions
            Action::Merge => {
                if let View::Swarm(swarm_id) = &self.views.current
                    && let Some(swarm_state) = self.state.swarms.get_mut(swarm_id)
                {
                    // Show merge dialog if not already showing results
                    if !swarm_state.merge_results.is_visible() {
                        // TODO: Gather completed agents from swarm data
                        // For now, this will be wired up when swarm data is available
                    }
                }
            }
            Action::CopyToClipboard => {
                if let View::Swarm(swarm_id) = &self.views.current
                    && let Some(swarm_state) = self.state.swarms.get(swarm_id)
                    && swarm_state.merge_results.is_visible()
                {
                    let content = swarm_state.merge_results.as_markdown();
                    match crate::clipboard::copy_to_clipboard(&content) {
                        Ok(()) => {
                            tracing::info!("Copied merged results to clipboard");
                        }
                        Err(e) => {
                            tracing::warn!("Failed to copy to clipboard: {}", e);
                        }
                    }
                }
            }
            Action::SaveToFile => {
                if let View::Swarm(swarm_id) = &self.views.current
                    && let Some(swarm_state) = self.state.swarms.get(swarm_id)
                    && swarm_state.merge_results.is_visible()
                {
                    let content = swarm_state.merge_results.as_markdown();
                    // Generate timestamped filename
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let filename = format!("merged_results_{}.md", timestamp);
                    match std::fs::write(&filename, &content) {
                        Ok(()) => {
                            tracing::info!("Saved merged results to {}", filename);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to save to file: {}", e);
                        }
                    }
                }
            }
            Action::ScrollUp => {
                if let View::Swarm(swarm_id) = &self.views.current
                    && let Some(swarm_state) = self.state.swarms.get_mut(swarm_id)
                {
                    swarm_state.merge_results.scroll_up();
                }
            }
            Action::ScrollDown => {
                if let View::Swarm(swarm_id) = &self.views.current
                    && let Some(swarm_state) = self.state.swarms.get_mut(swarm_id)
                {
                    // TODO: Get actual viewport height from render context
                    swarm_state.merge_results.scroll_down(20);
                }
            }
            Action::PageUp => {
                if let View::Swarm(swarm_id) = &self.views.current
                    && let Some(swarm_state) = self.state.swarms.get_mut(swarm_id)
                {
                    // TODO: Get actual viewport height from render context
                    swarm_state.merge_results.page_up(20);
                }
            }
            Action::PageDown => {
                if let View::Swarm(swarm_id) = &self.views.current
                    && let Some(swarm_state) = self.state.swarms.get_mut(swarm_id)
                {
                    // TODO: Get actual viewport height from render context
                    swarm_state.merge_results.page_down(20);
                }
            }
            Action::CommandMode => {
                self.state.mode = Mode::Command;
                self.command_input.clear();
            }
            Action::OpenSettings => {
                // Initialize settings state with current theme
                self.settings_state = Some(SettingsState::new(&self.theme.name));
                self.views.push(View::Settings);
            }
            // Other actions - will be wired up in future stories
            Action::NavigateLeft
            | Action::NavigateRight
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
    /// Shows the command bar at the bottom when in command mode.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Reserve space for command bar if in command mode
        let (main_area, command_area) = if self.state.mode == Mode::Command {
            let chunks = Layout::default()
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        // Delegate rendering to the current view
        self.render_current_view(frame, main_area);

        // Render command bar if in command mode
        if let Some(cmd_area) = command_area {
            CommandBarWidget::render(frame, cmd_area, &self.command_input, &self.theme);
        }
    }

    /// Renders the current view from the view stack.
    fn render_current_view(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match &self.views.current {
            View::Dashboard => DashboardView.render(frame, area, self),
            View::Agent(agent_id) => AgentView::new(agent_id.clone()).render(frame, area, self),
            View::Swarm(swarm_id) => SwarmView::new(swarm_id.clone()).render(frame, area, self),
            View::Settings => SettingsView.render(frame, area, self),
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
    fn app_with_theme_uses_provided_theme() {
        let mut custom = vibes_default();
        custom.name = "custom".into();
        let app = App::with_theme(custom);
        assert_eq!(app.theme.name, "custom");
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

    // ==================== Command Mode Tests ====================

    #[test]
    fn handle_key_colon_enters_command_mode() {
        let mut app = App::new();
        assert_eq!(app.state.mode, Mode::Normal);

        let key = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
        app.handle_key(key);

        assert_eq!(app.state.mode, Mode::Command);
    }

    #[test]
    fn command_mode_escape_returns_to_normal() {
        let mut app = App::new();
        app.state.mode = Mode::Command;

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(key);

        assert_eq!(app.state.mode, Mode::Normal);
    }

    #[test]
    fn command_mode_typing_fills_buffer() {
        let mut app = App::new();
        app.state.mode = Mode::Command;

        // Type "theme"
        for c in "theme".chars() {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            app.handle_key(key);
        }

        assert_eq!(app.command_input.buffer, "theme");
    }

    #[test]
    fn command_mode_enter_executes_command() {
        let mut app = App::new();
        app.state.mode = Mode::Command;
        app.command_input.buffer = "theme dark".into();
        app.command_input.cursor = 10;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.handle_key(key);

        // Theme should have changed
        assert_eq!(app.theme.name, "dark");
        // Should return to normal mode
        assert_eq!(app.state.mode, Mode::Normal);
    }

    #[test]
    fn command_mode_backspace_deletes_char() {
        let mut app = App::new();
        app.state.mode = Mode::Command;
        app.command_input.buffer = "theme".into();
        app.command_input.cursor = 5;

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        app.handle_key(key);

        assert_eq!(app.command_input.buffer, "them");
    }

    #[test]
    fn command_mode_tab_triggers_completion() {
        let mut app = App::new();
        app.state.mode = Mode::Command;
        app.command_input.buffer = "th".into();
        app.command_input.cursor = 2;

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        app.handle_key(key);

        // Should have completions
        assert!(!app.command_input.completions.is_empty());
    }

    #[test]
    fn theme_command_changes_app_theme() {
        let mut app = App::new();
        assert_eq!(app.theme.name, "vibes");

        // Enter command mode and execute theme command
        app.state.mode = Mode::Command;
        app.command_input.buffer = "theme light".into();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.handle_key(key);

        assert_eq!(app.theme.name, "light");
    }

    // ==================== Settings View Tests ====================

    #[test]
    fn handle_key_s_opens_settings_view() {
        let mut app = App::new();
        assert_eq!(app.views.current, View::Dashboard);
        assert!(app.settings_state.is_none());

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);

        assert_eq!(app.views.current, View::Settings);
        assert!(app.settings_state.is_some());
    }

    #[test]
    fn settings_state_initialized_with_current_theme() {
        let mut app = App::new();
        assert_eq!(app.theme.name, "vibes");

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);

        let settings = app.settings_state.as_ref().unwrap();
        assert_eq!(settings.original_theme(), "vibes");
        assert_eq!(settings.preview_theme(), "vibes");
    }

    #[test]
    fn settings_view_esc_returns_to_dashboard() {
        let mut app = App::new();

        // Open settings
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);
        assert_eq!(app.views.current, View::Settings);

        // Press Esc to go back
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(key);

        assert_eq!(app.views.current, View::Dashboard);
    }

    #[test]
    fn settings_view_j_navigates_down() {
        let mut app = App::new();

        // Open settings
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);
        assert_eq!(app.views.current, View::Settings);

        // Initial selected index is 0
        assert_eq!(app.settings_state.as_ref().unwrap().selected_index(), 0);

        // Press j to go down
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(key);

        // Selected index should increase
        assert_eq!(app.settings_state.as_ref().unwrap().selected_index(), 1);
    }

    #[test]
    fn settings_view_k_navigates_up() {
        let mut app = App::new();

        // Open settings and move down first
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);

        // Move down twice
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(key);
        app.handle_key(key);
        assert_eq!(app.settings_state.as_ref().unwrap().selected_index(), 2);

        // Press k to go up
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        app.handle_key(key);

        assert_eq!(app.settings_state.as_ref().unwrap().selected_index(), 1);
    }

    #[test]
    fn settings_view_navigation_updates_preview_theme() {
        let mut app = App::new();

        // Open settings
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);

        // Initial preview is vibes
        assert_eq!(
            app.settings_state.as_ref().unwrap().preview_theme(),
            "vibes"
        );

        // Navigate down to select next theme
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(key);

        // Preview theme should change to the next theme in the list
        let themes = app.theme_loader.list();
        let expected_theme = themes.get(1).unwrap();
        assert_eq!(
            app.settings_state.as_ref().unwrap().preview_theme(),
            *expected_theme
        );
    }

    #[test]
    fn settings_view_navigation_wraps_at_boundaries() {
        let mut app = App::new();

        // Open settings
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);

        // Press k when at index 0 should stay at 0 (or wrap to end)
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        app.handle_key(key);

        // Should stay at 0 (no wrap up from start)
        assert_eq!(app.settings_state.as_ref().unwrap().selected_index(), 0);
    }

    #[test]
    fn settings_view_enter_applies_theme() {
        let mut app = App::new();
        assert_eq!(app.theme.name, "vibes");

        // Open settings
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        app.handle_key(key);

        // Navigate to dark theme (index depends on loader)
        let themes = app.theme_loader.list();
        if let Some(dark_idx) = themes.iter().position(|t| *t == "dark") {
            for _ in 0..dark_idx {
                let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
                app.handle_key(key);
            }

            // Press Enter to apply
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            app.handle_key(key);

            // Theme should be applied
            assert_eq!(app.theme.name, "dark");
            // Should return to Dashboard
            assert_eq!(app.views.current, View::Dashboard);
            // Settings state should be cleared
            assert!(app.settings_state.is_none());
        }
    }
}
