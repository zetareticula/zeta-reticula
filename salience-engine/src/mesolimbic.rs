use rand::Rng;
use crate::role_inference::{RoleInferer, RoleInferenceResult};
use std::sync::Arc;
use dashmap::DashMap;

use rand_distr::{Distribution, Normal};

// Represents the result of a role inference
#[derive(Serialize, Deserialize, Clone)]
pub struct RoleInferenceResult {
    pub token_id: u32,
    pub inferred_role: String, // e.g., "subject", "verb", "object", "modifier", "negation"
    pub confidence: f32, // Confidence score for the inferred role
}

// RoleTheory represents a theory of roles and their probabilities
// in the context of a token's features
// This is used to dynamically infer roles based on token features
// and update the probabilities based on observed data.
#[derive(Serialize, Deserialize, Clone)]
pub struct RoleTheory {
    pub roles: Vec<String>,          // Possible roles (e.g., "subject", "verb", "object", "modifier", "negation")
    pub probabilities: Vec<Vec<f32>>, // P(role | token_features)
}

// Represents a token's features relevant to salience
#[derive(Serialize, Deserialize, Clone)]
pub struct TokenFeatures {
    pub token_id: u32,
    pub frequency: f32,
    pub sentiment_score: f32,
    pub context_relevance: f32,
    pub role: String,  // Now dynamically inferred
}

// Represents the result of salience computation for a token
#[derive(Serialize, Deserialize)]
pub struct SalienceResult {
    pub token_id: u32,
    pub salience_score: f32,
    pub role: String,
    pub role_confidence: f32,
}

pub struct MesolimbicSystem {
    role_inferer: Arc<RoleInferer>,
}

impl MesolimbicSystem {
    pub fn new() -> Self {
        MesolimbicSystem {
            role_inferer: Arc::new(RoleInferer::new(10, 5)), // 10 outer, 5 inner iterations
        }
    }

    pub fn compute_salience(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<SalienceResult> {
        // Infer roles
        let role_results = self.role_inferer.infer_roles(features.clone(), theory_key);

        // Compute salience for each token
        features.into_iter().zip(role_results).map(|(feature, role_result)| {
            let mut rng = rand::thread_rng();

            let striatum_contribution = feature.frequency * 0.3;
            let amygdala_contribution = feature.sentiment_score.abs() * 0.25;
            let hippocampus_contribution = feature.context_relevance * 0.2;
            let parahippocampal_contribution = hippocampus_contribution * 0.1;
            let acc_contribution = rng.gen_range(-0.05..0.05);
            let insula_contribution = amygdala_contribution * 0.15;

            // Role-based modulation: e.g., "negation" increases salience
            let role_modulation = match role_result.inferred_role.as_str() {
                "negation" => 0.2,
                "subject" => 0.15,
                "object" => 0.1,
                _ => 0.0,
            };

            let salience_score = (striatum_contribution +
                                 amygdala_contribution +
                                 hippocampus_contribution +
                                 parahippocampal_contribution +
                                 acc_contribution +
                                 insula_contribution +
                                 role_modulation)
                .clamp(0.0, 1.0);

            SalienceResult {
                token_id: feature.token_id,
                salience_score,
                role: role_result.inferred_role,
                role_confidence: role_result.confidence,
            }
        }).collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MesolimbicConfig {
    pub outer_loop_iterations: usize,
    pub inner_loop_iterations: usize,
}

impl MesolimbicConfig {
    pub fn new(outer_loop_iterations: usize, inner_loop_iterations: usize) -> Self {
        MesolimbicConfig {
            outer_loop_iterations,
            inner_loop_iterations,
        }
    }
}

// RoleInferer is responsible for inferring roles based on token features
impl RoleInferer {
    pub fn new(outer_loop_iterations: usize, inner_loop_iterations: usize) -> Self {
        let theories = DashMap::new();
        // Initialize with a default theory
        theories.insert("default".to_string(), RoleTheory {
            roles: vec!["subject".to_string(), "verb".to_string(), "object".to_string(), "modifier".to_string(), "negation".to_string()],
            probabilities: vec![vec![0.2; 5]; 5], // Uniform distribution initially
        });
        RoleInferer {
            theories,
            outer_loop_iterations,
            inner_loop_iterations,
        }
    }

    // Stochastic search to infer roles
    pub fn infer_roles(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<RoleInferenceResult> {
        let mut rng = rand::thread_rng();
        let theory = self.theories.get(theory_key).unwrap_or_else(|| self.theories.get("default").unwrap()).clone();

        // Outer loop: Sample theories
        let mut best_theory = theory.clone();
        let mut best_likelihood = f32::NEG_INFINITY;

        for _ in 0..self.outer_loop_iterations {
            let mut candidate_theory = theory.clone();
            // Perturb probabilities (simplified perturbation)
            for probs in &mut candidate_theory.probabilities {
                let normal = Normal::new(0.0, 0.1).unwrap();
                for p in probs {
                    *p = (*p + normal.sample(&mut rng)).clamp(0.0, 1.0);
                }
                // Normalize
                let sum: f32 = probs.iter().sum();
                for p in probs {
                    *p /= sum;
                }
            }

            // Compute likelihood (placeholder)
            let likelihood = rng.gen_range(-1.0..1.0); // Replace with actual likelihood computation

            if likelihood > best_likelihood {
                best_likelihood = likelihood;
                best_theory = candidate_theory;
            }
        }

        // Inner loop: Infer roles based on the best theory
        features.into_iter().map(|feature| {
            let role_index = rng.gen_range(0..best_theory.roles.len());
            RoleInferenceResult {
                token_id: feature.token_id,
                inferred_role: best_theory.roles[role_index].clone(),
                confidence: best_theory.probabilities[role_index][role_index], // Placeholder for confidence
            }
        }).collect()
    }
}


