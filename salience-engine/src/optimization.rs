use dashmap::DashMap;
use rayon::prelude::*;
use serde_json::map;
use std::sync::Arc;
use rand_distr::{Distribution, Normal};
use bumpalo::Bump;
use serde::{Serialize, Deserialize};
use ndarray::s;
use ndarray::Array2;
use crate::role_inference::{RoleInferer, RoleInferenceResult};
use crate::quantization::{QuantizationResult, PrecisionLevel};


// ---- Salience Optimization Engine ----
// This module provides an optimized computation of salience scores for tokens
// using a Young tableau structure and parallel processing with caching.
// It leverages Gaussian weighting for frame-based convolution and supports dynamic role inference.
// It is designed to handle large-scale token processing efficiently.

#[derive(Serialize, Deserialize, Clone)]
pub struct SalienceEngine {
    pub role_inferer: Arc<RoleInferer>, // Role inference engine
    pub salience_optimizer: SalienceOptimizer, // Optimizer for salience computation
}

impl SalienceEngine {
    pub fn new(outer_iterations: usize, inner_iterations: usize) -> Self {
        SalienceEngine {
            role_inferer: Arc::new(RoleInferer::new(outer_iterations, inner_iterations)),
            salience_optimizer: SalienceOptimizer::new(),
        }
    }

