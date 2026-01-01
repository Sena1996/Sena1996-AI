use futures::Stream;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub chunk_index: u32,
    pub finish_reason: Option<String>,
}

impl StreamChunk {
    pub fn new(delta: impl Into<String>, chunk_index: u32) -> Self {
        Self {
            delta: delta.into(),
            chunk_index,
            finish_reason: None,
        }
    }

    pub fn with_finish_reason(mut self, reason: impl Into<String>) -> Self {
        self.finish_reason = Some(reason.into());
        self
    }

    pub fn is_final(&self) -> bool {
        self.finish_reason.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub event_type: StreamEventType,
    pub data: Option<StreamChunk>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventType {
    Start,
    Delta,
    Stop,
    Error,
}

impl StreamEvent {
    pub fn start() -> Self {
        Self {
            event_type: StreamEventType::Start,
            data: None,
            error: None,
        }
    }

    pub fn delta(chunk: StreamChunk) -> Self {
        Self {
            event_type: StreamEventType::Delta,
            data: Some(chunk),
            error: None,
        }
    }

    pub fn stop() -> Self {
        Self {
            event_type: StreamEventType::Stop,
            data: None,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            event_type: StreamEventType::Error,
            data: None,
            error: Some(message.into()),
        }
    }
}

pub type StreamResult = Result<StreamChunk>;
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;
pub type CompletionStream = BoxStream<StreamResult>;

pin_project! {
    pub struct ChunkCollector<S> {
        #[pin]
        stream: S,
        collected: String,
        chunks: Vec<StreamChunk>,
    }
}

impl<S> ChunkCollector<S>
where
    S: Stream<Item = StreamResult>,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            collected: String::new(),
            chunks: Vec::new(),
        }
    }
}

impl<S> Stream for ChunkCollector<S>
where
    S: Stream<Item = StreamResult>,
{
    type Item = StreamResult;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.stream.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                this.collected.push_str(&chunk.delta);
                this.chunks.push(chunk.clone());
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S> ChunkCollector<S> {
    pub fn collected_content(&self) -> &str {
        &self.collected
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn into_content(self) -> String {
        self.collected
    }
}

#[derive(Debug, Default)]
pub struct StreamBuffer {
    content: String,
    chunk_count: u32,
    is_complete: bool,
}

impl StreamBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &StreamChunk) {
        self.content.push_str(&chunk.delta);
        self.chunk_count += 1;

        if chunk.is_final() {
            self.is_complete = true;
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    pub fn into_content(self) -> String {
        self.content
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.chunk_count = 0;
        self.is_complete = false;
    }
}
