use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::role_inference::{RoleInferer, RoleInferenceResult};

// Represents a token's features relevant to salience
#[derive(Serialize, Deserialize, Clone)]
pub struct TokenFeatures {
    pub token_id: u32,
    pub frequency: f32,
    pub sentiment_score: f32,
    pub context_relevance: f32,
    pub role: String,  // Now dynamically inferred
}

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