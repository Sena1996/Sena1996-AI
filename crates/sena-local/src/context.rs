use sena_core::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::compression::{CompressionConfig, PromptCompressor};
use crate::textrank::TextRankSummarizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_tokens: u32,
    pub reserved_for_response: u32,
    pub system_prompt_budget: u32,
    pub sliding_window_size: usize,
    pub preserve_recent_count: usize,
    pub compress_threshold: f32,
    pub summarize_old_messages: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 128_000,
            reserved_for_response: 4_096,
            system_prompt_budget: 8_000,
            sliding_window_size: 50,
            preserve_recent_count: 5,
            compress_threshold: 0.7,
            summarize_old_messages: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextWindow {
    config: ContextConfig,
    messages: VecDeque<ContextMessage>,
    system_prompt: Option<String>,
    total_tokens: u32,
    compressor: PromptCompressor,
    summarizer: TextRankSummarizer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
    pub token_count: u32,
    pub is_compressed: bool,
    pub original_tokens: Option<u32>,
    pub importance: f32,
}

impl ContextMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        let content = content.into();
        let token_count = estimate_tokens(&content);

        Self {
            role: role.into(),
            content,
            token_count,
            is_compressed: false,
            original_tokens: None,
            importance: 1.0,
        }
    }

    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance;
        self
    }
}

fn estimate_tokens(text: &str) -> u32 {
    ((text.len() as f32) / 3.5).ceil() as u32
}

impl ContextWindow {
    pub fn new(config: ContextConfig) -> Self {
        Self {
            config,
            messages: VecDeque::new(),
            system_prompt: None,
            total_tokens: 0,
            compressor: PromptCompressor::new(CompressionConfig::balanced()),
            summarizer: TextRankSummarizer::new(),
        }
    }

    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        let prompt = prompt.into();
        let tokens = estimate_tokens(&prompt);

        if tokens > self.config.system_prompt_budget {
            if let Ok(compressed) = self.compressor.compress_to_token_limit(&prompt, self.config.system_prompt_budget) {
                self.system_prompt = Some(compressed.content);
            } else {
                self.system_prompt = Some(prompt);
            }
        } else {
            self.system_prompt = Some(prompt);
        }

