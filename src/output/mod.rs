//! SENA Output Formatting Module
//!
//! Beautiful Unicode tables, progress bars, and format boxes
//! SENA v5.0 - Personalized AI

pub mod tables;
pub mod progress;
pub mod format_box;

pub use tables::TableBuilder;
pub use progress::{
    ProgressBar, ProgressConfig, LiveProgress, MultiProgress, Spinner,
    render_progress_box, ansi, SPINNERS, SPINNER_DOTS, SPINNER_SIMPLE
};
pub use format_box::FormatBox;
