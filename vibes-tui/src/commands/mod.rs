//! Command system for TUI command mode.
//!
//! Provides the infrastructure for `:command` style commands like `:theme`.

mod input;
mod registry;
mod settings;
mod theme;

pub use input::CommandInput;
pub use registry::CommandRegistry;
pub use settings::SettingsCommand;
pub use theme::ThemeCommand;

/// Result of executing a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// Success with optional message to display.
    Ok(Option<String>),
    /// Error with message.
    Err(String),
    /// Command wants to quit the app.
    Quit,
}

impl CommandResult {
    /// Create a success result with a message.
    pub fn ok(msg: impl Into<String>) -> Self {
        Self::Ok(Some(msg.into()))
    }

    /// Create a success result with no message.
    pub fn ok_empty() -> Self {
        Self::Ok(None)
    }

    /// Create an error result.
    pub fn err(msg: impl Into<String>) -> Self {
        Self::Err(msg.into())
    }

    /// Check if the result is successful.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Check if the result is an error.
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
}

/// A TUI command that can be executed via `:command` syntax.
pub trait Command: Send + Sync {
    /// Command name (e.g., "theme", "quit").
    fn name(&self) -> &str;

    /// Execute the command with arguments.
    fn execute(&mut self, args: &[&str], app: &mut crate::App) -> CommandResult;

    /// Return completions for the given partial input.
    /// Called when user presses Tab.
    fn completions(&self, _args: &[&str], _app: &crate::App) -> Vec<String> {
        vec![]
    }

    /// Short help text for `:help` listing.
    fn help(&self) -> &str {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === CommandResult tests ===

    #[test]
    fn command_result_ok_with_message() {
        let result = CommandResult::ok("Success!");
        assert_eq!(result, CommandResult::Ok(Some("Success!".into())));
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    #[test]
    fn command_result_ok_empty() {
        let result = CommandResult::ok_empty();
        assert_eq!(result, CommandResult::Ok(None));
        assert!(result.is_ok());
    }

    #[test]
    fn command_result_err() {
        let result = CommandResult::err("Something failed");
        assert_eq!(result, CommandResult::Err("Something failed".into()));
        assert!(result.is_err());
        assert!(!result.is_ok());
    }

    #[test]
    fn command_result_quit() {
        let result = CommandResult::Quit;
        assert!(!result.is_ok());
        assert!(!result.is_err());
    }

    // === Command trait tests ===

    struct TestCommand {
        name: String,
        help: String,
    }

    impl Command for TestCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn execute(&mut self, args: &[&str], _app: &mut crate::App) -> CommandResult {
            if args.is_empty() {
                CommandResult::ok("No args")
            } else {
                CommandResult::ok(format!("Args: {}", args.join(", ")))
            }
        }

        fn completions(&self, _args: &[&str], _app: &crate::App) -> Vec<String> {
            vec!["option1".into(), "option2".into()]
        }

        fn help(&self) -> &str {
            &self.help
        }
    }

    #[test]
    fn command_trait_name() {
        let cmd = TestCommand {
            name: "test".into(),
            help: "Test command".into(),
        };
        assert_eq!(cmd.name(), "test");
    }

    #[test]
    fn command_trait_help() {
        let cmd = TestCommand {
            name: "test".into(),
            help: "Test command".into(),
        };
        assert_eq!(cmd.help(), "Test command");
    }

    #[test]
    fn command_trait_completions() {
        let cmd = TestCommand {
            name: "test".into(),
            help: "".into(),
        };
        let app = crate::App::new();
        let completions = cmd.completions(&[], &app);
        assert_eq!(completions, vec!["option1", "option2"]);
    }

    #[test]
    fn command_trait_execute_no_args() {
        let mut cmd = TestCommand {
            name: "test".into(),
            help: "".into(),
        };
        let mut app = crate::App::new();
        let result = cmd.execute(&[], &mut app);
        assert_eq!(result, CommandResult::ok("No args"));
    }

    #[test]
    fn command_trait_execute_with_args() {
        let mut cmd = TestCommand {
            name: "test".into(),
            help: "".into(),
        };
        let mut app = crate::App::new();
        let result = cmd.execute(&["arg1", "arg2"], &mut app);
        assert_eq!(result, CommandResult::ok("Args: arg1, arg2"));
    }
}
