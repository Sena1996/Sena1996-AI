//! Live Progress Bar Output
//!
//! Creates Unicode progress bars with:
//! - Live in-place updates (no duplicates)
//! - Custom emoji/prefix support
//! - Spinner animation
//! - Color support

use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use crate::config::SenaConfig;

/// ANSI Escape Codes for terminal control
pub mod ansi {
    pub const CLEAR_LINE: &str = "\x1b[2K";
    pub const CURSOR_START: &str = "\r";
    pub const CURSOR_UP: &str = "\x1b[1A";
    pub const CURSOR_HIDE: &str = "\x1b[?25l";
    pub const CURSOR_SHOW: &str = "\x1b[?25h";
    pub const CURSOR_SAVE: &str = "\x1b[s";
    pub const CURSOR_RESTORE: &str = "\x1b[u";

    // Colors
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const RED: &str = "\x1b[31m";
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
}

/// Spinner characters for animation
pub const SPINNERS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const SPINNER_DOTS: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
pub const SPINNER_SIMPLE: &[&str] = &["|", "/", "-", "\\"];

/// Configuration for progress bar appearance
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    pub prefix: String,
    pub emoji: String,
    pub width: usize,
    pub show_emoji: bool,
    pub show_percentage: bool,
    pub show_spinner: bool,
    pub use_colors: bool,
    pub filled_char: char,
    pub empty_char: char,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        let user = SenaConfig::user();
        Self {
            prefix: user.prefix.clone(),
            emoji: user.emoji.clone(),
            width: 30,
            show_emoji: true,
            show_percentage: true,
            show_spinner: false,
            use_colors: true,
            filled_char: '█',
            empty_char: '░',
        }
    }
}

impl ProgressConfig {
    pub fn from_user_config() -> Self {
        Self::default()
    }

    /// Create config with custom prefix and emoji
    pub fn custom(prefix: &str, emoji: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            emoji: emoji.to_string(),
            ..Default::default()
        }
    }
}

/// Live Progress Bar with in-place updates
#[derive(Debug)]
pub struct ProgressBar {
    label: String,
    percent: f32,
    config: ProgressConfig,
    spinner_index: usize,
    start_time: Instant,
}

impl ProgressBar {
    /// Create a new progress bar with default config
    pub fn new(label: &str, percent: f32) -> Self {
        Self {
            label: label.to_string(),
            percent: percent.clamp(0.0, 100.0),
            config: ProgressConfig::from_user_config(),
            spinner_index: 0,
            start_time: Instant::now(),
        }
    }

    /// Create with custom config
    pub fn with_config(label: &str, percent: f32, config: ProgressConfig) -> Self {
        Self {
            label: label.to_string(),
            percent: percent.clamp(0.0, 100.0),
            config,
            spinner_index: 0,
            start_time: Instant::now(),
        }
    }

    /// Set bar width
    pub fn width(mut self, width: usize) -> Self {
        self.config.width = width;
        self
    }

    /// Disable emoji in bar
    pub fn no_emoji(mut self) -> Self {
        self.config.show_emoji = false;
        self
    }

    /// Enable spinner
    pub fn with_spinner(mut self) -> Self {
        self.config.show_spinner = true;
        self
    }

    /// Disable colors
    pub fn no_colors(mut self) -> Self {
        self.config.use_colors = false;
        self
    }

    /// Update the percentage
    pub fn set_percent(&mut self, percent: f32) {
        self.percent = percent.clamp(0.0, 100.0);
    }

