//! Settings command implementation.

use super::{Command, CommandResult};
use crate::views::View;
use crate::{App, SettingsState};

/// Command for opening the settings view.
pub struct SettingsCommand;

impl Command for SettingsCommand {
    fn name(&self) -> &str {
        "settings"
    }

    fn execute(&mut self, args: &[&str], app: &mut App) -> CommandResult {
        if !args.is_empty() {
            return CommandResult::err("Usage: :settings");
        }

        // Initialize settings state with current theme
        app.settings_state = Some(SettingsState::new(&app.theme.name));
        app.views.push(View::Settings);

        CommandResult::ok_empty()
    }

    fn help(&self) -> &str {
        "Open theme settings"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::View;

    #[test]
    fn settings_command_name() {
        let cmd = SettingsCommand;
        assert_eq!(cmd.name(), "settings");
    }

    #[test]
    fn settings_command_help() {
        let cmd = SettingsCommand;
        assert_eq!(cmd.help(), "Open theme settings");
    }

    #[test]
    fn settings_command_opens_settings_view() {
        let mut cmd = SettingsCommand;
        let mut app = App::new();
        assert_eq!(app.views.current, View::Dashboard);
        assert!(app.settings_state.is_none());

        let result = cmd.execute(&[], &mut app);

        assert!(result.is_ok());
        assert_eq!(app.views.current, View::Settings);
        assert!(app.settings_state.is_some());
    }

    #[test]
    fn settings_command_initializes_state_with_current_theme() {
        let mut cmd = SettingsCommand;
        let mut app = App::new();
        assert_eq!(app.theme.name, "vibes");

        cmd.execute(&[], &mut app);

        let settings = app.settings_state.as_ref().unwrap();
        assert_eq!(settings.original_theme(), "vibes");
        assert_eq!(settings.preview_theme(), "vibes");
    }

    #[test]
    fn settings_command_rejects_extra_args() {
        let mut cmd = SettingsCommand;
        let mut app = App::new();

        let result = cmd.execute(&["extra"], &mut app);

        assert!(result.is_err());
        // Settings view should not be opened
        assert_eq!(app.views.current, View::Dashboard);
    }
}
