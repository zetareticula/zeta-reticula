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

//! Unified Salience and Mesolimbic System for Zeta Reticula
//! 
//! This module consolidates all salience and mesolimbic functionality from:
//! - salience-engine/src/mesolimbic.rs
//! - agentflow-rs/src/mesolimbic.rs
//! - kvquant_rs/src/mesolimbic_system.rs
//! - Multiple salience analysis implementations

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SalienceError {
    #[error("Invalid token: {0}")]
    InvalidToken(u32),
    #[error("Computation failed: {0}")]
    ComputationError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Memory allocation failed")]
    MemoryError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalienceConfig {
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub threshold: f64,
    pub outer_loop_iterations: usize,
    pub inner_loop_iterations: usize,
    pub phoneme_preservation: bool,
    pub enable_foraging: bool,
    pub adaptive_threshold: bool,
}

impl Default for SalienceConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            discount_factor: 0.95,
            threshold: 0.5,
            outer_loop_iterations: 100,
            inner_loop_iterations: 10,
            phoneme_preservation: true,
            enable_foraging: true,
            adaptive_threshold: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalienceResult {
    pub token_id: u32,
    pub salience_score: f32,
    pub confidence: f32,
    pub phoneme_preserved: bool,
    pub foraging_probability: f32,
    pub role_inference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MesolimbicState {
    pub dopamine_level: f64,
    pub attention_focus: Vec<u32>,
    pub reward_prediction: f64,
    pub exploration_factor: f64,
}

impl Default for MesolimbicState {
    fn default() -> Self {
        Self {
            dopamine_level: 0.5,
            attention_focus: Vec::new(),
            reward_prediction: 0.0,
            exploration_factor: 0.1,
        }
    }
}

/// Unified Salience and Mesolimbic System
pub struct UnifiedSalienceSystem {
    config: SalienceConfig,
    state: MesolimbicState,
    token_history: HashMap<u32, Vec<f32>>,
    phoneme_patterns: HashMap<u32, Vec<u32>>,
    role_mappings: HashMap<u32, String>,
}

impl UnifiedSalienceSystem {
    pub fn new(config: SalienceConfig) -> Self {
        Self {
            config,
            state: MesolimbicState::default(),
            token_history: HashMap::new(),
            phoneme_patterns: HashMap::new(),
            role_mappings: HashMap::new(),
        }
    }

    /// Compute salience scores for a batch of tokens
    pub fn compute_salience(&mut self, tokens: &[u32]) -> Result<Vec<SalienceResult>, SalienceError> {
        let mut results = Vec::with_capacity(tokens.len());
        
        for &token_id in tokens {
            let result = self.compute_token_salience(token_id)?;
            results.push(result);
        }

        // Update mesolimbic state based on results
        self.update_mesolimbic_state(&results);
        
        Ok(results)
    }

    fn compute_token_salience(&mut self, token_id: u32) -> Result<SalienceResult, SalienceError> {
        // Base salience computation
        let base_salience = self.compute_base_salience(token_id);
        
        // Phoneme preservation analysis
        let phoneme_preserved = if self.config.phoneme_preservation {
            self.analyze_phoneme_preservation(token_id)
        } else {
            true
        };

        // Foraging probability computation
        let foraging_probability = if self.config.enable_foraging {
            self.compute_foraging_probability(token_id)
        } else {
            0.5
        };

        // Role inference
        let role_inference = self.infer_token_role(token_id);

        // Confidence based on history and patterns
        let confidence = self.compute_confidence(token_id, base_salience);

        // Apply adaptive threshold
        let final_salience = if self.config.adaptive_threshold {
            self.apply_adaptive_threshold(base_salience, token_id)
        } else {
            base_salience
        };

        // Update token history
        self.update_token_history(token_id, final_salience);

        Ok(SalienceResult {
            token_id,
            salience_score: final_salience,
            confidence,
            phoneme_preserved,
            foraging_probability,
            role_inference,
        })
    }

    fn compute_base_salience(&self, token_id: u32) -> f32 {
        // Multi-factor salience computation
        let frequency_factor = self.compute_frequency_factor(token_id);
        let context_factor = self.compute_context_factor(token_id);
        let novelty_factor = self.compute_novelty_factor(token_id);
        let attention_factor = self.compute_attention_factor(token_id);

        // Weighted combination
        let salience = frequency_factor * 0.3 
                    + context_factor * 0.3 
                    + novelty_factor * 0.2 
                    + attention_factor * 0.2;

        salience.clamp(0.0, 1.0)
    }

    fn compute_frequency_factor(&self, token_id: u32) -> f32 {
        // Higher frequency = lower base salience (common words less salient)
        let history = self.token_history.get(&token_id);
        match history {
            Some(hist) if !hist.is_empty() => {
                let avg_occurrence = hist.len() as f32 / 1000.0; // Normalize
                (1.0 - avg_occurrence).max(0.1)
            }
            _ => 0.8 // New tokens are moderately salient
        }
    }

    fn compute_context_factor(&self, token_id: u32) -> f32 {
        // Context based on surrounding tokens in attention focus
        if self.state.attention_focus.contains(&token_id) {
            0.9
        } else {
            // Check if token is related to focused tokens
            let related_score = self.state.attention_focus.iter()
                .map(|&focus_token| self.compute_token_similarity(token_id, focus_token))
                .fold(0.0f32, |acc, sim| acc.max(sim));
            related_score * 0.7
        }
    }

    fn compute_novelty_factor(&self, token_id: u32) -> f32 {
        // Novel tokens (not seen recently) are more salient
        match self.token_history.get(&token_id) {
            Some(history) if !history.is_empty() => {
                let recent_occurrences = history.iter().rev().take(10).count();
                (10 - recent_occurrences) as f32 / 10.0
            }
            _ => 1.0 // Completely new tokens are highly novel
        }
    }

    fn compute_attention_factor(&self, token_id: u32) -> f32 {
        // Factor based on current mesolimbic state
        let dopamine_influence = (self.state.dopamine_level as f32).clamp(0.0, 1.0);
        let exploration_influence = (self.state.exploration_factor as f32).clamp(0.0, 1.0);
        
        // Tokens get attention boost based on dopamine and exploration
        let base_attention = 0.5;
        base_attention + (dopamine_influence * 0.3) + (exploration_influence * 0.2)
    }

    fn compute_token_similarity(&self, token1: u32, token2: u32) -> f32 {
        // Simplified similarity based on token ID proximity and patterns
        if token1 == token2 {
            return 1.0;
        }

        // Check phoneme patterns
        if let (Some(pattern1), Some(pattern2)) = (
            self.phoneme_patterns.get(&token1),
            self.phoneme_patterns.get(&token2)
        ) {
            let common_phonemes = pattern1.iter()
                .filter(|&p| pattern2.contains(p))
                .count();
            let total_phonemes = (pattern1.len() + pattern2.len()).max(1);
            (common_phonemes * 2) as f32 / total_phonemes as f32
        } else {
            // Fallback to ID-based similarity
            let diff = (token1 as i64 - token2 as i64).abs() as f32;
            (1.0 / (1.0 + diff / 1000.0)).clamp(0.0, 1.0)
        }
    }

    fn analyze_phoneme_preservation(&mut self, token_id: u32) -> bool {
        // Analyze if token preserves important phonemic information
        if let Some(pattern) = self.phoneme_patterns.get(&token_id) {
            // Check if pattern contains critical phonemes
            let critical_phonemes = [1, 2, 3, 5, 8, 13]; // Example critical phoneme IDs
            pattern.iter().any(|&p| critical_phonemes.contains(&p))
        } else {
            // Generate and store phoneme pattern for new token
            let pattern = self.generate_phoneme_pattern(token_id);
            let preserved = pattern.len() > 2; // Simple heuristic
            self.phoneme_patterns.insert(token_id, pattern);
            preserved
        }
    }

    fn generate_phoneme_pattern(&self, token_id: u32) -> Vec<u32> {
        // Generate phoneme pattern based on token characteristics
        let mut pattern = Vec::new();
        let mut id = token_id;
        
        // Extract phoneme-like features from token ID
        while id > 0 && pattern.len() < 5 {
            pattern.push(id % 20); // 20 possible phoneme types
            id /= 20;
        }
        
        if pattern.is_empty() {
            pattern.push(0); // Default phoneme
        }
        
        pattern
    }

    fn compute_foraging_probability(&self, token_id: u32) -> f32 {
        // Two-loop stochastic foraging search
        let mut total_probability = 0.0;
        
        // Outer loop
        for _ in 0..self.config.outer_loop_iterations {
            let mut inner_probability = 0.0;
            
            // Inner loop
            for _ in 0..self.config.inner_loop_iterations {
                let exploration_reward = self.compute_exploration_reward(token_id);
                let exploitation_reward = self.compute_exploitation_reward(token_id);
                
                // Balance exploration vs exploitation
                let probability = exploration_reward * self.state.exploration_factor as f32
                               + exploitation_reward * (1.0 - self.state.exploration_factor as f32);
                
                inner_probability += probability;
            }
            
            total_probability += inner_probability / self.config.inner_loop_iterations as f32;
        }
        
        (total_probability / self.config.outer_loop_iterations as f32).clamp(0.0, 1.0)
    }

    fn compute_exploration_reward(&self, token_id: u32) -> f32 {
        // Reward for exploring new or rare tokens
        let novelty = self.compute_novelty_factor(token_id);
        let uncertainty = 1.0 - self.compute_confidence(token_id, 0.5);
        (novelty + uncertainty) / 2.0
    }

    fn compute_exploitation_reward(&self, token_id: u32) -> f32 {
        // Reward for exploiting known valuable tokens
        let history_value = self.token_history.get(&token_id)
            .map(|hist| hist.iter().sum::<f32>() / hist.len() as f32)
            .unwrap_or(0.5);
        
        let attention_value = if self.state.attention_focus.contains(&token_id) { 0.8 } else { 0.2 };
        
        (history_value + attention_value) / 2.0
    }

    fn infer_token_role(&mut self, token_id: u32) -> Option<String> {
        // Infer semantic role of token based on patterns and context
        if let Some(existing_role) = self.role_mappings.get(&token_id) {
            return Some(existing_role.clone());
        }

        // Role inference based on token characteristics
        let role = if token_id < 100 {
            "function_word"
        } else if token_id < 1000 {
            "content_word"
        } else if token_id < 10000 {
            "domain_specific"
        } else {
            "rare_token"
        };

        let role_string = role.to_string();
        self.role_mappings.insert(token_id, role_string.clone());
        Some(role_string)
    }

    fn compute_confidence(&self, token_id: u32, salience: f32) -> f32 {
        // Confidence based on historical consistency and current state
        let history_consistency = self.token_history.get(&token_id)
            .map(|hist| {
                if hist.len() < 2 {
                    0.5
                } else {
                    let variance = self.compute_variance(hist);
                    (1.0 - variance).clamp(0.0, 1.0)
                }
            })
            .unwrap_or(0.3);

        let state_confidence = (self.state.dopamine_level as f32).clamp(0.0, 1.0);
        let salience_confidence = salience;

        (history_consistency + state_confidence + salience_confidence) / 3.0
    }

    fn compute_variance(&self, values: &[f32]) -> f32 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        variance
    }

    fn apply_adaptive_threshold(&mut self, salience: f32, token_id: u32) -> f32 {
        // Adaptive threshold based on recent performance and context
        let base_threshold = self.config.threshold as f32;
        
        // Adjust threshold based on recent salience distribution
        let recent_avg = self.compute_recent_average_salience();
        let adaptive_threshold = if recent_avg > base_threshold {
            base_threshold * 1.1 // Raise threshold if recent salience is high
        } else {
            base_threshold * 0.9 // Lower threshold if recent salience is low
        };

        // Apply threshold with some smoothing
        if salience > adaptive_threshold {
            salience
        } else {
            salience * 0.8 // Reduce salience below threshold
        }
    }

    fn compute_recent_average_salience(&self) -> f32 {
        let recent_values: Vec<f32> = self.token_history.values()
            .filter_map(|hist| hist.last().copied())
            .collect();
        
        if recent_values.is_empty() {
            0.5
        } else {
            recent_values.iter().sum::<f32>() / recent_values.len() as f32
        }
    }

    fn update_token_history(&mut self, token_id: u32, salience: f32) {
        let history = self.token_history.entry(token_id).or_insert_with(Vec::new);
        history.push(salience);
        
        // Keep only recent history (last 100 entries)
        if history.len() > 100 {
            history.remove(0);
        }
    }

    fn update_mesolimbic_state(&mut self, results: &[SalienceResult]) {
        // Update dopamine level based on salience results
        let avg_salience = results.iter().map(|r| r.salience_score).sum::<f32>() / results.len() as f32;
        let dopamine_delta = (avg_salience - 0.5) as f64 * self.config.learning_rate;
        self.state.dopamine_level = (self.state.dopamine_level + dopamine_delta).clamp(0.0, 1.0);

        // Update attention focus with high-salience tokens
        self.state.attention_focus.clear();
        for result in results {
            if result.salience_score > self.config.threshold as f32 {
                self.state.attention_focus.push(result.token_id);
            }
        }

        // Update exploration factor based on reward prediction
        let current_reward = avg_salience as f64;
        let prediction_error = current_reward - self.state.reward_prediction;
        self.state.reward_prediction += self.config.learning_rate * prediction_error;
        
        // Adjust exploration based on prediction accuracy
        if prediction_error.abs() > 0.1 {
            self.state.exploration_factor = (self.state.exploration_factor + 0.01).min(0.3);
        } else {
            self.state.exploration_factor = (self.state.exploration_factor - 0.01).max(0.05);
        }
    }

    /// Get current mesolimbic state
    pub fn get_state(&self) -> &MesolimbicState {
        &self.state
    }

    /// Reset the system state
    pub fn reset(&mut self) {
        self.state = MesolimbicState::default();
        self.token_history.clear();
        self.phoneme_patterns.clear();
        self.role_mappings.clear();
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SalienceConfig) {
        self.config = config;
    }
}

/// Factory function to create salience system instances
pub fn create_salience_system(config: SalienceConfig) -> UnifiedSalienceSystem {
    UnifiedSalienceSystem::new(config)
}

/// Convenience function for quick salience computation
pub fn compute_token_salience(tokens: &[u32]) -> Result<Vec<SalienceResult>, SalienceError> {
    let mut system = UnifiedSalienceSystem::new(SalienceConfig::default());
    system.compute_salience(tokens)
}
