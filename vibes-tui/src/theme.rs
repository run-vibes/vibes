//! CRT-inspired theme system for vibes TUI.

use ratatui::style::{Color, Modifier, Style};

/// Theme configuration for the TUI.
///
/// Contains all colors and styles needed to render the interface
/// with a consistent CRT phosphor aesthetic.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,

    // Base colors
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,

    // Status colors
    pub running: Color,
    pub paused: Color,
    pub completed: Color,
    pub failed: Color,

    // UI element colors
    pub border: Color,
    pub selection: Color,
    pub highlight: Color,

    // Text styles
    pub bold: Style,
    pub dim: Style,
    pub italic: Style,
}

/// Creates the default vibes theme with CRT phosphor green aesthetics.
///
/// This theme matches the design system colors:
/// - Phosphor green (#00ff88) for primary text and success states
/// - Cyan accent (#00c8ff) for highlights
/// - Dark background (#121212) for the CRT effect
pub fn vibes_default() -> Theme {
    let fg = Color::Rgb(0, 255, 136); // #00ff88 phosphor green

    Theme {
        name: "vibes".into(),

        // Base colors
        bg: Color::Rgb(18, 18, 18), // #121212
        fg,
        accent: Color::Rgb(0, 200, 255),  // #00c8ff cyan
        success: Color::Rgb(0, 255, 136), // #00ff88
        warning: Color::Rgb(255, 200, 0), // #ffc800
        error: Color::Rgb(255, 85, 85),   // #ff5555

        // Status colors
        running: Color::Rgb(0, 255, 136),     // #00ff88
        paused: Color::Rgb(255, 200, 0),      // #ffc800
        completed: Color::Rgb(100, 100, 100), // #646464
        failed: Color::Rgb(255, 85, 85),      // #ff5555

        // UI element colors
        border: Color::Rgb(60, 60, 60),     // #3c3c3c
        selection: Color::Rgb(40, 80, 40),  // #285028
        highlight: Color::Rgb(0, 150, 100), // #009664

        // Text styles
        bold: Style::default().fg(fg).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(fg).add_modifier(Modifier::DIM),
        italic: Style::default().fg(fg).add_modifier(Modifier::ITALIC),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vibes_default_has_correct_name() {
        let theme = vibes_default();
        assert_eq!(theme.name, "vibes");
    }

    #[test]
    fn vibes_default_has_phosphor_green_fg() {
        let theme = vibes_default();
        assert_eq!(theme.fg, Color::Rgb(0, 255, 136));
    }

    #[test]
    fn vibes_default_has_cyan_accent() {
        let theme = vibes_default();
        assert_eq!(theme.accent, Color::Rgb(0, 200, 255));
    }

    #[test]
    fn vibes_default_has_dark_background() {
        let theme = vibes_default();
        assert_eq!(theme.bg, Color::Rgb(18, 18, 18));
    }

    #[test]
    fn vibes_default_status_colors_match_semantic_meaning() {
        let theme = vibes_default();
        // Running uses success green
        assert_eq!(theme.running, theme.success);
        // Paused uses warning yellow
        assert_eq!(theme.paused, theme.warning);
        // Failed uses error red
        assert_eq!(theme.failed, theme.error);
    }

    #[test]
    fn theme_is_clone() {
        let theme = vibes_default();
        let cloned = theme.clone();
        assert_eq!(theme.name, cloned.name);
    }
}
