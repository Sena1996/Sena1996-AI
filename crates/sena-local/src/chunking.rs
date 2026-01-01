const DEFAULT_CHUNK_SIZE: usize = 512;
const DEFAULT_OVERLAP: usize = 50;
const CHARS_PER_TOKEN: f32 = 4.0;

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub max_tokens: usize,
    pub overlap_tokens: usize,
    pub preserve_sentences: bool,
    pub preserve_paragraphs: bool,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_tokens: DEFAULT_CHUNK_SIZE,
            overlap_tokens: DEFAULT_OVERLAP,
            preserve_sentences: true,
            preserve_paragraphs: false,
        }
    }
}

impl ChunkConfig {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            ..Default::default()
        }
    }

    pub fn with_overlap(mut self, overlap: usize) -> Self {
        self.overlap_tokens = overlap;
        self
    }

    pub fn preserve_sentences(mut self, preserve: bool) -> Self {
        self.preserve_sentences = preserve;
        self
    }

    pub fn preserve_paragraphs(mut self, preserve: bool) -> Self {
        self.preserve_paragraphs = preserve;
        self
    }
}

pub struct TextChunker {
    config: ChunkConfig,
}

impl TextChunker {
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    pub fn with_max_tokens(max_tokens: usize) -> Self {
        Self::new(ChunkConfig::new(max_tokens))
    }

    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() as f32 / CHARS_PER_TOKEN).ceil() as usize
    }

    pub fn needs_chunking(&self, text: &str) -> bool {
        Self::estimate_tokens(text) > self.config.max_tokens
    }

    pub fn chunk(&self, text: &str) -> Vec<TextChunk> {
        if !self.needs_chunking(text) {
            return vec![TextChunk {
                content: text.to_string(),
                index: 0,
                start_char: 0,
                end_char: text.len(),
                token_estimate: Self::estimate_tokens(text),
            }];
        }

        if self.config.preserve_paragraphs {
            self.chunk_by_paragraphs(text)
        } else if self.config.preserve_sentences {
            self.chunk_by_sentences(text)
        } else {
            self.chunk_by_tokens(text)
        }
    }

    fn chunk_by_paragraphs(&self, text: &str) -> Vec<TextChunk> {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut chunk_index = 0;

        for para in paragraphs {
            let para_tokens = Self::estimate_tokens(para);
            let current_tokens = Self::estimate_tokens(&current_chunk);

            if current_tokens + para_tokens > self.config.max_tokens && !current_chunk.is_empty() {
                let end_char = current_start + current_chunk.len();
                chunks.push(TextChunk {
                    content: current_chunk.trim().to_string(),
                    index: chunk_index,
                    start_char: current_start,
                    end_char,
                    token_estimate: Self::estimate_tokens(&current_chunk),
                });
                chunk_index += 1;

                if self.config.overlap_tokens > 0 {
                    let overlap_chars = (self.config.overlap_tokens as f32 * CHARS_PER_TOKEN) as usize;
                    let overlap_start = current_chunk.len().saturating_sub(overlap_chars);
                    current_chunk = current_chunk[overlap_start..].to_string();
                    current_start = end_char - (current_chunk.len());
                } else {
                    current_chunk.clear();
                    current_start = end_char;
                }
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(TextChunk {
                content: current_chunk.trim().to_string(),
                index: chunk_index,
                start_char: current_start,
                end_char: text.len(),
                token_estimate: Self::estimate_tokens(&current_chunk),
            });
        }

        chunks
    }

    fn chunk_by_sentences(&self, text: &str) -> Vec<TextChunk> {
        let sentences = self.split_sentences(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut chunk_index = 0;

        for sentence in sentences {
            let sentence_tokens = Self::estimate_tokens(&sentence);
            let current_tokens = Self::estimate_tokens(&current_chunk);

            if sentence_tokens > self.config.max_tokens {
                if !current_chunk.is_empty() {
                    let end_char = current_start + current_chunk.len();
                    chunks.push(TextChunk {
                        content: current_chunk.trim().to_string(),
                        index: chunk_index,
                        start_char: current_start,
                        end_char,
                        token_estimate: current_tokens,
                    });
                    chunk_index += 1;
                    current_chunk.clear();
                    current_start = end_char;
                }

                let sub_chunks = self.chunk_long_sentence(&sentence, chunk_index, current_start);
                for sub in sub_chunks {
                    current_start = sub.end_char;
                    chunk_index = sub.index + 1;
                    chunks.push(sub);
                }
                continue;
            }

            if current_tokens + sentence_tokens > self.config.max_tokens && !current_chunk.is_empty() {
                let end_char = current_start + current_chunk.len();
                chunks.push(TextChunk {
                    content: current_chunk.trim().to_string(),
                    index: chunk_index,
                    start_char: current_start,
                    end_char,
                    token_estimate: current_tokens,
                });
                chunk_index += 1;

                if self.config.overlap_tokens > 0 {
                    let overlap_chars = (self.config.overlap_tokens as f32 * CHARS_PER_TOKEN) as usize;
                    let overlap_start = current_chunk.len().saturating_sub(overlap_chars);
                    current_chunk = current_chunk[overlap_start..].to_string();
                    current_start = end_char - current_chunk.len();
                } else {
                    current_chunk.clear();
                    current_start = end_char;
                }
            }

            if !current_chunk.is_empty() && !current_chunk.ends_with(' ') {
                current_chunk.push(' ');
            }
            current_chunk.push_str(&sentence);
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(TextChunk {
                content: current_chunk.trim().to_string(),
                index: chunk_index,
                start_char: current_start,
                end_char: text.len(),
                token_estimate: Self::estimate_tokens(&current_chunk),
            });
        }

        chunks
    }

    fn chunk_by_tokens(&self, text: &str) -> Vec<TextChunk> {
        let max_chars = (self.config.max_tokens as f32 * CHARS_PER_TOKEN) as usize;
        let overlap_chars = (self.config.overlap_tokens as f32 * CHARS_PER_TOKEN) as usize;
        let step = max_chars.saturating_sub(overlap_chars).max(1);

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < text.len() {
            let end = (start + max_chars).min(text.len());

            let adjusted_end = if end < text.len() {
                text[start..end]
                    .rfind(|c: char| c.is_whitespace())
                    .map(|pos| start + pos)
                    .unwrap_or(end)
            } else {
                end
            };

            let chunk_text = text[start..adjusted_end].trim();
            if !chunk_text.is_empty() {
                chunks.push(TextChunk {
                    content: chunk_text.to_string(),
                    index,
                    start_char: start,
                    end_char: adjusted_end,
                    token_estimate: Self::estimate_tokens(chunk_text),
                });
                index += 1;
            }

            start = if adjusted_end == end {
                start + step
            } else {
                adjusted_end.saturating_sub(overlap_chars)
            };
        }

        chunks
    }

    fn chunk_long_sentence(&self, sentence: &str, start_index: usize, start_char: usize) -> Vec<TextChunk> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = start_index;
        let mut current_start = start_char;

        for word in words {
            let word_tokens = Self::estimate_tokens(word);
            let current_tokens = Self::estimate_tokens(&current_chunk);

            if current_tokens + word_tokens > self.config.max_tokens && !current_chunk.is_empty() {
                let end_char = current_start + current_chunk.len();
                chunks.push(TextChunk {
                    content: current_chunk.trim().to_string(),
                    index: chunk_index,
                    start_char: current_start,
                    end_char,
                    token_estimate: current_tokens,
                });
                chunk_index += 1;
                current_chunk.clear();
                current_start = end_char;
            }

            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(word);
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(TextChunk {
                content: current_chunk.trim().to_string(),
                index: chunk_index,
                start_char: current_start,
                end_char: start_char + sentence.len(),
                token_estimate: Self::estimate_tokens(&current_chunk),
            });
        }

        chunks
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            current.push(c);

            if c == '.' || c == '!' || c == '?' {
                let next_is_space = chars.get(i + 1).map(|nc| nc.is_whitespace()).unwrap_or(true);
                let next_is_upper = chars.get(i + 2).map(|nc| nc.is_uppercase()).unwrap_or(true);

                if next_is_space && (next_is_upper || i + 1 >= chars.len()) {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        sentences.push(trimmed);
                    }
                    current.clear();
                }
            }

            i += 1;
        }

        if !current.trim().is_empty() {
            sentences.push(current.trim().to_string());
        }

        sentences
    }

    pub fn merge_chunks(chunks: &[TextChunk], separator: &str) -> String {
        chunks.iter().map(|c| c.content.as_str()).collect::<Vec<_>>().join(separator)
    }

    pub fn average_embeddings(embeddings: &[Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }

        let dim = embeddings[0].len();
        let count = embeddings.len() as f32;

        let mut averaged = vec![0.0; dim];
        for embedding in embeddings {
            for (i, val) in embedding.iter().enumerate() {
                averaged[i] += val / count;
            }
        }

        averaged
    }

    pub fn weighted_average_embeddings(embeddings: &[Vec<f32>], weights: &[f32]) -> Vec<f32> {
        if embeddings.is_empty() || embeddings.len() != weights.len() {
            return Vec::new();
        }

        let dim = embeddings[0].len();
        let total_weight: f32 = weights.iter().sum();

        if total_weight == 0.0 {
            return Self::average_embeddings(embeddings);
        }

        let mut weighted = vec![0.0; dim];
        for (embedding, weight) in embeddings.iter().zip(weights.iter()) {
            for (i, val) in embedding.iter().enumerate() {
                weighted[i] += val * weight / total_weight;
            }
        }

        weighted
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new(ChunkConfig::default())
    }
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub content: String,
    pub index: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub token_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_chunking_needed() {
        let chunker = TextChunker::with_max_tokens(512);
        let text = "This is a short text.";
        let chunks = chunker.chunk(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn test_chunking_long_text() {
        let chunker = TextChunker::with_max_tokens(15);
        let text = "One two three. Four five six. Seven eight nine. Ten eleven twelve. Thirteen fourteen fifteen.";
        let chunks = chunker.chunk(text);
        assert!(chunks.len() > 1, "Expected multiple chunks, got {}", chunks.len());

        for chunk in &chunks {
            assert!(chunk.token_estimate <= 25, "Chunk too large: {} tokens", chunk.token_estimate);
        }
    }

    #[test]
    fn test_token_estimation() {
        let text = "Hello world";
        let tokens = TextChunker::estimate_tokens(text);
        assert!(tokens > 0);
        assert!(tokens < 10);
    }

    #[test]
    fn test_average_embeddings() {
        let embeddings = vec![
            vec![1.0, 2.0, 3.0],
            vec![3.0, 4.0, 5.0],
        ];
        let avg = TextChunker::average_embeddings(&embeddings);
        assert_eq!(avg, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_chunk_preservation() {
        let chunker = TextChunker::new(ChunkConfig {
            max_tokens: 100,
            overlap_tokens: 20,
            preserve_sentences: true,
            preserve_paragraphs: false,
        });

        let text = "First sentence here. Second sentence here. Third sentence here.";
        let chunks = chunker.chunk(text);

        for chunk in &chunks {
            assert!(chunk.content.ends_with('.') || chunk.index == chunks.len() - 1);
        }
    }
}
