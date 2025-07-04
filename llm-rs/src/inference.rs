use crate::model::Model;
use crate::kv_cache::KVCache;
use crate::fusion_anns::FusionANNS;
use crate::utils::measure_latency;
use shared::{QuantizationResult, PrecisionLevel};
use crate::token_features::TokenFeatures;
use ns_router_rs::{NSRoutingPlan, ModelConfig, KVCacheConfig};
use agentflow_rs::{initialize_agent_flow, AgentFlowConfig, FederatedANSS, DistributedCache, DistributedIO, FederatedRoleInference};
use ndarray::Array1;
use tonic::{transport::Channel, Request};
use log;

mod pb {
    tonic::include_proto!("sidecar"); // Generated from zeta-sidecar/proto/sidecar.proto
}

#[derive(Serialize, Deserialize)]
pub struct InferenceEngine {
    model: Model,
    kv_cache: KVCache,
    fusion_anns: FusionANNS,
    agent_flow_server: agentflow_rs::server::AgentFlowServer,
    quantization_results: Vec<QuantizationResult>,
    sidecar_client: pb::sidecar_service_client::SidecarServiceClient<Channel>,
    // Placeholder for future tableau functionality
    _tableau_placeholder: (),
}

impl InferenceEngine {
    pub async fn new(model_size: usize) -> Self {
        let model = Model::new(model_size, &[]);
        let kv_cache = KVCache::new(0.5, vec![]);
        let fusion_anns = FusionANNS::new(768, 100);
        let agent_flow_config = AgentFlowConfig { num_clients: 4, privacy_epsilon: 1.0 };
        let agent_flow_server = initialize_agent_flow(agent_flow_config);
        let sidecar_client = pb::sidecar_service_client::SidecarServiceClient::connect("http://localhost:50051").await.unwrap();
        // Placeholder for future tableau functionality
        let _tableau_placeholder = ();
        InferenceEngine {
            model,
            kv_cache,
            fusion_anns,
            agent_flow_server,
            quantization_results: vec![],
            sidecar_client,
            _tableau_placeholder,
        }
    }

    pub async fn infer(&mut self, input: &str, routing_plan: &NSRoutingPlan) -> InferenceOutput {
        log::info!("Starting inference for input: {}", input);

        self.agent_flow_server.initialize().await;
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

        let quantizer = SalienceQuantizer::new(0.7);
        let token_features: Vec<TokenFeatures> = tokens.iter()
            .enumerate()
            .map(|(i, _)| TokenFeatures {
                token_id: i as u32,
                frequency: 0.5,
                sentiment_score: 0.0,
                context_relevance: 0.5,
                role: "".to_string(),
            })
            .collect();
        let (results, mut tableau) = quantizer.quantize_tokens(token_features, "default");
        self.quantization_results = results;
        self.tableau = tableau;
        self.tableau.cache_to_sidecar(&mut self.sidecar_client).await.unwrap();

        let (processed_text, latency) = measure_latency(|| {
            let salience_scores: Vec<(u32, f32)> = self.quantization_results.iter()
                .map(|r| (r.token_id, r.salience_score))
                .collect();

            let mut embeddings = Array1::zeros(self.model.get_d_model()); // Use the getter method
            for (i, _) in tokens.iter().enumerate() {
                embeddings[i] = i as f32 * 0.1;
            }

            let federated_anss = FederatedANSS;
            let query = Array1::from_vec(vec![0.1; self.model.get_d_model()]); // Use the getter method
            let candidates = rt.block_on(federated_anss.search(&self.agent_flow_server, &query, 100)).unwrap();
            let ranked = rt.block_on(self.fusion_anns.heuristic_rerank(&query, candidates)).unwrap();

            let distributed_io = DistributedIO;
            let _data = rt.block_on(distributed_io.parallel_read(&self.agent_flow_server, 32 * 1024)).unwrap();

            let ffn_output = self.model.compute_ffn(&embeddings);

            let active_neurons: Vec<usize> = self.quantization_results.iter()
                .enumerate()
                .filter(|(_, r)| r.salience_score > 0.7)
                .map(|(i, _)| i)
                .collect();
            self.model.set_last_k_active(active_neurons.into_iter().take(10).collect());

            let inactive_neurons: Vec<usize> = self.quantization_results.iter()
                .enumerate()
                .filter(|(_, r)| r.salience_score < 0.3)
                .map(|(i, _)| self.model.get_pointer(i)) // Use a public method to access pointers
                .collect();
            let new_neurons = vec![self.model.get_num_used()]; // Use a public method to access num_used
            let weights = vec![0.1; self.model.get_d_model() * 2]; // Use the getter method
            let biases = vec![0.0; 1];
            self.model.add_neurons(&new_neurons, &weights, &biases);

            let distributed_cache = DistributedCache;
            for (i, token) in tokens.iter().enumerate() {
                let token_id = i as u32;
                let layer = 0;
                let value = ffn_output[i];
                let salience_score = self.quantization_results[i].salience_score;
                distributed_cache.update(
                    &self.agent_flow_server,
                    token_id,
                    value,
                    salience_score,
                    self.model.get_pointer(i), // Use a public method to access pointers
                    self.model.get_bias(i), // Use a public method to access biases
                    token_id,
                    (i, vec![i + 1, i + 2]),
                );
                output_text.push_str(token);
                if i < tokens.len() - 1 {
                    output_text.push(' ');
                }
            }

            distributed_cache.invalidate_low_salience(&self.agent_flow_server, &salience_scores);
            distributed_cache.erase_full_spots(&self.agent_flow_server);

            output_text.clone()
        });

        InferenceOutput {
            text: processed_text,
            tokens_processed: token_count,
            latency_ms: latency,
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct InferenceOutput {
    pub text: String,
    pub tokens_processed: usize,
    pub latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_inference_engine() {
        let rt = Runtime::new().unwrap();
        let mut engine = rt.block_on(InferenceEngine::new(1024));
        let routing_plan = NSRoutingPlan {
            model_config: ModelConfig {
                precision: vec![PrecisionLevel::FP32],
                d_model: 768,
            },
            kv_cache_config: KVCacheConfig {
                sparsity: 0.5,
                priority_tokens: vec![],
            },
        };
        let input = "Hello world";
        let output = rt.block_on(engine.infer(input, &routing_plan));
        assert_eq!(output.text, "Hello world");
        assert!(output.tokens_processed > 0);
        assert!(output.latency_ms > 0);
    }
}