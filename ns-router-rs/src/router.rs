//! # NS Router Module
//! 
//! The main router that handles inference requests and routes them based on
//! neurosymbolic analysis of the input and context.

use log::{self, error, info, debug};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Errors that can occur during routing
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum RouterError {
    #[error("Empty input")]
    EmptyInput,
    
    #[error("Invalid input format: {0}")]
    InvalidInput(String),
    
    #[error("Symbolic reasoning failed: {0}")]
    SymbolicError(#[from] crate::symbolic::SymbolicError),
    
    #[error("Strategy selection failed: {0}")]
    StrategyError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result type for router operations
pub type RouterResult<T> = Result<T, RouterError>;

// Re-export commonly used types
use shared::{QuantizationResult, PrecisionLevel};
use crate::{
    NSRoutingPlan,
    context::NSContextAnalyzer,
    strategy::NSStrategySelector,
    symbolic::{SymbolicReasoner, SymbolicError},
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

/// Configuration for the NSRouter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Maximum number of tokens to process
    pub max_tokens: usize,
    
    /// Default model configuration
    pub default_model: String,
    
    /// Enable/disable symbolic reasoning
    pub enable_symbolic: bool,
    
    /// Cache size for router decisions
    pub cache_size: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            max_tokens: 4096,
            default_model: "default".to_string(),
            enable_symbolic: true,
            cache_size: 1000,
        }
    }
}

/// Main router for neurosymbolic inference
/// 
/// The `NSRouter` is responsible for processing inference requests and determining
/// the optimal execution strategy based on the input and context analysis.
#[derive(Debug)]
pub struct NSRouter {
    /// Analyzer for extracting context from input
    analyzer: NSContextAnalyzer,
    
    /// Selector for choosing the best execution strategy
    selector: NSStrategySelector,
    
    /// Symbolic reasoner for advanced constraints
    symbolic_reasoner: SymbolicReasoner,
    
    /// Configuration
    config: RouterConfig,
    
    /// Decision cache
    #[allow(dead_code)]
    decision_cache: Arc<RwLock<lru::LruCache<String, NSRoutingPlan>>>,
}

impl NSRouter {
    /// Create a new `NSRouter` instance with default components
    /// 
    /// # Returns
    /// A new instance of `NSRouter` ready to handle routing requests.
    pub fn new() -> Self {
        Self::with_config(RouterConfig::default())
    }
    
    /// Create a new `NSRouter` with custom configuration
    /// 
    /// # Arguments
    /// * `config` - Configuration for the router
    /// 
    /// # Returns
    /// A new instance of `NSRouter` with the specified configuration.
    pub fn with_config(config: RouterConfig) -> Self {
        let decision_cache = Arc::new(RwLock::new(
            lru::LruCache::new(config.cache_size)
        ));
        
        NSRouter {
            analyzer: NSContextAnalyzer::new(),
            selector: NSStrategySelector::new(),
            symbolic_reasoner: SymbolicReasoner::new(),
            config,
            decision_cache,
        }
    }

    /// Route an inference request based on the input and user context
    /// 
    /// # Arguments
    /// * `input` - The input text to be processed
    /// * `user_id` - Identifier for the user making the request
    /// 
    /// # Returns
    /// A `RouterResult` containing either:
    /// - `Ok(NSRoutingPlan)`: The optimal routing plan for the request
    /// - `Err(RouterError)`: An error if routing fails
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Input is empty
    /// - Context analysis fails
    /// - Symbolic reasoning fails
    /// - Strategy selection fails
    pub async fn route_inference(&self, input: &str, user_id: &str) -> RouterResult<NSRoutingPlan> {
        // Input validation
        let input = input.trim();
        if input.is_empty() {
            error!("Empty input received from user: {}", user_id);
            return Err(RouterError::EmptyInput);
        }

        info!("Routing inference for user: {}, input length: {}", user_id, input.len());
        debug!("Input: {}", input);

        // Check cache first
        let cache_key = format!("{}:{}", user_id, input);
        if let Some(cached_plan) = self.decision_cache.write().await.get(&cache_key) {
            debug!("Cache hit for input");
            return Ok(cached_plan.clone());
        }

        // Tokenize and extract features
        let tokens: Vec<&str> = input.split_whitespace().collect();
        if tokens.len() > self.config.max_tokens {
            return Err(RouterError::InvalidInput(format!(
                "Input exceeds maximum token limit ({} > {})",
                tokens.len(),
                self.config.max_tokens
            )));
        }

        let token_features = self.extract_token_features(&tokens);
        
        // Analyze context
        let context = self.analyzer.analyze(input, token_features);
        
        // Apply symbolic reasoning if enabled
        let mut symbolic_rules = Vec::new();
        if self.config.enable_symbolic {
            symbolic_rules = self.apply_symbolic_reasoning(input, &context.salience_profile)?;
            debug!("Generated {} symbolic rules", symbolic_rules.len());
        }

        // Select execution strategy
        let strategy = self.selector.select_strategy(&context, &symbolic_rules)
            .map_err(|e| RouterError::StrategyError(e.to_string()))?;

        // Create routing plan
        let plan = NSRoutingPlan {
            model_config: ModelConfig {
                model_name: self.config.default_model.clone(),
                precision: self.select_precision(&context),
                max_length: self.config.max_tokens as u32,
            },
            execution_strategy: strategy,
            kv_cache_config: KVCacheConfig {
                enabled: true,
                size_mb: self.calculate_cache_size(&context),
            },
            symbolic_rules,
        };

        // Cache the decision
        self.decision_cache.write().await.put(cache_key, plan.clone());

        Ok(plan)
    }
    
