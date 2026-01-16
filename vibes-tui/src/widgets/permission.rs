//! Permission approval widget for the agent detail view.
//!
//! Displays pending permission requests and provides controls to approve,
//! deny, or view details of the requested operation.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::Theme;

/// Unique identifier for a permission request.
pub type PermissionId = String;

/// Type of permission being requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionType {
    /// Request to write to a file.
    FileWrite,
    /// Request to execute a command.
    Command,
    /// Request to read a file.
    FileRead,
    /// Request to make a web request.
    WebRequest,
}

/// Details specific to each permission type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDetails {
    /// File write details.
    FileWrite {
        path: PathBuf,
        content: String,
        original: Option<String>,
    },
    /// Command execution details.
    Command {
        command: String,
        working_dir: PathBuf,
    },
    /// File read details.
    FileRead { path: PathBuf },
    /// Web request details.
    WebRequest { url: String, method: String },
}

/// A permission request from an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRequest {
    /// Unique request identifier.
    pub id: PermissionId,
    /// Type of permission requested.
    pub request_type: PermissionType,
    /// Human-readable description of the request.
    pub description: String,
    /// Detailed information about the request.
    pub details: PermissionDetails,
    /// When the request was made.
    pub timestamp: DateTime<Utc>,
}

/// Widget for displaying and handling permission requests.
#[derive(Debug, Clone, Default)]
pub struct PermissionWidget {
    /// Currently pending permission request, if any.
    pending: Option<PermissionRequest>,
}

impl PermissionWidget {
    /// Creates a new permission widget with no pending request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pending permission request.
    pub fn set_pending(&mut self, request: PermissionRequest) {
        self.pending = Some(request);
    }

    /// Clears the pending permission request.
    pub fn clear_pending(&mut self) {
        self.pending = None;
    }

    /// Returns the pending permission request, if any.
    pub fn pending(&self) -> Option<&PermissionRequest> {
        self.pending.as_ref()
    }

