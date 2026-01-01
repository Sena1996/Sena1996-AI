use sena_core::{Result, TextProcessor};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TextRankSummarizer {
    damping: f32,
    iterations: usize,
    min_sentence_length: usize,
}

impl TextRankSummarizer {
    pub fn new() -> Self {
        Self {
            damping: 0.85,
            iterations: 30,
            min_sentence_length: 10,
        }
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping.clamp(0.0, 1.0);
        self
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations.max(1);
        self
    }

    pub fn summarize(&self, text: &str, num_sentences: usize) -> Result<String> {
        let sentences = self.split_sentences(text);

        if sentences.len() <= num_sentences {
            return Ok(text.to_string());
        }

        let similarity_matrix = self.build_similarity_matrix(&sentences);
        let scores = self.compute_scores(&similarity_matrix);

        let mut indexed_scores: Vec<(usize, f32)> = scores.into_iter().enumerate().collect();
        indexed_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut selected_indices: Vec<usize> = indexed_scores
            .into_iter()
            .take(num_sentences)
            .map(|(i, _)| i)
            .collect();

        selected_indices.sort();

        let summary = selected_indices
            .iter()
            .filter_map(|&i| sentences.get(i))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");

        Ok(summary)
    }

    pub fn extract_keywords(&self, text: &str, num_keywords: usize) -> Vec<String> {
        let words = self.tokenize(text);

        if words.len() <= num_keywords {
            return words;
        }

        let cooccurrence = self.build_cooccurrence(&words, 4);
        let scores = self.compute_word_scores(&cooccurrence, &words);

        let mut word_scores: Vec<(String, f32)> = scores.into_iter().collect();
        word_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        word_scores
            .into_iter()
            .take(num_keywords)
            .map(|(w, _)| w)
            .collect()
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            current.push(c);
            if c == '.' || c == '!' || c == '?' {
                let trimmed = current.trim().to_string();
                if trimmed.len() >= self.min_sentence_length {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }

        if !current.trim().is_empty() && current.trim().len() >= self.min_sentence_length {
            sentences.push(current.trim().to_string());
        }

        sentences
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() > 2)
            .filter(|s| !self.is_stopword(s))
            .map(String::from)
            .collect()
    }

    fn is_stopword(&self, word: &str) -> bool {
        const STOPWORDS: &[&str] = &[
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had",
            "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
            "can", "this", "that", "these", "those", "it", "its", "they", "them", "their",
            "we", "our", "you", "your", "he", "she", "him", "her", "his", "not", "no", "yes",
        ];
        STOPWORDS.contains(&word)
    }

    fn build_similarity_matrix(&self, sentences: &[String]) -> Vec<Vec<f32>> {
        let n = sentences.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let sim = self.sentence_similarity(&sentences[i], &sentences[j]);
                matrix[i][j] = sim;
                matrix[j][i] = sim;
            }
        }

        matrix
    }

    fn sentence_similarity(&self, s1: &str, s2: &str) -> f32 {
        let words1: std::collections::HashSet<_> = self.tokenize(s1).into_iter().collect();
        let words2: std::collections::HashSet<_> = self.tokenize(s2).into_iter().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection = words1.intersection(&words2).count() as f32;
        let union = (words1.len() + words2.len()) as f32;

        if union == 0.0 {
            0.0
        } else {
            intersection / (union / 2.0).max(1.0)
        }
    }

    fn compute_scores(&self, matrix: &[Vec<f32>]) -> Vec<f32> {
        let n = matrix.len();
        let mut scores = vec![1.0 / n as f32; n];

        for _ in 0..self.iterations {
            let mut new_scores = vec![0.0; n];

            for i in 0..n {
                let mut sum = 0.0;
                for j in 0..n {
                    if i != j {
                        let row_sum: f32 = matrix[j].iter().sum();
                        if row_sum > 0.0 {
                            sum += matrix[j][i] * scores[j] / row_sum;
                        }
                    }
                }
                new_scores[i] = (1.0 - self.damping) / n as f32 + self.damping * sum;
            }

            scores = new_scores;
        }

        scores
    }

    fn build_cooccurrence(&self, words: &[String], window: usize) -> HashMap<(String, String), f32> {
        let mut cooccurrence = HashMap::new();

        for i in 0..words.len() {
            for j in (i + 1)..=(i + window).min(words.len() - 1) {
                let key = if words[i] < words[j] {
                    (words[i].clone(), words[j].clone())
                } else {
                    (words[j].clone(), words[i].clone())
                };
                *cooccurrence.entry(key).or_insert(0.0) += 1.0;
            }
        }

        cooccurrence
    }

    fn compute_word_scores(
        &self,
        cooccurrence: &HashMap<(String, String), f32>,
        words: &[String],
    ) -> HashMap<String, f32> {
        let unique_words: std::collections::HashSet<_> = words.iter().cloned().collect();
        let n = unique_words.len();

        let mut scores: HashMap<String, f32> = unique_words
            .iter()
            .map(|w| (w.clone(), 1.0 / n as f32))
            .collect();

        for _ in 0..self.iterations {
            let mut new_scores = HashMap::new();

            for word in &unique_words {
                let mut sum = 0.0;

                for (pair, weight) in cooccurrence {
                    let neighbor = if &pair.0 == word {
                        Some(&pair.1)
                    } else if &pair.1 == word {
                        Some(&pair.0)
                    } else {
                        None
                    };

                    if let Some(neighbor) = neighbor {
                        let neighbor_total: f32 = cooccurrence
                            .iter()
                            .filter(|(p, _)| &p.0 == neighbor || &p.1 == neighbor)
                            .map(|(_, w)| w)
                            .sum();

                        if neighbor_total > 0.0 {
                            sum += weight * scores.get(neighbor).unwrap_or(&0.0) / neighbor_total;
                        }
                    }
                }

                new_scores.insert(word.clone(), (1.0 - self.damping) / n as f32 + self.damping * sum);
            }

            scores = new_scores;
        }

        scores
    }
}

impl Default for TextRankSummarizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextProcessor for TextRankSummarizer {
    fn truncate(&self, text: &str, max_tokens: usize) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() <= max_tokens {
            text.to_string()
        } else {
            words[..max_tokens].join(" ")
        }
    }

    fn chunk(&self, text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.len() <= chunk_size {
            return vec![text.to_string()];
        }

        let step = chunk_size.saturating_sub(overlap).max(1);
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < words.len() {
            let end = (start + chunk_size).min(words.len());
            chunks.push(words[start..end].join(" "));
            start += step;
        }

        chunks
    }

    fn normalize(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f32 / 4.0).ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize() {
        let summarizer = TextRankSummarizer::new();
        let text = "This is the first sentence. This is the second sentence. This is the third sentence. This is the fourth sentence.";
        let summary = summarizer.summarize(text, 2).unwrap();
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_extract_keywords() {
        let summarizer = TextRankSummarizer::new();
        let text = "Machine learning is a subset of artificial intelligence. Artificial intelligence enables machines to learn from data.";
        let keywords = summarizer.extract_keywords(text, 3);
        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_chunk() {
        let summarizer = TextRankSummarizer::new();
        let text = "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10";
        let chunks = summarizer.chunk(text, 4, 1);
        assert!(chunks.len() > 1);
    }
}
