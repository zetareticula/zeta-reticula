// Copyright 2025 ZETA RETICULA INC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.



//! # NS Router Module
//! 
//! The main router that handles inference requests and routes them based on
//! neurosymbolic analysis of the input and context.

use log::{self, error, info, debug};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::num::NonZeroUsize;
use tokio::sync::{RwLock, Mutex};
use lru::LruCache;
use salience_engine::role_inference::SalienceResult;
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
    
    /// Position of the token in the sequence
    pub position: usize,
    
    /// Semantic role of the token (e.g., subject, object, modifier)
    pub role: String,
    
    /// Salience score of the token (0-1)
    pub salience: f32,
    
    /// Attention weights for the token
    pub attention_weights: Vec<f32>,
    
    /// Sentiment score of the token (-1.0 to 1.0)
    pub sentiment_score: f32,
}

/// Configuration for model execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Name of the model to use
    pub model_name: String,
    /// Supported precision levels
    pub precision: Vec<PrecisionLevel>,
    /// Maximum sequence length
    pub max_length: u32,
    /// Whether to use forward time direction (true) or backward (false)
    pub use_forward_time: bool,
    /// Confidence score for the time direction (0.0 to 1.0)
    pub time_direction_confidence: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_name: "default".to_string(),
            precision: vec![PrecisionLevel::FP16],
            max_length: 2048,
            use_forward_time: true,  // Default to forward time direction
            time_direction_confidence: 0.8,  // Default confidence
        }
    }
}

/// Configuration for the KV cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVCacheConfig {
    /// Whether KV caching is enabled
    pub enabled: bool,
    /// Size of the KV cache in MB
    pub size_mb: usize,
    /// Whether the cache should be time-aware
    pub time_aware: bool,
}

