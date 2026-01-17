//! CRT-inspired theme system for vibes TUI.

use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::path::PathBuf;

/// Errors that can occur when loading themes.
#[derive(Debug, thiserror::Error)]
pub enum ThemeLoadError {
    #[error("config file not found: {0}")]
    NotFound(PathBuf),

    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("invalid color format '{0}': expected #RRGGBB")]
    InvalidColor(String),

    #[error("theme '{0}' not found")]
    ThemeNotFound(String),
}

/// Root config structure matching config.toml.
#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    pub theme: ThemeSection,
}

/// The [theme] section of the config.
#[derive(Debug, Deserialize)]
pub struct ThemeSection {
    /// Name of the active theme.
    pub active: String,
    /// Custom theme definitions.
    #[serde(default)]
    pub custom: Vec<ThemeConfigRaw>,
}

/// Raw theme definition as stored in TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfigRaw {
    pub name: String,
    pub bg: String,
    pub fg: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub running: String,
    pub paused: String,
    pub completed: String,
    pub failed: String,
    pub border: String,
    pub selection: String,
    pub highlight: String,
}

impl ThemeConfigRaw {
    /// Convert raw config to a Theme by parsing all hex colors.
    pub fn to_theme(&self) -> Result<Theme, ThemeLoadError> {
        let fg = parse_hex_color(&self.fg)?;
        Ok(Theme {
            name: self.name.clone(),
            bg: parse_hex_color(&self.bg)?,
            fg,
            accent: parse_hex_color(&self.accent)?,
            success: parse_hex_color(&self.success)?,
            warning: parse_hex_color(&self.warning)?,
            error: parse_hex_color(&self.error)?,
            running: parse_hex_color(&self.running)?,
            paused: parse_hex_color(&self.paused)?,
            completed: parse_hex_color(&self.completed)?,
            failed: parse_hex_color(&self.failed)?,
            border: parse_hex_color(&self.border)?,
            selection: parse_hex_color(&self.selection)?,
            highlight: parse_hex_color(&self.highlight)?,
            bold: Style::default().fg(fg).add_modifier(Modifier::BOLD),
            dim: Style::default().fg(fg).add_modifier(Modifier::DIM),
            italic: Style::default().fg(fg).add_modifier(Modifier::ITALIC),
        })
    }
}

/// Loads and manages themes from config files.
pub struct ThemeLoader {
    builtin: Vec<Theme>,
    custom: Vec<Theme>,
    active: String,
}

impl ThemeLoader {
    /// Create a new loader with only builtin themes.
    pub fn new() -> Self {
        Self {
            builtin: vec![vibes_default()],
            custom: Vec::new(),
            active: "vibes".into(),
        }
    }

    /// Load themes from a TOML config string.
    pub fn from_toml_str(toml: &str) -> Result<Self, ThemeLoadError> {
        let config: ThemeConfig = toml::from_str(toml)?;

        let custom: Result<Vec<Theme>, ThemeLoadError> = config
            .theme
            .custom
            .iter()
            .map(|raw| raw.to_theme())
            .collect();

        Ok(Self {
            builtin: vec![vibes_default()],
            custom: custom?,
            active: config.theme.active,
        })
    }

    /// Load themes from a config file path.
    pub fn from_config(path: &std::path::Path) -> Result<Self, ThemeLoadError> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| ThemeLoadError::NotFound(path.to_path_buf()))?;
        Self::from_toml_str(&content)
    }

    /// Load themes from the default vibes config location.
    ///
    /// Looks for `~/.config/vibes/config.toml`. If the file doesn't exist
    /// or has no theme section, returns a default loader with builtin themes.
    pub fn from_default_config() -> Self {
        let config_path = vibes_paths::config_dir().join("config.toml");
        match Self::from_config(&config_path) {
            Ok(loader) => loader,
            Err(ThemeLoadError::NotFound(_)) => {
                // Config file doesn't exist - use defaults
                Self::new()
            }
            Err(e) => {
                // Log the error but fall back to defaults
                tracing::warn!("Failed to load theme config: {}", e);
                Self::new()
            }
        }
    }

    /// Get a theme by name (searches custom first, then builtin).
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.custom
            .iter()
            .find(|t| t.name == name)
            .or_else(|| self.builtin.iter().find(|t| t.name == name))
    }

    /// List all available theme names.
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.builtin.iter().map(|t| t.name.as_str()).collect();
        for theme in &self.custom {
            if !names.contains(&theme.name.as_str()) {
                names.push(&theme.name);
            }
        }
        names
    }

    /// Get the active theme name from config.
    pub fn active_name(&self) -> &str {
        &self.active
    }

    /// Get the currently active theme.
    pub fn active(&self) -> Option<&Theme> {
        self.get(&self.active)
    }
}

