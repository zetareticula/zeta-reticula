use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use std::sync::Arc;

// Role inference state
#[derive(Serialize, Deserialize, Clone)]
pub struct RoleTheory {
    pub roles: Vec<String>,          // Possible roles (e.g., "subject", "negation")
    pub probabilities: Vec<Vec<f32>>, // P(role | token_features)
}

#[derive(Serialize, Deserialize)]
pub struct RoleInferenceResult {
    pub token_id: u32,
    pub inferred_role: String,
    pub confidence: f32,
}

pub struct RoleInferer {
    theories: DashMap<String, RoleTheory>, // Concurrent theory storage
    outer_loop_iterations: usize,
    inner_loop_iterations: usize,
}

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

            // Inner loop: Sample models (role assignments)
            let mut best_model: Vec<(usize, f32)> = vec![(0, 0.0); features.len()];
            let mut best_model_likelihood = f32::NEG_INFINITY;

            for _ in 0..self.inner_loop_iterations {
                let mut model: Vec<(usize, f32)> = vec![];
                let mut likelihood = 0.0;

                for (i, feature) in features.iter().enumerate() {
                    let role_probs = &candidate_theory.probabilities[i % candidate_theory.roles.len()];
                    let role_idx = self.sample_role(role_probs, &mut rng);
                    let confidence = role_probs[role_idx];
                    likelihood += confidence.ln();
                    model.push((role_idx, confidence));
                }

                if likelihood > best_model_likelihood {
                    best_model_likelihood = likelihood;
                    best_model = model;
                }
            }

            if best_model_likelihood > best_likelihood {
                best_likelihood = best_model_likelihood;
                best_theory = candidate_theory;
            }
        }

        // Update the theory in the DashMap
        self.theories.insert(theory_key.to_string(), best_theory.clone());

        // Map results
        features.into_iter().enumerate().map(|(i, feature)| {
            let (role_idx, confidence) = best_model[i];
            RoleInferenceResult {
                token_id: feature.token_id,
                inferred_role: best_theory.roles[role_idx].clone(),
                confidence,
            }
        }).collect()
    }

    fn sample_role(&self, probabilities: &[f32], rng: &mut impl Rng) -> usize {
        let mut sum = 0.0;
        let r: f32 = rng.gen();
        for (i, &p) in probabilities.iter().enumerate() {
            sum += p;
            if r <= sum {
                return i;
            }
        }
        probabilities.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_inference() {
        let inferer = RoleInferer::new(10, 5);
        let features = vec![
            TokenFeatures {
                token_id: 0,
                frequency: 0.8,
                sentiment_score: 0.9,
                context_relevance: 0.7,
                role: "".to_string(), // Will be inferred
            },
            TokenFeatures {
                token_id: 1,
                frequency: 0.3,
                sentiment_score: 0.2,
                context_relevance: 0.4,
                role: "".to_string(),
            },
        ];

        let results = inferer.infer_roles(features, "default");
        assert_eq!(results.len(), 2);
        assert!(results[0].confidence > 0.0);
        assert!(results[0].inferred_role == "subject" || results[0].inferred_role == "negation"); // Likely roles
    }
}