/// Configuration for the NSRouter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Maximum number of tokens to process
    pub max_tokens: usize,
    /// Maximum number of cached routing plans
    pub max_cached_plans: usize,
    /// Default model configuration
    pub default_model: String,
    /// Default precision level for inference
    pub default_precision: PrecisionLevel,
    /// Whether to enable symbolic reasoning
    pub enable_symbolic: bool,
    /// Whether to enable salience analysis
    pub enable_salience: bool,
    /// Context window size for analysis
    pub context_window: usize,
    /// Whether to use time directionality (forward/backward) in routing
    pub enable_time_directionality: bool,
    /// Default time direction (true for forward, false for backward)
    pub default_time_direction: bool,
    /// Context length scaling factor for time directionality
    pub time_direction_context_scale: f32,
    /// Cache size for router decisions
    pub cache_size: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            max_tokens: 4096,
            default_model: "default".to_string(),
            max_cached_plans: 1000,
            default_precision: PrecisionLevel::FP16,
            enable_symbolic: true,
            enable_salience: true,
            context_window: 2048,
            enable_time_directionality: true,
            default_time_direction: true, // Default to forward direction
            time_direction_context_scale: 1.0,
            cache_size: 1024,
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
    context_analyzer: Arc<NSContextAnalyzer>,
    
    /// Salience analyzer for determining token importance
    salience_analyzer: Arc<SalienceAnalyzer>,
    
    /// Symbolic reasoner for handling logical constraints
    symbolic_reasoner: Arc<RwLock<SymbolicReasoner>>,
    
    /// Strategy selector for choosing execution strategies
    strategy_selector: Arc<NSStrategySelector>,
    
    /// Cache for storing routing decisions
    decision_cache: Arc<RwLock<LruCache<String, NSRoutingPlan>>>,
    
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
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap_or_else(|| NonZeroUsize::new(100).unwrap());
        let decision_cache = Arc::new(RwLock::new(LruCache::new(cache_size)));
        Self {
            context_analyzer: Arc::new(NSContextAnalyzer::default()),
            salience_analyzer: Arc::new(salience_analyzer),
            symbolic_reasoner: Arc::new(RwLock::new(SymbolicReasoner::default())),
            strategy_selector: Arc::new(NSStrategySelector::default()),
            decision_cache,
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
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap_or_else(|| NonZeroUsize::new(100).unwrap());
        let decision_cache = Arc::new(RwLock::new(LruCache::new(cache_size)));
        Self {
            context_analyzer: Arc::new(NSContextAnalyzer::default()),
            salience_analyzer: Arc::new(SalienceAnalyzer::new()),
            symbolic_reasoner: Arc::new(RwLock::new(SymbolicReasoner::default())),
            strategy_selector: Arc::new(NSStrategySelector::default()),
            decision_cache,
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
        if let Some(cached_plan) = {
            let cache = self.decision_cache.read().await;
            cache.peek(&cache_key).cloned()
        } {
            return Ok(cached_plan);
        }
        
        // Determine time direction based on config and input characteristics
        let use_forward = if self.config.enable_time_directionality {
            // Simple heuristic: use forward direction for most cases, 
            // but consider input length for direction decision
            let input_len = input.len();
            if input_len > (self.config.context_window * 2) {
                // For very long inputs, prefer forward direction
                true
            } else {
                // For shorter inputs, use config default
                self.config.default_time_direction
            }
        } else {
            // Time directionality disabled, use forward as default
            true
        };
        
        // Tokenize input and analyze salience
        let salience_results = self.salience_analyzer.analyze_text(input);
        let salient_phrases = self.salience_analyzer.extract_salient_phrases(input, 0.5);
        
        // Extract token features with salience information
        let token_features = self.extract_token_features_with_salience(&salience_results);
        
        // Analyze context with salience information and time directionality
        let mut context = self.context_analyzer.analyze(input, token_features, use_forward);
        
        // Update context with time directionality information
        context.use_forward_time = use_forward;
        
        // Apply symbolic reasoning if enabled
        let symbolic_constraints = if self.config.enable_symbolic {
            self.apply_symbolic_reasoning(input, &salience_results).await?
        } else {
            Vec::new()
        };
        
        // Log time directionality decision
        let direction = if use_forward { "forward" } else { "backward" };
        log::info!(
            "Using {} time direction (confidence: {:.2}%)", 
            direction, 
            context.time_direction_confidence * 100.0
        );
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
        
        // Create routing plan with time directionality and salience information
        let plan = NSRoutingPlan {
            model_config: ModelConfig {
                model_name: self.config.default_model.clone(),
                max_length: self.config.max_tokens as u32,
                precision: self.select_precision(&context),
                use_forward_time: context.use_forward_time,
                time_direction_confidence: context.time_direction_confidence,
            },
            execution_strategy: strategy,
            kv_cache_config: KVCacheConfig {
                enabled: true,
                size_mb: self.calculate_cache_size(&context),
                time_aware: context.use_forward_time,
            },
            symbolic_rules: context.symbolic_constraints,
            salient_phrases,
            time_context_scale: context.time_context_scale as i32,
        };
        
        // Log the routing decision with time directionality
        log::info!(
            "Routing with {} time direction (confidence: {:.2}%, context scale: {})",
            if context.use_forward_time { "forward" } else { "backward" },
            context.time_direction_confidence * 100.0,
            context.time_context_scale
        );
        
        // Cache the routing decision
        {
            let mut cache = self.decision_cache.write().await;
            cache.put(cache_key, plan.clone());
        }
        
        Ok(plan)
    }
    
    /// Extract features from tokens with salience information
    fn extract_token_features_with_salience(&self, salience_results: &[SalienceResult]) -> Vec<TokenFeatures> {
        salience_results.iter().map(|result| {
            TokenFeatures {
                token_id: result.token_id,
                position: result.position,
                role: result.role.clone(),
                salience: result.salience,
                attention_weights: result.attention_weights.clone(),
                sentiment_score: 0.0, // Default sentiment score
            }
        }).collect()
    }
    
    /// Extract features from tokens (legacy method)
    fn extract_token_features(&self, tokens: &[&str]) -> Vec<TokenFeatures> {
        tokens
            .iter()
            .enumerate()
            .map(|(i, _)| {
                TokenFeatures {
                    token_id: i as u32,
                    position: i,
                    role: "unknown".to_string(),
                    salience: 1.0,
                    attention_weights: vec![1.0],
                    sentiment_score: 0.0,
                }
            })
            .collect()
    }

    
    /// Apply symbolic reasoning to the input
    async fn apply_symbolic_reasoning(
        &self, 
        input: &str, 
        salience_profile: &[QuantizationResult]
    ) -> Result<Vec<String>, RouterError> {
        // Extract constraints from input
        let constraints = self.extract_constraints(input);
        
        // Get a write lock on the symbolic reasoner
        let mut reasoner = self.symbolic_reasoner.write().await;
        
        // Apply symbolic reasoning
        reasoner
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
    pub fn select_precision(&self, context: &NSContextAnalysis) -> Vec<PrecisionLevel> {
        // Simple heuristic: use 16-bit for large batches, 32-bit otherwise
        if context.batch_size > 32 {
            vec![PrecisionLevel::Bit16]
        } else {
            vec![PrecisionLevel::Bit32]
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
    
    /// Time directionality information
    pub time_context_scale: i32,
}

impl Default for NSRoutingPlan {
    fn default() -> Self {
        Self {
            model_config: ModelConfig::default(),
            execution_strategy: "local".to_string(),
            kv_cache_config: KVCacheConfig {
                enabled: true,
                size_mb: 1024,
                time_aware: false,
            },
            symbolic_rules: Vec::new(),
            salient_phrases: Vec::new(),
            time_context_scale: 1,  // Default to scale of 1 (no scaling)
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
            time_context_scale: 1,  // Default to scale of 1 (no scaling)
        }
    }
}
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
    use tokio::test;

    #[test]
    fn test_router_initialization() {
        let router = NSRouter::new();
        // Verify the router was created with default components
        assert!(router.context_analyzer.is_some());
        assert!(router.strategy_selector.is_some());
    }

    #[tokio::test]
    async fn test_route_inference_success() {
        let router = NSRouter::new();
        
        // Test with a simple input
        let result = router.route_inference("The quick brown fox", "user123").await;
        assert!(result.is_ok(), "Routing should succeed for valid input");
        
        let plan = result.unwrap();
        
        // Verify the routing plan contains expected values
        assert!(!plan.model_config.model_name.is_empty(), "Model name should not be empty");
        assert!(!plan.execution_strategy.is_empty(), "Execution strategy should be set");
    }

    #[tokio::test]
    async fn test_route_inference_empty_input() {
        let router = NSRouter::new();
        
        // Test with empty input
        let result = router.route_inference("", "user123").await;
        assert!(result.is_err(), "Should return error for empty input");
        
        // Test with whitespace-only input
        let result = router.route_inference("   ", "user123").await;
        assert!(result.is_err(), "Should return error for whitespace-only input");
    }

    #[tokio::test]
    async fn test_token_features_extraction() {
        let router = NSRouter::new();
        
        // Test with a known input pattern
        let result = router.route_inference("Rust is awesome", "user123").await.unwrap();
        
        // Verify the execution strategy is set
        assert!(!result.execution_strategy.is_empty());
    }

    #[tokio::test]
    async fn test_token_role_assignment() {
        let router = NSRouter::new();
        
        // Test with words that should be subjects (end with 's' or at index 0, 3, etc.)
        let result = router.route_inference("Rust is awesome", "user123").await.unwrap();
        
        // Verify we got a valid routing plan
        assert!(!result.execution_strategy.is_empty());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let router = NSRouter::new();
        
        // Test with invalid input
        let result = router.route_inference(" ", "user123").await;
        assert!(result.is_err(), "Should return error for empty input");
        
        // Test with very long input (to test potential edge cases)
        let long_input = "word ".repeat(1000);
        let result = router.route_inference(&long_input, "user123").await;
        assert!(result.is_ok(), "Should handle long input gracefully");
    }
}


