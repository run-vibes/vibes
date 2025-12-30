//! Command registry for plugin CLI commands

use std::collections::HashMap;
use vibes_plugin_api::CommandSpec;

/// Registry of all plugin commands
pub struct CommandRegistry {
    /// Map from full command path to registration info
    commands: HashMap<Vec<String>, RegisteredPluginCommand>,
}

/// A command registered by a plugin
pub struct RegisteredPluginCommand {
    /// Name of the plugin that owns this command
    pub plugin_name: String,
    /// Command specification
    pub spec: CommandSpec,
}

impl CommandRegistry {
    /// Create a new empty command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register commands for a plugin
    ///
    /// Commands are stored with full path including plugin name prefix
    pub fn register(&mut self, plugin_name: &str, commands: Vec<CommandSpec>) {
        for spec in commands {
            let mut full_path = vec![plugin_name.to_string()];
            full_path.extend(spec.path.clone());

            self.commands.insert(
                full_path,
                RegisteredPluginCommand {
                    plugin_name: plugin_name.to_string(),
                    spec,
                },
            );
        }
    }

    /// Check if a command path would conflict with existing registrations
    ///
    /// Returns the name of the plugin that owns the conflicting command, if any
    pub fn check_conflict(&self, plugin_name: &str, path: &[String]) -> Option<&str> {
        let mut full_path = vec![plugin_name.to_string()];
        full_path.extend(path.iter().cloned());

        self.commands
            .get(&full_path)
            .map(|c| c.plugin_name.as_str())
    }

    /// Find a command by its full path
    pub fn find(&self, path: &[String]) -> Option<&RegisteredPluginCommand> {
        self.commands.get(path)
    }

    /// Find the longest matching command path
    ///
    /// Given a path like `["groove", "trust", "role", "admin"]`, this finds
    /// the longest registered command (e.g., `["groove", "trust", "role"]`)
    /// and returns the match length so the caller knows where arguments begin.
    ///
    /// Returns (command, match_length) if found, where match_length is the
    /// number of path elements that form the command.
    pub fn find_longest_match(&self, path: &[String]) -> Option<(&RegisteredPluginCommand, usize)> {
        // Try progressively shorter prefixes until we find a match
        for len in (1..=path.len()).rev() {
            let prefix = &path[..len];
            if let Some(cmd) = self.commands.get(prefix) {
                return Some((cmd, len));
            }
        }
        None
    }

    /// Get all registered commands
    pub fn all_commands(&self) -> impl Iterator<Item = (&[String], &RegisteredPluginCommand)> {
        self.commands.iter().map(|(k, v)| (k.as_slice(), v))
    }

    /// Unregister all commands for a plugin
    pub fn unregister(&mut self, plugin_name: &str) {
        self.commands.retain(|_, v| v.plugin_name != plugin_name);
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

    #[test]
    fn test_register_commands() {
        let mut registry = CommandRegistry::new();

        let commands = vec![CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show levels".into(),
            args: vec![],
        }];

        registry.register("groove", commands);

        let found = registry.find(&["groove".into(), "trust".into(), "levels".into()]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().plugin_name, "groove");
    }

    #[test]
    fn test_check_conflict() {
        let mut registry = CommandRegistry::new();

        let commands = vec![CommandSpec {
            path: vec!["foo".into()],
            description: "Foo".into(),
            args: vec![],
        }];

        registry.register("plugin-a", commands);

        // plugin-b trying to register under its own namespace won't conflict
        // because plugin-a registered ["plugin-a", "foo"], not ["plugin-b", "foo"]
        let conflict = registry.check_conflict("plugin-b", &["foo".into()]);
        assert!(conflict.is_none());

        // But checking plugin-a's own namespace will find the conflict
        let conflict = registry.check_conflict("plugin-a", &["foo".into()]);
        assert_eq!(conflict, Some("plugin-a"));
    }

    #[test]
    fn test_no_conflict_with_self() {
        let registry = CommandRegistry::new();

        // No conflict if plugin doesn't exist yet
        let conflict = registry.check_conflict("new-plugin", &["foo".into()]);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_unregister_removes_all_plugin_commands() {
        let mut registry = CommandRegistry::new();

        let commands = vec![
            CommandSpec {
                path: vec!["cmd1".into()],
                description: "Command 1".into(),
                args: vec![],
            },
            CommandSpec {
                path: vec!["cmd2".into()],
                description: "Command 2".into(),
                args: vec![],
            },
        ];

        registry.register("test-plugin", commands);

        // Verify commands are registered
        assert!(
            registry
                .find(&["test-plugin".into(), "cmd1".into()])
                .is_some()
        );
        assert!(
            registry
                .find(&["test-plugin".into(), "cmd2".into()])
                .is_some()
        );

        // Unregister
        registry.unregister("test-plugin");

        // Verify commands are gone
        assert!(
            registry
                .find(&["test-plugin".into(), "cmd1".into()])
                .is_none()
        );
        assert!(
            registry
                .find(&["test-plugin".into(), "cmd2".into()])
                .is_none()
        );
    }

    #[test]
    fn test_all_commands_iterator() {
        let mut registry = CommandRegistry::new();

        let commands = vec![
            CommandSpec {
                path: vec!["foo".into()],
                description: "Foo".into(),
                args: vec![],
            },
            CommandSpec {
                path: vec!["bar".into()],
                description: "Bar".into(),
                args: vec![],
            },
        ];

        registry.register("test", commands);

        let all: Vec<_> = registry.all_commands().collect();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_default_creates_empty_registry() {
        let registry = CommandRegistry::default();
        assert_eq!(registry.all_commands().count(), 0);
    }

    #[test]
    fn test_find_longest_match() {
        let mut registry = CommandRegistry::new();

        let commands = vec![
            CommandSpec {
                path: vec!["trust".into(), "levels".into()],
                description: "Show levels".into(),
                args: vec![],
            },
            CommandSpec {
                path: vec!["trust".into(), "role".into()],
                description: "Show role".into(),
                args: vec![],
            },
        ];

        registry.register("groove", commands);

        // Exact match
        let path: Vec<String> = vec!["groove".into(), "trust".into(), "levels".into()];
        let (cmd, len) = registry.find_longest_match(&path).unwrap();
        assert_eq!(cmd.plugin_name, "groove");
        assert_eq!(len, 3);

        // Path with extra args
        let path: Vec<String> = vec![
            "groove".into(),
            "trust".into(),
            "role".into(),
            "admin".into(),
        ];
        let (cmd, len) = registry.find_longest_match(&path).unwrap();
        assert_eq!(cmd.plugin_name, "groove");
        assert_eq!(len, 3); // "groove", "trust", "role" - "admin" is an arg
        assert_eq!(&path[len..], &["admin".to_string()]);

        // No match
        let path: Vec<String> = vec!["unknown".into(), "cmd".into()];
        assert!(registry.find_longest_match(&path).is_none());
    }
}
