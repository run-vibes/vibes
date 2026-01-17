//! Command registry for dispatching commands.

use super::{Command, CommandResult};
use crate::App;

/// Manages registered commands and dispatches input.
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
}

impl CommandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    /// Register a command.
    pub fn register(&mut self, cmd: Box<dyn Command>) {
        self.commands.push(cmd);
    }

    /// Find command by name.
    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands
            .iter()
            .find(|c| c.name() == name)
            .map(|c| c.as_ref())
    }

    /// Get mutable reference for execution.
    fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn Command>> {
        self.commands.iter_mut().find(|c| c.name() == name)
    }

    /// List all command names.
    pub fn list(&self) -> Vec<&str> {
        self.commands.iter().map(|c| c.name()).collect()
    }

    /// Parse and execute a command string like "theme dark".
    pub fn execute(&mut self, input: &str, app: &mut App) -> CommandResult {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let (name, args) = match parts.split_first() {
            Some((n, a)) => (*n, a),
            None => return CommandResult::err("Empty command"),
        };

        match self.get_mut(name) {
            Some(cmd) => cmd.execute(args, app),
            None => CommandResult::err(format!("Unknown command: {}", name)),
        }
    }

    /// Get tab completions for partial input.
    pub fn completions(&self, input: &str, app: &App) -> Vec<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            // No input or partial command name - complete command names
            [] => self.commands.iter().map(|c| c.name().to_string()).collect(),
            [partial] if !input.ends_with(' ') => {
                // Partial command name - filter by prefix
                self.commands
                    .iter()
                    .filter(|c| c.name().starts_with(partial))
                    .map(|c| c.name().to_string())
                    .collect()
            }
            _ => {
                // Command name complete, delegate to command for arg completions
                let cmd_name = parts[0];
                let args = &parts[1..];
                self.get(cmd_name)
                    .map(|cmd| cmd.completions(args, app))
                    .unwrap_or_default()
            }
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::CommandResult;

    // Test command for registry tests
    struct EchoCommand;

    impl Command for EchoCommand {
        fn name(&self) -> &str {
            "echo"
        }

        fn execute(&mut self, args: &[&str], _app: &mut App) -> CommandResult {
            if args.is_empty() {
                CommandResult::ok("(empty)")
            } else {
                CommandResult::ok(args.join(" "))
            }
        }

        fn completions(&self, _args: &[&str], _app: &App) -> Vec<String> {
            vec!["hello".into(), "world".into()]
        }

        fn help(&self) -> &str {
            "Echo arguments"
        }
    }

    struct QuitCommand;

    impl Command for QuitCommand {
        fn name(&self) -> &str {
            "quit"
        }

        fn execute(&mut self, _args: &[&str], _app: &mut App) -> CommandResult {
            CommandResult::Quit
        }

        fn help(&self) -> &str {
            "Quit the application"
        }
    }

    #[test]
    fn registry_new_is_empty() {
        let registry = CommandRegistry::new();
        assert!(registry.commands.is_empty());
    }

    #[test]
    fn registry_register_adds_command() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        assert_eq!(registry.commands.len(), 1);
    }

    #[test]
    fn registry_get_finds_command_by_name() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        registry.register(Box::new(QuitCommand));

        let cmd = registry.get("echo");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name(), "echo");

        let cmd = registry.get("quit");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name(), "quit");
    }

    #[test]
    fn registry_get_returns_none_for_unknown() {
        let registry = CommandRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn registry_execute_dispatches_to_command() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        let mut app = App::new();

        let result = registry.execute("echo hello world", &mut app);
        assert_eq!(result, CommandResult::ok("hello world"));
    }

    #[test]
    fn registry_execute_handles_no_args() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        let mut app = App::new();

        let result = registry.execute("echo", &mut app);
        assert_eq!(result, CommandResult::ok("(empty)"));
    }

    #[test]
    fn registry_execute_returns_error_for_unknown_command() {
        let mut registry = CommandRegistry::new();
        let mut app = App::new();

        let result = registry.execute("unknown", &mut app);
        assert!(result.is_err());
    }

    #[test]
    fn registry_execute_returns_error_for_empty_input() {
        let mut registry = CommandRegistry::new();
        let mut app = App::new();

        let result = registry.execute("", &mut app);
        assert!(result.is_err());

        let result = registry.execute("   ", &mut app);
        assert!(result.is_err());
    }

    #[test]
    fn registry_list_returns_all_command_names() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        registry.register(Box::new(QuitCommand));

        let names = registry.list();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"echo"));
        assert!(names.contains(&"quit"));
    }

    #[test]
    fn registry_completions_returns_command_names_when_no_input() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        registry.register(Box::new(QuitCommand));
        let app = App::new();

        let completions = registry.completions("", &app);
        assert!(completions.contains(&"echo".to_string()));
        assert!(completions.contains(&"quit".to_string()));
    }

    #[test]
    fn registry_completions_filters_command_names_by_prefix() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        registry.register(Box::new(QuitCommand));
        let app = App::new();

        let completions = registry.completions("e", &app);
        assert!(completions.contains(&"echo".to_string()));
        assert!(!completions.contains(&"quit".to_string()));
    }

    #[test]
    fn registry_completions_delegates_to_command_for_args() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(EchoCommand));
        let app = App::new();

        let completions = registry.completions("echo ", &app);
        assert!(completions.contains(&"hello".to_string()));
        assert!(completions.contains(&"world".to_string()));
    }
}
