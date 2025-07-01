use shared::{QuantizationResult, PrecisionLevel};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NSContextAnalysis {
    pub token_count: usize,
    pub salience_profile: Vec<QuantizationResult>,
    pub theory_complexity: f32,
    pub symbolic_constraints: Vec<String>,  // e.g., "subjects > modifiers in salience"
}

pub struct NSContextAnalyzer;

impl NSContextAnalyzer {
    pub fn new() -> Self {
        NSContextAnalyzer
    }

    pub fn analyze(&self, input: &str, quantization_results: Vec<QuantizationResult>) -> NSContextAnalysis {
        let token_count = input.split_whitespace().count();
        let salience_profile = quantization_results;
        let theory_complexity = salience_profile.iter()
            .map(|r| if matches!(r.precision, PrecisionLevel::Bit16) { 1.0 } else { 0.5 })
            .sum::<f32>() / token_count as f32;

        // Simplified symbolic constraints (in production, derive from theory)
        let symbolic_constraints = vec![
            "subjects > modifiers in salience".to_string(),
            "negations require Bit16 precision".to_string(),
        ];

        NSContextAnalysis {
            token_count,
            salience_profile,
            theory_complexity,
            symbolic_constraints,
        }
    }
}