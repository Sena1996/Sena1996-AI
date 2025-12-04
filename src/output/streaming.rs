use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamEventType {
    Start,
    Text,
    ToolCall,
    ToolResult,
    Thinking,
    Progress,
    Error,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub event_type: StreamEventType,
    pub content: String,
    pub timestamp_ms: u64,
    pub metadata: Option<serde_json::Value>,
}

impl StreamEvent {
    pub fn new(event_type: StreamEventType, content: &str) -> Self {
        Self {
            event_type,
            content: content.to_string(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn start() -> Self {
        Self::new(StreamEventType::Start, "")
    }

    pub fn text(content: &str) -> Self {
        Self::new(StreamEventType::Text, content)
    }

    pub fn thinking(content: &str) -> Self {
        Self::new(StreamEventType::Thinking, content)
    }

    pub fn tool_call(tool_name: &str) -> Self {
        Self::new(StreamEventType::ToolCall, tool_name)
    }

    pub fn tool_result(result: &str) -> Self {
        Self::new(StreamEventType::ToolResult, result)
    }

    pub fn progress(message: &str) -> Self {
        Self::new(StreamEventType::Progress, message)
    }

    pub fn error(message: &str) -> Self {
        Self::new(StreamEventType::Error, message)
    }

    pub fn complete() -> Self {
        Self::new(StreamEventType::Complete, "")
    }
}

pub struct StreamWriter {
    sender: Sender<StreamEvent>,
    start_time: Instant,
    buffer: String,
    flush_threshold: usize,
}

impl StreamWriter {
    pub fn new(sender: Sender<StreamEvent>) -> Self {
        let _ = sender.send(StreamEvent::start());
        Self {
            sender,
            start_time: Instant::now(),
            buffer: String::new(),
            flush_threshold: 100,
        }
    }

    pub fn with_flush_threshold(mut self, threshold: usize) -> Self {
        self.flush_threshold = threshold;
        self
    }

    pub fn write_text(&mut self, text: &str) {
        self.buffer.push_str(text);
        if self.buffer.len() >= self.flush_threshold {
            self.flush();
        }
    }

    pub fn write_char(&mut self, c: char) {
        self.buffer.push(c);
        if self.buffer.len() >= self.flush_threshold {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let _ = self.sender.send(StreamEvent::text(&self.buffer));
            self.buffer.clear();
        }
    }

    pub fn thinking(&self, content: &str) {
        let _ = self.sender.send(StreamEvent::thinking(content));
    }

    pub fn tool_call(&self, tool_name: &str) {
        let _ = self.sender.send(StreamEvent::tool_call(tool_name));
    }

    pub fn tool_result(&self, result: &str) {
        let _ = self.sender.send(StreamEvent::tool_result(result));
    }

    pub fn progress(&self, message: &str) {
        let _ = self.sender.send(StreamEvent::progress(message));
    }

    pub fn error(&self, message: &str) {
        let _ = self.sender.send(StreamEvent::error(message));
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn complete(mut self) {
        self.flush();
        let _ = self.sender.send(StreamEvent::complete());
    }
}

impl Drop for StreamWriter {
    fn drop(&mut self) {
        self.flush();
    }
}

pub struct StreamReader {
    receiver: Receiver<StreamEvent>,
    accumulated: String,
    events: Vec<StreamEvent>,
}

impl StreamReader {
    pub fn new(receiver: Receiver<StreamEvent>) -> Self {
        Self {
            receiver,
            accumulated: String::new(),
            events: Vec::new(),
        }
    }

    pub fn try_recv(&mut self) -> Option<StreamEvent> {
        match self.receiver.try_recv() {
            Ok(event) => {
                if event.event_type == StreamEventType::Text {
                    self.accumulated.push_str(&event.content);
                }
                self.events.push(event.clone());
                Some(event)
            }
            Err(_) => None,
        }
    }

    pub fn recv_timeout(&mut self, timeout: Duration) -> Option<StreamEvent> {
        match self.receiver.recv_timeout(timeout) {
            Ok(event) => {
                if event.event_type == StreamEventType::Text {
                    self.accumulated.push_str(&event.content);
                }
                self.events.push(event.clone());
                Some(event)
            }
            Err(_) => None,
        }
    }

    pub fn accumulated_text(&self) -> &str {
        &self.accumulated
    }

    pub fn all_events(&self) -> &[StreamEvent] {
        &self.events
    }

    pub fn is_complete(&self) -> bool {
        self.events
            .iter()
            .any(|e| e.event_type == StreamEventType::Complete)
    }
}

pub fn create_stream() -> (StreamWriter, StreamReader) {
    let (sender, receiver) = mpsc::channel();
    let writer = StreamWriter::new(sender);
    let reader = StreamReader::new(receiver);
    (writer, reader)
}

pub struct ConsoleStreamRenderer {
    show_thinking: bool,
    show_tools: bool,
    typing_effect: bool,
    typing_delay_ms: u64,
}

impl ConsoleStreamRenderer {
    pub fn new() -> Self {
        Self {
            show_thinking: true,
            show_tools: true,
            typing_effect: false,
            typing_delay_ms: 10,
        }
    }

    pub fn with_typing_effect(mut self, delay_ms: u64) -> Self {
        self.typing_effect = true;
        self.typing_delay_ms = delay_ms;
        self
    }

    pub fn show_thinking(mut self, show: bool) -> Self {
        self.show_thinking = show;
        self
    }

    pub fn show_tools(mut self, show: bool) -> Self {
        self.show_tools = show;
        self
    }

    pub fn render_event(&self, event: &StreamEvent) {
        match event.event_type {
            StreamEventType::Start => {
                print!("\x1b[2K\r");
            }
            StreamEventType::Text => {
                if self.typing_effect {
                    for c in event.content.chars() {
                        print!("{}", c);
                        let _ = io::stdout().flush();
                        std::thread::sleep(Duration::from_millis(self.typing_delay_ms));
                    }
                } else {
                    print!("{}", event.content);
                    let _ = io::stdout().flush();
                }
            }
            StreamEventType::Thinking => {
                if self.show_thinking {
                    print!("\x1b[90m{}\x1b[0m", event.content);
                    let _ = io::stdout().flush();
                }
            }
            StreamEventType::ToolCall => {
                if self.show_tools {
                    print!("\x1b[36m[Tool: {}]\x1b[0m ", event.content);
                    let _ = io::stdout().flush();
                }
            }
            StreamEventType::ToolResult => {
                if self.show_tools {
                    print!(
                        "\x1b[32m[Result: {}...]\x1b[0m ",
                        event.content.chars().take(50).collect::<String>()
                    );
                    let _ = io::stdout().flush();
                }
            }
            StreamEventType::Progress => {
                print!("\x1b[33m{}\x1b[0m", event.content);
                let _ = io::stdout().flush();
            }
            StreamEventType::Error => {
                println!("\x1b[31mError: {}\x1b[0m", event.content);
                let _ = io::stdout().flush();
            }
            StreamEventType::Complete => {
                println!();
                let _ = io::stdout().flush();
            }
        }
    }

    pub fn render_stream(&self, reader: &mut StreamReader, timeout_ms: u64) {
        let timeout = Duration::from_millis(timeout_ms);
        loop {
            match reader.recv_timeout(timeout) {
                Some(event) => {
                    let is_complete = event.event_type == StreamEventType::Complete;
                    self.render_event(&event);
                    if is_complete {
                        break;
                    }
                }
                None => {
                    if reader.is_complete() {
                        break;
                    }
                }
            }
        }
    }
}

impl Default for ConsoleStreamRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JsonStreamRenderer;

impl JsonStreamRenderer {
    pub fn render_event(event: &StreamEvent) -> String {
        serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn render_sse(event: &StreamEvent) -> String {
        let event_type = match event.event_type {
            StreamEventType::Start => "start",
            StreamEventType::Text => "text",
            StreamEventType::ToolCall => "tool_call",
            StreamEventType::ToolResult => "tool_result",
            StreamEventType::Thinking => "thinking",
            StreamEventType::Progress => "progress",
            StreamEventType::Error => "error",
            StreamEventType::Complete => "complete",
        };

        let data = serde_json::json!({
            "type": event_type,
            "content": event.content,
            "timestamp_ms": event.timestamp_ms,
            "metadata": event.metadata,
        });

        format!("event: {}\ndata: {}\n\n", event_type, data)
    }
}

pub struct TypewriterEffect {
    delay_ms: u64,
}

impl TypewriterEffect {
    pub fn new(delay_ms: u64) -> Self {
        Self { delay_ms }
    }

    pub fn print(&self, text: &str) {
        for c in text.chars() {
            print!("{}", c);
            let _ = io::stdout().flush();
            std::thread::sleep(Duration::from_millis(self.delay_ms));
        }
    }

    pub fn println(&self, text: &str) {
        self.print(text);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_event_creation() {
        let event = StreamEvent::text("Hello");
        assert_eq!(event.event_type, StreamEventType::Text);
        assert_eq!(event.content, "Hello");
    }

    #[test]
    fn test_create_stream() {
        let (mut writer, mut reader) = create_stream();

        writer.write_text("Hello");
        writer.flush();

        let event = reader.try_recv();
        assert!(event.is_some());
    }

    #[test]
    fn test_stream_complete() {
        let (writer, mut reader) = create_stream();
        writer.complete();

        std::thread::sleep(Duration::from_millis(10));

        while reader.try_recv().is_some() {}

        assert!(reader.is_complete());
    }

    #[test]
    fn test_json_stream_renderer() {
        let event = StreamEvent::text("Test content");
        let json = JsonStreamRenderer::render_event(&event);

        assert!(json.contains("Test content"));
        assert!(json.contains("\"event_type\":\"Text\""));
    }

    #[test]
    fn test_sse_format() {
        let event = StreamEvent::text("Hello world");
        let sse = JsonStreamRenderer::render_sse(&event);

        assert!(sse.starts_with("event: text\n"));
        assert!(sse.contains("Hello world"));
    }
}
