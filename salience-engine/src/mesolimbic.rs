use serde::{Serialize, Deserialize};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use dashmap::DashMap;
use std::sync::Arc;
use crate::role_inference::{RoleInferer, RoleInferenceResult};

#[derive(Serialize, Deserialize, Clone)]
pub struct RoleInferenceResult {
    pub token_id: u32,
    pub inferred_role: String,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoleTheory {
    pub roles: Vec<String>,
    pub probabilities: Vec<Vec<f32>>, // P(role | token_features)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenFeatures {
    pub token_id: u32,
    pub frequency: f32,
    pub sentiment_score: f32,
    pub context_relevance: f32,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct SalienceResult {
    pub token_id: u32,
    pub salience_score: f32,
    pub role: String,
    pub role_confidence: f32,
}

#[derive(Serialize, Deserialize)]
pub struct MesolimbicConfig {
    pub outer_loop_iterations: usize,
    pub inner_loop_iterations: usize,
}

impl MesolimbicConfig {
    pub fn new(outer_loop_iterations: usize, inner_loop_iterations: usize) -> Self {
        Self {
            outer_loop_iterations,
            inner_loop_iterations,
        }
    }
}

pub struct MesolimbicSystem {
    role_inferer: Arc<RoleInferer>,
}

impl MesolimbicSystem {
    pub fn with_config(config: MesolimbicConfig) -> Self {
        MesolimbicSystem {
            role_inferer: Arc::new(RoleInferer::new(
                config.outer_loop_iterations,
                config.inner_loop_iterations,
            )),
        }
    }

    pub fn compute_salience(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<SalienceResult> {
        let role_results = self.role_inferer.infer_roles(features.clone(), theory_key);

        features
            .into_iter()
            .zip(role_results)
            .map(|(feature, role_result)| {
                let mut rng = rand::thread_rng();

                let weights = [0.3, 0.25, 0.2, 0.1, 0.05, 0.15];
                let inputs = [
                    feature.frequency,
                    feature.sentiment_score.abs(),
                    feature.context_relevance,
                    feature.context_relevance * 0.1,
                    rng.gen_range(-0.05..0.05),
                    feature.sentiment_score.abs() * 0.15,
                ];

                let mut salience_score: f32 = weights
                    .iter()
                    .zip(inputs.iter())
                    .map(|(w, v)| w * v)
                    .sum();

                let role_modulation = match role_result.inferred_role.as_str() {
                    "negation" => 0.2,
                    "subject" => 0.15,
                    "object" => 0.1,
                    _ => 0.0,
                };

                salience_score += role_modulation;
                salience_score = salience_score.clamp(0.0, 1.0);

                SalienceResult {
                    token_id: feature.token_id,
                    salience_score,
                    role: role_result.inferred_role,
                    role_confidence: role_result.confidence,
                }
            })
            .collect()
    }
}

impl RoleInferer {
    pub fn new(outer_loop_iterations: usize, inner_loop_iterations: usize) -> Self {
        let theories = DashMap::new();
        theories.insert(
            "default".to_string(),
            RoleTheory {
                roles: vec![
                    "subject".to_string(),
                    "verb".to_string(),
                    "object".to_string(),
                    "modifier".to_string(),
                    "negation".to_string(),
                ],
                probabilities: vec![vec![0.2; 5]; 5],
            },
        );
        Self {
            theories,
            outer_loop_iterations,
            inner_loop_iterations,
        }
    }

    pub fn infer_roles(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<RoleInferenceResult> {
        let mut rng = rand::thread_rng();
        let theory = self
            .theories
            .get(theory_key)
            .unwrap_or_else(|| self.theories.get("default").unwrap())
            .clone();

        let mut best_theory = theory.clone();
        let mut best_likelihood = f32::NEG_INFINITY;

        let normal = Normal::new(0.0, 0.1).unwrap();

        for _ in 0..self.outer_loop_iterations {
            let mut candidate = theory.clone();

            for probs in &mut candidate.probabilities {
                let k = rng.gen_range(1..=probs.len());
                for _ in 0..k {
                    let i = rng.gen_range(0..probs.len());
                    probs[i] = (probs[i] + normal.sample(&mut rng)).clamp(0.0, 1.0);
                }
                probs.normalize();
            }

            let likelihood = rng.gen_range(-1.0..1.0); // Placeholder
            if likelihood > best_likelihood {
                best_theory = candidate;
                best_likelihood = likelihood;
            }
        }

        features
            .into_iter()
            .map(|feature| {
                let feature_vec = vec![
                    feature.frequency,
                    feature.sentiment_score,
                    feature.context_relevance,
                    feature.context_relevance * 0.1,
                    feature.sentiment_score * 0.15,
                ];

                let (role_index, confidence) = best_theory
                    .probabilities
                    .iter()
                    .enumerate()
                    .map(|(i, probs)| {
                        let sim = cosine_similarity(&feature_vec, probs);
                        (i, sim)
                    })
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap_or((0, 0.0));

                RoleInferenceResult {
                    token_id: feature.token_id,
                    inferred_role: best_theory.roles[role_index].clone(),
                    confidence: confidence.clamp(0.0, 1.0),
                }
            })
            .collect()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|y| y * y).sum::<f32>().sqrt();
    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

trait Normalize {
    fn normalize(&mut self);
}

impl Normalize for Vec<f32> {
    fn normalize(&mut self) {
        let sum: f32 = self.iter().sum();
        if sum > 0.0 {
            for val in self.iter_mut() {
                *val /= sum;
            }
        }
    }
}