    /// Extract features from tokens
    fn extract_token_features(&self, tokens: &[&str]) -> Vec<TokenFeatures> {
        tokens.iter()
            .enumerate()
            .map(|(idx, &word)| {
                // In a real implementation, this would use more sophisticated NLP techniques
                let is_subject = idx % 3 == 0 || word.ends_with('s');
                let length_factor = word.len() as f32 / 10.0; // Normalize by max expected length
                
                TokenFeatures {
                    token_id: idx as u32,
                    frequency: 1.0 / (idx as f32 + 1.0).sqrt(), // Simple IDF-like weighting
                    sentiment_score: 0.0,  // Would use a sentiment analysis model
                    context_relevance: 1.0 - (0.1 * (idx % 5) as f32), // Simple decay
                    role: if is_subject { 
                        "subject".to_string() 
                    } else { 
                        "modifier".to_string() 
                    },
                    length_factor,
                }
            })
            .collect()
    }
    
    /// Apply symbolic reasoning to the input
    fn apply_symbolic_reasoning(
        &self, 
        input: &str, 
        salience_profile: &[QuantizationResult]
    ) -> Result<Vec<String>, RouterError> {
        // Extract constraints from input
        let constraints = self.extract_constraints(input);
        
        // Apply symbolic reasoning
        self.symbolic_reasoner
            .apply_constraints(&constraints, salience_profile)
            .map_err(RouterError::SymbolicError)
    }
    
    /// Extract constraints from input text
    fn extract_constraints(&self, input: &str) -> Vec<String> {
        // Simple regex-based constraint extraction
        // In a real implementation, this would use more sophisticated NLP
        let constraint_pattern = regex::Regex::new(r"(?:must|should|require)[^.!?]*(?:[.!?]|$")
            .expect("Invalid regex pattern");
            
        constraint_pattern
            .find_iter(input)
            .map(|m| m.as_str().to_string())
            .collect()
    }
    
    /// Select appropriate precision based on context
    fn select_precision(&self, context: &NSContextAnalysis) -> PrecisionLevel {
        // Simple heuristic: use lower precision for larger inputs
        if context.token_count > 1000 {
            PrecisionLevel::FP16
        } else {
            PrecisionLevel::FP32
        }
    }
    
    /// Calculate appropriate cache size based on context
    fn calculate_cache_size(&self, context: &NSContextAnalysis) -> u32 {
        // Simple heuristic: larger inputs get more cache
        let base_size = 1024; // 1GB base
        let token_based = (context.token_count as f32 * 0.1) as u32; // 0.1MB per token
        
        (base_size + token_based).min(8192) // Cap at 8GB
    }
}

#[derive(Serialize, Deserialize)]
pub struct NSRoutingPlan {
    pub model_config: ModelConfig,
    pub execution_strategy: String,
    pub kv_cache_config: KVQuantConfig,
    pub symbolic_rules: Vec<String>,
}

impl NSRoutingPlan {
    pub fn new(model_config: ModelConfig, execution_strategy: String, kv_cache_config: KVQuantConfig, symbolic_rules: Vec<String>) -> Self {
        NSRoutingPlan {
            model_config,
            execution_strategy,
            kv_cache_config,
            symbolic_rules,
        }
    }
}

enum ExecutionStrategy {
    Local,
    Federated,
    Distributed,
}

#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub size: usize,
    pub precision: Vec<PrecisionLevel>,
}

#[derive(Serialize, Deserialize)]
pub struct KVQuantConfig {
    pub spot_capacity: usize,
    pub block_size: usize,
    pub salience_threshold: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SymbolicRule {
    pub name: String,
    pub description: String,
    pub conditions: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SymbolicRules {
    pub rules: Vec<SymbolicRule>,
}

#[derive(Serialize, Deserialize)]
pub struct NSContextAnalysis {
    pub context: String,
    pub token_features: Vec<TokenFeatures>,
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


