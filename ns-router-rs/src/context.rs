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
use crate::model::Model;
use crate::kv_cache::KVCache;
use crate::fusion_anns::FusionANNS;
use agentflow_rs::server::AgentFlowServer;
use tonic::transport::Channel;
use ns_router_rs::pb;





//bring crates forward
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NSContextAnalyzer {
    //branch off
    let mut model: Model;
    let mut kv_cache: KVCache;
    let mut fusion_anns: FusionANNS;
    let mut agent_flow_server: agentflow_rs::server::AgentFlowServer;
    let mut quantization_results: Vec<QuantizationResult>;
    let mut sidecar_client: pb::sidecar_service_client::SidecarServiceClient<Channel>;
    // Placeholder for future tableau functionality
    let mut _tableau_placeholder: ();
}


/// Analysis of the context for neurosymbolic routing
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NSContextAnalysis {
    /// The input text being analyzed
    pub input: String,
    
    /// Features extracted from each token in the input
    pub token_features: Vec<TokenFeatures>,
    
    /// Number of tokens in the input
    pub token_count: usize,
    
    /// Profile of salience scores for the input
    pub salience_profile: Vec<QuantizationResult>,
    
    /// Estimated complexity of the input
    pub theory_complexity: f32,
    
    /// Any symbolic constraints derived from the input
    pub symbolic_constraints: Vec<String>,
    
    /// User ID for the current request
    pub user_id: Option<String>,
    
    /// Model configuration for the request
    pub model_config: Option<ModelConfig>,
    
    /// Cache configuration for the request
    pub cache_config: Option<KVCacheConfig>,
}

impl NSContextAnalysis {
    /// Create a new NSContextAnalysis with default values
    pub fn new() -> Self {
        NSContextAnalysis {
            input: String::new(),
            token_features: Vec::new(),
            token_count: 0,
            salience_profile: Vec::new(),
            theory_complexity: 0.0,
            symbolic_constraints: Vec::new(),
            user_id: None,
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
    pub fn analyze(&self, input: &str, token_features: Vec<TokenFeatures>) -> NSContextAnalysis {
        let token_count = token_features.len();
        
        // Calculate theory complexity (simplified for now)
        let theory_complexity = if token_count > 0 {
            let avg_freq: f32 = token_features.iter()
                .map(|f| f.frequency)
                .sum::<f32>() / token_count as f32;
            
            // Higher complexity for less frequent words
            (1.0 - avg_freq).max(0.1)
        } else {
            0.0
        };
        
        // Create salience profile (simplified for now)
        let salience_profile = token_features.iter()
            .map(|f| {
                let mut qr = QuantizationResult::default();
                qr.token_id = f.token_id;
                qr.score = f.context_relevance;
                // Set other required fields with reasonable defaults
                qr.precision = PrecisionLevel::FP32;
                qr.quantized = false;
                qr
            })
            .collect();
        
        NSContextAnalysis::new()
            .with_input(input)
            .with_token_features(token_features)
            .with_token_count(token_count)
            .with_theory_complexity(theory_complexity)
            .with_salience_profile(salience_profile)
    }
}

impl Default for NSContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}


