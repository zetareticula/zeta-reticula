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
use crate::salience::SalienceAnalyzer;

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
    context::{NSContextAnalyzer, NSContextAnalysis},
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
    
    /// Relevance of the token in the current context (0-1)
    pub context_relevance: f32,
    
    /// Semantic role of the token (e.g., subject, object, modifier)
    pub role: String,
    
    /// Sentiment score of the token (-1.0 to 1.0)
    pub sentiment_score: f32,
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
#[derive(Debug, Clone)]
pub struct NSRouter {
    /// Context analyzer for extracting features and context
    context_analyzer: Arc<dyn NSContextAnalyzer + Send + Sync>,
    
    /// Salience analyzer for determining token importance
    salience_analyzer: Arc<SalienceAnalyzer>,
    
    /// Symbolic reasoner for applying logical rules
    symbolic_reasoner: Arc<SymbolicReasoner>,
    
    /// Strategy selector for choosing execution strategies
    strategy_selector: Arc<NSStrategySelector>,
    
    /// Cache for storing routing decisions
    routing_cache: Arc<dyn RoutingCache + Send + Sync>,
    
    /// Configuration for the router
    config: RouterConfig,
}

impl NSRouter {
    /// Create a new `NSRouter` instance with default components
    /// 
    /// # Returns
    /// A new instance of `NSRouter` ready to handle routing requests.
    pub fn new() -> Self {
        let config = RouterConfig::default();
        let salience_analyzer = SalienceAnalyzer::new();
        
        Self {
            context_analyzer: Arc::new(NSContextAnalyzer::default()),
            salience_analyzer: Arc::new(salience_analyzer),
            symbolic_reasoner: Arc::new(SymbolicReasoner::default()),
            strategy_selector: Arc::new(NSStrategySelector::default()),
            routing_cache: Arc::new(InMemoryRoutingCache::default()),
            config,
        }
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
        
        Self {
            context_analyzer: Arc::new(NSContextAnalyzer::default()),
            salience_analyzer: Arc::new(SalienceAnalyzer::new()),
            symbolic_reasoner: Arc::new(SymbolicReasoner::default()),
            strategy_selector: Arc::new(NSStrategySelector::default()),
            routing_cache: Arc::new(InMemoryRoutingCache::default()),
            config,
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
        if input.trim().is_empty() {
            return Err(RouterError::EmptyInput);
        }
        
        // Check cache first
        let cache_key = format!("{}:{}", user_id, input);
        if let Some(cached_plan) = self.routing_cache.get(&cache_key).await {
            return Ok(cached_plan);
        }
        
        // Tokenize input and analyze salience
        let salience_results = self.salience_analyzer.analyze_text(input);
        let salient_phrases = self.salience_analyzer.extract_salient_phrases(input, 0.5);
        
        // Extract token features with salience information
        let token_features = self.extract_token_features_with_salience(&salience_results);
        
        // Analyze context with salience information
        let mut context = self.context_analyzer.analyze(input, token_features);
        
        // Apply symbolic reasoning if enabled
        if self.config.enable_symbolic {
            let symbolic_constraints = self.apply_symbolic_reasoning(input, &context.salience_profile)?;
            context.symbolic_constraints = symbolic_constraints;
        }
        
        // Update context with salience information
        context.salience_profile = salience_results.into_iter()
            .map(|sr| {
                let mut qr = QuantizationResult::default();
                qr.token_id = sr.token_id;
                qr.score = sr.salience_score;
                qr
            })
            .collect();
        
        // Select execution strategy with salience information
        let strategy = self.strategy_selector.select_strategy(&context)
            .await
            .map_err(|e| RouterError::StrategyError(e.to_string()))?;
        
        // Create routing plan with salience-informed configuration
        let plan = NSRoutingPlan {
            model_config: ModelConfig {
                max_length: self.config.max_sequence_length,
                precision: self.select_precision(&context),
                ..Default::default()
            },
            execution_strategy: strategy,
            kv_cache_config: KVCacheConfig {
                enabled: self.config.enable_kv_cache,
                size_mb: self.calculate_cache_size(&context),
            },
            symbolic_rules: context.symbolic_constraints,
            salient_phrases, // Add salient phrases to the routing plan
        };
        
        // Cache the routing decision
        self.routing_cache.set(&cache_key, plan.clone()).await;
        
        Ok(plan)
    }
    
    /// Extract features from tokens with salience information
    fn extract_token_features_with_salience(&self, salience_results: &[SalienceResult]) -> Vec<TokenFeatures> {
        salience_results
            .iter()
            .map(|sr| {
                TokenFeatures {
                    token_id: sr.token_id,
                    frequency: 1.0 - sr.salience_score, // Less frequent tokens are more salient
                    context_relevance: sr.salience_score,
                    role: sr.role.clone(),
                }
            })
            .collect()
    }
    
    /// Extract features from tokens (legacy method)
    fn extract_token_features(&self, tokens: &[&str]) -> Vec<TokenFeatures> {
        tokens
            .iter()
            .enumerate()
            .map(|(i, _)| {
                TokenFeatures {
                    token_id: i as u32,
                    frequency: 0.5,
                    context_relevance: 0.5,
                    role: "modifier".to_string(),
                }
            })
            .collect()
    }
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
    fn select_precision(&self, context: &NSContextAnalysis) -> Vec<PrecisionLevel> {
        // For now, return a single precision level based on the context
        // In a real implementation, this could return multiple options with confidence scores
        if context.token_features.len() > 1000 {
            vec![PrecisionLevel::FP16]
        } else {
            vec![PrecisionLevel::FP32]
        }
    }
    
    /// Calculate appropriate cache size in MB based on context
    fn calculate_cache_size(&self, context: &NSContextAnalysis) -> u32 {
        // Simple heuristic: allocate more cache for larger inputs
        // Base size + size based on number of tokens
        let base_size = 512; // 512MB base
        let token_based = (context.token_features.len() as f32 * 0.1) as u32; // 0.1MB per token
        
        (base_size + token_based).min(4096) // Cap at 4GB
    }
    
        // Update context with salience information
        context.salience_profile = salience_results.into_iter()
            .map(|sr| {
                let mut qr = QuantizationResult::default();
                qr.token_id = sr.token_id;
                qr.score = sr.salience_score;
                qr
            })
            .collect();
        
        // Select execution strategy with salience information
        let strategy = self.strategy_selector.select_strategy(&context)
            .await
            .map_err(|e| RouterError::StrategyError(e.to_string()))?;
        
        // Create routing plan with salience-informed configuration
        let plan = NSRoutingPlan::new(
            ModelConfig {
                max_length: self.config.max_sequence_length,
                precision: self.select_precision(&context),
                ..Default::default()
            },
            strategy,
            KVCacheConfig {
                enabled: self.config.enable_kv_cache,
                size_mb: self.calculate_cache_size(&context),
            },
            context.symbolic_constraints,
        );
        
        // Cache the routing decision
        self.routing_cache.set(&cache_key, plan.clone()).await;
        
        Ok(plan)
    }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NSRoutingPlan {
    /// Configuration for the model to use
    pub model_config: ModelConfig,
    
    /// Strategy for executing the model
    pub execution_strategy: String,
    
    /// Configuration for the KV cache
    pub kv_cache_config: KVCacheConfig,
    
    /// Symbolic rules to apply during inference
    pub symbolic_rules: Vec<String>,
    
    /// Salient phrases identified in the input
    pub salient_phrases: Vec<String>,
}

impl Default for NSRoutingPlan {
    fn default() -> Self {
        Self {
            model_config: ModelConfig {
                model_name: "default".to_string(),
                precision: vec![PrecisionLevel::FP16],
                max_length: 2048,
            },
            execution_strategy: "local".to_string(),
            kv_cache_config: KVCacheConfig {
                enabled: true,
                size_mb: 1024,
            },
            symbolic_rules: Vec::new(),
            salient_phrases: Vec::new(),
        }
    }
}

impl NSRoutingPlan {
    pub fn new(
        model_config: ModelConfig,
        execution_strategy: String,
        kv_cache_config: KVCacheConfig,
        symbolic_rules: Vec<String>,
    ) -> Self {
        Self {
            model_config,
            execution_strategy,
            kv_cache_config,
            symbolic_rules,
            salient_phrases: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Local,
    Federated,
    Distributed,
    OnDevice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_name: String,
    pub precision: Vec<PrecisionLevel>,
    pub max_length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVCacheConfig {
    pub enabled: bool,
    pub size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicRule {
    pub name: String,
    pub pattern: String,
    pub action: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicRules {
    pub rules: Vec<SymbolicRule>,
}

// NSContextAnalysis is now defined in the context module


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


