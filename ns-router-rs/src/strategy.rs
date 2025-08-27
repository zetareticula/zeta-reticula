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



//! # Strategy Selection Module
//!
//! This module provides functionality for selecting the optimal execution strategy
//! based on the context analysis of an inference request. It determines the best
//! model configuration, execution strategy, and KV cache settings for a given input.

use serde::{Serialize, Deserialize};
use crate::context::NSContextAnalysis;
use shared::PrecisionLevel;

/// Configuration for a model to be used for inference
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelConfig {
    /// Size of the model in parameters
    pub size: usize,
    
    /// Precision levels for the model
    pub precision: Vec<PrecisionLevel>,
}

/// Configuration for the KV cache
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KVCacheConfig {
    /// Sparsity level (0.0 = dense, 1.0 = fully sparse)
    pub sparsity: f32,
    
    /// Tokens that should be prioritized in the cache
    pub priority_tokens: Vec<u32>,
}

/// Strategy for executing model inference
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ExecutionStrategy {
    /// Run inference locally
    Local,
    
    /// Run inference in a federated manner
    Federated,
    
    /// Distribute inference across multiple nodes
    Distributed,
}

/// Selects the optimal execution strategy based on context analysis
#[derive(Debug, Clone)]
pub struct NSStrategySelector;

impl NSStrategySelector {
    /// Create a new strategy selector
    pub fn new() -> Self {
        NSStrategySelector
    }

    /// Select the best strategy based on context analysis
    /// 
    /// # Arguments
    /// * `context` - The analyzed context of the inference request
    /// 
    /// # Returns
    /// A tuple containing:
    /// 1. The selected model configuration
    /// 2. The execution strategy
    /// 3. The KV cache configuration
    /// 4. Any symbolic rules to apply
    pub fn select_strategy(
        &self, 
        context: &NSContextAnalysis
    ) -> (ModelConfig, ExecutionStrategy, KVCacheConfig, Vec<String>) {
        // Simple strategy selection based on input length and tokens
        let input_len = context.token_features.len();
        
        // Count subjects and modifiers
        let subject_count = context.token_features.iter()
            .filter(|t| t.role == "subject")
            .count();
        
        // Choose model size based on input length
        let model_size = match input_len {
            0..=10 => 100_000_000,  // 100M parameters
            11..=50 => 500_000_000,  // 500M parameters
            _ => 1_000_000_000,      // 1B parameters
        };
        
        // Choose precision based on subject count
        let precision = if subject_count > 2 {
            vec![PrecisionLevel::Bit16]
        } else {
            vec![PrecisionLevel::Bit32]
        };
        
        // Create model config
        let model_config = ModelConfig {
            size: model_size,
            precision,
        };
        
        // Simple execution strategy based on input length
        let execution_strategy = match input_len {
            0..=20 => ExecutionStrategy::Local,
            21..=100 => ExecutionStrategy::Federated,
            _ => ExecutionStrategy::Distributed,
        };
        
        // Simple KV cache config
        let kv_cache_config = KVCacheConfig {
            sparsity: 0.5,
            priority_tokens: vec![0, 1],  // BOS and EOS tokens
        };
        
        // No symbolic rules for now
        let symbolic_rules = vec![];
        
        (model_config, execution_strategy, kv_cache_config, symbolic_rules)
    }
}

impl Default for NSStrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TokenFeatures;

    #[test]
    fn test_strategy_selection() {
        let selector = NSStrategySelector::new();
        let context = NSContextAnalysis {
            input: "test input".to_string(),
            token_features: vec![
                TokenFeatures {
                    token_id: 1,
                    frequency: 0.5,
                    sentiment_score: 0.0,
                    context_relevance: 1.0,
                    role: "subject".to_string(),
                },
                TokenFeatures {
                    token_id: 2,
                    frequency: 0.5,
                    sentiment_score: 0.0,
                    context_relevance: 1.0,
                    role: "modifier".to_string(),
                },
            ],
            token_count: 2,
            salience_profile: vec![],
            theory_complexity: 0.0,
            symbolic_constraints: vec![],
        };

        let (model_config, strategy, _, _) = selector.select_strategy(&context);
        assert!(model_config.size > 0);
        assert!(!model_config.precision.is_empty());
        assert!(matches!(strategy, ExecutionStrategy::Local));
    }
}