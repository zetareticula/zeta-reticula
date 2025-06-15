use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use llm_rs::inference::InferenceEngine;
use salience_engine::quantizer::{SalienceQuantizer, TokenFeatures, PrecisionLevel};
use salience_engine::tableaux::YoungTableau;
use tonic::transport::Channel;
use model_store::ModelStore;
use thiserror::Error;
use validator::Validate;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::future_to_promise;
use ndarray::{Array2, array};
use half::f16;
use zeta_vault::{ZetaVault, VaultConfig, KVCache, CacheLayer};

#[derive(Error, Debug)]
enum InferenceError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Quantization error: {0}")]
    Quantization(String),
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
        "2" | "4" | "8" | "16" => Ok(()), // Added 2-bit support from KIVI
        _ => Err(validator::ValidationError::new("Precision must be 2, 4, 8, or 16")),
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
    #[cfg(feature = "server")]
    sidecar_client: pb::sidecar_service_client::SidecarServiceClient<Channel>,
    model_store: ModelStore,
}

impl InferenceHandler {
    pub async fn new(model_store: ModelStore) -> Self {
        let engine = InferenceEngine::new(768).await;
        #[cfg(feature = "server")]
        let sidecar_client = pb::sidecar_service_client::SidecarServiceClient::connect("http://localhost:50051").await.unwrap();
        InferenceHandler { engine, sidecar_client, model_store }
    }

    #[cfg(feature = "server")]
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
        let (mut results, mut tableau) = quantizer.quantize_tokens(token_features, "default");

        // KIVI 2-bit quantization
        let precision = match req.precision.as_str() {
            "2" => PrecisionLevel::Bit2, // Custom 2-bit precision
            "4" => PrecisionLevel::Bit4,
            "8" => PrecisionLevel::Bit8,
            "16" => PrecisionLevel::Bit16,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid precision")),
        };
        let (grouped_keys, residual_keys, grouped_values, residual_values) = self.apply_kivi_quantization(&mut results, &mut tableau, &precision)?;