    /// Increment spinner animation
    pub fn tick(&mut self) {
        self.spinner_index = (self.spinner_index + 1) % SPINNERS.len();
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Render the progress bar to a string
    pub fn render(&self) -> String {
        let filled = ((self.percent / 100.0) * self.config.width as f32) as usize;
        let empty = self.config.width.saturating_sub(filled);

        // Build the bar with optional emoji marker
        let bar: String = if self.config.show_emoji {
            let filled_str = self.config.filled_char.to_string().repeat(filled.saturating_sub(1));
            let empty_str = self.config.empty_char.to_string().repeat(empty);
            if filled > 0 {
                format!("{}{}{}", filled_str, self.config.emoji, empty_str)
            } else {
                format!("{}{}", self.config.emoji, empty_str)
            }
        } else {
            let filled_str = self.config.filled_char.to_string().repeat(filled);
            let empty_str = self.config.empty_char.to_string().repeat(empty);
            format!("{}{}", filled_str, empty_str)
        };

        // Status indicator
        let status = if self.percent >= 100.0 {
            " ✅"
        } else {
            ""
        };

        // Spinner
        let spinner = if self.config.show_spinner && self.percent < 100.0 {
            format!("{} ", SPINNERS[self.spinner_index])
        } else {
            String::new()
        };

        // Color the bar
        let colored_bar = if self.config.use_colors {
            let color = if self.percent >= 100.0 {
                ansi::GREEN
            } else if self.percent >= 50.0 {
                ansi::CYAN
            } else {
                ansi::YELLOW
            };
            format!("{}[{}]{}", color, bar, ansi::RESET)
        } else {
            format!("[{}]", bar)
        };

        // Percentage
        let pct = if self.config.show_percentage {
            format!(" {:.0}%", self.percent)
        } else {
            String::new()
        };

        format!("{}{}: {}{}{}", spinner, self.label, colored_bar, pct, status)
    }

    /// Print the progress bar (updates in-place)
    pub fn print(&self) {
        print!("{}{}{}", ansi::CURSOR_START, ansi::CLEAR_LINE, self.render());
        io::stdout().flush().unwrap_or(());
    }

    /// Print and move to next line (for final state)
    pub fn println(&self) {
        println!("{}{}{}", ansi::CURSOR_START, ansi::CLEAR_LINE, self.render());
    }
}

/// Live Multi-Task Progress Display
pub struct LiveProgress {
    tasks: Vec<(String, f32)>,
    config: ProgressConfig,
    lines_printed: usize,
    spinner_index: usize,
}

impl LiveProgress {
    /// Create new multi-progress display
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            config: ProgressConfig::from_user_config(),
            lines_printed: 0,
            spinner_index: 0,
        }
    }

    /// Create with custom config
    pub fn with_config(config: ProgressConfig) -> Self {
        Self {
            tasks: Vec::new(),
            config,
            lines_printed: 0,
            spinner_index: 0,
        }
    }

    /// Add a task
    pub fn add_task(&mut self, name: &str, percent: f32) {
        self.tasks.push((name.to_string(), percent));
    }

    /// Update a task by index
    pub fn update_task(&mut self, index: usize, percent: f32) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.1 = percent.clamp(0.0, 100.0);
        }
    }

    /// Update a task by name
    pub fn update_by_name(&mut self, name: &str, percent: f32) {
        for task in &mut self.tasks {
            if task.0 == name {
                task.1 = percent.clamp(0.0, 100.0);
                break;
            }
        }
    }

    /// Tick spinner
    pub fn tick(&mut self) {
        self.spinner_index = (self.spinner_index + 1) % SPINNERS.len();
    }

    /// Check if all tasks are complete
    pub fn is_complete(&self) -> bool {
        self.tasks.iter().all(|(_, pct)| *pct >= 100.0)
    }

    /// Clear previously printed lines
    fn clear_previous(&self) {
        if self.lines_printed > 0 {
            // Move cursor up and clear each line
            for _ in 0..self.lines_printed {
                print!("{}{}", ansi::CURSOR_UP, ansi::CLEAR_LINE);
            }
        }
    }

    /// Render to string (static, no ANSI)
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Title box
        let title = format!("{} {} TASK PROGRESS", self.config.prefix, self.config.emoji);
        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str(&format!("║{:^62}║\n", title));
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        // Progress bars box
        output.push_str("┌──────────────────────────────────────────────────────────────┐\n");
        for (name, pct) in &self.tasks {
            let bar = ProgressBar::with_config(name, *pct, self.config.clone());
            output.push_str(&format!("│ {:58} │\n", bar.render()));
        }
        output.push_str("└──────────────────────────────────────────────────────────────┘");

        output
    }

    /// Print with live update (clears previous, prints new)
    pub fn print(&mut self) {
        // Clear previous output
        self.clear_previous();

        // Calculate lines to print
        let header_lines = 5; // Title box + empty line
        let content_lines = self.tasks.len() + 2; // Tasks + border lines
        self.lines_printed = header_lines + content_lines;

        // Title box
        let title = format!("{} {} TASK PROGRESS", self.config.prefix, self.config.emoji);
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║{:^62}║", title);
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        // Progress bars box
        println!("┌──────────────────────────────────────────────────────────────┐");
        for (name, pct) in &self.tasks {
            let mut bar = ProgressBar::with_config(name, *pct, self.config.clone());
            bar.spinner_index = self.spinner_index;
            println!("│ {:58} │", bar.render());
        }
        println!("└──────────────────────────────────────────────────────────────┘");

        io::stdout().flush().unwrap_or(());
    }

    /// Finish and show final state
    pub fn finish(&mut self) {
        self.clear_previous();
        self.lines_printed = 0;

        // Mark all complete
        for task in &mut self.tasks {
            task.1 = 100.0;
        }

        // Print final state (without clearing)
        let title = format!("{} {} COMPLETE!", self.config.prefix, self.config.emoji);
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║{}{:^62}{}║", ansi::GREEN, title, ansi::RESET);
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("┌──────────────────────────────────────────────────────────────┐");
        for (name, pct) in &self.tasks {
            let bar = ProgressBar::with_config(name, *pct, self.config.clone());
            println!("│ {:58} │", bar.render());
        }
        println!("└──────────────────────────────────────────────────────────────┘");
    }
}

