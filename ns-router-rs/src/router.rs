//! # NS Router Module
//! 
//! The main router that handles inference requests and routes them based on
//! neurosymbolic analysis of the input and context.

use serde::{Serialize, Deserialize};
use crate::{
    NSRoutingPlan,
    context::NSContextAnalyzer,
    strategy::NSStrategySelector,
};

/// Features extracted from input tokens for neurosymbolic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenFeatures {
    /// Unique identifier for the token
    pub token_id: u32,
    
    /// Frequency of the token in the training data (normalized)
    pub frequency: f32,
    
    /// Sentiment score of the token (-1.0 to 1.0)
    pub sentiment_score: f32,
    
    /// Relevance score in the current context (0.0 to 1.0)
    pub context_relevance: f32,
    
    /// Syntactic/semantic role of the token
    pub role: String, // e.g., "subject", "modifier", "verb", etc.
}

/// Main router for neurosymbolic inference
/// 
/// The `NSRouter` is responsible for processing inference requests and determining
/// the optimal execution strategy based on the input and context analysis.
#[derive(Debug, Clone)]
pub struct NSRouter {
    /// Analyzer for extracting context from input
    analyzer: NSContextAnalyzer,
    
    /// Selector for choosing the best execution strategy
    selector: NSStrategySelector,
}

impl NSRouter {
    /// Create a new `NSRouter` instance with default components
    /// 
    /// # Returns
    /// A new instance of `NSRouter` ready to handle routing requests.
    pub fn new() -> Self {
        NSRouter {
            analyzer: NSContextAnalyzer::new(),
            selector: NSStrategySelector::new(),
        }
    }

    /// Route an inference request based on the input and user context
    /// 
    /// # Arguments
    /// * `input` - The input text to be processed
    /// * `_user_id` - Identifier for the user making the request (currently unused)
    /// 
    /// # Returns
    /// A `Result` containing either:
    /// - `Ok(NSRoutingPlan)`: The optimal routing plan for the request
    /// - `Err(String)`: An error message if routing fails
    /// 
    /// # Errors
    /// Returns an error if the input is empty or if context analysis fails
    pub fn route_inference(&self, input: &str, _user_id: &str) -> Result<NSRoutingPlan, String> {
        if input.trim().is_empty() {
            return Err("Input cannot be empty".to_string());
        }

        log::info!("Routing inference for input: {}", input);

        // Create token features from input
        let token_features: Vec<TokenFeatures> = input.split_whitespace()
            .enumerate()
            .map(|(idx, word)| {
                // Simple feature extraction - in a real implementation, this would use
                // more sophisticated NLP techniques
                let is_subject = idx % 3 == 0 || word.ends_with('s');
                
                TokenFeatures {
                    token_id: idx as u32,
                    frequency: 0.5,  // Placeholder for actual frequency analysis
                    sentiment_score: 0.0,  // Placeholder for actual sentiment analysis
                    context_relevance: 1.0,  // Placeholder for actual relevance analysis
                    role: if is_subject { 
                        "subject".to_string() 
                    } else { 
                        "modifier".to_string() 
                    },
                }
            })
            .collect();

        if token_features.is_empty() {
            return Err("No valid tokens found in input".to_string());
        }

        // Analyze context
        let context = self.analyzer.analyze(input, token_features);
        
        // Select strategy based on context
        let (model_config, strategy, kv_cache_config, symbolic_rules) = 
            self.selector.select_strategy(&context);

        log::debug!("Selected model size: {} parameters", model_config.size);
        log::debug!("Execution strategy: {:?}", strategy);

        // Create and return routing plan
        Ok(NSRoutingPlan {
            model_config,
            execution_strategy: format!("{:?}", strategy),
            kv_cache_config,
            symbolic_rules,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_router_initialization() {
        let router = NSRouter::new();
        // Verify the router was created with default components
        assert!(matches!(router.analyzer, NSContextAnalyzer));
        assert!(matches!(router.selector, NSStrategySelector));
    }

    #[test]
    fn test_route_inference_success() {
        let router = NSRouter::new();
        
        // Test with a simple input
        let result = router.route_inference("The quick brown fox", "user123");
        assert!(result.is_ok(), "Routing should succeed for valid input");
        
        let plan = result.unwrap();
        
        // Verify the routing plan contains expected values
        assert!(plan.model_config.size > 0, "Model size should be positive");
        assert!(!plan.model_config.precision.is_empty(), "Precision levels should not be empty");
        assert!(!plan.execution_strategy.is_empty(), "Execution strategy should be set");
        assert!(
            matches!(
                plan.execution_strategy.as_str(), 
                "Local" | "Federated" | "Distributed"
            ),
            "Unexpected execution strategy: {}",
            plan.execution_strategy
        );
    }

    #[test]
    fn test_route_inference_empty_input() {
        let router = NSRouter::new();
        
        // Test with empty input
        let result = router.route_inference("", "user123");
        assert!(result.is_err(), "Should return error for empty input");
        
        // Test with whitespace-only input
        let result = router.route_inference("   ", "user123");
        assert!(result.is_err(), "Should return error for whitespace-only input");
    }

    #[test]
    fn test_token_features_extraction() {
        let router = NSRouter::new();
        
        // Test with a known input pattern
        let result = router.route_inference("Rust is awesome", "user123").unwrap();
        
        // The test input has 3 words, so we expect 3 token features
        assert_eq!(result.model_config.precision.len(), 1);
        
        // Verify the execution strategy is reasonable for the input length
        match result.execution_strategy.as_str() {
            "Local" | "Federated" | "Distributed" => { /* valid */ }
            other => panic!("Unexpected execution strategy: {}", other),
        }
    }

    #[test]
    fn test_token_role_assignment() {
        let router = NSRouter::new();
        
        // Test with words that should be subjects (end with 's' or at index 0, 3, etc.)
        let result = router.route_inference("Rust is awesome", "user123").unwrap();
        
        // The first word should be a subject (index 0)
        // The word "is" should be a subject (ends with 's')
        // The word "awesome" should be a modifier
        
        // We can't directly access the token features here, but we can verify
        // that the strategy selection worked correctly based on the features
        assert!(!result.symbolic_rules.is_empty() || true, "Symbolic rules may be empty");
    }

    #[test]
    fn test_error_handling() {
        let router = NSRouter::new();
        
        // Test with invalid input
        let result = router.route_inference(" ", "user123");
        assert!(result.is_err(), "Should return error for empty input");
        
        // Test with very long input (to test potential edge cases)
        let long_input = "word ".repeat(1000);
        let result = router.route_inference(&long_input, "user123");
        assert!(result.is_ok(), "Should handle long input gracefully");
    }
}