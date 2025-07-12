// Copyright 2025 ZETA RETICULA INC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use actix_web::{web, HttpResponse, Error};
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
use zeta_vault::{ZetaVault, VaultConfig, KVCache, CacheLayer, SecretStore};
use crate::lua::LuaEngine;
use crate::python::PythonEngine;
use crate::USAGE_TRACKER;
use sqlx::PgPool;
use std::sync::Mutex;

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
    #[error("Lua execution error: {0}")]
    Lua(#[from] rlua::Error),
    #[error("Python execution error: {0}")]
    Python(#[from] pyo3::PyErr),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Compaction error: {0}")]
    Compaction(String),
}

#[derive(Validate, Deserialize)]
pub struct InferenceRequest {
    #[validate(length(min = 1, message = "Input cannot be empty"))]
    pub(crate) input: String,
    #[validate(length(min = 1, message = "Model name is required"))]
    model_name: String,
    #[validate(custom = "validate_precision")]
    precision: String,
    user_id: String,
}

#[derive(Validate, Deserialize)]
pub struct SalienceRequest {
    #[validate(length(min = 1, message = "Text cannot be empty"))]
    text: String,
    user_id: String,
}

fn validate_precision(precision: &str) -> Result<(), validator::ValidationError> {
    match precision {
        "2" | "4" | "8" | "16" => Ok(()),
        _ => Err(validator::ValidationError::new("Precision must be 2, 4, 8, or 16")),
    }
}

#[derive(Serialize, Deserialize)]
pub struct InferenceResponse {
    text: String,
    tokens_processed: usize,
    latency_ms: f64,
    upgrade_prompt: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SalienceResponse {
    salient_phrases: Vec<String>,
    tokens_processed: usize,
    upgrade_prompt: Option<String>,
}

pub struct InferenceHandler {
    engine: InferenceEngine,
    #[cfg(feature = "server")]
    sidecar_client: pb::sidecar_service_client::SidecarServiceClient<Channel>,
    model_store: ModelStore,
    lua_engine: LuaEngine,
    python_engine: PythonEngine,
    db_pool: PgPool,
    secret_store: Option<SecretStore>,
    chunk_size: usize,
}

impl InferenceHandler {
    pub async fn new(model_store: ModelStore, db_pool: PgPool) -> Self {
        let engine = InferenceEngine::new(768).await;
        #[cfg(feature = "server")]
        let sidecar_client = pb::sidecar_service_client::SidecarServiceClient::connect("http://localhost:50051")
            .await
            .unwrap_or_else(|e| panic!("Sidecar connection failed: {:?}", e));
        let lua_engine = LuaEngine::new();
        let python_engine = PythonEngine::new();
        lua_engine.load_script("lua/inference.lua").unwrap_or_else(|e| log::error!("Lua load failed: {:?}", e));
        let secret_store = if cfg!(feature = "enterprise") {
            Some(SecretStore::new(VaultConfig::default()).await.unwrap_or_else(|e| {
                log::error!("SecretStore init failed: {:?}", e);
                std::process::exit(1)
            }))
        } else {
            None
        };
        InferenceHandler {
            engine,
            sidecar_client,
            model_store,
            lua_engine,
            python_engine,
            db_pool,
            secret_store,
            chunk_size: 32 * 1024,
        }
    }

    #[cfg(feature = "server")]
    pub async fn infer(&self, req: web::Json<InferenceRequest>) -> Result<HttpResponse, Error> {
        req.validate().map_err(|e| actix_web::error::ErrorBadRequest(InferenceError::Validation(e.to_string())))?;

        let subscription = self.get_subscription_data(&req.user_id).await?;
        let is_enterprise = subscription.plan == "enterprise" && cfg!(feature = "enterprise") && subscription.status == "active";
        let usage_limit = if is_enterprise { u32::MAX } else { 50_000 };

        let model_path = self.model_store.get_model_path(&req.model_name)
            .ok_or_else(|| InferenceError::ModelNotFound(req.model_name.clone()))?;
        let weights = self.model_store.get_model_weights(&req.model_name).await
            .ok_or_else(|| InferenceError::ModelNotFound(req.model_name.clone()))?;
        self.engine.load_weights(weights).await;

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
            _ => return Err(actix_web::error::ErrorBadRequest(InferenceError::Validation("Invalid precision".to_string()))),
        };
        let (grouped_keys, residual_keys, grouped_values, residual_values) = self.apply_kivi_quantization(&mut results, &mut tableau, &precision)?;

        let kv_cache = KVCache {
            key: bincode::serialize(&grouped_keys).unwrap(),
            value: bincode::serialize(&grouped_values).unwrap(),
            layer: CacheLayer::HBM,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        privileged_store!(self.secret_store, format!("kv_grouped_{}", req.model_name), bincode::serialize(&kv_cache).unwrap());
        self.model_store.vault.store(format!("kv_grouped_{}", req.model_name), bincode::serialize(&kv_cache).unwrap())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(InferenceError::Io(e)))?;
        self.model_store.vault.store(format!("kv_residual_{}", req.model_name), bincode::serialize(&(residual_keys, residual_values)).unwrap())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(InferenceError::Io(e)))?;

        tableau.cache_to_sidecar(&mut self.sidecar_client.clone())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(InferenceError::Io(e.into())))?;

        let routing_plan = ns_router_rs::NSRoutingPlan {
            model_config: ns_router_rs::ModelConfig { precision: results.clone() },
            kv_cache_config: ns_router_rs::KVCacheConfig { sparsity: 0.7, priority_tokens: vec![] },
        };

        let customized_input = self.lua_engine.customize_inference(&req.input)?;
        let quantized_input = self.python_engine.execute_quantization(&customized_input)?;
        let start = std::time::Instant::now();
        let output = self.engine.infer(&quantized_input, &routing_plan, &[]).await; // Empty weights as loaded earlier
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Enqueue compaction for updated KV cache
        let compaction_request = model_store::CompactionRequest {
            model_id: req.model_name.clone(),
            level: 0,
            data: bincode::serialize(&kv_cache).unwrap(),
        };
        self.model_store.enqueue_compaction(compaction_request).await;

        let upgrade_prompt = self.check_usage_limit(&req.user_id, output.tokens_processed as u32, "inference", &subscription)?;

        // Error handling for compaction
        if let Err(e) = self.handle_compaction_errors().await {
            log::warn!("Compaction error: {:?}", e);
        }

        Ok(HttpResponse::Ok().json(InferenceResponse {
            text: output.text,
            tokens_processed: output.tokens_processed,
            latency_ms,
            upgrade_prompt,
        }))
    }

    #[cfg(feature = "server")]
    pub async fn salience(&self, req: web::Json<SalienceRequest>) -> Result<HttpResponse, Error> {
        let subscription = self.get_subscription_data(&req.user_id).await?;
        let is_enterprise = subscription.plan == "enterprise" && cfg!(feature = "enterprise") && subscription.status == "active";
        let usage_limit = if is_enterprise { u32::MAX } else { 20_000 };

        let quantizer = SalienceQuantizer::new(0.7);
        let tokens: Vec<&str> = req.text.split_whitespace().collect();
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
        let (results, _) = quantizer.quantize_tokens(token_features, "default");
        let salient_phrases: Vec<String> = results.iter()
            .filter(|r| r.salience_score > 0.6)
            .map(|r| format!("Token_{}", r.token_id))
            .collect();

        let tokens_processed = tokens.len() as usize;
        let upgrade_prompt = self.check_usage_limit(&req.user_id, tokens_processed as u32, "salience", &subscription)?;

        Ok(HttpResponse::Ok().json(SalienceResponse {
            salient_phrases,
            tokens_processed,
            upgrade_prompt,
        }))
    }

    fn apply_kivi_quantization(&self, results: &mut Vec<QuantizationResult>, tableau: &mut YoungTableau, precision: &PrecisionLevel) -> Result<(Array2<f16>, Array2<f32>, Array2<f16>, Array2<f32>), InferenceError> {
        let group_size = 16;
        let token_count = results.len();
        let dim = tableau.data.dim().0;

        let mut grouped_keys = Array2::<f16>::zeros((token_count / group_size, dim));
        let mut residual_keys = Array2::<f32>::zeros((token_count % group_size, dim));
        for i in 0..token_count {
            let row = i / group_size;
            let col = i % group_size;
            if row < grouped_keys.dim().0 {
                for d in 0..dim {
                    grouped_keys[[row, d]] = f16::from_f32(tableau.data[[i, d]]).quantize_2bit();
                }
            } else {
                for d in 0..dim {
                    residual_keys[[col, d]] = tableau.data[[i, d]];
                }
            }
        }

        let mut grouped_values = Array2::<f16>::zeros((token_count / group_size, dim));
        let mut residual_values = Array2::<f32>::zeros((token_count % group_size, dim));
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

        let attention_scores = self.compute_attention_scores(&grouped_keys, &grouped_values, &residual_keys, &residual_values)?;
        tableau.data = attention_scores.mapv(|v| v.to_f32());

        Ok((grouped_keys, residual_keys, grouped_values, residual_values))
    }

    fn compute_attention_scores(&self, grouped_keys: &Array2<f16>, grouped_values: &Array2<f16>, residual_keys: &Array2<f32>, residual_values: &Array2<f32>) -> Result<Array2<f16>, InferenceError> {
        let mut scores = Array2::<f16>::zeros((grouped_keys.dim().0 + residual_keys.dim().0, grouped_values.dim().1));
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

    async fn get_subscription_data(&self, user_id: &str) -> Result<SubscriptionData, Error> {
        let row = sqlx::query("SELECT subscription_plan, subscription_status FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(InferenceError::Database)?;
        Ok(SubscriptionData {
            plan: row.get::<String, _>("subscription_plan"),
            status: row.get::<String, _>("subscription_status"),
        })
    }

    fn check_usage_limit(&self, user_id: &str, tokens_processed: u32, service: &str, subscription: &SubscriptionData) -> Result<Option<String>, Error> {
        let mut tracker = USAGE_TRACKER.lock().unwrap();
        let usage = tracker.iter_mut().find(|(id, _)| id == user_id).map(|(_, count)| {
            *count += tokens_processed;
            *count
        }).unwrap_or_else(|| {
            tracker.push((user_id.to_string(), tokens_processed));
            tokens_processed
        });

        let is_enterprise = subscription.plan == "enterprise" && cfg!(feature = "enterprise") && subscription.status == "active";
        let usage_limit = match service {
            "inference" => if is_enterprise { u32::MAX } else { 50_000 },
            "salience" => if is_enterprise { u32::MAX } else { 20_000 },
            _ => u32::MAX,
        };

        Ok(if usage > usage_limit && !is_enterprise {
            Some(format!("Upgrade to Enterprise for unlimited {}!", service))
        } else {
            None
        })
    }

    async fn handle_compaction_errors(&self) -> Result<(), InferenceError> {
        if rand::random::<f32>() < 0.1 { // 10% chance of error
            return Err(InferenceError::Compaction("Transient compaction failure".to_string()));
        }
        Ok(())
    }

    #[cfg(feature = "wasm")]
    #[wasm_bindgen]
    pub fn infer_wasm(input: String, model_name: String, precision: String, user_id: String) -> js_sys::Promise {
        future_to_promise(async move {
            let model_store = ModelStore::new().await;
            let db_pool = PgPool::connect(&std::env::var("NEON_CONNECTION_STRING").unwrap()).await.unwrap();
            let handler = InferenceHandler::new(model_store, db_pool).await;
            let req = InferenceRequest { input, model_name, precision, user_id };
            if let Err(e) = req.validate() {
                return Err(js_sys::Error::new(&e.to_string()).into());
            }

            let weights = handler.model_store.get_model_weights(&req.model_name)
                .ok_or_else(|| js_sys::Error::new(&InferenceError::ModelNotFound(req.model_name).to_string()))?;
            handler.engine.load_weights(weights).await;

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
            handler.model_store.vault.store(format!("kv_grouped_{}", req.model_name), bincode::serialize(&kv_cache).unwrap())
                .await
                .map_err(|e| js_sys::Error::new(&e.to_string()))?;
            handler.model_store.vault.store(format!("kv_residual_{}", req.model_name), bincode::serialize(&(residual_keys, residual_values)).unwrap())
                .await
                .map_err(|e| js_sys::Error::new(&e.to_string()))?;

            let customized_input = handler.lua_engine.customize_inference(&req.input)?;
            let quantized_input = handler.python_engine.execute_quantization(&customized_input)?;
            let output = handler.engine.infer(&quantized_input, &ns_router_rs::NSRoutingPlan {
                model_config: ns_router_rs::ModelConfig { precision: results.clone() },
                kv_cache_config: ns_router_rs::KVCacheConfig { sparsity: 0.7, priority_tokens: vec![] },
            }, &[]).await;

            let subscription = handler.get_subscription_data(&req.user_id).await?;
            let upgrade_prompt = handler.check_usage_limit(&req.user_id, output.tokens_processed as u32, "inference", &subscription)?;

            Ok(js_sys::Array::of1(&JsValue::from_serde(&InferenceResponse {
                text: output.text,
                tokens_processed: output.tokens_processed,
                latency_ms: output.latency_ms,
                upgrade_prompt,
            }).unwrap()).into())
        })
    }
}

#[derive(Debug)]
struct SubscriptionData {
    plan: String,
    status: String,
}