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

use actix_web::{web, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log;
use crate::zeta_vault_synergy::ZetaVaultSynergy;
use crate::api::petri_engine::PetriEngine;

#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Engine error: {0}")]
    Engine(String),
    #[error("Vault error: {0}")]
    Vault(#[from] ZetaVaultSynergyError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_name: String,
    pub input: Vec<String>,
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
    max_retries: usize,
}

impl InferenceHandler {
    pub fn new(vault: Arc<ZetaVaultSynergy>, petri_engine: Arc<PetriEngine>) -> Self {
        InferenceHandler {
            vault,
            petri_engine,
            max_retries: 3,
        }
    }

    pub async fn infer(&self, req: web::Json<InferenceRequest>) -> Result<HttpResponse, Error> {
        // Validation checks
        req.validate().map_err(|e| actix_web::error::ErrorBadRequest(InferenceError::Validation(e.to_string())))?;
        if req.input.is_empty() {
            return Err(actix_web::error::ErrorBadRequest(InferenceError::Validation("Input cannot be empty".to_string())));
        }
        if req.model_name.is_empty() {
            return Err(actix_web::error::ErrorBadRequest(InferenceError::Validation("Model name cannot be empty".to_string())));
        }

        // Retry mechanism for compaction errors
        let mut attempt = 0;
        loop {
            match self.infer_internal(&req).await {
                Ok(output) => {
                    log::info!("Inference completed for model {}: {} tokens, {}ms", req.model_name, output.tokens_processed, output.latency_ms);
                    return Ok(HttpResponse::Ok().json(output));
                }
                Err(e) if attempt < self.max_retries => {
                    log::warn!("Inference attempt {} failed: {:?}, retrying...", attempt, e);
                    sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await; // Exponential backoff
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
            }
        }
    }

    async fn infer_internal(&self, req: &InferenceRequest) -> Result<InferenceOutput, InferenceError> {
        let start = std::time::Instant::now();
        let kv_cache = self.vault.get_kv_cache(&req.model_name).await.ok_or_else(|| InferenceError::Vault(ZetaVaultSynergyError::Validation("No KV cache found".to_string())))?;
        if kv_cache.key.is_empty() || kv_cache.value.is_empty() {
            return Err(InferenceError::Validation("Invalid KV cache".to_string()));
        }

        let keys = bincode::deserialize(&kv_cache.key)?;
        let values = bincode::deserialize(&kv_cache.value)?;
        self.petri_engine.update_kv_cache(&req.model_name, &[KVCache { key: keys, value: values, ..kv_cache }], 8, false).await;

        let output = InferenceOutput {
            text: req.input.join(" ").to_string(),
            tokens_processed: req.input.len(),
            latency_ms: start.elapsed().as_millis() as u64,
        };
        if output.text.is_empty() {
            return Err(InferenceError::Validation("Empty inference output".to_string()));
        }

        self.vault.store_kv_cache(&req.model_name, Array2::zeros((1, 1)), Array2::zeros((1, 1))).await?; // Update cache
        Ok(output)
    }

    async fn handle_compaction_errors(&self) -> Result<(), InferenceError> {
        // Mock compaction error handling
        log::warn!("Handling compaction error...");
        Ok(())
    }
}

impl InferenceRequest {
    fn validate(&self) -> Result<(), String> {
        if self.model_name.is_empty() || self.input.is_empty() {
            Err("Invalid request data".to_string())
        } else {
            Ok(())
        }
    }
}