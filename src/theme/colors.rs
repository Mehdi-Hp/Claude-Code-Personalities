use colored::{ColoredString, Colorize};

/// Color wrapper for theme support
#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }

    /// Apply this color to a string
    pub fn apply<T: AsRef<str>>(&self, text: T) -> ColoredString {
        text.as_ref().truecolor(self.r, self.g, self.b)
    }

    /// Apply this color with bold formatting
    pub fn apply_bold<T: AsRef<str>>(&self, text: T) -> ColoredString {
        text.as_ref().truecolor(self.r, self.g, self.b).bold()
    }
}

/// Complete color scheme for a theme
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub personality: Color,
    pub activity: Color,
    pub directory: Color,
    pub file: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    pub separator: Color,
    pub model_opus: Color,
    pub model_sonnet: Color,
    pub model_haiku: Color,
}

impl ThemeColors {
    /// Dark theme colors (current default)
    pub fn dark() -> Self {
        Self {
            personality: Color::new(255, 255, 255), // White
            activity: Color::new(0, 255, 255),      // Cyan
            directory: Color::new(85, 170, 255),    // Light blue
            file: Color::new(255, 255, 170),        // Light yellow
            error: Color::new(255, 85, 85),         // Red
            warning: Color::new(255, 184, 108),     // Orange
            success: Color::new(80, 250, 123),      // Green
            info: Color::new(139, 233, 253),        // Light cyan
            separator: Color::new(128, 128, 128),   // Gray
            model_opus: Color::new(255, 85, 255),   // Magenta
            model_sonnet: Color::new(85, 255, 255), // Cyan
            model_haiku: Color::new(85, 255, 85),   // Green
        }
    }

    /// Light theme colors
    pub fn light() -> Self {
        Self {
            personality: Color::new(64, 64, 64),   // Dark gray
            activity: Color::new(0, 102, 204),     // Blue
            directory: Color::new(85, 85, 170),    // Dark blue
            file: Color::new(170, 136, 0),         // Dark yellow
            error: Color::new(170, 0, 0),          // Dark red
            warning: Color::new(204, 102, 0),      // Dark orange
            success: Color::new(0, 136, 0),        // Dark green
            info: Color::new(0, 136, 170),         // Dark cyan
            separator: Color::new(170, 170, 170),  // Light gray
            model_opus: Color::new(170, 0, 170),   // Dark magenta
            model_sonnet: Color::new(0, 136, 170), // Dark cyan
            model_haiku: Color::new(0, 136, 0),    // Dark green
        }
    }

    /// Solarized Dark theme
    pub fn solarized() -> Self {
        Self {
            personality: Color::from_hex(0xfdf6e3),  // Base3
            activity: Color::from_hex(0x2aa198),     // Cyan
            directory: Color::from_hex(0x268bd2),    // Blue
            file: Color::from_hex(0xb58900),         // Yellow
            error: Color::from_hex(0xdc322f),        // Red
            warning: Color::from_hex(0xcb4b16),      // Orange
            success: Color::from_hex(0x859900),      // Green
            info: Color::from_hex(0x2aa198),         // Cyan
            separator: Color::from_hex(0x586e75),    // Base01
            model_opus: Color::from_hex(0xd33682),   // Magenta
            model_sonnet: Color::from_hex(0x2aa198), // Cyan
            model_haiku: Color::from_hex(0x859900),  // Green
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            personality: Color::from_hex(0xf8f8f2),  // Foreground
            activity: Color::from_hex(0x8be9fd),     // Cyan
            directory: Color::from_hex(0x8be9fd),    // Cyan
            file: Color::from_hex(0xf1fa8c),         // Yellow
            error: Color::from_hex(0xff5555),        // Red
            warning: Color::from_hex(0xffb86c),      // Orange
            success: Color::from_hex(0x50fa7b),      // Green
            info: Color::from_hex(0xbd93f9),         // Purple
            separator: Color::from_hex(0x6272a4),    // Comment
            model_opus: Color::from_hex(0xff79c6),   // Pink
            model_sonnet: Color::from_hex(0x8be9fd), // Cyan
            model_haiku: Color::from_hex(0x50fa7b),  // Green
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            personality: Color::from_hex(0xeceff4),  // Snow Storm
            activity: Color::from_hex(0x88c0d0),     // Frost
            directory: Color::from_hex(0x5e81ac),    // Frost
            file: Color::from_hex(0xebcb8b),         // Aurora Yellow
            error: Color::from_hex(0xbf616a),        // Aurora Red
            warning: Color::from_hex(0xd08770),      // Aurora Orange
            success: Color::from_hex(0xa3be8c),      // Aurora Green
            info: Color::from_hex(0x81a1c1),         // Frost
            separator: Color::from_hex(0x4c566a),    // Polar Night
            model_opus: Color::from_hex(0xb48ead),   // Aurora Purple
            model_sonnet: Color::from_hex(0x88c0d0), // Frost
            model_haiku: Color::from_hex(0xa3be8c),  // Aurora Green
        }
    }

    /// Gruvbox theme
    pub fn gruvbox() -> Self {
        Self {
            personality: Color::from_hex(0xfbf1c7),  // Light fg
            activity: Color::from_hex(0x83a598),     // Blue
            directory: Color::from_hex(0x83a598),    // Blue
            file: Color::from_hex(0xd79921),         // Yellow
            error: Color::from_hex(0xfb4934),        // Red
            warning: Color::from_hex(0xfe8019),      // Orange
            success: Color::from_hex(0xb8bb26),      // Green
            info: Color::from_hex(0x8ec07c),         // Aqua
            separator: Color::from_hex(0x665c54),    // Gray
            model_opus: Color::from_hex(0xd3869b),   // Purple
            model_sonnet: Color::from_hex(0x8ec07c), // Aqua
            model_haiku: Color::from_hex(0xb8bb26),  // Green
        }
    }

    /// High contrast theme
    pub fn high_contrast() -> Self {
        Self {
            personality: Color::new(255, 255, 255), // Pure white
            activity: Color::new(0, 255, 255),      // Bright cyan
            directory: Color::new(85, 170, 255),    // Light blue
            file: Color::new(255, 255, 0),          // Pure yellow
            error: Color::new(255, 0, 0),           // Pure red
            warning: Color::new(255, 165, 0),       // Pure orange
            success: Color::new(0, 255, 0),         // Pure green
            info: Color::new(0, 191, 255),          // Bright blue
            separator: Color::new(192, 192, 192),   // Light gray
            model_opus: Color::new(255, 0, 255),    // Pure magenta
            model_sonnet: Color::new(0, 255, 255),  // Pure cyan
            model_haiku: Color::new(0, 255, 0),     // Pure green
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex(0xff8040);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_theme_colors_dark() {
        let colors = ThemeColors::dark();
        // Verify dark theme has proper colors (not all zeros)
        assert!(colors.personality.r > 0 || colors.personality.g > 0 || colors.personality.b > 0);
        assert!(colors.error.r > 0 || colors.error.g > 0 || colors.error.b > 0);
    }
}