        // Store quantized KV caches in ZetaVault
        let kv_cache = KVCache {
            key: bincode::serialize(&grouped_keys).unwrap(),
            value: bincode::serialize(&grouped_values).unwrap(),
            layer: CacheLayer::HBM,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        self.model_store.vault.store(format!("kv_grouped_{}", req.model_name), bincode::serialize(&kv_cache).unwrap()).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        self.model_store.vault.store(format!("kv_residual_{}", req.model_name), bincode::serialize(&(residual_keys, residual_values)).unwrap()).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

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

    fn apply_kivi_quantization(&self, results: &mut Vec<QuantizationResult>, tableau: &mut YoungTableau, precision: &PrecisionLevel) -> Result<(Array2<f16>, Array2<f32>, Array2<f16>, Array2<f32>), InferenceError> {
        let group_size = 16; // Arbitrary group size for KIVI
        let token_count = results.len();
        let dim = tableau.data.dim().0;

        // Split key cache (per-channel quantization)
        let mut grouped_keys = Array2::<f16>::zeros((token_count / group_size, dim));
        let mut residual_keys = Array2::<f32>::zeros(((token_count % group_size), dim));
        for i in 0..token_count {
            let row = i / group_size;
            let col = i % group_size;
            if row < grouped_keys.dim().0 {
                for d in 0..dim {
                    grouped_keys[[row, d]] = f16::from_f32(tableau.data[[i, d]]).quantize_2bit(); // Mock 2-bit quantization
                }
            } else {
                for d in 0..dim {
                    residual_keys[[col, d]] = tableau.data[[i, d]];
                }
            }
        }

        // Split value cache (per-token quantization)
        let mut grouped_values = Array2::<f16>::zeros((token_count / group_size, dim));
        let mut residual_values = Array2::<f32>::zeros(((token_count % group_size), dim));
        for i in 0..token_count {
            let row = i / group_size;
            let col = i % group_size;
            if row < grouped_values.dim().0 {
                for d in 0..dim {
                    grouped_values[[row, d]] = f16::from_f32(results[i].salience_score).quantize_2bit();
                }
            } else {
                for d in 0..dim {
                    residual_values[[col, d]] = results[i].salience_score;
                }
            }
        }

        // Tiled matrix multiplication for attention scores (hardware-friendly)
        let attention_scores = self.compute_attention_scores(&grouped_keys, &grouped_values, &residual_keys, &residual_values)?;
        tableau.data = attention_scores.mapv(|v| v as f32); // Update tableau with quantized scores

        Ok((grouped_keys, residual_keys, grouped_values, residual_values))
    }

    fn compute_attention_scores(&self, grouped_keys: &Array2<f16>, grouped_values: &Array2<f16>, residual_keys: &Array2<f32>, residual_values: &Array2<f32>) -> Result<Array2<f16>, InferenceError> {
        let mut scores = Array2::<f16>::zeros((grouped_keys.dim().0 + residual_keys.dim().0, grouped_values.dim().1));
        // Mock tiled multiplication (replace with optimized BLAS in production)
        for i in 0..grouped_keys.dim().0 {
            for j in 0..grouped_values.dim().1 {
                let mut sum = f16::from_f32(0.0);
                for d in 0..grouped_keys.dim().1 {
                    sum += grouped_keys[[i, d]] * grouped_values[[i, d]];
                }
                scores[[i, j]] = sum;
            }
        }
        for i in 0..residual_keys.dim().0 {
            for j in 0..residual_values.dim().1 {
                let mut sum = f16::from_f32(0.0);
                for d in 0..residual_keys.dim().1 {
                    sum += f16::from_f32(residual_keys[[i, d]]) * f16::from_f32(residual_values[[i, d]]);
                }
                scores[[grouped_keys.dim().0 + i, j]] = sum;
            }
        }
        Ok(scores)
    }

    #[cfg(feature = "wasm")]
    #[wasm_bindgen]
    pub fn infer_wasm(input: String, model_name: String, precision: String) -> js_sys::Promise {
        future_to_promise(async move {
            let model_store = ModelStore::new().await;
            let handler = InferenceHandler::new(model_store).await;
            let req = InferenceRequest { input, model_name, precision };
            if let Err(e) = req.validate() {
                return Err(js_sys::Error::new(&e.to_string()).into());
            }

            let model_path = handler.model_store.get_model_path(&req.model_name)
                .ok_or_else(|| js_sys::Error::new(&InferenceError::ModelNotFound(req.model_name).to_string()))?;
            let model_data = wasm_bindgen::JsValue::from_str(&model_path);
            handler.engine.model.load_from_wasm(&model_data).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;

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
            let (mut results, mut tableau) = quantizer.quantize_tokens(token_features, "default");

            let precision = match req.precision.as_str() {
                "2" => PrecisionLevel::Bit2,
                "4" => PrecisionLevel::Bit4,
                "8" => PrecisionLevel::Bit8,
                "16" => PrecisionLevel::Bit16,
                _ => return Err(js_sys::Error::new("Invalid precision").into()),
            };
            let (grouped_keys, residual_keys, grouped_values, residual_values) = handler.apply_kivi_quantization(&mut results, &mut tableau, &precision)?;

            let kv_cache = KVCache {
                key: bincode::serialize(&grouped_keys).unwrap(),
                value: bincode::serialize(&grouped_values).unwrap(),
                layer: CacheLayer::HBM,
                timestamp: chrono::Utc::now().timestamp() as u64,
            };
            handler.model_store.vault.store(format!("kv_grouped_{}", req.model_name), bincode::serialize(&kv_cache).unwrap()).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;
            handler.model_store.vault.store(format!("kv_residual_{}", req.model_name), bincode::serialize(&(residual_keys, residual_values)).unwrap()).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;

            let output = handler.engine.infer(&req.input, &ns_router_rs::NSRoutingPlan {
                model_config: ns_router_rs::ModelConfig { precision: results.clone() },
                kv_cache_config: ns_router_rs::KVCacheConfig { sparsity: 0.5, priority_tokens: vec![] },
            }).await;

            Ok(js_sys::Array::of1(&JsValue::from_serde(&InferenceResponse {
                text: output.text,
                tokens_processed: output.tokens_processed,
                latency_ms: output.latency_ms,
            }).unwrap()).into())
        })
    }
}