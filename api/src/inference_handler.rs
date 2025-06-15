use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use llm_rs::inference::InferenceEngine;
use salience_engine::quantizer::{SalienceQuantizer, TokenFeatures, PrecisionLevel};
use salience_engine::tableaux::YoungTableau;
use tonic::transport::Channel;
use model_store::ModelStore;
use thiserror::Error;
use validator::Validate;

#[derive(Error, Debug)]
enum InferenceError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Validate, Deserialize)]
pub struct InferenceRequest {
    #[validate(length(min = 1, message = "Input cannot be empty"))]
    input: String,
    #[validate(length(min = 1, message = "Model name is required"))]
    model_name: String,
    #[validate(custom = "validate_precision")]
    precision: String,
}

fn validate_precision(precision: &str) -> Result<(), validator::ValidationError> {
    match precision {
        "4" | "8" | "16" => Ok(()),
        _ => Err(validator::ValidationError::new("Precision must be 4, 8, or 16")),
    }
}

#[derive(Serialize, Deserialize)]
pub struct InferenceResponse {
    text: String,
    tokens_processed: usize,
    latency_ms: f64,
}

pub struct InferenceHandler {
    engine: InferenceEngine,
    sidecar_client: pb::sidecar_service_client::SidecarServiceClient<Channel>,
    model_store: ModelStore,
}

impl InferenceHandler {
    pub async fn new(model_store: ModelStore) -> Self {
        let engine = InferenceEngine::new(768).await;
        let sidecar_client = pb::sidecar_service_client::SidecarServiceClient::connect("http://localhost:50051").await.unwrap();
        InferenceHandler { engine, sidecar_client, model_store }
    }

    pub async fn infer(&self, req: web::Json<InferenceRequest>) -> Result<HttpResponse, actix_web::Error> {
        req.validate().map_err(|e| actix_web::error::ErrorBadRequest(e))?;

        let model_path = self.model_store.get_model_path(&req.model_name)
            .ok_or_else(|| InferenceError::ModelNotFound(req.model_name.clone()))?;
        self.engine.model.load_from_flash(&model_path).await.map_err(InferenceError::Io)?;

        let quantizer = SalienceQuantizer::new(0.7);
        let tokens: Vec<&str> = req.input.split_whitespace().collect();
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

        let precision = match req.precision.as_str() {
            "4" => PrecisionLevel::Bit4,
            "8" => PrecisionLevel::Bit8,
            "16" => PrecisionLevel::Bit16,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid precision")),
        };
        for result in &mut results {
            result.precision = precision.clone();
        }

        tableau.cache_to_sidecar(&mut self.sidecar_client.clone()).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let routing_plan = ns_router_rs::NSRoutingPlan {
            model_config: ns_router_rs::ModelConfig { precision: results.clone() },
            kv_cache_config: ns_router_rs::KVCacheConfig { sparsity: 0.5, priority_tokens: vec![] },
        };
        let output = self.engine.infer(&req.input, &routing_plan).await;

        Ok(HttpResponse::Ok().json(InferenceResponse {
            text: output.text,
            tokens_processed: output.tokens_processed,
            latency_ms: output.latency_ms,
        }))
    }
}