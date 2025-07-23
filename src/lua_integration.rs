// Copyright 2025 ZETA RETICULA
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

// Placeholder for ZetaVault (now replaced by zeta_vault_synergy)
pub use zeta_vault_synergy::{ZetaVaultSynergy, VaultConfig, KVCache, CacheLayer, SecretStore};

use mlua::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::inference_handler::{InferenceHandler, InferenceRequest, InferenceOutput, InferenceError};
use crate::quantize::{QuantizationHandler, QuantizationRequest, QuantizationResponse, QuantizationOptions, QuantizeError};
use crate::zeta_vault_synergy::{ZetaVaultSynergy, VaultConfig, ZetaVaultSynergyError};
use crate::api::petri_engine::PetriEngine;
use crate::attention_store::AttentionStore;
use crate::agentflow::AgentFlow;
use ndarray::Array2;
use half::f16;
use std::time::Instant;
use rand::Rng;
use log::{info, error};

#[derive(Debug)]
pub struct QuantizationOptions {
    pub sparse_gpt: bool,
    pub moe: bool,
    pub dynamic: bool,
    pub precision_granularity: String,
    pub kv_cache_compression: bool,
    pub attention_prefetch: bool,
    pub learning_rate: f32,
    pub calibration_tau: f32,
    pub quantize_activations: bool, // New: Quantize activations
    pub mixed_precision_config: Option<Vec<(String, u8)>>, // New: Layer-bit mapping
}

pub fn create_lua_module(lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
    let globals = lua.globals();

    let zeta_reticula = lua.create_table()?;

    // Inference Handler
    let inference_handler = lua.create_table()?;
    inference_handler.set("new", lua.create_async_function(|lua, ()| async move {
        let attention_store = Arc::new(AttentionStore::new().unwrap());
        let agent_flow = Arc::new(AgentFlow::new().await.unwrap());
        let vault = Arc::new(ZetaVaultSynergy::new(0, VaultConfig {
            node_count: 3,
            replication_factor: 2,
            sync_interval: std::time::Duration::from_secs(10),
        }).await.unwrap());
        let petri_engine = Arc::new(PetriEngine::new(Arc::clone(&attention_store), Arc::clone(&agent_flow), Arc::clone(&vault), 1.0).await);
        let handler = InferenceHandler::new(Arc::clone(&vault), Arc::clone(&petri_engine));
        let table = lua.create_table()?;
        table.set("handler", handler);
        Ok(mlua::Value::Table(table))
    })?)?;
    inference_handler.set("infer", lua.create_async_function(|lua, (handler_table, input, model_name, precision): (mlua::Table, String, String, String)| async move {
        let handler = handler_table.get::<_, InferenceHandler>("handler")?;
        let req = InferenceRequest {
            input: vec![input],
            model_name,
            precision,
        };
        if let Err(e) = req.validate() {
            return Err(mlua::Error::RuntimeError(e.to_string()));
        }
        let output = handler.infer(&req).await.map_err(|e| mlua::Error::RuntimeError(format!("Inference failed: {}", e)))?;
        Ok(mlua::Value::Table(lua.create_table_from([
            ("text", mlua::Value::String(lua.create_string(&output.text)?)),
            ("tokens_processed", mlua::Value::Integer(output.tokens_processed as i64)),
            ("latency_ms", mlua::Value::Number(output.latency_ms as f64)),
        ])?))
    })?)?;

    // Quantization Handler
    let quantization_handler = lua.create_table()?;
    quantization_handler.set("new", lua.create_async_function(|lua, ()| async move {
        let attention_store = Arc::new(AttentionStore::new().unwrap());
        let agent_flow = Arc::new(AgentFlow::new().await.unwrap());
        let vault = Arc::new(ZetaVaultSynergy::new(0, VaultConfig {
            node_count: 3,
            replication_factor: 2,
            sync_interval: std::time::Duration::from_secs(10),
        }).await.unwrap());
        let petri_engine = Arc::new(PetriEngine::new(Arc::clone(&attention_store), Arc::clone(&agent_flow), Arc::clone(&vault), 1.0).await);
        let handler = QuantizationHandler::new(Arc::clone(&vault), Arc::clone(&petri_engine));
        let table = lua.create_table()?;
        table.set("handler", handler);
        Ok(mlua::Value::Table(table))
    })?)?;
    quantization_handler.set("quantize", lua.create_async_function(|lua, (handler_table, model_name, bit_depth, options): (mlua::Table, String, u8, mlua::Table)| async move {
        let handler = handler_table.get::<_, QuantizationHandler>("handler")?;
        let mut req = QuantizationRequest {
            model_name,
            bit_depth: bit_depth.to_string(),
        };
        if let Err(e) = req.validate() {
            return Err(mlua::Error::RuntimeError(e.to_string()));
        }

        let sparse_gpt = options.get::<_, bool>("sparse_gpt").unwrap_or(false);
        let moe = options.get::<_, bool>("moe").unwrap_or(false);
        let dynamic = options.get::<_, bool>("dynamic").unwrap_or(false);
        let precision_granularity = options.get::<_, String>("precision_granularity").unwrap_or("per-tensor".to_string());
        let kv_cache_compression = options.get::<_, bool>("kv_cache_compression").unwrap_or(false);
        let attention_prefetch = options.get::<_, bool>("attention_prefetch").unwrap_or(false);
        let learning_rate = options.get::<_, f32>("learning_rate").unwrap_or(0.1);
        let calibration_tau = options.get::<_, f32>("calibration_tau").unwrap_or(0.5); // Default ICQ tau

        let output = handler.quantize(&req, QuantizationOptions {
            sparse_gpt,
            moe,
            dynamic,
            precision_granularity,
            kv_cache_compression,
            attention_prefetch,
            learning_rate,
            calibration_tau,
        }).await.map_err(|e| mlua::Error::RuntimeError(format!("Quantization failed: {}", e)))?;
        Ok(mlua::Value::String(lua.create_string(&output.quantized_path)?))
    })?)?;

    zeta_reticula.set("inference", mlua::Value::Table(inference_handler))?;
    zeta_reticula.set("quantization", mlua::Value::Table(quantization_handler))?;
    globals.set("zeta_reticula", zeta_reticula)?;

    Ok(mlua::Value::Nil)
}

