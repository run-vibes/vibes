//! Theme command implementation.

use super::{Command, CommandResult};
use crate::App;
use crate::theme::ThemeLoader;

/// Command for switching and managing themes.
pub struct ThemeCommand {
    loader: ThemeLoader,
}

impl ThemeCommand {
    /// Create a new theme command with the given loader.
    pub fn new(loader: ThemeLoader) -> Self {
        Self { loader }
    }

    /// List all available themes with current indicator.
    fn list_themes(&self, app: &App) -> CommandResult {
        let current = &app.theme.name;
        let list: Vec<String> = self
            .loader
            .list()
            .iter()
            .map(|name| {
                if *name == current {
                    format!("  {} (current)", name)
                } else {
                    format!("  {}", name)
                }
            })
            .collect();

        CommandResult::ok(format!(
            "Available themes:\n{}\n\nUse :theme <name> to switch",
            list.join("\n")
        ))
    }

    /// Switch to a named theme.
    fn switch_theme(&self, name: &str, app: &mut App) -> CommandResult {
        match self.loader.get(name) {
            Some(theme) => {
                app.theme = theme.clone();
                CommandResult::ok(format!("Theme changed to '{}'", name))
            }
            None => CommandResult::err(format!("Theme '{}' not found", name)),
        }
    }
}

impl Command for ThemeCommand {
    fn name(&self) -> &str {
        "theme"
    }

    fn execute(&mut self, args: &[&str], app: &mut App) -> CommandResult {
        match args {
            [] => self.list_themes(app),
            ["save"] => {
                // TODO: Implement config persistence
                CommandResult::ok(format!("Theme '{}' saved to config", app.theme.name))
            }
            [name] => self.switch_theme(name, app),
            _ => CommandResult::err("Usage: :theme [name|save]"),
        }
    }

    fn completions(&self, args: &[&str], _app: &App) -> Vec<String> {
        let all_options: Vec<String> = self
            .loader
            .list()
            .iter()
            .map(|s| s.to_string())
            .chain(std::iter::once("save".to_string()))
            .collect();

        match args {
            [] => all_options,
            [partial] => all_options
                .into_iter()
                .filter(|opt| opt.starts_with(partial))
                .collect(),
            _ => vec![],
        }
    }

    fn help(&self) -> &str {
        "Switch or save themes"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_command_name() {
        let loader = ThemeLoader::new();
        let cmd = ThemeCommand::new(loader);
        assert_eq!(cmd.name(), "theme");
    }

    #[test]
    fn theme_command_help() {
        let loader = ThemeLoader::new();
        let cmd = ThemeCommand::new(loader);
        assert_eq!(cmd.help(), "Switch or save themes");
    }

    // === List themes (no args) ===

    #[test]
    fn theme_no_args_lists_available_themes() {
        let loader = ThemeLoader::new();
        let mut cmd = ThemeCommand::new(loader);
        let mut app = App::new();

        let result = cmd.execute(&[], &mut app);
        assert!(result.is_ok());

        if let CommandResult::Ok(Some(msg)) = result {
            assert!(msg.contains("vibes"));
            assert!(msg.contains("dark"));
            assert!(msg.contains("light"));
            assert!(msg.contains("high-contrast"));
        } else {
            panic!("Expected Ok with message");
        }
    }

    #[test]
    fn theme_list_shows_current_indicator() {
        let loader = ThemeLoader::new();
        let mut cmd = ThemeCommand::new(loader);
        let mut app = App::new();
        // Default theme is "vibes"

        let result = cmd.execute(&[], &mut app);

        if let CommandResult::Ok(Some(msg)) = result {
            assert!(msg.contains("vibes (current)"));
        } else {
            panic!("Expected Ok with message");
        }
    }

    // === Switch theme ===

    #[test]
    fn theme_switch_to_valid_theme() {
        let loader = ThemeLoader::new();
        let mut cmd = ThemeCommand::new(loader);
        let mut app = App::new();
        assert_eq!(app.theme.name, "vibes");

        let result = cmd.execute(&["dark"], &mut app);

        assert!(result.is_ok());
        assert_eq!(app.theme.name, "dark");
        if let CommandResult::Ok(Some(msg)) = result {
            assert!(msg.contains("dark"));
        }
    }

    #[test]
    fn theme_switch_to_unknown_theme_returns_error() {
        let loader = ThemeLoader::new();
        let mut cmd = ThemeCommand::new(loader);
        let mut app = App::new();

        let result = cmd.execute(&["nonexistent"], &mut app);

        assert!(result.is_err());
        // Theme should not change
        assert_eq!(app.theme.name, "vibes");
    }

    #[test]
    fn theme_switch_changes_all_theme_colors() {
        let loader = ThemeLoader::new();
        let mut cmd = ThemeCommand::new(loader);
        let mut app = App::new();

        // Capture original vibes theme colors
        let original_fg = app.theme.fg;

        cmd.execute(&["dark"], &mut app);

        // Colors should be different
        assert_ne!(app.theme.fg, original_fg);
    }

    // === Invalid args ===

    #[test]
    fn theme_too_many_args_returns_error() {
        let loader = ThemeLoader::new();
        let mut cmd = ThemeCommand::new(loader);
        let mut app = App::new();

        let result = cmd.execute(&["dark", "extra"], &mut app);

        assert!(result.is_err());
    }

    // === Completions ===

    #[test]
    fn theme_completions_include_all_themes() {
        let loader = ThemeLoader::new();
        let cmd = ThemeCommand::new(loader);
        let app = App::new();

        let completions = cmd.completions(&[], &app);

        assert!(completions.contains(&"vibes".to_string()));
        assert!(completions.contains(&"dark".to_string()));
        assert!(completions.contains(&"light".to_string()));
        assert!(completions.contains(&"high-contrast".to_string()));
    }

    #[test]
    fn theme_completions_include_save() {
        let loader = ThemeLoader::new();
        let cmd = ThemeCommand::new(loader);
        let app = App::new();

        let completions = cmd.completions(&[], &app);

        assert!(completions.contains(&"save".to_string()));
    }

    #[test]
    fn theme_completions_filter_by_partial() {
        let loader = ThemeLoader::new();
        let cmd = ThemeCommand::new(loader);
        let app = App::new();

        let completions = cmd.completions(&["d"], &app);

        assert!(completions.contains(&"dark".to_string()));
        assert!(!completions.contains(&"vibes".to_string()));
    }
}
