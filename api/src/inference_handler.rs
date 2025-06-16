use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use llm-rs::inference::InferenceEngine;
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
use rustc_hash::FxHashMap as HashMap;
use argmin::core::{Executor, GradientDescent};
use argmin::solver::linesearch::MoreThuenteLineSearch;
use argmin::solver::gradientdescent::SteepestDescent;

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
    #[error("Optimization error: {0}")]
    Optimization(String),
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
        "2" | "4" | "8" | "16" => Ok(()),
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
    scaling_experts: HashMap<String, Vec<f32>>, // Store scaling experts per model
    teacher_model: Option<InferenceEngine>, // Full-precision teacher
}

impl InferenceHandler {
    pub async fn new(model_store: ModelStore) -> Self {
        let engine = InferenceEngine::new(768).await;
        #[cfg(feature = "server")]
        let sidecar_client = pb::sidecar_service_client::SidecarServiceClient::connect("http://localhost:50051").await.unwrap();
        let scaling_experts = HashMap::default();
        let teacher_model = Some(InferenceEngine::new(768).await); // Mock teacher
        InferenceHandler { engine, sidecar_client, model_store, scaling_experts, teacher_model }
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

        let precision = match req.precision.as_str() {
            "2" => PrecisionLevel::Bit2,
            "4" => PrecisionLevel::Bit4,
            "8" => PrecisionLevel::Bit8,
            "16" => PrecisionLevel::Bit16,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid precision")),
        };
        let (grouped_keys, residual_keys, grouped_values, residual_values, lora_delta) = self.apply_chunked_quantization_and_lora(&mut results, &mut tableau, &precision, &req.model_name)?;
        let (quantized_keys, quantized_values) = self.apply_binary_mos(&grouped_keys, &grouped_values, &req.input, &req.model_name)?;
        let (distilled_keys, distilled_values) = self.apply_qakd(&quantized_keys, &quantized_values, &req.input, &req.model_name)?;

        let kv_cache = KVCache {
            key: bincode::serialize(&distilled_keys).unwrap(),
            value: bincode::serialize(&distilled_values).unwrap(),
            layer: CacheLayer::HBM,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        self.model_store.vault.store(format!("kv_grouped_{}", req.model_name), bincode::serialize(&kv_cache).unwrap()).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        self.model_store.vault.store(format!("kv_residual_{}", req.model_name), bincode::serialize(&(residual_keys, residual_values)).unwrap()).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        self.model_store.vault.store(format!("lora_{}", req.model_name), bincode::serialize(&lora_delta).unwrap()).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

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

    fn apply_chunked_quantization_and_lora(&self, results: &mut Vec<QuantizationResult>, tableau: &mut YoungTableau, precision: &PrecisionLevel, model_name: &str) -> Result<(Array2<f16>, Array2<f32>, Array2<f16>, Array2<f32>, (Array2<f32>, Array2<f32>)), InferenceError> {
        let block_size = 64;
        let token_count = results.len();
        let dim = tableau.data.dim().0;
        let n_blocks = (token_count * dim) / block_size;

        let mut grouped_keys = Array2::<f16>::zeros((n_blocks, block_size));
        let mut residual_keys = Array2::<f32>::zeros(((token_count * dim) % block_size, dim));
        let mut grouped_values = Array2::<f16>::zeros((n_blocks, block_size));
        let mut residual_values = Array2::<f32>::zeros(((token_count * dim) % block_size, dim));
        let mut constants = Vec::with_capacity(n_blocks);

        for i in 0..n_blocks {
            let start = i * block_size;
            let end = std::cmp::min((i + 1) * block_size, token_count * dim);
            let block = tableau.data.slice(s![start..end, ..]).to_owned();
            let c_i = block.mapv(|v| v.abs()).max().unwrap_or(1.0);
            constants.push(c_i);
            for j in 0..block.len_of(Axis(0)) {
                for d in 0..dim {
                    let quantized = f16::from_f32(block[[j, d]] / c_i).quantize_2bit();
                    grouped_keys[[i, j]] = quantized;
                }
            }
        }

        let residual_start = n_blocks * block_size;
        if residual_start < token_count * dim {
            let residual_block = tableau.data.slice(s![residual_start.., ..]).to_owned();
            for j in 0..residual_block.len_of(Axis(0)) {
                for d in 0..dim {
                    residual_keys[[j, d]] = residual_block[[j, d]];
                }
            }
        }

        for i in 0..n_blocks {
            let start = i * block_size;
            let end = std::cmp::min((i + 1) * block_size, token_count * dim);
            for j in start..end {
                for d in 0..dim {
                    let quantized = f16::from_f32(results[j].salience_score / constants[i / dim]).quantize_2bit();
                    grouped_values[[i, j - start]] = quantized;
                }
            }
        }
        if residual_start < token_count * dim {
            let residual_block = results[residual_start..].iter().map(|r| r.salience_score).collect::<Vec<f32>>();
            for j in 0..residual_block.len() {
                for d in 0..dim {
                    residual_values[[j, d]] = residual_block[j];
                }
            }
        }

        let rank = 16;
        let a = Array2::<f32>::random((dim, rank), ndarray::rand::rand_distr::Uniform::new(0.0, 1.0));
        let b = Array2::<f32>::random((rank, dim), ndarray::rand::rand_distr::Uniform::new(0.0, 1.0));
        let delta_w = a.dot(&b);
        self.lora_adapters.insert(model_name.to_string(), (a, b));

        Ok((grouped_keys, residual_keys, grouped_values, residual_values, (delta_w, delta_w.t().to_owned())))
    }

    fn apply_binary_mos(&self, grouped_keys: &Array2<f16>, grouped_values: &Array2<f16>, input: &str, model_name: &str) -> Result<(Array2<f16>, Array2<f16>), InferenceError> {
        if !self.scaling_experts.contains_key(model_name) {
            let experts = vec![1.0, 0.8, 0.6, 0.4, 0.2];
            self.scaling_experts.insert(model_name.to_string(), experts);
        }
        let experts = self.scaling_experts.get(model_name).unwrap();

        let token_count = input.split_whitespace().count();
        let weights: Vec<f32> = (0..experts.len())
            .map(|i| {
                let weight = (token_count as f32 / experts.len() as f32) * (i as f32 + 1.0).recip();
                weight / weights.iter().sum::<f32>()
            })
            .collect();

        let mut combined_scaling = 0.0;
        for (w, s) in weights.iter().zip(experts.iter()) {
            combined_scaling += w * s;
        }

        let mut quantized_keys = grouped_keys.mapv(|v| f16::from_f32(v.to_f32() * combined_scaling));
        let mut quantized_values = grouped_values.mapv(|v| f16::from_f32(v.to_f32() * combined_scaling));

        let weight_matrix = Array2::<f16>::ones((quantized_keys.dim().1, 768));
        let keys_output = quantized_keys.dot(&weight_matrix);
        let values_output = quantized_values.dot(&weight_matrix);
        quantized_keys = keys_output;
        quantized_values = values_output;

        Ok((quantized_keys, quantized_values))
    }

    fn apply_qakd(&self, quantized_keys: &Array2<f16>, quantized_values: &Array2<f16>, input: &str, model_name: &str) -> Result<(Array2<f16>, Array2<f16>), InferenceError> {
        let teacher = self.teacher_model.as_ref().ok_or_else(|| InferenceError::Quantization("Teacher model not initialized".to_string()))?;
        let student = &self.engine;

        // Mock teacher inference (replace with actual teacher model inference)
        let teacher_logits = teacher.infer(input, &ns_router_rs::NSRoutingPlan::default()).await.logits; // Assume logits are available
        let student_logits = student.infer(input, &ns_router_rs::NSRoutingPlan::default()).await.logits;

        // Compute CE loss and optimize student
        let ce_loss = self.compute_ce_loss(&teacher_logits, &student_logits)?;
        let optimized_keys = self.optimize_with_qakd(quantized_keys, &ce_loss)?;
        let optimized_values = self.optimize_with_qakd(quantized_values, &ce_loss)?;

        Ok((optimized_keys, optimized_values))
    }

    fn compute_ce_loss(&self, teacher_logits: &Array2<f32>, student_logits: &Array2<f32>) -> Result<f32, InferenceError> {
        let n_samples = teacher_logits.len_of(Axis(0)) as f32;
        let ce_loss = -teacher_logits.mapv(|t| t.exp() / teacher_logits.sum_axis(Axis(1)).mapv(|s| s.exp()))
            .into_iter()
            .zip(student_logits.into_iter())
            .map(|(t, s)| t * s.ln())
            .sum::<f32>() / n_samples;
        Ok(ce_loss)
    }

    fn optimize_with_qakd(&self, tensor: &Array2<f16>, loss: &f32) -> Result<Array2<f16>, InferenceError> {
        // Mock gradient descent optimization using argmin
        let problem = argmin::core::Problem::new(
            || tensor.mapv(|v| v.to_f32()).into_raw_vec(),
            |param: &Vec<f32>| {
                let param_array = Array2::from_shape_vec((tensor.dim().0, tensor.dim().1), param.clone()).unwrap();
                let new_tensor = param_array.mapv(|v| f16::from_f32(v).quantize_2bit());
                let new_loss = self.compute_ce_loss(&Array2::ones(tensor.dim()), &new_tensor.mapv(|v| v.to_f32()))?; // Mock
                Ok(new_loss)
            }
        );
        let init_param = tensor.mapv(|v| v.to_f32()).into_raw_vec();
        let linesearch = MoreThuenteLineSearch::new();
        let solver = SteepestDescent::new(linesearch);
        let res = Executor::new(problem, solver)
            .configure(|state| state.param(init_param).max_iters(100))
            .run()
            .map_err(|e| InferenceError::Optimization(e.to_string()))?;
        let optimized = Array2::from_shape_vec(tensor.dim(), res.state.param).unwrap().mapv(|v| f16::from_f32(v));
        Ok(optimized)
    }

    #[cfg(feature = "wasm")]
    #[wasm_bindgen]
    pub fn infer_wasm(input: String, model_name: String, precision: String) -> js_sys::Promise {
        future_to_promise(async move {
            let model_store = ModelStore::new().await;
            let mut handler = InferenceHandler::new(model_store).await;
            let req = InferenceRequest { input: input.clone(), model_name: model_name.clone(), precision };
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
            let (grouped_keys, residual_keys, grouped_values, residual_values, lora_delta) = handler.apply_chunked_quantization_and_lora(&mut results, &mut tableau, &precision, &req.model_name)?;
            let (quantized_keys, quantized_values) = handler.apply_binary_mos(&grouped_keys, &grouped_values, &req.input, &req.model_name)?;
            let (distilled_keys, distilled_values) = handler.apply_qakd(&quantized_keys, &quantized_values, &req.input, &req.model_name)?;

            let kv_cache = KVCache {
                key: bincode::serialize(&distilled_keys).unwrap(),
                value: bincode::serialize(&distilled_values).unwrap(),
                layer: CacheLayer::HBM,
                timestamp: chrono::Utc::now().timestamp() as u64,
            };
            handler.model_store.vault.store(format!("kv_grouped_{}", req.model_name), bincode::serialize(&kv_cache).unwrap()).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;
            handler.model_store.vault.store(format!("kv_residual_{}", req.model_name), bincode::serialize(&(residual_keys, residual_values)).unwrap()).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;
            handler.model_store.vault.store(format!("lora_{}", req.model_name), bincode::serialize(&lora_delta).unwrap()).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;

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