// Completed Inference Handler
pub mod inference_handler {
    use serde::{Serialize, Deserialize};
    use thiserror::Error;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::zeta_vault_synergy::{ZetaVaultSynergy, KVCache};
    use crate::api::petri_engine::PetriEngine;
    use ndarray::Array2;
    use half::f16;
    use std::time::Instant;
    use rand::Rng;

    #[derive(Error, Debug)]
    pub enum InferenceError {
        #[error("Validation error: {0}")]
        Validation(String),
        #[error("Vault error: {0}")]
        Vault(#[from] ZetaVaultSynergyError),
        #[error("Petri engine error: {0}")]
        PetriEngine(String),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InferenceRequest {
        pub input: Vec<String>,
        pub model_name: String,
        pub precision: String,
    }

    impl InferenceRequest {
        pub fn validate(&self) -> Result<(), String> {
            if self.model_name.is_empty() || self.input.is_empty() {
                Err("Invalid request data".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InferenceOutput {
        pub text: String,
        pub tokens_processed: usize,
        pub latency_ms: u64,
    }

    pub struct InferenceHandler {
        vault: Arc<ZetaVaultSynergy>,
        petri_engine: Arc<PetriEngine>,
        latent_weights: Arc<RwLock<Vec<f32>>>, // High-precision latent weights
        lora_l1: Arc<RwLock<Array2<f32>>>,    // LoRA parameter l1
        lora_l2: Arc<RwLock<Array2<f32>>>,    // LoRA parameter l2
    }

    impl InferenceHandler {
        pub fn new(vault: Arc<ZetaVaultSynergy>, petri_engine: Arc<PetriEngine>) -> Self {
            let h, o, r = (4096, 4096, 64); // Example dimensions
            InferenceHandler {
                vault,
                petri_engine,
                latent_weights: Arc::new(RwLock::new(Vec::new())),
                lora_l1: Arc::new(RwLock::new(Array2::zeros((h, r)))),
                lora_l2: Arc::new(RwLock::new(Array2::zeros((r, o)))),
            }
        }

        pub async fn infer(&self, req: &InferenceRequest) -> Result<InferenceOutput, InferenceError> {
            let start = Instant::now();

            // Retrieve KVCache
            let kv_cache = self.vault.get_kv_cache(&req.model_name).await
                .ok_or_else(|| InferenceError::Vault(ZetaVaultSynergyError::Validation("No KV cache found".to_string())))?;
            let keys = bincode::deserialize::<Array2<f16>>(&kv_cache.key)
                .map_err(|e| InferenceError::Vault(ZetaVaultSynergyError::Serialization(e)))?;
            let values = bincode::deserialize::<Array2<f16>>(&kv_cache.value)
                .map_err(|e| InferenceError::Vault(ZetaVaultSynergyError::Serialization(e)))?;

            // Process input with FusionANNs via PetriEngine
            let input_text = req.input.join(" ");
            let tokens_processed = req.input.len();
            info!("Processing inference for model {} with {} tokens", req.model_name, tokens_processed);

            // Bit precision and quantization
            let bit_precision = match req.precision.as_str() {
                "f16" => 16,
                "q1" => 1,
                "q2" => 2,
                "q4" => 4,
                "q8" => 8,
                _ => return Err(InferenceError::Validation("Unsupported precision".to_string())),
            };

            let (q_keys, q_values) = if bit_precision == 1 {
                let (qk, qv) = self.quantize_1bit(&keys, &values).await?;
                (qk, qv)
            } else {
                let qk = self.quantize_nfk(&keys, bit_precision).await?;
                let qv = self.quantize_nfk(&values, bit_precision).await?;
                (qk, qv)
            };
            self.vault.store_kv_cache(&req.model_name, q_keys.clone(), q_values.clone()).await?;

            // Dequantize for inference
            let s_fp8_1 = self.double_quantize_scale(q_keys.mapv(|x| x.to_f32()).fold(f32::abs, |a, b| a.max(b)));
            let s_fp16_2 = self.dequant_scale(s_fp8_1);
            let dequant_keys = self.dequant_nfk(&q_keys, s_fp8_1, s_fp16_2);
            let dequant_values = self.dequant_nfk(&q_values, s_fp8_1, s_fp16_2);

            // LoRA enhancement
            let alpha = 0.1; // LoRA scalar
            let l1 = self.lora_l1.read().await.clone();
            let l2 = self.lora_l2.read().await.clone();
            let lora_output = alpha * self.apply_lora(&dequant_keys, &l1, &l2); // Mock input x as dequant_keys

            // Inference with FusionANNs and LoRA
            let result = self.petri_engine.infer_fusion_ann(&req.model_name, &input_text, &dequant_keys, &dequant_values, bit_precision as f32)
                .await.map_err(|e| InferenceError::PetriEngine(e.to_string()))? + lora_output; // Add LoRA output

            let latency_ms = start.elapsed().as_millis() as u64;
            info!("Inference completed for {} in {}ms", req.model_name, latency_ms);

            Ok(InferenceOutput {
                text: result,
                tokens_processed,
                latency_ms,
            })
        }

        async fn quantize_nfk(&self, w: &Array2<f16>, k: u8) -> Result<Array2<f16>, InferenceError> {
            let s = w.mapv(|x| x.to_f32()).fold(f32::abs, |a, b| a.max(b)); // absmax(w)
            let block_size = 64;
            let mut quantized = Array2::zeros(w.dim());

            for i in (0..w.len()).step_by(block_size) {
                let end = (i + block_size).min(w.len());
                let block = w.slice(s![i..end, ..]).mapv(|x| x.to_f32());
                let block_s = block.fold(f32::abs, |a, b| a.max(b));

                
                for (j, val) in block.iter().enumerate() {
                    let qi = self.nf_quantize(*val / block_s, k);
                    quantized[[i + j, 0]] = f16::from_f32(qi * block_s);
                }
            }
            Ok(quantized)
        }

        fn nf_quantize(&self, val: f32, k: u8) -> f32 {
            let levels = 2_u32.pow(k as u32) + 1;
            let q = rand::thread_rng().sample_iter(&rand_distr::Normal::new(0.0, 1.0).unwrap())
                .take(levels as usize)
                .collect::<Vec<f64>>();
            let quantiles = q.into_iter().map(|x| x as f32).collect::<Vec<f32>>();
            quantiles.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx = (val * (levels as f32 - 1.0)).round() as usize;
            (quantiles[idx] + quantiles[idx + 1]) / 2.0
        }

        fn double_quantize_scale(&self, s: f32) -> f32 {
            // Mock FP8 quantization of scale
            (s / 255.0).round() * 255.0 // Simplified FP8
        }

        fn dequant_scale(&self, s_fp8_1: f32) -> f32 {
            // Mock FP16 dequantization
            s_fp8_1 * 256.0 // Simplified FP16
        }

        fn dequant_nfk(&self, w_q: &Array2<f16>, s_fp8_1: f32, s_fp16_2: f32) -> Array2<f16> {
            w_q.mapv(|x| f16::from_f32(x.to_f32() * s_fp8_1 * s_fp16_2))
        }

        fn apply_lora(&self, x: &Array2<f16>, l1: &Array2<f32>, l2: &Array2<f32>) -> String {
            // Mock LoRA computation
            format!(" + LoRA adjustment")
        }

        async fn quantize_1bit(&self, keys: &Array2<f16>, values: &Array2<f16>) -> Result<(Array2<f16>, Array2<f16>), InferenceError> {
            let mut q_keys = Array2::zeros(keys.dim());
            let mut q_values = Array2::zeros(values.dim());

            let mut latent = self.latent_weights.write().await;
            if latent.is_empty() {
                *latent = keys.mapv(|x| x.to_f32()).into_raw_vec();
            }
            let latent_values = values.mapv(|x| x.to_f32()).into_raw_vec();

            for ((i, j), k) in keys.indexed_iter() {
                let idx = i * keys.dim().1 + j;
                let latent_k = latent[idx];
                q_keys[[i, j]] = f16::from_f32(if latent_k >= 0.0 { 1.0 } else { -1.0 });
            }
            for ((i, j), v) in values.indexed_iter() {
                let idx = i * values.dim().1 + j;
                let latent_v = latent_values[idx];
                q_values[[i, j]] = f16::from_f32(if latent_v >= 0.0 { 1.0 } else { -1.0 });
            }

            Ok((q_keys, q_values))
        }
    }
}

// Updated Quantize Module
pub mod quantize {
    use serde::{Serialize, Deserialize};
    use thiserror::Error;
    use ndarray::Array2;
    use half::f16;
    use crate::zeta_vault_synergy::ZetaVaultSynergy;

    #[derive(Error, Debug)]
    pub enum QuantizeError {
        #[error("Validation error: {0}")]
        Validation(String),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct QuantizationRequest {
        pub model_name: String,
        pub bit_depth: String,
    }

    impl QuantizationRequest {
        pub fn validate(&self) -> Result<(), String> {
            let valid_bits = ["1", "2", "4", "8"];
            if self.model_name.is_empty() || !valid_bits.contains(&self.bit_depth.as_str()) {
                Err("Invalid request data or unsupported bit depth (use 1, 2, 4, or 8)".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct QuantizationResponse {
        pub quantized_path: String,
    }

    #[derive(Debug)]
    pub struct QuantizationOptions {
        pub sparse_gpt: bool,
        pub moe: bool,
        pub dynamic: bool,
        pub precision_granularity: String,
        pub kv_cache_compression: bool,
        pub attention_prefetch: bool,
        pub learning_rate: f32,
        pub calibration_tau: f32,
    }

    pub struct QuantizationHandler {
        vault: Arc<ZetaVaultSynergy>,
        petri_engine: Arc<PetriEngine>,
        latent_weights: Arc<RwLock<Vec<f32>>>,
    }

    impl QuantizationHandler {
        pub fn new(vault: Arc<ZetaVaultSynergy>, petri_engine: Arc<PetriEngine>) -> Self {
            QuantizationHandler {
                vault,
                petri_engine,
                latent_weights: Arc::new(RwLock::new(Vec::new())),
            }
        }

        pub async fn quantize(&self, req: &QuantizationRequest, options: QuantizationOptions) -> Result<QuantizationResponse, QuantizeError> {
            let bit_depth = req.bit_depth.parse::<u8>().map_err(|_| QuantizeError::Validation("Invalid bit depth".to_string()))?;
            if ![1, 2, 4, 8].contains(&bit_depth) {
                return Err(QuantizeError::Validation("Bit depth must be 1, 2, 4, or 8".to_string()));
            }
            let quantized_path = format!("{}-q{}.gguf", req.model_name, bit_depth);

            // Initialize KVCache with mock data
            let mut keys = Array2::zeros((128, 1));
            let mut values = Array2::zeros((128, 1));

            // Asymmetric clipping and ICQ
            let (clipped_keys, clipped_values) = self.asymmetric_clip(&keys, &values, options.calibration_tau).await?;

            // Quantize with NFk or 1-bit
            let (q_keys, q_values) = if bit_depth == 1 {
                let mut latent = self.latent_weights.write().await;
                if latent.is_empty() {
                    *latent = clipped_keys.mapv(|x| x.to_f32()).into_raw_vec();
                }
                self.quantize_1bit(&clipped_keys, &clipped_values, options.learning_rate).await?
            } else {
                let qk = self.quantize_nfk(&clipped_keys, bit_depth).await?;
                let qv = self.quantize_nfk(&clipped_values, bit_depth).await?;
                (qk, qv)
            };

            // Apply additional techniques
            if options.sparse_gpt {
                info!("Applying SparseGPT with brute-force approximations");
                for layer in (0..128).step_by(64) {
                    let end = (layer + 64).min(128);
                    let mut sum = f16::from_f32(0.0);
                    for token in layer..end {
                        sum += q_keys[[token, 0]] + q_values[[token, 0]];
                    }
                    let avg = sum / f16::from_u32((end - layer) as u32);
                    for token in layer..end {
                        q_keys[[token, 0]] = if q_keys[[token, 0]] > avg { f16::from_f32(1.0) } else { f16::from_f32(0.0) };
                        q_values[[token, 0]] = if q_values[[token, 0]] > avg { f16::from_f32(1.0) } else { f16::from_f32(0.0) };
                    }
                }
            }

            if options.moe {
                info!("Activating Mixture of Experts subset with brute force");
                let num_experts = 4;
                let active_experts: Vec<usize> = (0..num_experts).step_by(2).collect();
                for (i, &expert) in active_experts.iter().enumerate() {
                    for token in (expert * 32)..((expert + 1) * 32).min(128) {
                        if i % 2 == 0 {
                            q_keys[[token, 0]] *= f16::from_f32(1.0);
                            q_values[[token, 0]] *= f16::from_f32(1.0);
                        } else {
                            q_keys[[token, 0]] = f16::from_f32(0.0);
                            q_values[[token, 0]] = f16::from_f32(0.0);
                        }
                    }
                }
            }

            if options.dynamic {
                info!("Adjusting quantization dynamically with brute force");
                for token in 0..128 {
                    let dynamic_depth = if token % 2 == 0 { bit_depth } else { bit_depth / 2 };
                    for _ in 0..dynamic_depth {
                        q_keys[[token, 0]] = q_keys[[token, 0]].clamp(f16::from_f32(0.0), f16::from_f32(1.0));
                        q_values[[token, 0]] = q_values[[token, 0]].clamp(f16::from_f32(0.0), f16::from_f32(1.0));
                    }
                }
            }

            if options.precision_granularity == "per-channel" {
                info!("Using per-channel precision granularity with brute force");
                for channel in 0..1 {
                    let mut channel_sum = f16::from_f32(0.0);
                    for token in 0..128 {
                        channel_sum += q_keys[[token, channel]];
                    }
                    let channel_avg = channel_sum / f16::from_u32(128);
                    for token in 0..128 {
                        q_keys[[token, channel]] = if q_keys[[token, channel]] > channel_avg { f16::from_f32(1.0) } else { f16::from_f32(0.0) };
                    }
                }
            }

            if options.kv_cache_compression {
                info!("Compressing KVCache with brute-force grouping");
                let group_size = 32;
                for group in (0..128).step_by(group_size) {
                    let mut group_sum = f16::from_f32(0.0);
                    for token in group..(group + group_size).min(128) {
                        group_sum += q_keys[[token, 0]];
                    }
                    let group_avg = group_sum / f16::from_u32(group_size as u32);
                    for token in group..(group + group_size).min(128) {
                        q_keys[[token, 0]] = if q_keys[[token, 0]] > group_avg { f16::from_f32(1.0) } else { f16::from_f32(0.0) };
                    }
                }
            }

            if options.attention_prefetch {
                info!("Prefetching attention data with brute force");
                let mut prefetch_buffer = Array2::zeros((128, 1));
                for token in 0..128 {
                    prefetch_buffer[[token, 0]] = q_keys[[token, 0]] + q_values[[token, 0]];
                }
                q_keys = prefetch_buffer.clone();
            }

            self.vault.store_kv_cache(&req.model_name, q_keys, q_values).await.map_err(|e| QuantizeError::Validation(e.to_string()))?;

            Ok(QuantizationResponse { quantized_path })
        }

        async fn quantize_nfk(&self, w: &Array2<f16>, k: u8) -> Result<Array2<f16>, QuantizeError> {
            let s = w.mapv(|x| x.to_f32()).fold(f32::abs, |a, b| a.max(b)); // absmax(w)
            let block_size = 64;
            let mut quantized = Array2::zeros(w.dim());

            for i in (0..w.len()).step_by(block_size) {
                let end = (i + block_size).min(w.len());
                let block = w.slice(s![i..end, ..]).mapv(|x| x.to_f32());
                let block_s = block.fold(f32::abs, |a, b| a.max(b));
                for (j, val) in block.iter().enumerate() {
                    let qi = self.nf_quantize(*val / block_s, k);
                    quantized[[i + j, 0]] = f16::from_f32(qi * block_s);
                }
            }
            Ok(quantized)
        }

        fn nf_quantize(&self, val: f32, k: u8) -> f32 {
            let levels = 2_u32.pow(k as u32) + 1;
            let q = rand::thread_rng().sample_iter(&rand_distr::Normal::new(0.0, 1.0).unwrap())
                .take(levels as usize)
                .collect::<Vec<f64>>();
            let quantiles = q.into_iter().map(|x| x as f32).collect::<Vec<f32>>();
            quantiles.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx = (val * (levels as f32 - 1.0)).round() as usize;
            (quantiles[idx] + quantiles[idx + 1]) / 2.0
        }

        fn double_quantize_scale(&self, s: f32) -> f32 {
            (s / 255.0).round() * 255.0 // Simplified FP8
        }

        fn dequant_scale(&self, s_fp8_1: f32) -> f32 {
            s_fp8_1 * 256.0 // Simplified FP16
        }

        fn dequant_nfk(&self, w_q: &Array2<f16>, s_fp8_1: f32, s_fp16_2: f32) -> Array2<f16> {
            w_q.mapv(|x| f16::from_f32(x.to_f32() * s_fp8_1 * s_fp16_2))
        }

        async fn quantize_1bit(&self, keys: &Array2<f16>, values: &Array2<f16>, learning_rate: f32) -> Result<(Array2<f16>, Array2<f16>), QuantizeError> {
            let mut q_keys = Array2::zeros(keys.dim());
            let mut q_values = Array2::zeros(values.dim());

            let mut latent = self.latent_weights.write().await;
            if latent.is_empty() {
                *latent = keys.mapv(|x| x.to_f32()).into_raw_vec();
            }
            let mut latent_values = values.mapv(|x| x.to_f32()).into_raw_vec();

            for ((i, j), k) in keys.indexed_iter() {
                let idx = i * keys.dim().1 + j;
                let latent_k = latent[idx];
                q_keys[[i, j]] = f16::from_f32(if latent_k >= 0.0 { 1.0 } else { -1.0 });
                latent[idx] += learning_rate * f32::from(k); // STE update
            }
            for ((i, j), v) in values.indexed_iter() {
                let idx = i * values.dim().1 + j;
                let latent_v = latent_values[idx];
                q_values[[i, j]] = f16::from_f32(if latent_v >= 0.0 { 1.0 } else { -1.0 });
                latent_values[idx] += learning_rate * f32::from(v); // STE update
            }

            Ok((q_keys, q_values))
        }

        async fn asymmetric_clip(&self, keys: &Array2<f16>, values: &Array2<f16>, tau: f32) -> Result<(Array2<f16>, Array2<f16>), QuantizeError> {
            let mut clipped_keys = keys.clone();
            let mut clipped_values = values.clone();

            let min_val = keys.mapv(|x| x.to_f32()).fold(f32::INFINITY, |a, b| a.min(b));
            let max_val = keys.mapv(|x| x.to_f32()).fold(f32::NEG_INFINITY, |a, b| a.max(b));
            let alpha = min_val * (1.0 - tau);
            let beta = max_val * (1.0 + tau);

            for ((i, j), k) in clipped_keys.indexed_iter_mut() {
                let val = k.to_f32();
                *k = f16::from_f32(if val < 0.0 { val.clamp(alpha, 0.0) } else { val.clamp(0.0, beta) });
            }
            for ((i, j), v) in clipped_values.indexed_iter_mut() {
                let val = v.to_f32();
                *v = f16::from_f32(if val < 0.0 { val.clamp(alpha, 0.0) } else { val.clamp(0.0, beta) });
            }

            Ok((clipped_keys, clipped_values))
        }
    }
}