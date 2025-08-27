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


//! # Context Analysis Module
//!
//! This module provides functionality for analyzing the context of inference requests
//! to inform routing decisions. It extracts features from input text and generates
//! analysis that can be used to select the most appropriate execution strategy.

use shared::{QuantizationResult, PrecisionLevel};
use serde::{Serialize, Deserialize};
use super::{TokenFeatures, ModelConfig, KVCacheConfig};
use log;

/// Analysis of the context for neurosymbolic routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NSContextAnalysis {
    /// Salience scores for each token
    pub token_salience: Vec<f32>,
    /// Average salience score
    pub avg_salience: f32,
    /// Maximum salience score
    pub max_salience: f32,
    /// Salient phrases and their scores
    pub salient_phrases: Vec<(String, f32)>,
    /// Overall sentiment score (-1.0 to 1.0)
    pub sentiment: f32,
    /// Detected language
    pub language: String,
    /// Whether the input contains sensitive content
    pub is_sensitive: bool,
    /// Complexity score (higher means more complex)
    pub complexity: f32,
    /// Whether the input contains questions
    pub has_questions: bool,
    /// Whether the input contains commands
    pub has_commands: bool,
    /// Whether to use forward time direction (true) or backward (false)
    pub use_forward_time: bool,
    /// Time direction confidence score (0.0 to 1.0)
    pub time_direction_confidence: f32,
    /// Context length scaling factor for time directionality
    pub time_context_scale: f32,
    /// Any symbolic constraints derived from the input
    pub symbolic_constraints: Vec<String>,
    /// User ID for the current request
    pub user_id: Option<String>,
    /// Batch size for the current request
    pub batch_size: usize,
    
    /// Model configuration for the request
    pub model_config: Option<ModelConfig>,
    
    /// Cache configuration for the request
    pub cache_config: Option<KVCacheConfig>,
}

impl NSContextAnalysis {
    /// Create a new NSContextAnalysis with default values
    pub fn new() -> Self {
        Self {
            input: String::new(),
            token_features: Vec::new(),
            token_count: 0,
            salience_profile: Vec::new(),
            theory_complexity: 0.0,
            symbolic_constraints: Vec::new(),
            user_id: None,
            batch_size: 1,  // Default batch size is 1
            model_config: None,
            cache_config: None,
        }
    }
    
    /// Set the user ID for this context
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    
    /// Set the model configuration for this context
    pub fn with_model_config(mut self, config: ModelConfig) -> Self {
        self.model_config = Some(config);
        self
    }
    
    /// Set the cache configuration for this context
    pub fn with_cache_config(mut self, config: KVCacheConfig) -> Self {
        self.cache_config = Some(config);
        self
    }
    
    /// Set the input text
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.input = input.into();
        self
    }
    
    /// Set the token features
    pub fn with_token_features(mut self, features: Vec<TokenFeatures>) -> Self {
        self.token_features = features;
        self
    }
    
    /// Set the token count
    pub fn with_token_count(mut self, count: usize) -> Self {
        self.token_count = count;
        self
    }
    
    /// Set the theory complexity
    pub fn with_theory_complexity(mut self, complexity: f32) -> Self {
        self.theory_complexity = complexity;
        self
    }
    
    /// Set the salience profile
    pub fn with_salience_profile(mut self, profile: Vec<QuantizationResult>) -> Self {
        self.salience_profile = profile;
        self
    }
    
    /// Add a symbolic constraint
    pub fn with_symbolic_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.symbolic_constraints.push(constraint.into());
        self
    }
}

impl Default for NSContextAnalysis {
    fn default() -> Self {
        Self::new()
    }
}



/// Analyzes the context of an inference request
#[derive(Debug, Clone)]
pub struct NSContextAnalyzer;

impl NSContextAnalyzer {
    /// Create a new context analyzer
    pub fn new() -> Self {
        NSContextAnalyzer
    }

    /// Analyze the input and token features to produce context analysis
    pub fn analyze(&self, text: &str, token_features: Vec<TokenFeatures>, use_forward_time: bool) -> NSContextAnalysis {
        // Calculate basic statistics
        let salience_scores: Vec<f32> = token_features.iter()
            .map(|f| f.salience)
            .collect();
            
        let token_count = token_features.len();
        let avg_salience = if !salience_scores.is_empty() {
            salience_scores.iter().sum::<f32>() / salience_scores.len() as f32
        } else {
            0.0
        };
        
        let max_salience = salience_scores.iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
            
        // Analyze time directionality
        let (time_direction_confidence, time_context_scale) = self.analyze_time_directionality(
            text, 
            &token_features,
            use_forward_time
        );
            
        // For now, use placeholder values for other fields
        NSContextAnalysis {
            token_salience: salience_scores,
            avg_salience,
            max_salience,
            salient_phrases: Vec::new(),
            sentiment: 0.0,
            language: "en".to_string(),
            is_sensitive: false,
            complexity: 0.5,
            has_questions: false,
            has_commands: false,
            use_forward_time,
            time_direction_confidence,
            time_context_scale: time_context_scale as f32,
            symbolic_constraints: Vec::new(),
            user_id: None,
            batch_size: 1,
            model_config: None,
            cache_config: None,
        }
    }

    /// Analyze time directionality of the input text
    /// 
    /// Returns a tuple of (confidence, context_scale) where:
    /// - confidence: 0.0 to 1.0 indicating confidence in the time direction
    /// - context_scale: Suggested context scaling factor based on input length
    fn analyze_time_directionality(&self, text: &str, token_features: &[TokenFeatures], use_forward_time: bool) -> (f32, i32) {
        let text_len = text.len();
        let token_count = token_features.len();
        
        // Calculate basic statistics
        let avg_token_len = if token_count > 0 {
            text_len as f32 / token_count as f32
        } else {
            0.0
        };
        
        // Simple heuristic: longer inputs benefit more from forward direction
        let length_confidence = (text_len as f32 / 1000.0).min(1.0);
        
        // Check for time-related words/phrases that might indicate direction
        let time_indicators = [
            ("after", 0.2), ("before", -0.2),
            ("later", 0.15), ("earlier", -0.15),
            ("next", 0.1), ("previous", -0.1),
            ("will", 0.1), ("was", -0.1),
        ];
        
        let mut time_bias = 0.0;
        let text_lower = text.to_lowercase();
        
        for (word, bias) in &time_indicators {
            if text_lower.contains(word) {
                time_bias += bias;
            }
        }
        
        // Calculate final confidence based on heuristics
        let base_confidence = if use_forward_time {
            0.6 + (length_confidence * 0.3) + time_bias
        } else {
            0.4 + (length_confidence * 0.2) - time_bias
        };
        
        // Calculate context scale factor based on input length
        let context_scale = match text_len {
            0..=100 => 1,
            101..=500 => 2,
            501..=2000 => 4,
            _ => 8
        };
        
        (base_confidence.max(0.0).min(1.0), context_scale)
    }
}

impl Default for NSContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}


