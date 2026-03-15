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

//! # Role Inference for KVQuant
//!
//! This module implements belief propagation-based role inference for tokens in LLM contexts.
//! Roles determine how tokens should be quantized based on their semantic importance.
//!
//! ## Role Categories
//!
//! - **Subject (0)**: Primary entities, high importance
//! - **Predicate (1)**: Actions/verbs, medium-high importance
//! - **Object (2)**: Secondary entities, medium importance
//! - **Modifier (3)**: Adjectives/adverbs, lower importance
//! - **Connector (4)**: Prepositions/conjunctions, lowest importance
//!
//! ## Algorithm
//!
//! Uses iterative belief propagation with configurable convergence thresholds.
//! The algorithm analyzes token position, frequency patterns, and contextual features.

use serde::{Deserialize, Serialize};

/// Number of distinct roles in the system
pub const NUM_ROLES: usize = 5;

/// Role identifiers with semantic meaning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TokenRole {
    /// Primary entity (nouns as subjects)
    Subject = 0,
    /// Action or state (verbs)
    Predicate = 1,
    /// Secondary entity (nouns as objects)
    Object = 2,
    /// Descriptive modifier (adjectives, adverbs)
    Modifier = 3,
    /// Structural connector (prepositions, conjunctions)
    Connector = 4,
}

impl TokenRole {
    /// Get the quantization priority for this role (higher = more important)
    #[inline]
    pub fn priority(&self) -> f32 {
        match self {
            TokenRole::Subject => 1.0,
            TokenRole::Predicate => 0.9,
            TokenRole::Object => 0.7,
            TokenRole::Modifier => 0.5,
            TokenRole::Connector => 0.3,
        }
    }

    /// Convert from role index
    #[inline]
    pub fn from_index(idx: usize) -> Self {
        match idx % NUM_ROLES {
            0 => TokenRole::Subject,
            1 => TokenRole::Predicate,
            2 => TokenRole::Object,
            3 => TokenRole::Modifier,
            _ => TokenRole::Connector,
        }
    }
}

/// Role inference result containing assignments and confidence scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInferenceResult {
    /// Role ID for each token (0-4)
    pub roles: Vec<usize>,
    /// Confidence score for each role assignment [0.0, 1.0]
    pub confidences: Vec<f32>,
}

impl RoleInferenceResult {
    /// Get the TokenRole for a specific position
    #[inline]
    pub fn get_role(&self, idx: usize) -> Option<TokenRole> {
        self.roles.get(idx).map(|&r| TokenRole::from_index(r))
    }

    /// Get average confidence across all assignments
    pub fn avg_confidence(&self) -> f32 {
        if self.confidences.is_empty() {
            return 0.0;
        }
        self.confidences.iter().sum::<f32>() / self.confidences.len() as f32
    }
}

/// Role inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInferenceRequest {
    /// Input tokens
    pub tokens: Vec<u32>,
    /// Optional role hints to guide inference
    pub role_hints: Option<Vec<usize>>,
}

/// Role inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInferenceResponse {
    /// Inference result
    pub result: RoleInferenceResult,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Role inferer using belief propagation for semantic role assignment.
///
/// # Algorithm Details
///
/// The inference uses a two-phase iterative approach:
/// 1. **Outer loop**: Updates global role distribution estimates
/// 2. **Inner loop**: Refines local token-role assignments using neighbor context
///
/// Convergence is determined by the threshold parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RoleInferer {
    /// Convergence threshold for belief propagation (lower = more precise)
    pub threshold: f64,
    /// Number of outer iterations for global updates
    pub outer_iterations: usize,
    /// Number of inner iterations for local refinement
    pub inner_iterations: usize,
    /// Belief matrix: beliefs[token_idx][role_idx] = probability
    #[serde(skip)]
    beliefs: Vec<[f64; NUM_ROLES]>,
}

impl Default for RoleInferer {
    fn default() -> Self {
        Self {
            threshold: 0.01,
            outer_iterations: 10,
            inner_iterations: 5,
            beliefs: Vec::new(),
        }
    }
}

impl RoleInferer {
    /// Create a new RoleInferer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new RoleInferer with custom iteration settings
    ///
    /// # Arguments
    /// * `threshold` - Convergence threshold (0.001 to 0.1 recommended)
    /// * `outer` - Outer loop iterations (5-20 recommended)
    /// * `inner` - Inner loop iterations (3-10 recommended)
    pub fn with_iterations(threshold: f64, outer: usize, inner: usize) -> Self {
        Self {
            threshold,
            outer_iterations: outer.max(1),
            inner_iterations: inner.max(1),
            beliefs: Vec::new(),
        }
    }

