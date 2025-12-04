use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptedOutput {
    pub content: String,
    pub original: String,
    pub was_blocked: bool,
    pub hallucination_score: f64,
    pub warnings: Vec<String>,
}

impl InterceptedOutput {
    pub fn passthrough(content: String) -> Self {
        Self {
            content: content.clone(),
            original: content,
            was_blocked: false,
            hallucination_score: 0.0,
            warnings: Vec::new(),
        }
    }

    pub fn blocked(original: String, reason: String) -> Self {
        Self {
            content: format!("[BLOCKED: {}]", reason),
            original,
            was_blocked: true,
            hallucination_score: 1.0,
            warnings: vec![reason],
        }
    }

    pub fn with_warning(content: String, warning: String, score: f64) -> Self {
        Self {
            content: content.clone(),
            original: content,
            was_blocked: false,
            hallucination_score: score,
            warnings: vec![warning],
        }
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn is_safe(&self) -> bool {
        !self.was_blocked && self.hallucination_score < 0.5
    }
}

pub struct StreamInterceptor {
    buffer: String,
    line_processor: Box<dyn Fn(&str) -> InterceptedOutput + Send + Sync>,
}

impl StreamInterceptor {
    pub fn new<F>(processor: F) -> Self
    where
        F: Fn(&str) -> InterceptedOutput + Send + Sync + 'static,
    {
        Self {
            buffer: String::new(),
            line_processor: Box::new(processor),
        }
    }

    pub fn passthrough() -> Self {
        Self::new(|line| InterceptedOutput::passthrough(line.to_string()))
    }

    pub fn process_chunk(&mut self, chunk: &str) -> Vec<InterceptedOutput> {
        self.buffer.push_str(chunk);

        let mut results = Vec::new();

        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].to_string();
            self.buffer = self.buffer[newline_pos + 1..].to_string();

            if !line.is_empty() {
                results.push((self.line_processor)(&line));
            }
        }

        results
    }

    pub fn flush(&mut self) -> Option<InterceptedOutput> {
        if self.buffer.is_empty() {
            return None;
        }

        let remaining = std::mem::take(&mut self.buffer);
        Some((self.line_processor)(&remaining))
    }

    pub fn process_complete(&mut self, content: &str) -> InterceptedOutput {
        (self.line_processor)(content)
    }
}

impl Default for StreamInterceptor {
    fn default() -> Self {
        Self::passthrough()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        let output = InterceptedOutput::passthrough("test content".to_string());
        assert!(!output.was_blocked);
        assert!(output.is_safe());
        assert!(!output.has_warnings());
    }

    #[test]
    fn test_blocked() {
        let output =
            InterceptedOutput::blocked("bad content".to_string(), "Policy violation".to_string());
        assert!(output.was_blocked);
        assert!(!output.is_safe());
        assert!(output.has_warnings());
    }

    #[test]
    fn test_stream_interceptor_chunks() {
        let mut interceptor = StreamInterceptor::passthrough();

        let results = interceptor.process_chunk("line1\nline2\npartial");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].content, "line1");
        assert_eq!(results[1].content, "line2");

        let final_result = interceptor.flush();
        assert!(final_result.is_some());
        assert_eq!(final_result.unwrap().content, "partial");
    }

    #[test]
    fn test_stream_interceptor_custom_processor() {
        let mut interceptor = StreamInterceptor::new(|line| {
            if line.contains("bad") {
                InterceptedOutput::blocked(line.to_string(), "Contains bad word".to_string())
            } else {
                InterceptedOutput::passthrough(line.to_string())
            }
        });

        let results = interceptor.process_chunk("good line\nbad line\n");
        assert_eq!(results.len(), 2);
        assert!(!results[0].was_blocked);
        assert!(results[1].was_blocked);
    }
}