    /// Returns true if there is a pending permission request.
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Converts the widget to a renderable Paragraph with the given theme.
    pub fn to_paragraph(&self, theme: &Theme) -> Paragraph<'_> {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let lines = match &self.pending {
            None => vec![Line::from(Span::styled(
                "No pending permissions",
                Style::default().fg(theme.fg),
            ))],
            Some(request) => {
                // Line 1: Permission request with warning icon and description
                let request_line = Line::from(vec![
                    Span::styled("\u{26A0} ", Style::default().fg(theme.warning)), // ⚠
                    Span::styled("Permission Request: ", Style::default().fg(theme.fg)),
                    Span::styled(&request.description, Style::default().fg(theme.fg)),
                ]);

                // Line 2: Controls
                let controls_line = Line::from(vec![
                    Span::styled("[y]", Style::default().fg(theme.accent)),
                    Span::styled(" Approve  ", Style::default().fg(theme.fg)),
                    Span::styled("[n]", Style::default().fg(theme.accent)),
                    Span::styled(" Deny  ", Style::default().fg(theme.fg)),
                    Span::styled("[v]", Style::default().fg(theme.accent)),
                    Span::styled(" View diff  ", Style::default().fg(theme.fg)),
                    Span::styled("[e]", Style::default().fg(theme.accent)),
                    Span::styled(" Edit", Style::default().fg(theme.fg)),
                ]);

                vec![request_line, controls_line]
            }
        };

        Paragraph::new(lines).block(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // ==================== PermissionType Tests ====================

    #[test]
    fn permission_type_variants_are_distinct() {
        assert_ne!(PermissionType::FileWrite, PermissionType::Command);
        assert_ne!(PermissionType::Command, PermissionType::FileRead);
        assert_ne!(PermissionType::FileRead, PermissionType::WebRequest);
    }

    // ==================== PermissionDetails Tests ====================

    #[test]
    fn permission_details_file_write_stores_path_and_content() {
        let details = PermissionDetails::FileWrite {
            path: PathBuf::from("/src/main.rs"),
            content: "fn main() {}".to_string(),
            original: Some("fn main() { println!(\"old\"); }".to_string()),
        };

        match details {
            PermissionDetails::FileWrite {
                path,
                content,
                original,
            } => {
                assert_eq!(path, PathBuf::from("/src/main.rs"));
                assert_eq!(content, "fn main() {}");
                assert_eq!(
                    original,
                    Some("fn main() { println!(\"old\"); }".to_string())
                );
            }
            _ => panic!("Expected FileWrite variant"),
        }
    }

    #[test]
    fn permission_details_command_stores_command_and_dir() {
        let details = PermissionDetails::Command {
            command: "cargo build".to_string(),
            working_dir: PathBuf::from("/project"),
        };

        match details {
            PermissionDetails::Command {
                command,
                working_dir,
            } => {
                assert_eq!(command, "cargo build");
                assert_eq!(working_dir, PathBuf::from("/project"));
            }
            _ => panic!("Expected Command variant"),
        }
    }

    #[test]
    fn permission_details_file_read_stores_path() {
        let details = PermissionDetails::FileRead {
            path: PathBuf::from("/etc/config.toml"),
        };

        match details {
            PermissionDetails::FileRead { path } => {
                assert_eq!(path, PathBuf::from("/etc/config.toml"));
            }
            _ => panic!("Expected FileRead variant"),
        }
    }

    #[test]
    fn permission_details_web_request_stores_url_and_method() {
        let details = PermissionDetails::WebRequest {
            url: "https://api.example.com/data".to_string(),
            method: "POST".to_string(),
        };

        match details {
            PermissionDetails::WebRequest { url, method } => {
                assert_eq!(url, "https://api.example.com/data");
                assert_eq!(method, "POST");
            }
            _ => panic!("Expected WebRequest variant"),
        }
    }

    // ==================== PermissionRequest Tests ====================

    #[test]
    fn permission_request_has_all_fields() {
        let request = PermissionRequest {
            id: "req-123".to_string(),
            request_type: PermissionType::FileWrite,
            description: "Write to src/main.rs".to_string(),
            details: PermissionDetails::FileWrite {
                path: PathBuf::from("/src/main.rs"),
                content: "fn main() {}".to_string(),
                original: None,
            },
            timestamp: Utc::now(),
        };

        assert_eq!(request.id, "req-123");
        assert_eq!(request.request_type, PermissionType::FileWrite);
        assert_eq!(request.description, "Write to src/main.rs");
    }

    // ==================== PermissionWidget Tests ====================

    #[test]
    fn permission_widget_new_has_no_pending() {
        let widget = PermissionWidget::new();
        assert!(!widget.has_pending());
        assert!(widget.pending().is_none());
    }

    #[test]
    fn permission_widget_set_pending_stores_request() {
        let mut widget = PermissionWidget::new();
        let request = PermissionRequest {
            id: "req-456".to_string(),
            request_type: PermissionType::Command,
            description: "Execute cargo test".to_string(),
            details: PermissionDetails::Command {
                command: "cargo test".to_string(),
                working_dir: PathBuf::from("/project"),
            },
            timestamp: Utc::now(),
        };

        widget.set_pending(request.clone());

        assert!(widget.has_pending());
        assert_eq!(widget.pending().unwrap().id, "req-456");
    }

    #[test]
    fn permission_widget_clear_pending_removes_request() {
        let mut widget = PermissionWidget::new();
        let request = PermissionRequest {
            id: "req-789".to_string(),
            request_type: PermissionType::FileRead,
            description: "Read config file".to_string(),
            details: PermissionDetails::FileRead {
                path: PathBuf::from("/config.toml"),
            },
            timestamp: Utc::now(),
        };

        widget.set_pending(request);
        assert!(widget.has_pending());

        widget.clear_pending();
        assert!(!widget.has_pending());
    }

    #[test]
    fn permission_widget_default_equals_new() {
        let widget1 = PermissionWidget::new();
        let widget2 = PermissionWidget::default();

        assert_eq!(widget1.has_pending(), widget2.has_pending());
    }

    // ==================== Rendering Tests ====================

    #[test]
    fn permission_widget_renders_no_pending_state() {
        let theme = crate::vibes_default();
        let widget = PermissionWidget::new();

        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("No pending permissions"),
            "Expected 'No pending permissions' in: {}",
            content
        );
    }

    #[test]
    fn permission_widget_renders_file_write_request() {
        let theme = crate::vibes_default();
        let mut widget = PermissionWidget::new();
        widget.set_pending(PermissionRequest {
            id: "req-1".to_string(),
            request_type: PermissionType::FileWrite,
            description: "Write to src/auth/login.rs".to_string(),
            details: PermissionDetails::FileWrite {
                path: PathBuf::from("src/auth/login.rs"),
                content: "fn login() {}".to_string(),
                original: None,
            },
            timestamp: Utc::now(),
        });

        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Write to src/auth/login.rs"),
            "Expected description in: {}",
            content
        );
    }

    #[test]
    fn permission_widget_renders_command_request() {
        let theme = crate::vibes_default();
        let mut widget = PermissionWidget::new();
        widget.set_pending(PermissionRequest {
            id: "req-2".to_string(),
            request_type: PermissionType::Command,
            description: "Execute: rm -rf /".to_string(),
            details: PermissionDetails::Command {
                command: "rm -rf /".to_string(),
                working_dir: PathBuf::from("/"),
            },
            timestamp: Utc::now(),
        });

        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Execute: rm -rf /"),
            "Expected command in: {}",
            content
        );
    }

    #[test]
    fn permission_widget_renders_controls_hint() {
        let theme = crate::vibes_default();
        let mut widget = PermissionWidget::new();
        widget.set_pending(PermissionRequest {
            id: "req-3".to_string(),
            request_type: PermissionType::FileRead,
            description: "Read config.toml".to_string(),
            details: PermissionDetails::FileRead {
                path: PathBuf::from("config.toml"),
            },
            timestamp: Utc::now(),
        });

        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        // Check for control hints
        assert!(content.contains("[y]"), "Expected [y] in: {}", content);
        assert!(content.contains("[n]"), "Expected [n] in: {}", content);
    }
}