impl Default for ThemeLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a hex color string (#RRGGBB) to ratatui Color.
pub fn parse_hex_color(hex: &str) -> Result<Color, ThemeLoadError> {
    if !hex.starts_with('#') || hex.len() != 7 {
        return Err(ThemeLoadError::InvalidColor(hex.to_string()));
    }

    let r = u8::from_str_radix(&hex[1..3], 16)
        .map_err(|_| ThemeLoadError::InvalidColor(hex.to_string()))?;
    let g = u8::from_str_radix(&hex[3..5], 16)
        .map_err(|_| ThemeLoadError::InvalidColor(hex.to_string()))?;
    let b = u8::from_str_radix(&hex[5..7], 16)
        .map_err(|_| ThemeLoadError::InvalidColor(hex.to_string()))?;

    Ok(Color::Rgb(r, g, b))
}

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

    // === ThemeLoader tests (TDD: RED first) ===

    #[test]
    fn theme_loader_new_has_builtin_vibes() {
        let loader = ThemeLoader::new();
        assert!(loader.get("vibes").is_some());
        assert_eq!(loader.get("vibes").unwrap().name, "vibes");
    }

    #[test]
    fn theme_loader_list_includes_builtins() {
        let loader = ThemeLoader::new();
        let names = loader.list();
        assert!(names.contains(&"vibes"));
    }

    #[test]
    fn theme_loader_from_config_string() {
        let toml = r##"
[theme]
active = "custom"

[[theme.custom]]
name = "custom"
bg = "#000000"
fg = "#ffffff"
accent = "#00ffff"
success = "#00ff00"
warning = "#ffff00"
error = "#ff0000"
running = "#00ff00"
paused = "#ffff00"
completed = "#888888"
failed = "#ff0000"
border = "#444444"
selection = "#333333"
highlight = "#666666"
"##;

        let loader = ThemeLoader::from_toml_str(toml).unwrap();
        assert_eq!(loader.active_name(), "custom");
        assert!(loader.get("custom").is_some());
        assert_eq!(loader.get("custom").unwrap().fg, Color::Rgb(255, 255, 255));
    }

    #[test]
    fn theme_loader_custom_overrides_builtin() {
        let toml = r##"
[theme]
active = "vibes"

[[theme.custom]]
name = "vibes"
bg = "#000000"
fg = "#ff0000"
accent = "#00ffff"
success = "#00ff00"
warning = "#ffff00"
error = "#ff0000"
running = "#00ff00"
paused = "#ffff00"
completed = "#888888"
failed = "#ff0000"
border = "#444444"
selection = "#333333"
highlight = "#666666"
"##;

        let loader = ThemeLoader::from_toml_str(toml).unwrap();
        // Custom theme overrides builtin - fg should be red, not green
        assert_eq!(loader.get("vibes").unwrap().fg, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn theme_loader_active_defaults_to_vibes() {
        let loader = ThemeLoader::new();
        assert_eq!(loader.active_name(), "vibes");
    }

    // === Config parsing tests (TDD: RED first) ===

    #[test]
    fn parse_theme_config_from_toml() {
        let toml = r##"
[theme]
active = "my-theme"

[[theme.custom]]
name = "my-theme"
bg = "#1a1a2e"
fg = "#eaeaea"
accent = "#e94560"
success = "#00ff88"
warning = "#ffc800"
error = "#ff5555"
running = "#00ff88"
paused = "#ffc800"
completed = "#646464"
failed = "#ff5555"
border = "#3c3c3c"
selection = "#285028"
highlight = "#009664"
"##;

        let config: ThemeConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.theme.active, "my-theme");
        assert_eq!(config.theme.custom.len(), 1);
        assert_eq!(config.theme.custom[0].name, "my-theme");
        assert_eq!(config.theme.custom[0].fg, "#eaeaea");
    }

    #[test]
    fn theme_config_to_theme() {
        let raw = ThemeConfigRaw {
            name: "test".into(),
            bg: "#121212".into(),
            fg: "#00ff88".into(),
            accent: "#00c8ff".into(),
            success: "#00ff88".into(),
            warning: "#ffc800".into(),
            error: "#ff5555".into(),
            running: "#00ff88".into(),
            paused: "#ffc800".into(),
            completed: "#646464".into(),
            failed: "#ff5555".into(),
            border: "#3c3c3c".into(),
            selection: "#285028".into(),
            highlight: "#009664".into(),
        };

        let theme = raw.to_theme().unwrap();
        assert_eq!(theme.name, "test");
        assert_eq!(theme.fg, Color::Rgb(0, 255, 136));
        assert_eq!(theme.bg, Color::Rgb(18, 18, 18));
    }

    // === Hex color parsing tests (TDD: RED first) ===

    #[test]
    fn parse_hex_color_valid_rgb() {
        let color = parse_hex_color("#ff5500").unwrap();
        assert_eq!(color, Color::Rgb(255, 85, 0));
    }

    #[test]
    fn parse_hex_color_lowercase() {
        let color = parse_hex_color("#00ff88").unwrap();
        assert_eq!(color, Color::Rgb(0, 255, 136));
    }

    #[test]
    fn parse_hex_color_uppercase() {
        let color = parse_hex_color("#00FF88").unwrap();
        assert_eq!(color, Color::Rgb(0, 255, 136));
    }

    #[test]
    fn parse_hex_color_missing_hash() {
        let result = parse_hex_color("ff5500");
        assert!(result.is_err());
    }

    #[test]
    fn parse_hex_color_wrong_length() {
        let result = parse_hex_color("#fff");
        assert!(result.is_err());
    }

    #[test]
    fn parse_hex_color_invalid_chars() {
        let result = parse_hex_color("#gggggg");
        assert!(result.is_err());
    }

    // === Existing tests ===

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
