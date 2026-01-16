//! Agent detail view - displays a single agent's status, output, and controls.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::traits::ViewRenderer;
use crate::App;
use crate::state::AgentId;
use crate::widgets::{ControlBar, OutputPanelWidget, PermissionWidget};

/// The agent detail view showing output, context, and controls for a single agent.
#[derive(Debug, Clone)]
pub struct AgentView {
    agent_id: AgentId,
}

impl AgentView {
    /// Creates a new AgentView for the given agent.
    pub fn new(agent_id: AgentId) -> Self {
        Self { agent_id }
    }

    /// Returns the agent ID (used in tests).
    #[cfg(test)]
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }

    /// Renders the header section with agent metadata.
    fn render_header(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Row 1: Session ID and Model
        let row1 = Line::from(vec![
            Span::styled("Session: ", Style::default().fg(app.theme.fg)),
            Span::styled("session-abc", app.theme.dim),
            Span::raw("   "),
            Span::styled("Model: ", Style::default().fg(app.theme.fg)),
            Span::styled("claude-sonnet-4-20250514", app.theme.dim),
        ]);

        // Row 2: Status and Task
        let row2 = Line::from(vec![
            Span::styled("Status: ", Style::default().fg(app.theme.fg)),
            Span::styled("Running", Style::default().fg(app.theme.running)),
            Span::raw("   "),
            Span::styled("Task: ", Style::default().fg(app.theme.fg)),
            Span::styled("implement login flow", app.theme.dim),
        ]);

        let header = Paragraph::new(vec![row1, row2]);
        frame.render_widget(header, area);
    }

    /// Renders the output panel with real-time agent output.
    fn render_output_panel(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Get the output buffer from the agent state, or use an empty widget
        let widget = if let Some(agent_state) = app.state.agents.get(&self.agent_id) {
            OutputPanelWidget {
                buffer: agent_state.output.clone(),
            }
        } else {
            OutputPanelWidget::new()
        };

        let paragraph = widget.to_paragraph(&app.theme, area.height as usize);
        frame.render_widget(paragraph, area);
    }

    /// Renders the context panel with stats (right side).
    fn render_context_panel(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .title(" Context ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        // Stats display
        let stats = vec![
            Line::from(vec![
                Span::styled("Files: ", Style::default().fg(app.theme.fg)),
                Span::styled("12", app.theme.dim),
            ]),
            Line::from(vec![
                Span::styled("Tokens: ", Style::default().fg(app.theme.fg)),
                Span::styled("45,231", app.theme.dim),
            ]),
            Line::from(vec![
                Span::styled("Tools: ", Style::default().fg(app.theme.fg)),
                Span::styled("8 calls", app.theme.dim),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(app.theme.fg)),
                Span::styled("4m 32s", app.theme.dim),
            ]),
        ];

        let content = Paragraph::new(stats).block(block);
        frame.render_widget(content, area);
    }

    /// Renders the permission request area.
    fn render_permission_area(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Get the permission widget from the agent state, or use an empty one
        let widget = if let Some(agent_state) = app.state.agents.get(&self.agent_id) {
            agent_state.permission.clone()
        } else {
            PermissionWidget::new()
        };

        let paragraph = widget.to_paragraph(&app.theme);
        frame.render_widget(paragraph, area);
    }

    /// Renders the control bar from agent state.
    fn render_control_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Get the control bar widget from the agent state, or use a default
        let widget = if let Some(agent_state) = app.state.agents.get(&self.agent_id) {
            agent_state.control_bar.clone()
        } else {
            ControlBar::default()
        };

        let paragraph = widget.to_paragraph(&app.theme);
        frame.render_widget(paragraph, area);
    }
}

