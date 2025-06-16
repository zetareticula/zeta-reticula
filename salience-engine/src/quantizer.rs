use serde::{Serialize, Deserialize};
use crate::tableaux::YoungTableau;
use crate::quantizer::{QuantizationResult, PrecisionLevel};
use std::sync::Arc;
use dashmap::DashMap;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::RwLock;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicI32;

use crate::tableaux::YoungTableau;
use crate::role_inference::{RoleInferer, RoleInferenceResult};
use crate::mesolimbic::{MesolimbicSystem, SalienceResult};
use crate::role_inference::RoleTheory;



#[derive(Serialize, Deserialize, Clone)]
pub struct TokenFeatures {
    pub token_id: u32,
    pub frequency: f32,
    pub sentiment_score: f32,
    pub context_relevance: f32,
    pub role: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QuantizationResult {
    pub token_id: u32,
    pub precision: PrecisionLevel,
    pub salience_score: f32,
    pub row: usize,
    pub role: String,
    pub role_confidence: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum PrecisionLevel {
    Bit4,
    Bit8,
    Bit16,
}

pub struct SalienceQuantizer {
    threshold: f32,
}

impl SalienceQuantizer {
    pub fn new(threshold: f32) -> Self {
        SalienceQuantizer { threshold }
    }

    pub fn quantize_tokens(&self, features: Vec<TokenFeatures>, theory_key: &str) -> (Vec<QuantizationResult>, YoungTableau) {
        let mut results = Vec::new();
        let dimensions = (features.len() as f32).sqrt().ceil() as usize;
        let mut tableau = YoungTableau::new(dimensions, self.threshold);

        for (i, feature) in features.iter().enumerate() {
            let precision = if feature.context_relevance > 0.8 {
                PrecisionLevel::Bit16
            } else if feature.context_relevance > 0.5 {
                PrecisionLevel::Bit8
            } else {
                PrecisionLevel::Bit4
            };
            let salience_score = feature.context_relevance * feature.frequency;
            results.push(QuantizationResult {
                token_id: feature.token_id,
                precision,
                salience_score,
                row: i,
                role: feature.role.clone(),
                role_confidence: 0.9, // Mock
            });
        }

        tableau = YoungTableau::from_quantization_results(&results, dimensions);
        tableau.sparsify();
        (results, tableau)
    }
}