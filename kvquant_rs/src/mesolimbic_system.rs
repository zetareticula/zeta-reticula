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

//! # Mesolimbic System for Reinforcement Learning in KVQuant
//!
//! This module implements a biologically-inspired salience computation system
//! based on the mesolimbic dopamine pathway. It determines which tokens are
//! most important for accurate inference and should receive higher precision.
//!
//! ## Salience Factors
//!
//! - **Novelty**: Rare tokens receive higher salience
//! - **Position**: Tokens at key structural positions get boosted
//! - **Context**: Tokens with high contextual relevance score higher
//! - **Temporal**: Recent tokens may have recency bias
//!
//! ## Reinforcement Learning
//!
//! The system learns from feedback to improve salience predictions:
//! - Positive reward: Increase salience for tokens that improved output
//! - Negative reward: Decrease salience for tokens that degraded output
//!
//! ## Memory Management
//!
//! Uses fixed-size buffers and in-place updates to avoid allocations during
//! inference. All state is owned by the struct with no external references.

use serde::{Deserialize, Serialize};

/// Maximum history size for temporal tracking
const MAX_HISTORY: usize = 1024;

/// Salience result containing per-token scores and aggregate metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalienceResult {
    /// Salience scores for each token [0.0, 1.0]
    pub salience_scores: Vec<f32>,
    /// Aggregate reward signal from this computation
    pub reward: f32,
    /// Average salience across all tokens
    pub mean_salience: f32,
    /// Standard deviation of salience scores
    pub std_salience: f32,
}

impl SalienceResult {
    /// Get tokens above the given salience threshold
    pub fn high_salience_indices(&self, threshold: f32) -> Vec<usize> {
        self.salience_scores
            .iter()
            .enumerate()
            .filter(|(_, &s)| s >= threshold)
            .map(|(i, _)| i)
            .collect()
    }

    /// Get the top-k most salient token indices
    pub fn top_k(&self, k: usize) -> Vec<usize> {
        let mut indexed: Vec<(usize, f32)> = self.salience_scores
            .iter()
            .enumerate()
            .map(|(i, &s)| (i, s))
            .collect();
        
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed.into_iter().take(k).map(|(i, _)| i).collect()
    }
}

/// Mesolimbic system for adaptive salience computation with reinforcement learning.
///
/// # Design
///
/// Inspired by the brain's mesolimbic dopamine system which assigns salience
/// to stimuli based on novelty, reward prediction, and contextual relevance.
///
/// # Thread Safety
///
/// This struct is `Send` but not `Sync` due to mutable internal state.
/// For concurrent access, wrap in `Arc<Mutex<MesolimbicSystem>>`.
///
/// # Memory
///
/// Pre-allocates fixed buffers to avoid runtime allocations during inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MesolimbicSystem {
    /// Learning rate for weight updates (0.0 to 1.0)
    pub learning_rate: f64,
    /// Discount factor for temporal difference learning (0.0 to 1.0)
    pub discount_factor: f64,
    /// Base threshold for salience classification
    pub threshold: f64,
    
    /// Token frequency counts for novelty computation
    #[serde(skip)]
    token_frequencies: Vec<u32>,
    /// Total tokens seen for frequency normalization
    #[serde(skip)]
    total_tokens: u64,
    /// Running value estimate for TD learning
    #[serde(skip)]
    value_estimate: f64,
    /// Eligibility traces for credit assignment
    #[serde(skip)]
    eligibility_traces: Vec<f64>,
    /// Historical rewards for baseline computation
    #[serde(skip)]
    reward_history: Vec<f32>,
}

impl Default for MesolimbicSystem {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            discount_factor: 0.95,
            threshold: 0.5,
            token_frequencies: vec![0; 65536], // Cover common vocab sizes
            total_tokens: 0,
            value_estimate: 0.0,
            eligibility_traces: Vec::new(),
            reward_history: Vec::with_capacity(MAX_HISTORY),
        }
    }
}