impl ViewRenderer for AgentView {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Main container block with agent ID in title
        let title = format!(" Agent: {} ", self.agent_id);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into: header (2 lines), main content (flex), permission (4 lines), control bar (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(4),    // Main content area
                Constraint::Length(4), // Permission area (2 lines content + 2 lines borders)
                Constraint::Length(1), // Control bar
            ])
            .split(inner);

        // Render header
        self.render_header(frame, chunks[0], app);

        // Split main content into two columns: Output (60%) | Context (40%)
        let main_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // Render output panel (left)
        self.render_output_panel(frame, main_columns[0], app);

        // Render context panel (right)
        self.render_context_panel(frame, main_columns[1], app);

        // Render permission area
        self.render_permission_area(frame, chunks[2], app);

        // Render control bar
        self.render_control_bar(frame, chunks[3], app);

        // Render modal overlays if visible (render on top of everything)
        if let Some(agent_state) = app.state.agents.get(&self.agent_id) {
            // Diff modal has lower priority - render first
            agent_state.diff_modal.render(frame, &app.theme);
            // Confirmation dialog has higher priority - render on top
            agent_state.confirmation.render(frame, &app.theme);
        }
    }

    fn title(&self) -> &str {
        "Agent"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{PermissionDetails, PermissionRequest, PermissionType};
    use chrono::Utc;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    #[test]
    fn agent_view_stores_agent_id() {
        let view = AgentView::new("agent-123".into());
        assert_eq!(view.agent_id(), "agent-123");
    }

    #[test]
    fn agent_view_has_correct_title() {
        let view = AgentView::new("agent-1".into());
        assert_eq!(view.title(), "Agent");
    }

    #[test]
    fn agent_view_clone_preserves_id() {
        let view = AgentView::new("agent-clone-test".into());
        let cloned = view.clone();
        assert_eq!(view.agent_id(), cloned.agent_id());
    }

    #[test]
    fn agent_view_renders_permission_widget_no_pending() {
        let mut app = App::default();
        app.state
            .agents
            .insert("agent-1".to_string(), Default::default());

        let view = AgentView::new("agent-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should show "No pending permissions" when no permission request
        assert!(
            content.contains("No pending permissions"),
            "Expected 'No pending permissions' in rendered view"
        );
    }

    #[test]
    fn agent_view_renders_permission_widget_with_request() {
        let mut app = App::default();
        let mut agent_state = crate::state::AgentState::default();
        agent_state.permission.set_pending(PermissionRequest {
            id: "req-test".to_string(),
            request_type: PermissionType::FileWrite,
            description: "Write to src/test.rs".to_string(),
            details: PermissionDetails::FileWrite {
                path: PathBuf::from("src/test.rs"),
                content: "fn test() {}".to_string(),
                original: None,
            },
            timestamp: Utc::now(),
        });
        app.state.agents.insert("agent-1".to_string(), agent_state);

        let view = AgentView::new("agent-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should show the permission description
        assert!(
            content.contains("Write to src/test.rs"),
            "Expected permission description in rendered view"
        );

        // Should show control hints
        assert!(content.contains("[y]"), "Expected [y] control hint");
        assert!(content.contains("[n]"), "Expected [n] control hint");
    }

    #[test]
    fn agent_view_renders_diff_modal_when_visible() {
        let mut app = App::default();
        let mut agent_state = crate::state::AgentState::default();
        agent_state
            .diff_modal
            .show("src/main.rs", Some("fn old() {}"), "fn new() {}");
        app.state.agents.insert("agent-1".to_string(), agent_state);

        let view = AgentView::new("agent-1".into());
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should show the diff modal title
        assert!(
            content.contains("Diff: src/main.rs"),
            "Expected diff modal title in rendered view"
        );

        // Should show Original and Proposed labels
        assert!(content.contains("Original"), "Expected 'Original' label");
        assert!(content.contains("Proposed"), "Expected 'Proposed' label");
    }

    #[test]
    fn agent_view_does_not_render_diff_modal_when_hidden() {
        let mut app = App::default();
        let agent_state = crate::state::AgentState::default();
        // diff_modal is hidden by default
        app.state.agents.insert("agent-1".to_string(), agent_state);

        let view = AgentView::new("agent-1".into());
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should NOT show the diff modal
        assert!(
            !content.contains("Diff:"),
            "Expected no diff modal in rendered view when hidden"
        );
    }

    // ==================== Control Bar Widget Tests ====================

    #[test]
    fn agent_view_renders_control_bar_with_pause_when_running() {
        use crate::widgets::AgentStatus;

        let mut app = App::default();
        let mut agent_state = crate::state::AgentState::default();
        agent_state.control_bar.set_status(AgentStatus::Running);
        app.state.agents.insert("agent-1".to_string(), agent_state);

        let view = AgentView::new("agent-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should show Pause action when running
        assert!(
            content.contains("Pause"),
            "Expected 'Pause' in control bar when running"
        );
        // Should NOT show Resume
        assert!(
            !content.contains("Resume"),
            "Expected no 'Resume' in control bar when running"
        );
    }

    #[test]
    fn agent_view_renders_control_bar_with_resume_when_paused() {
        use crate::widgets::AgentStatus;

        let mut app = App::default();
        let mut agent_state = crate::state::AgentState::default();
        agent_state.control_bar.set_status(AgentStatus::Paused);
        app.state.agents.insert("agent-1".to_string(), agent_state);

        let view = AgentView::new("agent-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should show Resume action when paused
        assert!(
            content.contains("Resume"),
            "Expected 'Resume' in control bar when paused"
        );
        // Should NOT show Pause
        assert!(
            !content.contains("Pause"),
            "Expected no 'Pause' in control bar when paused"
        );
    }

    #[test]
    fn agent_view_renders_confirmation_dialog_when_visible() {
        use crate::widgets::ConfirmationType;

        let mut app = App::default();
        let mut agent_state = crate::state::AgentState::default();
        agent_state.confirmation.show(ConfirmationType::Cancel);
        app.state.agents.insert("agent-1".to_string(), agent_state);

        let view = AgentView::new("agent-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should show the confirmation dialog
        assert!(
            content.contains("Cancel Agent"),
            "Expected 'Cancel Agent' in confirmation dialog"
        );
    }
}
