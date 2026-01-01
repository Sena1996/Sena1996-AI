use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerType {
    Cl100k,
    P50kBase,
    O200kBase,
    Simple,
}

impl Default for TokenizerType {
    fn default() -> Self {
        Self::Cl100k
    }
}

pub struct Tokenizer {
    tokenizer_type: TokenizerType,
    _vocab: HashMap<String, u32>,
    special_tokens: HashMap<String, u32>,
    common_tokens: HashSet<&'static str>,
    subword_patterns: Vec<&'static str>,
}

impl Tokenizer {
    pub fn new(tokenizer_type: TokenizerType) -> Self {
        let mut tokenizer = Self {
            tokenizer_type,
            _vocab: HashMap::new(),
            special_tokens: HashMap::new(),
            common_tokens: Self::build_common_tokens(),
            subword_patterns: Self::build_subword_patterns(),
        };
        tokenizer.init_special_tokens();
        tokenizer
    }

    fn build_common_tokens() -> HashSet<&'static str> {
        [
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "I",
            "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
            "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
            "or", "an", "will", "my", "one", "all", "would", "there", "their", "what",
            "so", "up", "out", "if", "about", "who", "get", "which", "go", "me",
            "when", "make", "can", "like", "time", "no", "just", "him", "know", "take",
            "people", "into", "year", "your", "good", "some", "could", "them", "see", "other",
            "than", "then", "now", "look", "only", "come", "its", "over", "think", "also",
            "back", "after", "use", "two", "how", "our", "work", "first", "well", "way",
            "even", "new", "want", "because", "any", "these", "give", "day", "most", "us",
            "is", "are", "was", "were", "been", "being", "has", "had", "does", "did",
            "fn", "let", "mut", "pub", "impl", "struct", "enum", "trait", "mod", "use",
            "const", "static", "async", "await", "match", "self", "Self", "return", "where",
            "if", "else", "while", "for", "loop", "break", "continue", "move", "ref", "type",
            "function", "class", "import", "export", "default", "interface", "extends", "implements",
            "public", "private", "protected", "abstract", "final", "static", "void", "null",
            "true", "false", "undefined", "console", "log", "error", "warn", "print", "println",
            "def", "lambda", "pass", "raise", "except", "try", "finally", "with", "as", "from",
            "None", "True", "False", "yield", "global", "nonlocal", "assert", "del", "exec",
        ].iter().copied().collect()
    }

    fn build_subword_patterns() -> Vec<&'static str> {
        vec![
            "ing", "tion", "ness", "ment", "able", "ible", "ous", "ive", "ful", "less",
            "ly", "er", "est", "ed", "es", "en", "al", "ity", "ty", "ry",
            "ure", "ence", "ance", "ant", "ent", "ism", "ist", "ize", "ise", "ify",
            "ation", "ition", "ution", "sion", "ology", "ography", "graphy", "metry",
            "pre", "post", "un", "re", "dis", "mis", "non", "anti", "semi", "auto",
            "super", "sub", "over", "under", "out", "up", "down", "multi", "poly", "mono",
        ]
    }

    fn init_special_tokens(&mut self) {
        match self.tokenizer_type {
            TokenizerType::Cl100k | TokenizerType::O200kBase => {
                self.special_tokens.insert("<|endoftext|>".to_string(), 100257);
                self.special_tokens.insert("<|fim_prefix|>".to_string(), 100258);
                self.special_tokens.insert("<|fim_middle|>".to_string(), 100259);
                self.special_tokens.insert("<|fim_suffix|>".to_string(), 100260);
                self.special_tokens.insert("<|endofprompt|>".to_string(), 100276);
            }
            TokenizerType::P50kBase => {
                self.special_tokens.insert("<|endoftext|>".to_string(), 50256);
            }
            TokenizerType::Simple => {}
        }
    }

    pub fn count_tokens(&self, text: &str) -> u32 {
        match self.tokenizer_type {
            TokenizerType::Cl100k => self.count_cl100k(text),
            TokenizerType::P50kBase => self.count_p50k(text),
            TokenizerType::O200kBase => self.count_o200k(text),
            TokenizerType::Simple => self.count_simple(text),
        }
    }

    fn count_simple(&self, text: &str) -> u32 {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();

        let estimate = ((char_count as f64 / 4.0) + (word_count as f64 * 0.75)) / 2.0;
        estimate.ceil() as u32
    }

    fn count_cl100k(&self, text: &str) -> u32 {
        let mut tokens = 0u32;
        let bytes = text.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            let b = bytes[i];

            if b.is_ascii_alphanumeric() || b == b'_' {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = &bytes[start..i];
                tokens += self.count_word_tokens_cl100k(word);
            } else if b == b' ' {
                let start = i;
                while i < bytes.len() && bytes[i] == b' ' {
                    i += 1;
                }
                let space_count = i - start;
                tokens += ((space_count + 3) / 4) as u32;
            } else if b == b'\n' || b == b'\t' || b == b'\r' {
                tokens += 1;
                i += 1;
            } else if b < 128 {
                tokens += 1;
                i += 1;
            } else {
                let char_start = i;
                i += 1;
                while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
                    i += 1;
                }
                let char_bytes = i - char_start;
                tokens += match char_bytes {
                    1 => 1,
                    2 => 1,
                    3 => 2,
                    4 => 2,
                    _ => ((char_bytes + 1) / 2) as u32,
                };
            }
        }

        tokens.max(1)
    }

    fn count_word_tokens_cl100k(&self, word: &[u8]) -> u32 {
        let len = word.len();

        if len == 0 {
            return 0;
        }
        if len == 1 {
            return 1;
        }

        let word_str = std::str::from_utf8(word).unwrap_or("");
        let word_lower = word_str.to_lowercase();

        if self.common_tokens.contains(word_str) || self.common_tokens.contains(word_lower.as_str()) {
            return 1;
        }

        let is_camel_case = word.iter().skip(1).any(|b| b.is_ascii_uppercase());
        let is_snake_case = word.contains(&b'_');
        let has_numbers = word.iter().any(|b| b.is_ascii_digit());

        if is_camel_case {
            let mut tokens = 0u32;
            let mut current_len = 0usize;
            for (i, b) in word.iter().enumerate() {
                if i > 0 && b.is_ascii_uppercase() {
                    tokens += self.estimate_subword_tokens(current_len);
                    current_len = 1;
                } else {
                    current_len += 1;
                }
            }
            tokens += self.estimate_subword_tokens(current_len);
            return tokens;
        }

        if is_snake_case {
            return word_str.split('_').map(|part| {
                if part.is_empty() { 0 } else { self.estimate_subword_tokens(part.len()) }
            }).sum();
        }

        if has_numbers {
            let mut tokens = 0u32;
            let mut digit_run = 0usize;
            let mut alpha_run = 0usize;

            for b in word {
                if b.is_ascii_digit() {
                    if alpha_run > 0 {
                        tokens += self.estimate_subword_tokens(alpha_run);
                        alpha_run = 0;
                    }
                    digit_run += 1;
                } else {
                    if digit_run > 0 {
                        tokens += ((digit_run + 2) / 3) as u32;
                        digit_run = 0;
                    }
                    alpha_run += 1;
                }
            }

            if digit_run > 0 {
                tokens += ((digit_run + 2) / 3) as u32;
            }
            if alpha_run > 0 {
                tokens += self.estimate_subword_tokens(alpha_run);
            }
            return tokens;
        }

        for suffix in &self.subword_patterns {
            if word_lower.ends_with(suffix) && word_lower.len() > suffix.len() + 2 {
                let prefix_len = word_lower.len() - suffix.len();
                return self.estimate_subword_tokens(prefix_len) + 1;
            }
        }

        self.estimate_subword_tokens(len)
    }

    fn estimate_subword_tokens(&self, len: usize) -> u32 {
        match len {
            0 => 0,
            1..=4 => 1,
            5..=8 => 2,
            9..=12 => 3,
            _ => ((len as f32 / 4.0).ceil()) as u32,
        }
    }

    fn count_p50k(&self, text: &str) -> u32 {
        let base = self.count_cl100k(text);
        ((base as f32) * 1.1) as u32
    }

    fn count_o200k(&self, text: &str) -> u32 {
        let base = self.count_cl100k(text);
        ((base as f32) * 0.85) as u32
    }

    pub fn count_messages(&self, messages: &[(String, String)]) -> u32 {
        let mut total = 0u32;

        for (role, content) in messages {
            total += 4;
            total += self.count_tokens(role);
            total += self.count_tokens(content);
        }

        total += 3;
        total
    }

    pub fn fits_context(&self, text: &str, max_tokens: u32) -> bool {
        self.count_tokens(text) <= max_tokens
    }

    pub fn truncate_to_tokens(&self, text: &str, max_tokens: u32) -> String {
        if self.fits_context(text, max_tokens) {
            return text.to_string();
        }

        let chars: Vec<char> = text.chars().collect();
        let estimated_chars = (max_tokens as f32 * 4.0) as usize;

        if estimated_chars >= chars.len() {
            return text.to_string();
        }

        let mut end = estimated_chars.min(chars.len());

        while end > 0 && !chars[end - 1].is_whitespace() {
            end -= 1;
        }

        if end == 0 {
            end = estimated_chars.min(chars.len());
        }

        chars[..end].iter().collect()
    }

    pub fn split_by_tokens(&self, text: &str, chunk_size: u32) -> Vec<String> {
        let mut chunks = Vec::new();
        let sentences: Vec<&str> = text
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut current_chunk = String::new();
        let mut current_tokens = 0u32;

        for sentence in sentences {
            let sentence_text = format!("{}. ", sentence.trim());
            let sentence_tokens = self.count_tokens(&sentence_text);

            if current_tokens + sentence_tokens > chunk_size && !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
                current_tokens = 0;
            }

            current_chunk.push_str(&sentence_text);
            current_tokens += sentence_tokens;
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new(TokenizerType::Simple)
    }
}

