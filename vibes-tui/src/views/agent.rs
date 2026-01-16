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
use crate::widgets::OutputPanelWidget;

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

    /// Renders the permission request area placeholder.
    fn render_permission_area(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        let content = Paragraph::new(Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(app.theme.warning)),
            Span::styled("Permission area", app.theme.dim),
            Span::styled(" (coming in m43-feat-03)", app.theme.dim),
        ]))
        .block(block);

        frame.render_widget(content, area);
    }

    /// Renders the control bar placeholder.
    fn render_control_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let controls = Line::from(vec![
            Span::styled("[p]", Style::default().fg(app.theme.accent)),
            Span::styled(" Pause  ", app.theme.dim),
            Span::styled("[c]", Style::default().fg(app.theme.accent)),
            Span::styled(" Cancel  ", app.theme.dim),
            Span::styled("[r]", Style::default().fg(app.theme.accent)),
            Span::styled(" Restart  ", app.theme.dim),
            Span::styled("[Esc]", Style::default().fg(app.theme.accent)),
            Span::styled(" Back", app.theme.dim),
        ]);

        let bar = Paragraph::new(controls);
        frame.render_widget(bar, area);
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

        // Split into: header (2 lines), main content (flex), permission (3 lines), control bar (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(4),    // Main content area
                Constraint::Length(3), // Permission area
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
    }

    fn title(&self) -> &str {
        "Agent"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
