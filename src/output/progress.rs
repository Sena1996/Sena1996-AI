//! Progress Bar Output
//!
//! Creates Unicode progress bars with SENA branding

/// Progress bar renderer
#[derive(Debug)]
pub struct ProgressBar {
    label: String,
    percent: f32,
    width: usize,
    show_emoji: bool,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(label: &str, percent: f32) -> Self {
        Self {
            label: label.to_string(),
            percent: percent.clamp(0.0, 100.0),
            width: 30,
            show_emoji: true,
        }
    }

    /// Set bar width
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Disable emoji
    pub fn no_emoji(mut self) -> Self {
        self.show_emoji = false;
        self
    }

    /// Render the progress bar
    pub fn render(&self) -> String {
        let filled = ((self.percent / 100.0) * self.width as f32) as usize;
        let empty = self.width.saturating_sub(filled);

        // Build the bar
        let bar: String = if self.show_emoji {
            let filled_str = "█".repeat(filled.saturating_sub(1));
            let empty_str = "░".repeat(empty);
            if filled > 0 {
                format!("{}🦁{}", filled_str, empty_str)
            } else {
                format!("🦁{}", empty_str)
            }
        } else {
            let filled_str = "█".repeat(filled);
            let empty_str = "░".repeat(empty);
            format!("{}{}", filled_str, empty_str)
        };

        // Status indicator
        let status = if self.percent >= 100.0 {
            " ✅"
        } else if self.percent > 0.0 {
            ""
        } else {
            ""
        };

        format!("{}: [{}] {:.0}%{}", self.label, bar, self.percent, status)
    }
}

/// Render multiple progress bars in a box
pub fn render_progress_box(title: &str, bars: &[ProgressBar]) -> String {
    let mut output = String::new();

    // Title box
    output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    output.push_str(&format!("║              {}                           ║\n", title));
    output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

    // Progress bars box
    output.push_str("┌──────────────────────────────────────────────────────────────┐\n");
    for bar in bars {
        output.push_str(&format!("│ {:60} │\n", bar.render()));
    }
    output.push_str("└──────────────────────────────────────────────────────────────┘");

    output
}

/// Create a multi-task progress display
pub struct MultiProgress {
    tasks: Vec<(String, f32)>,
}

impl MultiProgress {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(mut self, name: &str, percent: f32) -> Self {
        self.tasks.push((name.to_string(), percent));
        self
    }

    pub fn render(&self) -> String {
        let bars: Vec<ProgressBar> = self.tasks
            .iter()
            .map(|(name, pct)| ProgressBar::new(name, *pct))
            .collect();

        render_progress_box("SENA 🦁 TASK PROGRESS", &bars)
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_zero() {
        let bar = ProgressBar::new("Task", 0.0);
        let output = bar.render();
        assert!(output.contains("Task"));
        assert!(output.contains("0%"));
        assert!(output.contains("🦁"));
    }

    #[test]
    fn test_progress_bar_fifty() {
        let bar = ProgressBar::new("Task", 50.0);
        let output = bar.render();
        assert!(output.contains("50%"));
        assert!(output.contains("█"));
    }

    #[test]
    fn test_progress_bar_complete() {
        let bar = ProgressBar::new("Task", 100.0);
        let output = bar.render();
        assert!(output.contains("100%"));
        assert!(output.contains("✅"));
    }

    #[test]
    fn test_progress_bar_no_emoji() {
        let bar = ProgressBar::new("Task", 50.0).no_emoji();
        let output = bar.render();
        assert!(!output.contains("🦁"));
    }

    #[test]
    fn test_multi_progress() {
        let mp = MultiProgress::new()
            .add_task("Initialize", 100.0)
            .add_task("Process", 50.0)
            .add_task("Finalize", 0.0);

        let output = mp.render();
        assert!(output.contains("Initialize"));
        assert!(output.contains("Process"));
        assert!(output.contains("TASK PROGRESS"));
    }
}