impl Default for LiveProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Spinner for indeterminate progress
pub struct Spinner {
    message: String,
    index: usize,
    #[allow(dead_code)]
    running: Arc<AtomicBool>,
    config: ProgressConfig,
}

impl Spinner {
    /// Create a new spinner
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            index: 0,
            running: Arc::new(AtomicBool::new(false)),
            config: ProgressConfig::from_user_config(),
        }
    }

    /// Tick the spinner animation
    pub fn tick(&mut self) {
        self.index = (self.index + 1) % SPINNERS.len();
        print!("{}{}{} {} {}...",
            ansi::CURSOR_START,
            ansi::CLEAR_LINE,
            self.config.emoji,
            SPINNERS[self.index],
            self.message
        );
        io::stdout().flush().unwrap_or(());
    }

    /// Stop with success message
    pub fn success(&self, message: &str) {
        println!("{}{}{} ✅ {}",
            ansi::CURSOR_START,
            ansi::CLEAR_LINE,
            self.config.emoji,
            message
        );
    }

    /// Stop with error message
    pub fn error(&self, message: &str) {
        println!("{}{}{} ❌ {}",
            ansi::CURSOR_START,
            ansi::CLEAR_LINE,
            self.config.emoji,
            message
        );
    }
}

// ============================================================================
// Legacy API (for backward compatibility)
// ============================================================================

/// Render multiple progress bars in a box (static, legacy)
pub fn render_progress_box(title: &str, bars: &[ProgressBar]) -> String {
    let mut output = String::new();

    output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    output.push_str(&format!("║{:^62}║\n", title));
    output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

    output.push_str("┌──────────────────────────────────────────────────────────────┐\n");
    for bar in bars {
        output.push_str(&format!("│ {:58} │\n", bar.render()));
    }
    output.push_str("└──────────────────────────────────────────────────────────────┘");

    output
}

/// Create a multi-task progress display (legacy)
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
        let config = ProgressConfig::from_user_config();
        let title = format!("{} {} TASK PROGRESS", config.prefix, config.emoji);

        let bars: Vec<ProgressBar> = self.tasks
            .iter()
            .map(|(name, pct)| ProgressBar::with_config(name, *pct, config.clone()))
            .collect();

        render_progress_box(&title, &bars)
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_zero() {
        let bar = ProgressBar::new("Task", 0.0).no_colors();
        let output = bar.render();
        assert!(output.contains("Task"));
        assert!(output.contains("0%"));
    }

    #[test]
    fn test_progress_bar_fifty() {
        let bar = ProgressBar::new("Task", 50.0).no_colors();
        let output = bar.render();
        assert!(output.contains("50%"));
        assert!(output.contains("█"));
    }

    #[test]
    fn test_progress_bar_complete() {
        let bar = ProgressBar::new("Task", 100.0).no_colors();
        let output = bar.render();
        assert!(output.contains("100%"));
        assert!(output.contains("✅"));
    }

    #[test]
    fn test_progress_bar_no_emoji() {
        let bar = ProgressBar::new("Task", 50.0).no_emoji().no_colors();
        let output = bar.render();
        assert!(output.contains("["));
        assert!(output.contains("█"));
    }

    #[test]
    fn test_custom_config() {
        let config = ProgressConfig::custom("JARVIS", "🤖");
        let bar = ProgressBar::with_config("Task", 50.0, config);
        let output = bar.render();
        assert!(output.contains("🤖"));
    }

    #[test]
    fn test_live_progress() {
        let mut lp = LiveProgress::new();
        lp.add_task("Task 1", 100.0);
        lp.add_task("Task 2", 50.0);

        assert!(!lp.is_complete());
        lp.update_task(1, 100.0);
        assert!(lp.is_complete());
    }

    #[test]
    fn test_multi_progress_legacy() {
        let mp = MultiProgress::new()
            .add_task("Initialize", 100.0)
            .add_task("Process", 50.0)
            .add_task("Finalize", 0.0);

        let output = mp.render();
        assert!(output.contains("Initialize"));
        assert!(output.contains("Process"));
        assert!(output.contains("TASK PROGRESS"));
    }

    #[test]
    fn test_progress_config_default() {
        let config = ProgressConfig::default();
        let user = SenaConfig::user();
        assert_eq!(config.prefix, user.prefix);
        assert_eq!(config.emoji, user.emoji);
        assert_eq!(config.width, 30);
    }
}