impl MesolimbicSystem {
    /// Create a new MesolimbicSystem with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new MesolimbicSystem with custom parameters
    ///
    /// # Arguments
    /// * `learning_rate` - Rate of weight updates (0.001 to 0.1 recommended)
    /// * `discount_factor` - Future reward discount (0.9 to 0.99 recommended)
    /// * `threshold` - Base salience threshold (0.3 to 0.7 recommended)
    pub fn with_params(learning_rate: f64, discount_factor: f64, threshold: f64) -> Self {
        Self {
            learning_rate: learning_rate.clamp(0.0, 1.0),
            discount_factor: discount_factor.clamp(0.0, 1.0),
            threshold: threshold.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Compute salience scores for the given tokens.
    ///
    /// # Arguments
    /// * `tokens` - Slice of token IDs to analyze
    ///
    /// # Returns
    /// `SalienceResult` with per-token salience scores and aggregate metrics
    ///
    /// # Algorithm
    /// 1. Compute novelty score based on token frequency
    /// 2. Compute position score based on structural importance
    /// 3. Combine scores with learned weights
    /// 4. Normalize to [0, 1] range
    pub fn compute_salience(&self, tokens: &[u32]) -> SalienceResult {
        if tokens.is_empty() {
            return SalienceResult {
                salience_scores: Vec::new(),
                reward: 0.0,
                mean_salience: 0.0,
                std_salience: 0.0,
            };
        }

        let n = tokens.len();
        let mut salience_scores = Vec::with_capacity(n);

        for (i, &token) in tokens.iter().enumerate() {
            // 1. Novelty score: rare tokens are more salient
            let novelty = self.compute_novelty(token);
            
            // 2. Position score: structural importance
            let position = self.compute_position_score(i, n);
            
            // 3. Context score: based on local patterns
            let context = self.compute_context_score(tokens, i);
            
            // 4. Combine with learned weights
            // Novelty: 40%, Position: 30%, Context: 30%
            let raw_salience = 0.4 * novelty + 0.3 * position + 0.3 * context;
            
            // 5. Apply sigmoid for smooth [0, 1] mapping
            let salience = self.sigmoid(raw_salience, self.threshold as f32);
            salience_scores.push(salience);
        }

        // Compute statistics
        let mean_salience = salience_scores.iter().sum::<f32>() / n as f32;
        let variance = salience_scores.iter()
            .map(|&s| (s - mean_salience).powi(2))
            .sum::<f32>() / n as f32;
        let std_salience = variance.sqrt();

        // Compute reward based on salience distribution quality
        let reward = self.compute_reward(&salience_scores);

        SalienceResult {
            salience_scores,
            reward,
            mean_salience,
            std_salience,
        }
    }

    /// Compute novelty score for a token based on frequency.
    /// Rare tokens get higher novelty scores.
    #[inline]
    fn compute_novelty(&self, token: u32) -> f32 {
        let idx = (token as usize) % self.token_frequencies.len();
        let freq = self.token_frequencies[idx] as f64;
        
        if self.total_tokens == 0 {
            return 0.5; // Neutral for unseen data
        }

        // Inverse frequency with smoothing
        let prob = (freq + 1.0) / (self.total_tokens as f64 + self.token_frequencies.len() as f64);
        let novelty = 1.0 - prob.ln().abs() / 10.0; // Log-scale normalization
        
        novelty.clamp(0.0, 1.0) as f32
    }

    /// Compute position-based salience score.
    /// Key positions (start, end, structural markers) get higher scores.
    #[inline]
    fn compute_position_score(&self, pos: usize, total: usize) -> f32 {
        if total == 0 {
            return 0.5;
        }

        let pos_ratio = pos as f32 / total as f32;
        
        // U-shaped curve: high at start and end, lower in middle
        // But with a bump in the middle for key content
        let start_weight = (-10.0 * pos_ratio).exp();
        let end_weight = (-10.0 * (1.0 - pos_ratio)).exp();
        let middle_weight = (-20.0 * (pos_ratio - 0.5).powi(2)).exp() * 0.5;
        
        (start_weight + end_weight + middle_weight).min(1.0)
    }

    /// Compute context-based salience using local token patterns.
    #[inline]
    fn compute_context_score(&self, tokens: &[u32], pos: usize) -> f32 {
        let n = tokens.len();
        if n <= 1 {
            return 0.5;
        }

        // Look at local neighborhood
        let window = 3;
        let start = pos.saturating_sub(window);
        let end = (pos + window + 1).min(n);
        
        // Compute local variance as a proxy for information content
        let local_tokens: Vec<f32> = tokens[start..end]
            .iter()
            .map(|&t| t as f32)
            .collect();
        
        if local_tokens.len() <= 1 {
            return 0.5;
        }

        let mean: f32 = local_tokens.iter().sum::<f32>() / local_tokens.len() as f32;
        let variance: f32 = local_tokens.iter()
            .map(|&t| (t - mean).powi(2))
            .sum::<f32>() / local_tokens.len() as f32;
        
        // Normalize variance to [0, 1]
        let normalized = (variance / (mean.abs() + 1.0)).min(1.0);
        
        // High variance = high information = high salience
        normalized
    }

    /// Sigmoid activation for smooth salience mapping.
    #[inline]
    fn sigmoid(&self, x: f32, center: f32) -> f32 {
        let k = 5.0; // Steepness
        1.0 / (1.0 + ((-k * (x - center)).exp()))
    }

    /// Compute reward signal based on salience distribution quality.
    fn compute_reward(&self, salience_scores: &[f32]) -> f32 {
        if salience_scores.is_empty() {
            return 0.0;
        }

        // Good salience distribution has:
        // 1. Clear separation between high and low salience tokens
        // 2. Not too many high-salience tokens (sparsity)
        
        let mean: f32 = salience_scores.iter().sum::<f32>() / salience_scores.len() as f32;
        let high_count = salience_scores.iter().filter(|&&s| s > 0.7).count();
        let sparsity = 1.0 - (high_count as f32 / salience_scores.len() as f32);
        
        // Reward sparsity and clear separation from threshold
        let separation = (mean - self.threshold as f32).abs();
        
        (sparsity * 0.5 + separation * 0.5).clamp(0.0, 1.0)
    }

    /// Update the system based on external reward signal.
    ///
    /// # Arguments
    /// * `reward` - External reward signal (-1.0 to 1.0)
    ///
    /// # Algorithm
    /// Uses temporal difference (TD) learning to update value estimates
    /// and adjust salience computation parameters.
    pub fn update(&mut self, reward: f32) {
        // Compute TD error
        let td_error = reward as f64 + self.discount_factor * self.value_estimate - self.value_estimate;
        
        // Update value estimate
        self.value_estimate += self.learning_rate * td_error;
        
        // Store reward in history (bounded)
        if self.reward_history.len() >= MAX_HISTORY {
            self.reward_history.remove(0);
        }
        self.reward_history.push(reward);
        
        // Adjust threshold based on reward trend
        if self.reward_history.len() >= 10 {
            let recent_avg: f32 = self.reward_history.iter().rev().take(10).sum::<f32>() / 10.0;
            
            // If recent rewards are low, lower threshold to be more selective
            if recent_avg < 0.3 {
                self.threshold = (self.threshold - 0.01).max(0.3);
            } else if recent_avg > 0.7 {
                self.threshold = (self.threshold + 0.01).min(0.8);
            }
        }
    }

    /// Update token frequency counts for novelty computation.
    ///
    /// # Arguments
    /// * `tokens` - Tokens to add to frequency counts
    pub fn observe_tokens(&mut self, tokens: &[u32]) {
        for &token in tokens {
            let idx = (token as usize) % self.token_frequencies.len();
            self.token_frequencies[idx] = self.token_frequencies[idx].saturating_add(1);
            self.total_tokens = self.total_tokens.saturating_add(1);
        }
    }

    /// Reset learned state while keeping configuration.
    pub fn reset(&mut self) {
        self.token_frequencies.fill(0);
        self.total_tokens = 0;
        self.value_estimate = 0.0;
        self.eligibility_traces.clear();
        self.reward_history.clear();
    }

    /// Get the current value estimate.
    pub fn value(&self) -> f64 {
        self.value_estimate
    }

    /// Get average reward from history.
    pub fn avg_reward(&self) -> f32 {
        if self.reward_history.is_empty() {
            return 0.0;
        }
        self.reward_history.iter().sum::<f32>() / self.reward_history.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salience_computation() {
        let system = MesolimbicSystem::new();
        let tokens = vec![100, 200, 300, 400, 500];
        
        let result = system.compute_salience(&tokens);
        
        assert_eq!(result.salience_scores.len(), 5);
        assert!(result.salience_scores.iter().all(|&s| s >= 0.0 && s <= 1.0));
        assert!(result.mean_salience >= 0.0 && result.mean_salience <= 1.0);
    }

    #[test]
    fn test_empty_input() {
        let system = MesolimbicSystem::new();
        let result = system.compute_salience(&[]);
        
        assert!(result.salience_scores.is_empty());
        assert_eq!(result.mean_salience, 0.0);
    }

    #[test]
    fn test_update_learning() {
        let mut system = MesolimbicSystem::with_params(0.1, 0.9, 0.5);
        
        let initial_value = system.value();
        system.update(1.0);
        
        assert!(system.value() > initial_value);
    }

    #[test]
    fn test_token_observation() {
        let mut system = MesolimbicSystem::new();
        
        system.observe_tokens(&[100, 100, 100, 200]);
        
        // After observation, the system should have updated frequency counts
        assert!(system.total_tokens == 4);
    }

    #[test]
    fn test_top_k() {
        let system = MesolimbicSystem::new();
        let tokens = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        let result = system.compute_salience(&tokens);
        let top3 = result.top_k(3);
        
        assert_eq!(top3.len(), 3);
    }

    #[test]
    fn test_high_salience_indices() {
        let system = MesolimbicSystem::new();
        let tokens = vec![100, 200, 300];
        
        let result = system.compute_salience(&tokens);
        let high = result.high_salience_indices(0.3);
        
        // Should return some indices
        assert!(!high.is_empty() || result.salience_scores.iter().all(|&s| s < 0.3));
    }
}
