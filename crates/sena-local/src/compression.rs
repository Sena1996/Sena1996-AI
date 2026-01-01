use regex::Regex;
use sena_core::Result;
use std::collections::HashSet;

use crate::textrank::TextRankSummarizer;

#[derive(Debug, Clone)]
pub struct PromptCompressor {
    summarizer: TextRankSummarizer,
    target_ratio: f32,
    preserve_code: bool,
    preserve_urls: bool,
    min_sentence_length: usize,
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub original_length: usize,
    pub compressed_length: usize,
    pub compression_ratio: f32,
    pub content: String,
    pub tokens_saved_estimate: u32,
}

#[derive(Debug, Clone, Default)]
pub struct CompressionConfig {
    pub target_ratio: f32,
    pub preserve_code: bool,
    pub preserve_urls: bool,
    pub preserve_lists: bool,
    pub min_sentence_length: usize,
    pub max_output_tokens: Option<u32>,
}

impl CompressionConfig {
    pub fn aggressive() -> Self {
        Self {
            target_ratio: 0.3,
            preserve_code: true,
            preserve_urls: false,
            preserve_lists: false,
            min_sentence_length: 15,
            max_output_tokens: None,
        }
    }

    pub fn balanced() -> Self {
        Self {
            target_ratio: 0.5,
            preserve_code: true,
            preserve_urls: true,
            preserve_lists: true,
            min_sentence_length: 10,
            max_output_tokens: None,
        }
    }

    pub fn conservative() -> Self {
        Self {
            target_ratio: 0.7,
            preserve_code: true,
            preserve_urls: true,
            preserve_lists: true,
            min_sentence_length: 5,
            max_output_tokens: None,
        }
    }
}

impl Default for PromptCompressor {
    fn default() -> Self {
        Self::new(CompressionConfig::balanced())
    }
}

