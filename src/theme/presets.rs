use super::{Theme, ThemeColors};

/// Helper functions for applying theme colors consistently
impl Theme {
    /// Apply personality color with bold formatting
    pub fn apply_personality(&self, text: &str) -> String {
        let colors = self.colors();
        colors.personality.apply_bold(text).to_string()
    }

    /// Apply activity color
    pub fn apply_activity(&self, text: &str) -> String {
        let colors = self.colors();
        colors.activity.apply(text).to_string()
    }

    /// Apply directory/workspace color
    pub fn apply_directory(&self, text: &str) -> String {
        let colors = self.colors();
        colors.directory.apply(text).to_string()
    }

    /// Apply file color
    pub fn apply_file(&self, text: &str) -> String {
        let colors = self.colors();
        colors.file.apply(text).to_string()
    }

    /// Apply error color
    pub fn apply_error(&self, text: &str) -> String {
        let colors = self.colors();
        colors.error.apply(text).to_string()
    }

    /// Apply warning color
    pub fn apply_warning(&self, text: &str) -> String {
        let colors = self.colors();
        colors.warning.apply(text).to_string()
    }

    /// Apply success color
    pub fn apply_success(&self, text: &str) -> String {
        let colors = self.colors();
        colors.success.apply(text).to_string()
    }

    /// Apply info color
    pub fn apply_info(&self, text: &str) -> String {
        let colors = self.colors();
        colors.info.apply(text).to_string()
    }

    /// Apply separator color
    pub fn apply_separator(&self, text: &str) -> String {
        let colors = self.colors();
        colors.separator.apply(text).to_string()
    }

    /// Apply model-specific color
    pub fn apply_model_color(&self, text: &str, model_name: &str) -> String {
        let colors = self.colors();
        let color = if model_name.to_lowercase().contains("opus") {
            &colors.model_opus
        } else if model_name.to_lowercase().contains("sonnet") {
            &colors.model_sonnet
        } else if model_name.to_lowercase().contains("haiku") {
            &colors.model_haiku
        } else {
            &colors.personality
        };
        color.apply(text).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_application() {
        let theme = Theme::Dark;

        let personality = theme.apply_personality("Test");
        let error = theme.apply_error("Error");
        let model = theme.apply_model_color("[Opus]", "Opus");

        // Just verify we get colored strings back
        assert!(personality.contains("Test"));
        assert!(error.contains("Error"));
        assert!(model.contains("Opus"));
    }

    #[test]
    fn test_model_color_matching() {
        let theme = Theme::Dark;

        let opus = theme.apply_model_color("Test", "Claude-3-Opus");
        let sonnet = theme.apply_model_color("Test", "Claude-3-Sonnet");
        let haiku = theme.apply_model_color("Test", "Claude-3-Haiku");
        let unknown = theme.apply_model_color("Test", "GPT-4");

        // All should be colored (non-empty)
        assert!(opus.contains("Test"));
        assert!(sonnet.contains("Test"));
        assert!(haiku.contains("Test"));
        assert!(unknown.contains("Test"));
    }
}
