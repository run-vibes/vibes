//! Setup wizard prompt helpers.
//!
//! These functions provide consistent output formatting for CLI setup wizards.
//! They are infrastructure for future wizard commands (tunnel setup, auth setup, etc.).

use dialoguer::console::style;
use std::io::{self, Write};

const HEADER_WIDTH: usize = 60;

/// Draws a boxed header with the given title.
pub fn print_header(title: &str) {
    print_header_to(&mut io::stdout(), title).unwrap();
}

/// Draws a boxed header to a writer (for testing).
pub fn print_header_to<W: Write>(w: &mut W, title: &str) -> io::Result<()> {
    let border = "─".repeat(HEADER_WIDTH);
    writeln!(w, "┌{}┐", border)?;
    writeln!(w, "│ {:<width$} │", title, width = HEADER_WIDTH - 2)?;
    writeln!(w, "└{}┘", border)?;
    writeln!(w)?;
    Ok(())
}

/// Prints a step message with a trailing space (no newline).
pub fn print_step(message: &str) {
    print_step_to(&mut io::stdout(), message).unwrap();
}

/// Prints a step message to a writer (for testing).
pub fn print_step_to<W: Write>(w: &mut W, message: &str) -> io::Result<()> {
    write!(w, "{} ", message)?;
    w.flush()
}

/// Prints a success message with a green checkmark.
pub fn print_success(message: &str) {
    print_success_to(&mut io::stdout(), message).unwrap();
}

/// Prints a success message to a writer (for testing).
pub fn print_success_to<W: Write>(w: &mut W, message: &str) -> io::Result<()> {
    writeln!(
        w,
        "\n{} {}",
        style("✓").green().bold(),
        style(message).green()
    )
}

/// Prints an error message with a red X.
#[allow(dead_code)]
pub fn print_error(message: &str) {
    print_error_to(&mut io::stdout(), message).unwrap();
}

/// Prints an error message to a writer (for testing).
#[allow(dead_code)]
pub fn print_error_to<W: Write>(w: &mut W, message: &str) -> io::Result<()> {
    writeln!(w, "\n{} {}", style("✗").red().bold(), style(message).red())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_header_draws_box_with_title() {
        let mut output = Vec::new();
        print_header_to(&mut output, "Test Title").unwrap();
        let result = String::from_utf8(output).unwrap();

        // Should have top border
        assert!(result.contains("┌"), "Missing top-left corner");
        assert!(result.contains("┐"), "Missing top-right corner");
        // Should have bottom border
        assert!(result.contains("└"), "Missing bottom-left corner");
        assert!(result.contains("┘"), "Missing bottom-right corner");
        // Should contain the title
        assert!(result.contains("Test Title"), "Missing title");
        // Should have horizontal lines
        assert!(result.contains("─"), "Missing horizontal border");
    }

    #[test]
    fn print_header_pads_title_to_width() {
        let mut output = Vec::new();
        print_header_to(&mut output, "Short").unwrap();
        let result = String::from_utf8(output).unwrap();

        // Header should be fixed width (60 chars inner + borders)
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 3, "Header should have at least 3 lines");

        // All lines should have consistent width
        let border_line = lines[0];
        assert!(
            border_line.chars().filter(|&c| c == '─').count() == HEADER_WIDTH,
            "Border should be {} chars wide",
            HEADER_WIDTH
        );
    }

    #[test]
    fn print_step_ends_with_space_no_newline() {
        let mut output = Vec::new();
        print_step_to(&mut output, "Loading").unwrap();
        let result = String::from_utf8(output).unwrap();

        // Should end with space, not newline
        assert!(result.ends_with(' '), "Should end with space");
        assert!(!result.ends_with('\n'), "Should not end with newline");
        assert!(result.contains("Loading"), "Should contain message");
    }

    #[test]
    fn print_success_shows_green_checkmark() {
        let mut output = Vec::new();
        print_success_to(&mut output, "Done!").unwrap();
        let result = String::from_utf8(output).unwrap();

        // Should contain checkmark and message
        assert!(result.contains('✓'), "Should contain checkmark");
        assert!(result.contains("Done!"), "Should contain message");
        // Should end with newline
        assert!(result.ends_with('\n'), "Should end with newline");
    }

    #[test]
    fn print_error_shows_red_x() {
        let mut output = Vec::new();
        print_error_to(&mut output, "Failed!").unwrap();
        let result = String::from_utf8(output).unwrap();

        // Should contain X and message
        assert!(result.contains('✗'), "Should contain X");
        assert!(result.contains("Failed!"), "Should contain message");
        // Should end with newline
        assert!(result.ends_with('\n'), "Should end with newline");
    }
}