        self.recalculate_tokens();
    }

    pub fn add_message(&mut self, message: ContextMessage) {
        self.total_tokens += message.token_count;
        self.messages.push_back(message);

        if self.messages.len() > self.config.sliding_window_size {
            self.compact();
        }

        if self.is_over_budget() {
            self.prune();
        }
    }

    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(ContextMessage::new("user", content));
    }

    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(ContextMessage::new("assistant", content));
    }

    fn available_tokens(&self) -> u32 {
        let system_tokens = self.system_prompt.as_ref().map(|p| estimate_tokens(p)).unwrap_or(0);
        let budget = self.config.max_tokens
            .saturating_sub(self.config.reserved_for_response)
            .saturating_sub(system_tokens);
        budget
    }

    fn is_over_budget(&self) -> bool {
        self.total_tokens > self.available_tokens()
    }

    fn compact(&mut self) {
        let preserve_count = self.config.preserve_recent_count;
        let window_size = self.config.sliding_window_size;

        if self.messages.len() <= window_size {
            return;
        }

        let remove_count = self.messages.len() - window_size;

        if !self.config.summarize_old_messages {
            for _ in 0..remove_count {
                if let Some(removed) = self.messages.pop_front() {
                    self.total_tokens = self.total_tokens.saturating_sub(removed.token_count);
                }
            }
            return;
        }

        if self.messages.len() <= preserve_count * 2 {
            return;
        }

        let old_messages: Vec<_> = self.messages
            .drain(..self.messages.len() - preserve_count)
            .collect();

        if old_messages.is_empty() {
            return;
        }

        let combined_content: String = old_messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let original_tokens: u32 = old_messages.iter().map(|m| m.token_count).sum();

        if let Ok(summary) = self.summarizer.summarize(&combined_content, 3) {
            let summary_message = ContextMessage {
                role: "system".to_string(),
                content: format!("[Previous conversation summary: {}]", summary),
                token_count: estimate_tokens(&summary),
                is_compressed: true,
                original_tokens: Some(original_tokens),
                importance: 0.5,
            };
            self.messages.push_front(summary_message);
        }

        self.recalculate_tokens();
    }

    fn prune(&mut self) {
        while self.is_over_budget() && self.messages.len() > self.config.preserve_recent_count {
            let mut lowest_importance_idx = 0;
            let mut lowest_importance = f32::MAX;

            let check_range = self.messages.len().saturating_sub(self.config.preserve_recent_count);
            for (idx, msg) in self.messages.iter().enumerate().take(check_range) {
                let age_factor = 1.0 - (idx as f32 / self.messages.len() as f32);
                let effective_importance = msg.importance * age_factor;

                if effective_importance < lowest_importance {
                    lowest_importance = effective_importance;
                    lowest_importance_idx = idx;
                }
            }

            if lowest_importance_idx < self.messages.len() {
                if let Some(removed) = self.messages.remove(lowest_importance_idx) {
                    self.total_tokens = self.total_tokens.saturating_sub(removed.token_count);
                }
            } else {
                break;
            }
        }

        if self.is_over_budget() {
            self.compress_messages();
        }
    }

    fn compress_messages(&mut self) {
        let preserve_count = self.config.preserve_recent_count;
        let compress_range = self.messages.len().saturating_sub(preserve_count);

        for idx in 0..compress_range {
            if !self.is_over_budget() {
                break;
            }

            if let Some(msg) = self.messages.get_mut(idx) {
                if msg.is_compressed {
                    continue;
                }

                if let Ok(compressed) = self.compressor.compress(&msg.content) {
                    if compressed.compression_ratio < self.config.compress_threshold {
                        let original_tokens = msg.token_count;
                        msg.content = compressed.content;
                        msg.token_count = estimate_tokens(&msg.content);
                        msg.is_compressed = true;
                        msg.original_tokens = Some(original_tokens);

                        self.total_tokens = self.total_tokens
                            .saturating_sub(original_tokens)
                            + msg.token_count;
                    }
                }
            }
        }
    }

    fn recalculate_tokens(&mut self) {
        self.total_tokens = self.messages.iter().map(|m| m.token_count).sum();
    }

    pub fn to_messages(&self) -> Vec<Message> {
        let mut result = Vec::with_capacity(self.messages.len() + 1);

        if let Some(system) = &self.system_prompt {
            result.push(Message::system(system));
        }

        for msg in &self.messages {
            let message = match msg.role.as_str() {
                "user" => Message::user(&msg.content),
                "assistant" => Message::assistant(&msg.content),
                "system" => Message::system(&msg.content),
                _ => Message::user(&msg.content),
            };
            result.push(message);
        }

        result
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn total_tokens(&self) -> u32 {
        self.total_tokens
    }

    pub fn remaining_tokens(&self) -> u32 {
        self.available_tokens().saturating_sub(self.total_tokens)
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_tokens = 0;
    }

    pub fn get_messages(&self) -> &VecDeque<ContextMessage> {
        &self.messages
    }

    pub fn messages(&self) -> Vec<ContextMessage> {
        self.messages.iter().cloned().collect()
    }

    pub fn stats(&self) -> ContextStats {
        let compressed_count = self.messages.iter().filter(|m| m.is_compressed).count();
        let original_tokens: u32 = self.messages
            .iter()
            .filter_map(|m| m.original_tokens)
            .sum();

        ContextStats {
            message_count: self.messages.len(),
            total_tokens: self.total_tokens,
            available_tokens: self.available_tokens(),
            compressed_messages: compressed_count,
            tokens_saved: original_tokens.saturating_sub(self.total_tokens),
            utilization: self.total_tokens as f32 / self.available_tokens() as f32,
        }
    }
}

impl Default for ContextWindow {
    fn default() -> Self {
        Self::new(ContextConfig::default())
    }
}

#[derive(Debug, Clone)]
pub struct ContextStats {
    pub message_count: usize,
    pub total_tokens: u32,
    pub available_tokens: u32,
    pub compressed_messages: usize,
    pub tokens_saved: u32,
    pub utilization: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_window_basic() {
        let mut window = ContextWindow::default();

        window.add_user_message("Hello, how are you?");
        window.add_assistant_message("I'm doing well, thank you!");

        assert_eq!(window.message_count(), 2);
        assert!(window.total_tokens() > 0);
    }

    #[test]
    fn test_token_estimation() {
        let tokens = estimate_tokens("Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 10);
    }

    #[test]
    fn test_system_prompt() {
        let mut window = ContextWindow::default();
        window.set_system_prompt("You are a helpful assistant.");

        let messages = window.to_messages();
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_sliding_window() {
        let config = ContextConfig {
            sliding_window_size: 10,
            preserve_recent_count: 3,
            summarize_old_messages: false,
            ..Default::default()
        };

        let mut window = ContextWindow::new(config);

        for i in 0..15 {
            window.add_user_message(format!("Message {}", i));
        }

        assert!(window.message_count() <= 12);
    }

    #[test]
    fn test_context_stats() {
        let mut window = ContextWindow::default();
        window.add_user_message("Test message");

        let stats = window.stats();
        assert_eq!(stats.message_count, 1);
        assert!(stats.utilization < 1.0);
    }
}
