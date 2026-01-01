use bytes::Bytes;
use futures::stream::Stream;
use sena_core::{CompletionStream, Error, Result, StreamChunk};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct SseParser {
    buffer: String,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn parse_line(&mut self, line: &str) -> Option<String> {
        if line.starts_with("data: ") {
            let data = line.strip_prefix("data: ").unwrap();
            if data != "[DONE]" {
                return Some(data.to_string());
            }
        }
        None
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<String> {
        let text = String::from_utf8_lossy(chunk);
        self.buffer.push_str(&text);

        let mut events = Vec::new();
        while let Some(idx) = self.buffer.find("\n\n") {
            let event_block = self.buffer[..idx].to_string();
            self.buffer = self.buffer[idx + 2..].to_string();

            for line in event_block.lines() {
                if let Some(data) = self.parse_line(line) {
                    events.push(data);
                }
            }
        }

        events
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AnthropicStreamAdapter<S> {
    inner: S,
    parser: SseParser,
    chunk_index: u32,
    pending_chunks: Vec<StreamChunk>,
    finished: bool,
}

impl<S> AnthropicStreamAdapter<S> {
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            parser: SseParser::new(),
            chunk_index: 0,
            pending_chunks: Vec::new(),
            finished: false,
        }
    }
}

impl<S> Stream for AnthropicStreamAdapter<S>
where
    S: Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<StreamChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(chunk) = self.pending_chunks.pop() {
            return Poll::Ready(Some(Ok(chunk)));
        }

        if self.finished {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let events = self.parser.feed(&bytes);

                for event_data in events {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&event_data) {
                        let event_type = event.get("type").and_then(|v| v.as_str());

                        match event_type {
                            Some("content_block_delta") => {
                                if let Some(delta) = event.get("delta") {
                                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                        let chunk = StreamChunk::new(text, self.chunk_index);
                                        self.chunk_index += 1;
                                        self.pending_chunks.push(chunk);
                                    }
                                }
                            }
                            Some("message_stop") => {
                                self.finished = true;
                                let final_chunk = StreamChunk::new("", self.chunk_index)
                                    .with_finish_reason("stop");
                                self.pending_chunks.push(final_chunk);
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(chunk) = self.pending_chunks.pop() {
                    Poll::Ready(Some(Ok(chunk)))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(Error::network(format!("stream error: {}", e)))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct OpenAIStreamAdapter<S> {
    inner: S,
    parser: SseParser,
    chunk_index: u32,
    pending_chunks: Vec<StreamChunk>,
    finished: bool,
}

impl<S> OpenAIStreamAdapter<S> {
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            parser: SseParser::new(),
            chunk_index: 0,
            pending_chunks: Vec::new(),
            finished: false,
        }
    }
}

impl<S> Stream for OpenAIStreamAdapter<S>
where
    S: Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<StreamChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(chunk) = self.pending_chunks.pop() {
            return Poll::Ready(Some(Ok(chunk)));
        }

        if self.finished {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let events = self.parser.feed(&bytes);

                for event_data in events {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&event_data) {
                        if let Some(choices) = event.get("choices").and_then(|v| v.as_array()) {
                            for choice in choices {
                                let finish_reason =
                                    choice.get("finish_reason").and_then(|v| v.as_str());

                                if let Some(delta) = choice.get("delta") {
                                    if let Some(content) =
                                        delta.get("content").and_then(|v| v.as_str())
                                    {
                                        let mut chunk = StreamChunk::new(content, self.chunk_index);
                                        self.chunk_index += 1;

                                        if let Some(reason) = finish_reason {
                                            chunk = chunk.with_finish_reason(reason);
                                            self.finished = true;
                                        }

                                        self.pending_chunks.push(chunk);
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(chunk) = self.pending_chunks.pop() {
                    Poll::Ready(Some(Ok(chunk)))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(Error::network(format!("stream error: {}", e)))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub fn create_anthropic_stream<S>(stream: S) -> CompletionStream
where
    S: Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send + Unpin + 'static,
{
    Box::pin(AnthropicStreamAdapter::new(stream))
}

pub fn create_openai_stream<S>(stream: S) -> CompletionStream
where
    S: Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send + Unpin + 'static,
{
    Box::pin(OpenAIStreamAdapter::new(stream))
}
