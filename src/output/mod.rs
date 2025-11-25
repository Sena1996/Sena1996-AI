//! SENA Output Formatting Module
//!
//! Beautiful Unicode tables, progress bars, and format boxes

pub mod tables;
pub mod progress;
pub mod format_box;

pub use tables::TableBuilder;
pub use progress::ProgressBar;
pub use format_box::FormatBox;
