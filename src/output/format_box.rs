//! Format Box Output
//!
//! Creates SENA-branded format boxes for various output types

/// Format box builder
#[derive(Debug)]
pub struct FormatBox {
    title: String,
    width: usize,
    border_style: BorderStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
    Double,
    Single,
}

impl FormatBox {
    /// Create a new format box with title
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            width: 64,
            border_style: BorderStyle::Double,
        }
    }

    /// Set box width
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Use single line borders
    pub fn single_border(mut self) -> Self {
        self.border_style = BorderStyle::Single;
        self
    }

    /// Render the format box
    pub fn render(&self) -> String {
        match self.border_style {
            BorderStyle::Double => self.render_double(),
            BorderStyle::Single => self.render_single(),
        }
    }

    fn render_double(&self) -> String {
        let inner_width = self.width - 2;
        let title_len = self.title.chars().count();
        let left_pad = (inner_width.saturating_sub(title_len)) / 2;
        let right_pad = inner_width
            .saturating_sub(title_len)
            .saturating_sub(left_pad);

        let mut output = String::new();

        // Top border
        output.push('╔');
        output.push_str(&"═".repeat(inner_width));
        output.push_str("╗\n");

        // Empty line
        output.push('║');
        output.push_str(&" ".repeat(inner_width));
        output.push_str("║\n");

        // Title line
        output.push('║');
        output.push_str(&" ".repeat(left_pad));
        output.push_str(&self.title);
        output.push_str(&" ".repeat(right_pad));
        output.push_str("║\n");

        // Empty line
        output.push('║');
        output.push_str(&" ".repeat(inner_width));
        output.push_str("║\n");

        // Bottom border
        output.push('╚');
        output.push_str(&"═".repeat(inner_width));
        output.push('╝');

        output
    }

    fn render_single(&self) -> String {
        let inner_width = self.width - 2;
        let title_len = self.title.chars().count();
        let left_pad = (inner_width.saturating_sub(title_len)) / 2;
        let right_pad = inner_width
            .saturating_sub(title_len)
            .saturating_sub(left_pad);

        let mut output = String::new();

        // Top border
        output.push('┌');
        output.push_str(&"─".repeat(inner_width));
        output.push_str("┐\n");

        // Title line
        output.push('│');
        output.push_str(&" ".repeat(left_pad));
        output.push_str(&self.title);
        output.push_str(&" ".repeat(right_pad));
        output.push_str("│\n");

        // Bottom border
        output.push('└');
        output.push_str(&"─".repeat(inner_width));
        output.push('┘');

        output
    }
}

pub fn brilliant_thinking_box() -> String {
    use crate::config::SenaConfig;
    FormatBox::new(&SenaConfig::brand_title("BRILLIANT THINKING")).render()
}

pub fn truth_verification_box() -> String {
    use crate::config::SenaConfig;
    FormatBox::new(&SenaConfig::brand_title("TRUTH VERIFICATION SYSTEM")).render()
}

pub fn code_analysis_box() -> String {
    use crate::config::SenaConfig;
    FormatBox::new(&SenaConfig::brand_title("CODE QUALITY ANALYSIS")).render()
}

/// Create a section separator
pub fn section_separator(title: &str) -> String {
    let mut output = String::new();
    output.push_str(&"═".repeat(64));
    output.push('\n');
    output.push_str("  ");
    output.push_str(title);
    output.push('\n');
    output.push_str(&"═".repeat(64));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_box_double() {
        let box_output = FormatBox::new("Test Title").render();
        assert!(box_output.contains("╔"));
        assert!(box_output.contains("╚"));
        assert!(box_output.contains("Test Title"));
    }

    #[test]
    fn test_format_box_single() {
        let box_output = FormatBox::new("Test").single_border().render();
        assert!(box_output.contains("┌"));
        assert!(box_output.contains("└"));
    }

    #[test]
    fn test_brilliant_thinking_box() {
        let output = brilliant_thinking_box();
        assert!(output.contains("BRILLIANT THINKING"));
        assert!(output.contains("🦁"));
    }

    #[test]
    fn test_truth_verification_box() {
        let output = truth_verification_box();
        assert!(output.contains("TRUTH VERIFICATION"));
    }

    #[test]
    fn test_section_separator() {
        let output = section_separator("ANALYSIS");
        assert!(output.contains("═"));
        assert!(output.contains("ANALYSIS"));
    }
}
