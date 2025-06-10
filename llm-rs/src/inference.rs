use crate::model::Model;
use crate::kv_cache::KVCache;
use crate::fusion_anns::FusionANNS;
use crate::utils::measure_latency;
use salience_engine::quantizer::QuantizationResult;
use ns_router_rs::{NSRoutingPlan, ModelConfig, KVCacheConfig};
use serde::{Serialize, Deserialize};
use ndarray::Array1;
use log;

#[derive(Serialize, Deserialize)]
pub struct InferenceEngine {
    model: Model,
    kv_cache: KVCache,
    fusion_anns: FusionANNS,
    quantization_results: Vec<QuantizationResult>,
}

impl InferenceEngine {
    pub fn new(model_size: usize) -> Self {
        let model = Model::new(model_size, &[]);
        let kv_cache = KVCache::new(0.5, vec![]);
        let fusion_anns = FusionANNS::new(768, 100); // Example dimension, batch size
        InferenceEngine {
            model,
            kv_cache,
            fusion_anns,
            quantization_results: vec![],
        }
    }

    pub async fn infer(&mut self, input: &str, routing_plan: &NSRoutingPlan) -> InferenceOutput {
        log::info!("Starting inference for input: {}", input);

        self.model.load_from_flash("model_weights.bin").await;
        self.fusion_anns.initialize().await;

        self.model.quantize(&routing_plan.model_config.precision);
        self.kv_cache = KVCache::new(
            routing_plan.kv_cache_config.sparsity,
            routing_plan.kv_cache_config.priority_tokens.clone(),
        );
        self.quantization_results = routing_plan.model_config.precision.clone();

        let tokens: Vec<&str> = input.split_whitespace().collect();
        let token_count = tokens.len();
        let mut output_text = String::new();

        let (processed_text, latency) = measure_latency(|| {
            let salience_scores: Vec<(u32, f32)> = self.quantization_results.iter()
                .map(|r| (r.token_id, r.salience_score))
                .collect();

            let mut embeddings = Array1::zeros(self.model.d_model);
            for (i, token) in tokens.iter().enumerate() {
                embeddings[i] = i as f32 * 0.1;
            }

            // Use FusionANNS to retrieve relevant embeddings
            let query = Array1::from_vec(vec![0.1; self.model.d_model]);
            let candidates = self.fusion_anns.collaborative_filter(&query, 100);
            let ranked = rt.block_on(self.fusion_anns.heuristic_rerank(&query, candidates)).unwrap();

            let ffn_output = self.model.compute_ffn(&embeddings);

            let active_neurons: Vec<usize> = self.quantization_results.iter()
                .enumerate()
                .filter(|(_, r)| r.salience_score > 0.7)
                .map(|(i, _)| i)
                .collect();
            self.model.last_k_active = active_neurons.into_iter().take(10).collect();

            let inactive_neurons: Vec<usize> = self.quantization_results.iter()
                .enumerate()
                .filter(|(_, r)| r.salience_score < 0.3)
                .map(|(i, _)| self.model.pointers[i])
                .collect();
            self.model.delete_neurons(&inactive_neurons);

            let new_neurons = vec![self.model.num_used];
            let weights = vec![0.1; self.model.d_model * 2];
            let biases = vec![0.0; 1];
            self.model.add_neurons(&new_neurons, &weights, &biases);

            for (i, token) in tokens.iter().enumerate() {
                let token_id = i as u32;
                let layer = 0;
                let value = ffn_output[i];
                let salience_score = self.quantization_results[i].salience_score;
                self.kv_cache.update(token_id, layer, value, salience_score, self.model.pointers[i], self.model.bias[i]);
                output_text.push_str(token);
                if i < tokens.len() - 1 {
                    output_text.push(' ');
                }
            }

            self.kv_cache.invalidate_low_salience(&salience_scores);
            self.kv_cache.erase_full_spots();

            output_text.clone()
        });

        InferenceOutput {
            text: processed_text,
            tokens_processed: token_count,
            latency_ms: latency,
        }
    }
}