pub struct TokenBudget {
    max_tokens: u32,
    used_tokens: u32,
    reserved_tokens: u32,
    tokenizer: Tokenizer,
}

impl TokenBudget {
    pub fn new(max_tokens: u32) -> Self {
        Self {
            max_tokens,
            used_tokens: 0,
            reserved_tokens: 0,
            tokenizer: Tokenizer::default(),
        }
    }

    pub fn with_reserved(mut self, reserved: u32) -> Self {
        self.reserved_tokens = reserved;
        self
    }

    pub fn with_tokenizer(mut self, tokenizer: Tokenizer) -> Self {
        self.tokenizer = tokenizer;
        self
    }

    pub fn available(&self) -> u32 {
        self.max_tokens
            .saturating_sub(self.used_tokens)
            .saturating_sub(self.reserved_tokens)
    }

    pub fn can_fit(&self, text: &str) -> bool {
        self.tokenizer.count_tokens(text) <= self.available()
    }

    pub fn try_allocate(&mut self, text: &str) -> Option<u32> {
        let tokens = self.tokenizer.count_tokens(text);
        if tokens <= self.available() {
            self.used_tokens += tokens;
            Some(tokens)
        } else {
            None
        }
    }

    pub fn allocate(&mut self, tokens: u32) -> bool {
        if tokens <= self.available() {
            self.used_tokens += tokens;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, tokens: u32) {
        self.used_tokens = self.used_tokens.saturating_sub(tokens);
    }

    pub fn utilization(&self) -> f32 {
        if self.max_tokens == 0 {
            return 0.0;
        }
        self.used_tokens as f32 / self.max_tokens as f32
    }

    pub fn remaining_percent(&self) -> f32 {
        1.0 - self.utilization()
    }

    pub fn reset(&mut self) {
        self.used_tokens = 0;
    }
}

pub fn count_tokens(text: &str, model: &str) -> u32 {
    let tokenizer_type = match model {
        m if m.contains("gpt-4o") => TokenizerType::O200kBase,
        m if m.contains("gpt-4") || m.contains("gpt-3.5") => TokenizerType::Cl100k,
        m if m.contains("claude") => TokenizerType::Cl100k,
        m if m.contains("text-davinci") => TokenizerType::P50kBase,
        _ => TokenizerType::Simple,
    };

    Tokenizer::new(tokenizer_type).count_tokens(text)
}

pub fn estimate_tokens(text: &str) -> u32 {
    Tokenizer::default().count_tokens(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenizer() {
        let tokenizer = Tokenizer::new(TokenizerType::Simple);

        let count = tokenizer.count_tokens("Hello, world!");
        assert!(count > 0 && count < 10);

        let long_text = "This is a longer piece of text with multiple words.";
        let long_count = tokenizer.count_tokens(long_text);
        assert!(long_count > count);
    }

    #[test]
    fn test_cl100k_tokenizer() {
        let tokenizer = Tokenizer::new(TokenizerType::Cl100k);

        let count = tokenizer.count_tokens("Hello, world!");
        assert!(count > 0);

        let code = "fn main() { println!(\"Hello\"); }";
        let code_count = tokenizer.count_tokens(code);
        assert!(code_count > 0);
    }

    #[test]
    fn test_unicode_tokens() {
        let tokenizer = Tokenizer::new(TokenizerType::Cl100k);

        let japanese = "こんにちは世界";
        let count = tokenizer.count_tokens(japanese);
        assert!(count > 0);

        let emoji = "Hello 👋 World 🌍";
        let emoji_count = tokenizer.count_tokens(emoji);
        assert!(emoji_count > 0);
    }

    #[test]
    fn test_message_counting() {
        let tokenizer = Tokenizer::default();

        let messages = vec![
            ("user".to_string(), "Hello!".to_string()),
            ("assistant".to_string(), "Hi there!".to_string()),
        ];

        let count = tokenizer.count_messages(&messages);
        assert!(count > 0);
    }

    #[test]
    fn test_truncation() {
        let tokenizer = Tokenizer::default();
        let text = "This is a test sentence. Another sentence here. And one more.";

        let truncated = tokenizer.truncate_to_tokens(text, 5);
        assert!(truncated.len() < text.len());
    }

    #[test]
    fn test_token_budget() {
        let mut budget = TokenBudget::new(1000).with_reserved(100);

        assert_eq!(budget.available(), 900);
        assert!(budget.can_fit("Hello, world!"));

        let allocated = budget.try_allocate("Test message");
        assert!(allocated.is_some());
        assert!(budget.available() < 900);
    }

    #[test]
    fn test_split_by_tokens() {
        let tokenizer = Tokenizer::default();
        let text = "First sentence. Second sentence. Third sentence. Fourth sentence.";

        let chunks = tokenizer.split_by_tokens(text, 10);
        assert!(!chunks.is_empty());
    }
}
