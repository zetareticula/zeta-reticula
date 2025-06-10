use crate::mesolimbic::{MesolimbicSystem, TokenFeatures, SalienceResult};
use crate::tableaux::{YoungTableau, TokenRole};
use crate::optimization::SalienceOptimizer;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub enum PrecisionLevel {
    Bit4,
    Bit8,
    Bit16,
}

#[derive(Serialize, Deserialize)]
pub struct QuantizationResult {
    pub token_id: u32,
    pub precision: PrecisionLevel,
    pub salience_score: f32,
    pub row: usize,
    pub role: String,
    pub role_confidence: f32,
}

pub struct SalienceQuantizer {
    mesolimbic: Arc<MesolimbicSystem>,
    optimizer: Arc<SalienceOptimizer>,
    salience_threshold: f32,
}

impl SalienceQuantizer {
    pub fn new(salience_threshold: f32) -> Self {
        SalienceQuantizer {
            mesolimbic: Arc::new(MesolimbicSystem::new()),
            optimizer: Arc::new(SalienceOptimizer::new()),
            salience_threshold,
        }
    }

    pub fn quantize_tokens(&self, tokens: Vec<TokenFeatures>, theory_key: &str) -> (Vec<QuantizationResult>, YoungTableau) {
        // Compute salience with optimization
        let salience_results = self.mesolimbic.compute_salience(tokens.clone(), theory_key);

        let optimized_results = self.optimizer.optimize_computation(salience_results, |result| {
            (result.token_id, result.salience_score)
        });

        let token_roles: Vec<TokenRole> = optimized_results.iter()
            .map(|(result, _)| TokenRole {
                token_id: result.token_id,
                role: result.role.clone(),
                salience_score: result.salience_score,
            })
            .collect();

        let tableau = YoungTableau::from_tokens(token_roles);

        let quantization_results: Vec<QuantizationResult> = optimized_results.into_iter()
            .map(|(result, _)| {
                let row = tableau.get_row(result.token_id).unwrap_or(tableau.num_rows());
                let precision = if result.salience_score >= self.salience_threshold || row == 0 {
                    PrecisionLevel::Bit16
                } else if row < tableau.num_rows() / 2 {
                    PrecisionLevel::Bit8
                } else {
                    PrecisionLevel::Bit4
                };
                QuantizationResult {
                    token_id: result.token_id,
                    precision,
                    salience_score: result.salience_score,
                    row,
                    role: result.role,
                    role_confidence: result.role_confidence,
                }
            })
            .collect();

        (quantization_results, tableau)
    }
}