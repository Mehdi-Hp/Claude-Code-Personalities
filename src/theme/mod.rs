use serde::{Deserialize, Serialize};

pub mod colors;
pub mod context;
pub mod presets;

pub use colors::ThemeColors;

/// Built-in theme options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    Default, // New terminal-colors theme
    Dark,
    Light,
    Solarized,
    Dracula,
    Nord,
    Gruvbox,
    HighContrast,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Default // Use the new Default theme
    }
}

impl Theme {
    /// Get all available themes
    pub fn all() -> Vec<Theme> {
        vec![
            Theme::Default,
            Theme::Dark,
            Theme::Light,
            Theme::Solarized,
            Theme::Dracula,
            Theme::Nord,
            Theme::Gruvbox,
            Theme::HighContrast,
        ]
    }

    /// Get theme display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::Default => "Default (Terminal Colors)",
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Solarized => "Solarized Dark",
            Theme::Dracula => "Dracula",
            Theme::Nord => "Nord",
            Theme::Gruvbox => "Gruvbox Dark",
            Theme::HighContrast => "High Contrast",
        }
    }

    /// Get theme description
    pub fn description(&self) -> &'static str {
        match self {
            Theme::Default => "Uses terminal's 256-color palette with context-aware colors",
            Theme::Dark => "Default dark theme with vibrant colors",
            Theme::Light => "Clean light theme optimized for bright terminals",
            Theme::Solarized => "Professional Solarized Dark color palette",
            Theme::Dracula => "Popular dark theme with purple and pink accents",
            Theme::Nord => "Arctic, north-bluish color palette",
            Theme::Gruvbox => "Retro groove colors with warm, earthy tones",
            Theme::HighContrast => "High contrast colors for accessibility",
        }
    }

    /// Get colors for this theme
    pub fn colors(&self) -> ThemeColors {
        match self {
            Theme::Default => ThemeColors::default_terminal(),
            Theme::Dark => ThemeColors::dark(),
            Theme::Light => ThemeColors::light(),
            Theme::Solarized => ThemeColors::solarized(),
            Theme::Dracula => ThemeColors::dracula(),
            Theme::Nord => ThemeColors::nord(),
            Theme::Gruvbox => ThemeColors::gruvbox(),
            Theme::HighContrast => ThemeColors::high_contrast(),
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" | "terminal" => Ok(Theme::Default),
            "dark" => Ok(Theme::Dark),
            "light" => Ok(Theme::Light),
            "solarized" | "solarized-dark" => Ok(Theme::Solarized),
            "dracula" => Ok(Theme::Dracula),
            "nord" => Ok(Theme::Nord),
            "gruvbox" | "gruvbox-dark" => Ok(Theme::Gruvbox),
            "high-contrast" | "highcontrast" => Ok(Theme::HighContrast),
            _ => Err(format!("Unknown theme: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_parsing() {
        assert_eq!("dark".parse::<Theme>().unwrap(), Theme::Dark);
        assert_eq!("solarized".parse::<Theme>().unwrap(), Theme::Solarized);
        assert_eq!(
            "high-contrast".parse::<Theme>().unwrap(),
            Theme::HighContrast
        );
    }

    #[test]
    fn test_theme_display() {
        assert_eq!(Theme::Dark.to_string(), "Dark");
        assert_eq!(Theme::Solarized.to_string(), "Solarized Dark");
    }

    #[test]
    fn test_all_themes_have_colors() {
        for theme in Theme::all() {
            let colors = theme.colors();
            // Just verify we get some colors back
            // Verify personality color has some value (not all zeros)
            match &colors.personality {
                crate::theme::colors::Color::Rgb { r, g, b } => {
                    assert!(*r > 0 || *g > 0 || *b > 0);
                }
                crate::theme::colors::Color::Terminal256(_) => {
                    // Terminal256 colors are always valid
                    assert!(true);
                }
            }
        }
    }
}