    /// Infer roles for the given tokens using belief propagation.
    ///
    /// # Arguments
    /// * `tokens` - Slice of token IDs to analyze
    ///
    /// # Returns
    /// `RoleInferenceResult` with role assignments and confidence scores
    ///
    /// # Algorithm
    /// 1. Initialize beliefs based on token features (position, value patterns)
    /// 2. Run belief propagation to refine assignments
    /// 3. Extract final roles from converged beliefs
    pub fn infer_roles(&self, tokens: &[u32]) -> RoleInferenceResult {
        if tokens.is_empty() {
            return RoleInferenceResult {
                roles: Vec::new(),
                confidences: Vec::new(),
            };
        }

        let n = tokens.len();
        
        // Initialize beliefs based on token features
        let mut beliefs = self.initialize_beliefs(tokens);
        
        // Run belief propagation
        self.propagate_beliefs(&mut beliefs, n);
        
        // Extract final roles and confidences
        self.extract_results(&beliefs)
    }

    /// Initialize belief matrix based on token features.
    /// Uses position-based heuristics and token value patterns.
    fn initialize_beliefs(&self, tokens: &[u32]) -> Vec<[f64; NUM_ROLES]> {
        let n = tokens.len();
        let mut beliefs = vec![[0.2; NUM_ROLES]; n]; // Uniform prior

        for (i, &token) in tokens.iter().enumerate() {
            let pos_ratio = i as f64 / n.max(1) as f64;
            
            // Position-based priors:
            // - Early tokens more likely to be subjects
            // - Middle tokens more likely to be predicates/objects
            // - Connectors distributed throughout
            
            if pos_ratio < 0.2 {
                // Beginning: favor subjects
                beliefs[i][TokenRole::Subject as usize] = 0.4;
                beliefs[i][TokenRole::Predicate as usize] = 0.25;
                beliefs[i][TokenRole::Object as usize] = 0.15;
                beliefs[i][TokenRole::Modifier as usize] = 0.1;
                beliefs[i][TokenRole::Connector as usize] = 0.1;
            } else if pos_ratio < 0.5 {
                // Early-middle: favor predicates
                beliefs[i][TokenRole::Subject as usize] = 0.15;
                beliefs[i][TokenRole::Predicate as usize] = 0.4;
                beliefs[i][TokenRole::Object as usize] = 0.2;
                beliefs[i][TokenRole::Modifier as usize] = 0.15;
                beliefs[i][TokenRole::Connector as usize] = 0.1;
            } else if pos_ratio < 0.8 {
                // Late-middle: favor objects
                beliefs[i][TokenRole::Subject as usize] = 0.1;
                beliefs[i][TokenRole::Predicate as usize] = 0.15;
                beliefs[i][TokenRole::Object as usize] = 0.4;
                beliefs[i][TokenRole::Modifier as usize] = 0.2;
                beliefs[i][TokenRole::Connector as usize] = 0.15;
            } else {
                // End: mixed, often modifiers
                beliefs[i][TokenRole::Subject as usize] = 0.1;
                beliefs[i][TokenRole::Predicate as usize] = 0.15;
                beliefs[i][TokenRole::Object as usize] = 0.2;
                beliefs[i][TokenRole::Modifier as usize] = 0.35;
                beliefs[i][TokenRole::Connector as usize] = 0.2;
            }

            // Token value-based adjustments
            // Common connector tokens (low values often represent punctuation/common words)
            if token < 100 {
                beliefs[i][TokenRole::Connector as usize] += 0.2;
                self.normalize_beliefs(&mut beliefs[i]);
            }
            // High token IDs often represent content words
            else if token > 10000 {
                beliefs[i][TokenRole::Subject as usize] += 0.1;
                beliefs[i][TokenRole::Object as usize] += 0.1;
                self.normalize_beliefs(&mut beliefs[i]);
            }
        }

        beliefs
    }

