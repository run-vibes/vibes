//! Application state for the TUI.

use std::collections::HashMap;

use crate::widgets::{
    ConfirmationDialog, ControlBar, DiffModal, MergeDialog, MergeResultsView, OutputBuffer,
    PermissionWidget,
};

/// Unique identifier for a session.
pub type SessionId = String;

/// Unique identifier for an agent.
pub type AgentId = String;

/// Unique identifier for a swarm.
pub type SwarmId = String;

/// State for a single agent including output buffer, permission widget, and diff modal.
#[derive(Debug, Clone, Default)]
pub struct AgentState {
    /// Output buffer for the agent's output stream.
    pub output: OutputBuffer,
    /// Permission widget for handling permission requests.
    pub permission: PermissionWidget,
    /// Diff modal for viewing file changes.
    pub diff_modal: DiffModal,
    /// Control bar for agent actions.
    pub control_bar: ControlBar,
    /// Confirmation dialog for destructive actions.
    pub confirmation: ConfirmationDialog,
}

/// State for a swarm including merge dialog and results view.
#[derive(Debug, Clone, Default)]
pub struct SwarmState {
    /// Merge dialog for confirming result aggregation.
    pub merge_dialog: MergeDialog,
    /// Merged results view for displaying combined output.
    pub merge_results: MergeResultsView,
}

/// Focus state within the Settings view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsFocus {
    #[default]
    ThemeList,
    ApplyButton,
    CancelButton,
}

/// State for the Settings view with theme preview.
#[derive(Debug, Clone)]
pub struct SettingsState {
    original_theme: String,
    preview_theme: String,
    selected_index: usize,
    focus: SettingsFocus,
}

impl SettingsState {
    /// Create new settings state with the given current theme.
    pub fn new(current_theme: &str) -> Self {
        Self {
            original_theme: current_theme.to_string(),
            preview_theme: current_theme.to_string(),
            selected_index: 0,
            focus: SettingsFocus::default(),
        }
    }

    /// The theme name when the settings view was opened.
    pub fn original_theme(&self) -> &str {
        &self.original_theme
    }

    /// The currently previewed theme name.
    pub fn preview_theme(&self) -> &str {
        &self.preview_theme
    }

    /// The selected index in the theme list.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Set the selected index in the theme list.
    pub fn set_selected_index(&mut self, index: usize) {
        self.selected_index = index;
    }

    /// Set the preview theme name.
    pub fn set_preview_theme(&mut self, name: &str) {
        self.preview_theme = name.to_string();
    }

    /// Get the current focus.
    pub fn focus(&self) -> SettingsFocus {
        self.focus
    }

    /// Set the focus.
    pub fn set_focus(&mut self, focus: SettingsFocus) {
        self.focus = focus;
    }

    /// Returns true if the preview differs from the original.
    pub fn is_modified(&self) -> bool {
        self.preview_theme != self.original_theme
    }
}

/// The current UI mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Command,
    Search,
    Help,
}

/// Currently selected item in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    #[default]
    None,
    Session(usize),
    Agent(usize),
    Swarm(usize),
}

