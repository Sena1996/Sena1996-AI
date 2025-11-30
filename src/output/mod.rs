//! SENA Output Formatting Module
//!
//! Beautiful Unicode tables, progress bars, and format boxes
//! SENA v5.0 - Personalized AI

pub mod format_box;
pub mod progress;
pub mod streaming;
pub mod tables;
pub mod websocket;

pub use format_box::FormatBox;
pub use progress::{
    ansi, render_progress_box, LiveProgress, MultiProgress, ProgressBar, ProgressConfig, Spinner,
    SPINNERS, SPINNER_DOTS, SPINNER_SIMPLE,
};
pub use streaming::{
    create_stream, ConsoleStreamRenderer, JsonStreamRenderer, StreamEvent, StreamEventType,
    StreamReader, StreamWriter, TypewriterEffect,
};
pub use tables::TableBuilder;
pub use websocket::{
    ClientId, MessageType, StreamToWebSocket, WebSocketBroadcaster, WebSocketMessage,
};
