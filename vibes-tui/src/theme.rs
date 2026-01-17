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
#[derive(Clone)]
pub struct ThemeLoader {
    builtin: Vec<Theme>,
    custom: Vec<Theme>,
    active: String,
}

impl ThemeLoader {
    /// Create a new loader with only builtin themes.
    pub fn new() -> Self {
        Self {
            builtin: builtin_themes(),
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
            builtin: builtin_themes(),
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

/// Creates the dark theme with neutral gray tones for reduced eye strain.
pub fn dark() -> Theme {
    let fg = Color::Rgb(212, 212, 212); // #d4d4d4

    Theme {
        name: "dark".into(),

        // Base colors
        bg: Color::Rgb(30, 30, 30), // #1e1e1e
        fg,
        accent: Color::Rgb(86, 156, 214),   // #569cd6 blue
        success: Color::Rgb(78, 201, 176),  // #4ec9b0 teal
        warning: Color::Rgb(220, 220, 170), // #dcdcaa yellow
        error: Color::Rgb(244, 71, 71),     // #f44747 red

        // Status colors
        running: Color::Rgb(78, 201, 176),    // teal
        paused: Color::Rgb(220, 220, 170),    // yellow
        completed: Color::Rgb(128, 128, 128), // gray
        failed: Color::Rgb(244, 71, 71),      // red

        // UI element colors
        border: Color::Rgb(68, 68, 68),     // #444444
        selection: Color::Rgb(38, 79, 120), // #264f78 blue
        highlight: Color::Rgb(51, 51, 51),  // #333333

        // Text styles
        bold: Style::default().fg(fg).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(fg).add_modifier(Modifier::DIM),
        italic: Style::default().fg(fg).add_modifier(Modifier::ITALIC),
    }
}

/// Creates the light theme for bright environments.
pub fn light() -> Theme {
    let fg = Color::Rgb(51, 51, 51); // #333333

    Theme {
        name: "light".into(),

        // Base colors
        bg: Color::Rgb(255, 255, 255), // #ffffff white
        fg,
        accent: Color::Rgb(0, 122, 204),  // #007acc blue
        success: Color::Rgb(22, 163, 74), // #16a34a green
        warning: Color::Rgb(202, 138, 4), // #ca8a04 amber
        error: Color::Rgb(220, 38, 38),   // #dc2626 red

        // Status colors
        running: Color::Rgb(22, 163, 74),     // green
        paused: Color::Rgb(202, 138, 4),      // amber
        completed: Color::Rgb(156, 163, 175), // gray
        failed: Color::Rgb(220, 38, 38),      // red

        // UI element colors
        border: Color::Rgb(209, 213, 219), // #d1d5db visible on white
        selection: Color::Rgb(191, 219, 254), // #bfdbfe light blue
        highlight: Color::Rgb(243, 244, 246), // #f3f4f6 light gray

        // Text styles
        bold: Style::default().fg(fg).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(fg).add_modifier(Modifier::DIM),
        italic: Style::default().fg(fg).add_modifier(Modifier::ITALIC),
    }
}

/// Creates the high-contrast theme for maximum accessibility.
pub fn high_contrast() -> Theme {
    let fg = Color::White;

    Theme {
        name: "high-contrast".into(),

        // Base colors - pure black and white
        bg: Color::Black,
        fg,
        accent: Color::Rgb(0, 255, 255),  // #00ffff pure cyan
        success: Color::Rgb(0, 255, 0),   // #00ff00 pure green
        warning: Color::Rgb(255, 255, 0), // #ffff00 pure yellow
        error: Color::Rgb(255, 0, 0),     // #ff0000 pure red

        // Status colors - saturated for visibility
        running: Color::Rgb(0, 255, 0),       // pure green
        paused: Color::Rgb(255, 255, 0),      // pure yellow
        completed: Color::Rgb(128, 128, 128), // mid gray
        failed: Color::Rgb(255, 0, 0),        // pure red

        // UI element colors
        border: Color::White,
        selection: Color::Rgb(0, 0, 128),  // navy blue
        highlight: Color::Rgb(64, 64, 64), // dark gray

        // Text styles
        bold: Style::default().fg(fg).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(fg).add_modifier(Modifier::DIM),
        italic: Style::default().fg(fg).add_modifier(Modifier::ITALIC),
    }
}

/// Returns all builtin themes.
pub fn builtin_themes() -> Vec<Theme> {
    vec![vibes_default(), dark(), light(), high_contrast()]
}

/// Gets a builtin theme by name.
///
/// Used by the `:theme` command to select themes at runtime.
#[allow(dead_code)] // Used by m45-feat-03 runtime theme switching
pub fn builtin_theme(name: &str) -> Option<Theme> {
    builtin_themes().into_iter().find(|t| t.name == name)
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

    // === Builtin theme registry tests (TDD: RED first) ===

    #[test]
    fn builtin_themes_returns_all_four_themes() {
        let themes = builtin_themes();
        assert_eq!(themes.len(), 4);
    }

    #[test]
    fn builtin_themes_includes_vibes() {
        let themes = builtin_themes();
        assert!(themes.iter().any(|t| t.name == "vibes"));
    }

    #[test]
    fn builtin_themes_includes_dark() {
        let themes = builtin_themes();
        assert!(themes.iter().any(|t| t.name == "dark"));
    }

    #[test]
    fn builtin_themes_includes_light() {
        let themes = builtin_themes();
        assert!(themes.iter().any(|t| t.name == "light"));
    }

    #[test]
    fn builtin_themes_includes_high_contrast() {
        let themes = builtin_themes();
        assert!(themes.iter().any(|t| t.name == "high-contrast"));
    }

    #[test]
    fn builtin_theme_finds_vibes_by_name() {
        let theme = builtin_theme("vibes");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().name, "vibes");
    }

    #[test]
    fn builtin_theme_finds_dark_by_name() {
        let theme = builtin_theme("dark");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().name, "dark");
    }

    #[test]
    fn builtin_theme_returns_none_for_unknown() {
        let theme = builtin_theme("nonexistent");
        assert!(theme.is_none());
    }

    // === Dark theme tests (TDD: RED first) ===

    #[test]
    fn dark_theme_has_correct_name() {
        let theme = dark();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn dark_theme_has_dark_background() {
        let theme = dark();
        // #1e1e1e = RGB(30, 30, 30)
        assert_eq!(theme.bg, Color::Rgb(30, 30, 30));
    }

    #[test]
    fn dark_theme_has_light_foreground() {
        let theme = dark();
        // #d4d4d4 = RGB(212, 212, 212)
        assert_eq!(theme.fg, Color::Rgb(212, 212, 212));
    }

    #[test]
    fn dark_theme_has_blue_accent() {
        let theme = dark();
        // #569cd6 = RGB(86, 156, 214)
        assert_eq!(theme.accent, Color::Rgb(86, 156, 214));
    }

    // === Light theme tests (TDD: RED first) ===

    #[test]
    fn light_theme_has_correct_name() {
        let theme = light();
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn light_theme_has_white_background() {
        let theme = light();
        // #ffffff = RGB(255, 255, 255)
        assert_eq!(theme.bg, Color::Rgb(255, 255, 255));
    }

    #[test]
    fn light_theme_has_dark_foreground() {
        let theme = light();
        // #333333 = RGB(51, 51, 51)
        assert_eq!(theme.fg, Color::Rgb(51, 51, 51));
    }

    #[test]
    fn light_theme_has_visible_border() {
        let theme = light();
        // Border should contrast with white background
        // Should be darker than #dddddd for visibility
        if let Color::Rgb(r, g, b) = theme.border {
            assert!(
                r < 220 && g < 220 && b < 220,
                "border should be visible on white"
            );
        } else {
            panic!("border should be RGB");
        }
    }

    // === High-contrast theme tests (TDD: RED first) ===

    #[test]
    fn high_contrast_theme_has_correct_name() {
        let theme = high_contrast();
        assert_eq!(theme.name, "high-contrast");
    }

    #[test]
    fn high_contrast_theme_has_black_background() {
        let theme = high_contrast();
        assert_eq!(theme.bg, Color::Black);
    }

    #[test]
    fn high_contrast_theme_has_white_foreground() {
        let theme = high_contrast();
        assert_eq!(theme.fg, Color::White);
    }

    #[test]
    fn high_contrast_theme_has_cyan_accent() {
        let theme = high_contrast();
        // Pure cyan #00ffff
        assert_eq!(theme.accent, Color::Rgb(0, 255, 255));
    }

    #[test]
    fn high_contrast_theme_uses_saturated_colors() {
        let theme = high_contrast();
        // Success should be pure green
        assert_eq!(theme.success, Color::Rgb(0, 255, 0));
        // Warning should be pure yellow
        assert_eq!(theme.warning, Color::Rgb(255, 255, 0));
        // Error should be pure red
        assert_eq!(theme.error, Color::Rgb(255, 0, 0));
    }

    // === ThemeLoader builtin integration tests ===

    #[test]
    fn theme_loader_new_includes_all_builtins() {
        let loader = ThemeLoader::new();
        let names = loader.list();
        assert!(names.contains(&"vibes"));
        assert!(names.contains(&"dark"));
        assert!(names.contains(&"light"));
        assert!(names.contains(&"high-contrast"));
    }
}
