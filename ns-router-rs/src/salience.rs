use salience_engine::mesolimbic::{MesolimbicSystem, SalienceResult, TokenFeatures as SETokenFeatures};
use salience_engine::role_inference::RoleInferenceResult;
use std::collections::HashMap;

/// Wrapper around the salience engine's functionality
pub struct SalienceAnalyzer {
    mesolimbic: MesolimbicSystem,
    theory_key: String,
}

impl SalienceAnalyzer {
    /// Create a new SalienceAnalyzer with default configuration
    pub fn new() -> Self {
        Self {
            mesolimbic: MesolimbicSystem::default(),
            theory_key: "default".to_string(),
        }
    }
    
    /// Analyze text and return salience scores for each token
    pub fn analyze_text(&self, text: &str) -> Vec<SalienceResult> {
        // Convert text to token features
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let token_features = self.tokenize_text(&tokens);
        
        // Compute salience using the mesolimbic system
        self.mesolimbic.compute_salience(token_features, &self.theory_key)
    }
    
    /// Convert raw tokens to the format expected by the salience engine
    fn tokenize_text(&self, tokens: &[&str]) -> Vec<SETokenFeatures> {
        tokens.iter()
            .enumerate()
            .map(|(i, &token)| {
                // Simple tokenization - in a real implementation, you'd want to use a proper tokenizer
                // and extract actual features like frequency, sentiment, etc.
                SETokenFeatures {
                    token_id: i as u32,
                    frequency: 0.5,  // Placeholder
                    sentiment_score: 0.0,  // Neutral sentiment by default
                    context_relevance: 0.5,  // Medium relevance by default
                    role: "".to_string(),  // Will be filled by the role inference
                }
            })
            .collect()
    }
    
    /// Extract the most salient phrases from the text
    pub fn extract_salient_phrases(&self, text: &str, threshold: f32) -> Vec<String> {
        let results = self.analyze_text(text);
        
        // Group tokens by their salience scores
        let mut phrases = Vec::new();
        let mut current_phrase = Vec::new();
        
        for result in results {
            if result.salience_score >= threshold {
                current_phrase.push(result.token_id);
            } else if !current_phrase.is_empty() {
                // Convert token IDs back to words
                let phrase = current_phrase.iter()
                    .filter_map(|&id| text.split_whitespace().nth(id as usize))
                    .collect::<Vec<_>>()
                    .join(" ");
                phrases.push(phrase);
                current_phrase.clear();
            }
        }
        
        // Add the last phrase if any
        if !current_phrase.is_empty() {
            let phrase = current_phrase.iter()
                .filter_map(|&id| text.split_whitespace().nth(id as usize))
                .collect::<Vec<_>>()
                .join(" ");
            phrases.push(phrase);
        }
        
        phrases
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_salience_analysis() {
        let analyzer = SalienceAnalyzer::new();
        let text = "The quick brown fox jumps over the lazy dog";
        
        let results = analyzer.analyze_text(text);
        assert!(!results.is_empty(), "Should return results for non-empty text");
        
        // Check that we have one result per token
        let token_count = text.split_whitespace().count();
        assert_eq!(results.len(), token_count, "Should return one result per token");
    }
    
    #[test]
    fn test_salient_phrase_extraction() {
        let analyzer = SalienceAnalyzer::new();
        let text = "The quick brown fox jumps over the lazy dog";
        
        // With a very low threshold, should return all tokens as phrases
        let phrases = analyzer.extract_salient_phrases(text, 0.0);
        assert!(!phrases.is_empty(), "Should return at least one phrase");
        
        // With a very high threshold, should return no phrases
        let phrases = analyzer.extract_salient_phrases(text, 1.0);
        assert!(phrases.is_empty() || phrases.iter().all(|p| p.is_empty()));
    }
}