    /// Run belief propagation to refine role assignments.
    fn propagate_beliefs(&self, beliefs: &mut [[f64; NUM_ROLES]], n: usize) {
        if n <= 1 {
            return;
        }

        // Message buffers for belief propagation
        let mut messages_forward = vec![[0.0; NUM_ROLES]; n];
        let mut messages_backward = vec![[0.0; NUM_ROLES]; n];

        for _outer in 0..self.outer_iterations {
            let mut max_delta = 0.0;

            for _inner in 0..self.inner_iterations {
                // Forward pass: propagate beliefs left to right
                for i in 1..n {
                    for role in 0..NUM_ROLES {
                        // Transition probability based on role sequences
                        let transition = self.transition_prob(role, i, n);
                        messages_forward[i][role] = beliefs[i - 1][role] * transition;
                    }
                    self.normalize_beliefs(&mut messages_forward[i]);
                }

                // Backward pass: propagate beliefs right to left
                for i in (0..n - 1).rev() {
                    for role in 0..NUM_ROLES {
                        let transition = self.transition_prob(role, i, n);
                        messages_backward[i][role] = beliefs[i + 1][role] * transition;
                    }
                    self.normalize_beliefs(&mut messages_backward[i]);
                }

                // Update beliefs by combining messages
                for i in 0..n {
                    let old_beliefs = beliefs[i];
                    
                    for role in 0..NUM_ROLES {
                        let forward = if i > 0 { messages_forward[i][role] } else { 1.0 };
                        let backward = if i < n - 1 { messages_backward[i][role] } else { 1.0 };
                        
                        // Combine prior with messages
                        beliefs[i][role] *= forward * backward;
                    }
                    self.normalize_beliefs(&mut beliefs[i]);

                    // Track convergence
                    for role in 0..NUM_ROLES {
                        let delta = (beliefs[i][role] - old_beliefs[role]).abs();
                        max_delta = f64::max(max_delta, delta);
                    }
                }
            }

            // Check convergence
            if max_delta < self.threshold {
                break;
            }
        }
    }

    /// Compute transition probability for role at position.
    #[inline]
    fn transition_prob(&self, role: usize, pos: usize, total: usize) -> f64 {
        let pos_ratio = pos as f64 / total.max(1) as f64;
        
        // Role-specific position preferences
        match role {
            0 => 1.0 - pos_ratio * 0.5,           // Subject: prefer early
            1 => 1.0 - (pos_ratio - 0.3).abs(),   // Predicate: prefer early-middle
            2 => 1.0 - (pos_ratio - 0.6).abs(),   // Object: prefer late-middle
            3 => 0.5 + pos_ratio * 0.3,           // Modifier: slight late preference
            _ => 0.8,                              // Connector: uniform
        }
    }

    /// Normalize belief array to sum to 1.0
    #[inline]
    fn normalize_beliefs(&self, beliefs: &mut [f64; NUM_ROLES]) {
        let sum: f64 = beliefs.iter().sum();
        if sum > 0.0 {
            for b in beliefs.iter_mut() {
                *b /= sum;
            }
        } else {
            // Fallback to uniform
            for b in beliefs.iter_mut() {
                *b = 1.0 / NUM_ROLES as f64;
            }
        }
    }

    /// Extract final roles and confidences from converged beliefs.
    fn extract_results(&self, beliefs: &[[f64; NUM_ROLES]]) -> RoleInferenceResult {
        let mut roles = Vec::with_capacity(beliefs.len());
        let mut confidences = Vec::with_capacity(beliefs.len());

        for belief in beliefs {
            // Find role with maximum belief
            let (max_role, max_belief) = belief
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.2));

            roles.push(max_role);
            confidences.push(*max_belief as f32);
        }

        RoleInferenceResult { roles, confidences }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_inference_basic() {
        let inferer = RoleInferer::new();
        let tokens = vec![100, 200, 300, 400, 500];
        
        let result = inferer.infer_roles(&tokens);
        
        assert_eq!(result.roles.len(), 5);
        assert_eq!(result.confidences.len(), 5);
        assert!(result.confidences.iter().all(|&c| c >= 0.0 && c <= 1.0));
    }

    #[test]
    fn test_role_inference_empty() {
        let inferer = RoleInferer::new();
        let result = inferer.infer_roles(&[]);
        
        assert!(result.roles.is_empty());
        assert!(result.confidences.is_empty());
    }

    #[test]
    fn test_role_priority() {
        assert!(TokenRole::Subject.priority() > TokenRole::Connector.priority());
        assert!(TokenRole::Predicate.priority() > TokenRole::Modifier.priority());
    }

    #[test]
    fn test_custom_iterations() {
        let inferer = RoleInferer::with_iterations(0.001, 20, 10);
        let tokens = vec![1, 2, 3];
        
        let result = inferer.infer_roles(&tokens);
        assert_eq!(result.roles.len(), 3);
    }
}
