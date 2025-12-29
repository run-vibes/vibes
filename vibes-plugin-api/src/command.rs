//! CLI command types for plugin registration

/// Specification for a CLI command
#[derive(Debug, Clone)]
pub struct CommandSpec {
    /// Command path, e.g., ["trust", "levels"] -> `vibes <plugin> trust levels`
    pub path: Vec<String>,
    /// Short description for help text
    pub description: String,
    /// Argument specifications
    pub args: Vec<ArgSpec>,
}

/// Specification for a command argument
#[derive(Debug, Clone)]
pub struct ArgSpec {
    /// Argument name
    pub name: String,
    /// Description for help text
    pub description: String,
    /// Whether this argument is required
    pub required: bool,
}

/// Output from a CLI command handler
#[derive(Debug)]
pub enum CommandOutput {
    /// Plain text output (printed as-is)
    Text(String),
    /// Structured data (can be formatted as table, JSON, etc.)
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// Success with no output
    Success,
    /// Exit with specific code
    Exit(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_spec_creation() {
        let spec = CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust levels".into(),
            args: vec![],
        };
        assert_eq!(spec.path, vec!["trust", "levels"]);
        assert!(spec.args.is_empty());
    }

    #[test]
    fn test_arg_spec_required() {
        let arg = ArgSpec {
            name: "role".into(),
            description: "Role name".into(),
            required: true,
        };
        assert!(arg.required);
    }

    #[test]
    fn test_command_output_text() {
        let output = CommandOutput::Text("Hello".into());
        match output {
            CommandOutput::Text(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_command_output_table() {
        let output = CommandOutput::Table {
            headers: vec!["Name".into(), "Value".into()],
            rows: vec![vec!["foo".into(), "bar".into()]],
        };
        match output {
            CommandOutput::Table { headers, rows } => {
                assert_eq!(headers.len(), 2);
                assert_eq!(rows.len(), 1);
            }
            _ => panic!("Expected Table variant"),
        }
    }
}