    pub fn compute_salience(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<RoleInferenceResult> {
        // Infer roles using the role inferer
        self.role_inferer.infer_roles(features, theory_key)
    }
}

// ---- Data Structures ----
// Represents a token's features relevant to salience
#[derive(Serialize, Deserialize, Clone)]
pub struct TokenFeatures {
    pub token_id: u32,
    pub frequency: f32,
    pub sentiment_score: f32,
    pub context_relevance: f32,
    pub role: String, // Now dynamically inferred
}

// SalienceOptimizer optimizes the computation of salience scores for tokens
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]  
#[serde(deny_unknown_fields)]
#[derive(Debug, Clone)]
pub struct YoungTableau {
    pub rows: Vec<Vec<QuantizationResult>>, // Rows of the tableau
    pub dimensions: (usize, usize), // Dimensions of the tableau
    pub threshold: f32, // Threshold for salience
}
pub enum PrecisionLevel {
    Bit4,
    Bit8,
    Bit16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QuantizationResult {
    pub row: usize, // Row index in the tableau
    pub column: usize, // Column index in the tableau
    pub value: f32, // Value of the quantized result
    pub precision: PrecisionLevel, // Precision level of the quantization
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[derive(Debug, Clone)]
pub struct Frame<'a> {
    pub tokens: &'a [TokenFeatures], // Tokens in the frame
    pub aggregated_salience: f32, // Aggregated salience score for the frame
    pub frame_id: u32, // Unique identifier for the frame
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[derive(Debug, Clone)]
pub struct RoleInferer {
    pub outer_iterations: usize, // Number of outer iterations for role inference
    pub inner_iterations: usize, // Number of inner iterations for role inference
}

pub struct SalienceOptimizer {
    cache: Arc<DashMap<u32, f32>>, // Cache token salience scores
}

impl SalienceOptimizer {
    pub fn new() -> Self {
        SalienceOptimizer {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn optimize_computation<T, F>(&self, items: Vec<T>, compute: F) -> Vec<(T, f32)>
    where
        F: Fn(&T) -> (u32, f32) + Send + Sync,
        T: Send + Sync,
    {
        items.into_par_iter().map(|item| {
            let (id, salience) = compute(&item);
            if let Some(cached) = self.cache.get(&id) {
                (item, *cached)
            } else {
                self.cache.insert(id, salience);
                (item, salience)
            }
        }).collect()
    }
}

// ---- Role Inference ----
impl RoleInferer {
    pub fn new(outer_iterations: usize, inner_iterations: usize) -> Self {
        RoleInferer {
            outer_iterations,
            inner_iterations,
        }
    }

    pub fn infer_roles(&self, features: Vec<TokenFeatures>, theory_key: &str) -> Vec<RoleInferenceResult> {
        // Mock role inference logic
        features.into_iter().map(|feature| {
            let inferred_role = match feature.role.as_str() {
                "negation" => "negation".to_string(),
                "subject" => "subject".to_string(),
                "object" => "object".to_string(),
                _ => "unknown".to_string(),
            };
            RoleInferenceResult {
                inferred_role,
                confidence: 0.9, // Mock confidence
            }
        }).collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoleInferenceResult {
    pub inferred_role: String, // Inferred role of the token
    pub confidence: f32, // Confidence score of the inference
}

// Example usage
#[derive(Serialize, Deserialize)]
pub struct Token {
    pub id: u32,
    pub features: Vec<f32>,
}

impl Token {
    pub fn new(id: u32, features: Vec<f32>) -> Self {
        Token { id, features }
    }
}

pub fn compute_token_salience(token: &Token) -> (u32, f32) {
    // Mock computation of salience score based on token features
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let salience_score: f32 = token.features.iter().map(|&f| f * normal.sample(&mut rng)).sum();
    (token.id, salience_score)
}

fn find_nearest_neighbors(&self, query: &Array1<f32>, list_id: u32, top_m: usize) -> Vec<u32> {
    let mut candidates = Vec::new();
    
    if let Some(ids) = self.vector_ids.get(&list_id) {
        for &id in ids.iter() {
            let vector = self.pq_vectors.slice(s![id as usize, ..]);
            let distance = vector.dot(query);
            candidates.push((id, distance));
        }
    }

    // Sort candidates by distance and return top-m
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    candidates.iter().take(top_m).map(|&(id, _)| id).collect()
}


impl PrecisionLevel {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bit4" => Some(PrecisionLevel::Bit4),
            "bit8" => Some(PrecisionLevel::Bit8),
            "bit16" => Some(PrecisionLevel::Bit16),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PrecisionLevel::Bit4 => "Bit4".to_string(),
            PrecisionLevel::Bit8 => "Bit8".to_string(),
            PrecisionLevel::Bit16 => "Bit16".to_string(),
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            4 => Some(PrecisionLevel::Bit4),
            8 => Some(PrecisionLevel::Bit8),
            16 => Some(PrecisionLevel::Bit16),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            PrecisionLevel::Bit4 => 4,
            PrecisionLevel::Bit8 => 8,
            PrecisionLevel::Bit16 => 16,
        }
    }

// ---- Young Tableau Implementation ----
impl YoungTableau {
    pub fn new(rows: usize, threshold: f32) -> Self {
        YoungTableau {
            rows: vec![Vec::with_capacity(10); rows],
            dimensions: (rows, 10),
            threshold,
        }
    }

    pub fn insert(&mut self, result: QuantizationResult) {
        if result.row < self.dimensions.0 {
            self.rows[result.row].push(result);
        }
    }

    pub fn compute_salience(&self) -> Vec<QuantizationResult> {
        // Compute salience scores for each row in the tableau
        self.rows.iter().flat_map(|row| {
            row.iter().filter(|&r| r.value >= self.threshold).cloned()
        }).collect()
    }
}

// ---- Frame-Based Convolution with Gaussian Weighting ----
impl<'a> Frame<'a> {
    pub fn new(frame_id: u32, tokens: &'a [TokenFeatures]) -> Self {
        Frame {
            tokens,
            aggregated_salience: 0.0,
            frame_id,
        }
    }

    pub fn compute_salience(&mut self, threshold: f32, bump: &Bump) {
        let normal = Normal::new(0.0, 1.0).unwrap(); // Gaussian with mean=0, std=1
        let mut weights = bumpalo::vec![in bump; 0.0; self.tokens.len()];
        
        // Compute Gaussian weights based on token position
        for (i, weight) in weights.iter_mut().enumerate() {
            let position = i as f32 / self.tokens.len() as f32; // Normalize position
            *weight = normal.pdf(position);
        }

        // Normalize weights to sum to 1
        let weight_sum: f32 = weights.iter().sum();
        if weight_sum > 0.0 {
            weights.iter_mut().for_each(|w| *w /= weight_sum);
        }

        // Compute weighted salience
        self.aggregated_salience = self.tokens.iter()
            .zip(weights.iter())
            .filter(|(t, _)| t.frequency >= threshold)
            .map(|(t, w)| t.frequency * w)
            .sum();
    }
}