impl PromptCompressor {
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            summarizer: TextRankSummarizer::new(),
            target_ratio: config.target_ratio,
            preserve_code: config.preserve_code,
            preserve_urls: config.preserve_urls,
            min_sentence_length: config.min_sentence_length,
        }
    }

    pub fn compress(&self, text: &str) -> Result<CompressionResult> {
        let original_length = text.len();

        if original_length < 500 {
            return Ok(CompressionResult {
                original_length,
                compressed_length: original_length,
                compression_ratio: 1.0,
                content: text.to_string(),
                tokens_saved_estimate: 0,
            });
        }

        let (code_blocks, text_without_code) = self.extract_code_blocks(text);
        let cleaned = self.normalize_whitespace(&text_without_code);
        let cleaned = self.remove_redundant_phrases(&cleaned);
        let cleaned = self.deduplicate_sentences(&cleaned);

        let target_sentences = self.calculate_target_sentences(&cleaned);
        let summarized = if target_sentences > 0 {
            self.summarizer
                .summarize(&cleaned, target_sentences)
                .unwrap_or_else(|_| cleaned.clone())
        } else {
            cleaned
        };

        let final_content = if self.preserve_code && !code_blocks.is_empty() {
            self.reinsert_code_blocks(&summarized, &code_blocks)
        } else {
            summarized
        };

        let compressed_length = final_content.len();
        let compression_ratio = compressed_length as f32 / original_length as f32;
        let tokens_saved = ((original_length - compressed_length) / 4) as u32;

        Ok(CompressionResult {
            original_length,
            compressed_length,
            compression_ratio,
            content: final_content,
            tokens_saved_estimate: tokens_saved,
        })
    }

    pub fn compress_to_token_limit(&self, text: &str, max_tokens: u32) -> Result<CompressionResult> {
        let estimated_tokens = (text.len() / 4) as u32;

        if estimated_tokens <= max_tokens {
            return Ok(CompressionResult {
                original_length: text.len(),
                compressed_length: text.len(),
                compression_ratio: 1.0,
                content: text.to_string(),
                tokens_saved_estimate: 0,
            });
        }

        let target_chars = (max_tokens * 4) as usize;
        let mut result = self.compress(text)?;

        if result.compressed_length > target_chars {
            let sentences = self.split_sentences(&result.content);
            let mut truncated = String::new();

            for sentence in sentences {
                if truncated.len() + sentence.len() + 1 > target_chars {
                    break;
                }
                if !truncated.is_empty() {
                    truncated.push(' ');
                }
                truncated.push_str(&sentence);
            }

            result.compressed_length = truncated.len();
            result.compression_ratio = truncated.len() as f32 / result.original_length as f32;
            result.tokens_saved_estimate = ((result.original_length - truncated.len()) / 4) as u32;
            result.content = truncated;
        }

        Ok(result)
    }

    pub fn compress_conversation(&self, messages: &[(String, String)]) -> Result<Vec<(String, String)>> {
        let mut compressed = Vec::with_capacity(messages.len());
        let message_count = messages.len();

        for (idx, (role, content)) in messages.iter().enumerate() {
            let is_recent = idx >= message_count.saturating_sub(3);
            let ratio = if is_recent { 0.8 } else { self.target_ratio };

            let result = if is_recent {
                CompressionResult {
                    original_length: content.len(),
                    compressed_length: content.len(),
                    compression_ratio: 1.0,
                    content: content.clone(),
                    tokens_saved_estimate: 0,
                }
            } else {
                let temp_compressor = PromptCompressor {
                    summarizer: TextRankSummarizer::new(),
                    target_ratio: ratio,
                    preserve_code: self.preserve_code,
                    preserve_urls: self.preserve_urls,
                    min_sentence_length: self.min_sentence_length,
                };
                temp_compressor.compress(content)?
            };

            compressed.push((role.clone(), result.content));
        }

        Ok(compressed)
    }

    fn extract_code_blocks(&self, text: &str) -> (Vec<String>, String) {
        let code_pattern = Regex::new(r"```[\s\S]*?```|`[^`]+`").unwrap();
        let mut blocks = Vec::new();
        let mut text_without_code = text.to_string();
        let mut placeholder_idx = 0;

        for cap in code_pattern.find_iter(text) {
            blocks.push(cap.as_str().to_string());
            text_without_code = text_without_code.replacen(
                cap.as_str(),
                &format!("__CODE_BLOCK_{}__", placeholder_idx),
                1,
            );
            placeholder_idx += 1;
        }

        (blocks, text_without_code)
    }

    fn reinsert_code_blocks(&self, text: &str, blocks: &[String]) -> String {
        let mut result = text.to_string();

        for (idx, block) in blocks.iter().enumerate() {
            let placeholder = format!("__CODE_BLOCK_{}__", idx);
            result = result.replace(&placeholder, block);
        }

        result
    }

    fn normalize_whitespace(&self, text: &str) -> String {
        let ws_pattern = Regex::new(r"\s+").unwrap();
        let result = ws_pattern.replace_all(text, " ");
        result.trim().to_string()
    }

    fn remove_redundant_phrases(&self, text: &str) -> String {
        let redundant_patterns = [
            r"\b(basically|essentially|actually|literally|really|very|quite|rather)\b",
            r"\b(in order to)\b",
            r"\b(the fact that)\b",
            r"\b(it is important to note that)\b",
            r"\b(as a matter of fact)\b",
            r"\b(for all intents and purposes)\b",
            r"\b(at the end of the day)\b",
            r"\b(needless to say)\b",
            r"\b(in my opinion)\b",
            r"\b(I think that)\b",
            r"\b(I believe that)\b",
        ];

        let mut result = text.to_string();
        for pattern in &redundant_patterns {
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                result = re.replace_all(&result, "").to_string();
            }
        }

        self.normalize_whitespace(&result)
    }

    fn deduplicate_sentences(&self, text: &str) -> String {
        let sentences = self.split_sentences(text);
        let mut seen = HashSet::new();
        let mut unique_sentences = Vec::new();

        for sentence in sentences {
            let normalized = sentence.to_lowercase().trim().to_string();
            if normalized.len() >= self.min_sentence_length && !seen.contains(&normalized) {
                seen.insert(normalized);
                unique_sentences.push(sentence);
            }
        }

        unique_sentences.join(" ")
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        let sentence_pattern = Regex::new(r"[.!?]+\s*").unwrap();
        sentence_pattern
            .split(text)
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect()
    }

    fn calculate_target_sentences(&self, text: &str) -> usize {
        let sentences = self.split_sentences(text);
        let total = sentences.len();

        if total < 3 {
            return total;
        }

        let target = (total as f32 * self.target_ratio).ceil() as usize;
        target.max(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_compression() {
        let compressor = PromptCompressor::default();
        let long_text = "This is a test sentence. ".repeat(50);
        let result = compressor.compress(&long_text).unwrap();

        assert!(result.compression_ratio < 1.0);
        assert!(result.tokens_saved_estimate > 0);
    }

    #[test]
    fn test_short_text_no_compression() {
        let compressor = PromptCompressor::default();
        let short_text = "This is short.";
        let result = compressor.compress(short_text).unwrap();

        assert_eq!(result.compression_ratio, 1.0);
        assert_eq!(result.content, short_text);
    }

    #[test]
    fn test_code_preservation() {
        let compressor = PromptCompressor::new(CompressionConfig::balanced());
        let text_with_code = "Here is some explanation. ".repeat(30)
            + "\n```rust\nfn main() {}\n```\n"
            + &"More explanation. ".repeat(30);

        let result = compressor.compress(&text_with_code).unwrap();
        assert!(result.content.contains("```rust\nfn main() {}\n```"));
    }

    #[test]
    fn test_token_limit_compression() {
        let compressor = PromptCompressor::default();
        let long_text = "This is a sentence for testing. ".repeat(100);
        let result = compressor.compress_to_token_limit(&long_text, 100).unwrap();

        let estimated_tokens = (result.compressed_length / 4) as u32;
        assert!(estimated_tokens <= 100);
    }

    #[test]
    fn test_redundant_phrase_removal() {
        let compressor = PromptCompressor::default();
        let text = "Basically, this is essentially a very important test that is actually quite good. ".repeat(20);
        let result = compressor.compress(&text).unwrap();

        assert!(!result.content.to_lowercase().contains("basically"));
        assert!(!result.content.to_lowercase().contains("essentially"));
    }
}