/// Core application state.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub session: Option<SessionId>,
    pub agents: HashMap<AgentId, AgentState>,
    pub swarms: HashMap<SwarmId, SwarmState>,
    pub selected: Selection,
    pub mode: Mode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_defaults_to_normal() {
        let mode = Mode::default();
        assert_eq!(mode, Mode::Normal);
    }

    #[test]
    fn selection_defaults_to_none() {
        let selection = Selection::default();
        assert_eq!(selection, Selection::None);
    }

    #[test]
    fn app_state_defaults_to_empty() {
        let state = AppState::default();
        assert!(state.session.is_none());
        assert!(state.agents.is_empty());
        assert!(state.swarms.is_empty());
        assert_eq!(state.selected, Selection::None);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn mode_variants_are_distinct() {
        assert_ne!(Mode::Normal, Mode::Command);
        assert_ne!(Mode::Command, Mode::Search);
        assert_ne!(Mode::Search, Mode::Help);
    }

    #[test]
    fn selection_variants_hold_indices() {
        let session = Selection::Session(0);
        let agent = Selection::Agent(5);
        let swarm = Selection::Swarm(10);

        assert_eq!(session, Selection::Session(0));
        assert_eq!(agent, Selection::Agent(5));
        assert_eq!(swarm, Selection::Swarm(10));
    }

    #[test]
    fn app_state_can_store_session() {
        let state = AppState {
            session: Some("test-session".to_string()),
            ..Default::default()
        };
        assert_eq!(state.session, Some("test-session".to_string()));
    }

    #[test]
    fn app_state_can_store_agents() {
        let mut state = AppState::default();
        state
            .agents
            .insert("agent-1".to_string(), AgentState::default());
        assert_eq!(state.agents.len(), 1);
        assert!(state.agents.contains_key("agent-1"));
    }

    #[test]
    fn agent_state_has_output_buffer() {
        let state = AgentState::default();
        assert!(state.output.is_empty());
    }

    #[test]
    fn agent_state_has_permission_widget() {
        let state = AgentState::default();
        assert!(!state.permission.has_pending());
    }

    #[test]
    fn agent_state_permission_can_store_request() {
        use crate::widgets::{PermissionDetails, PermissionRequest, PermissionType};
        use chrono::Utc;
        use std::path::PathBuf;

        let mut state = AgentState::default();
        state.permission.set_pending(PermissionRequest {
            id: "req-1".to_string(),
            request_type: PermissionType::FileWrite,
            description: "Write to file".to_string(),
            details: PermissionDetails::FileWrite {
                path: PathBuf::from("test.rs"),
                content: "test".to_string(),
                original: None,
            },
            timestamp: Utc::now(),
        });

        assert!(state.permission.has_pending());
    }

    #[test]
    fn agent_state_has_diff_modal() {
        let state = AgentState::default();
        assert!(!state.diff_modal.is_visible());
    }

    #[test]
    fn agent_state_diff_modal_can_be_shown() {
        let mut state = AgentState::default();
        state.diff_modal.show("test.rs", Some("old"), "new");
        assert!(state.diff_modal.is_visible());
    }

    #[test]
    fn agent_state_has_control_bar() {
        use crate::widgets::AgentStatus;
        let state = AgentState::default();
        assert_eq!(state.control_bar.status(), AgentStatus::Running);
    }

    #[test]
    fn agent_state_control_bar_can_be_updated() {
        use crate::widgets::AgentStatus;
        let mut state = AgentState::default();
        state.control_bar.set_status(AgentStatus::Paused);
        assert_eq!(state.control_bar.status(), AgentStatus::Paused);
    }

    #[test]
    fn agent_state_has_confirmation_dialog() {
        let state = AgentState::default();
        assert!(!state.confirmation.is_visible());
    }

    #[test]
    fn agent_state_confirmation_can_be_shown() {
        use crate::widgets::ConfirmationType;
        let mut state = AgentState::default();
        state.confirmation.show(ConfirmationType::Cancel);
        assert!(state.confirmation.is_visible());
    }

    // ==================== SwarmState Tests ====================

    #[test]
    fn swarm_state_defaults_with_hidden_dialogs() {
        let state = SwarmState::default();
        assert!(!state.merge_dialog.is_visible());
        assert!(!state.merge_results.is_visible());
    }

    #[test]
    fn swarm_state_has_merge_dialog() {
        use crate::widgets::CompletedAgent;
        let mut state = SwarmState::default();
        state.merge_dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Test".into(),
                task_summary: "Task".into(),
            }],
            0,
        );
        assert!(state.merge_dialog.is_visible());
    }

    #[test]
    fn swarm_state_has_merge_results() {
        use crate::widgets::ResultSection;
        let mut state = SwarmState::default();
        state.merge_results.show(vec![ResultSection {
            agent_name: "Agent".into(),
            content: "Results".into(),
        }]);
        assert!(state.merge_results.is_visible());
    }

    #[test]
    fn app_state_can_store_swarms() {
        let mut state = AppState::default();
        state
            .swarms
            .insert("swarm-1".to_string(), SwarmState::default());
        assert_eq!(state.swarms.len(), 1);
        assert!(state.swarms.contains_key("swarm-1"));
    }

    // ==================== SettingsState Tests ====================

    #[test]
    fn settings_focus_defaults_to_theme_list() {
        let focus = SettingsFocus::default();
        assert_eq!(focus, SettingsFocus::ThemeList);
    }

    #[test]
    fn settings_state_stores_original_theme() {
        let state = SettingsState::new("vibes");
        assert_eq!(state.original_theme(), "vibes");
    }

    #[test]
    fn settings_state_preview_starts_as_original() {
        let state = SettingsState::new("dark");
        assert_eq!(state.preview_theme(), "dark");
    }

    #[test]
    fn settings_state_selected_index_starts_at_zero() {
        let state = SettingsState::new("vibes");
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn settings_state_can_set_selected_index() {
        let mut state = SettingsState::new("vibes");
        state.set_selected_index(2);
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn settings_state_can_set_preview_theme() {
        let mut state = SettingsState::new("vibes");
        state.set_preview_theme("dark");
        assert_eq!(state.preview_theme(), "dark");
    }

    #[test]
    fn settings_state_focus_can_be_changed() {
        let mut state = SettingsState::new("vibes");
        state.set_focus(SettingsFocus::ApplyButton);
        assert_eq!(state.focus(), SettingsFocus::ApplyButton);
    }

    #[test]
    fn settings_state_is_modified_when_preview_differs() {
        let mut state = SettingsState::new("vibes");
        assert!(!state.is_modified());
        state.set_preview_theme("dark");
        assert!(state.is_modified());
    }
}
